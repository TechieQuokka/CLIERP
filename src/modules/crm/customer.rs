use diesel::prelude::*;
use chrono::Utc;
use serde::Serialize;
use crate::core::result::CLIERPResult;
use crate::database::{
    DatabaseConnection, Customer, NewCustomer, CustomerType, CustomerStatus, CustomerWithStats,
    CustomerSummary
};
use crate::database::schema::{customers, leads, deals};
use crate::utils::validation::{validate_email, validate_required_string};
use crate::utils::pagination::{Paginate, PaginationParams, PaginatedResult};
use crate::utils::filters::FilterOptions;

pub struct CustomerService;

impl CustomerService {
    pub fn create_customer(
        conn: &mut DatabaseConnection,
        name: &str,
        customer_type: CustomerType,
        email: Option<&str>,
        phone: Option<&str>,
        address: Option<&str>,
        company_name: Option<&str>,
        tax_id: Option<&str>,
        credit_limit: Option<i32>,
        notes: Option<&str>,
    ) -> Result<Customer> {
        // Validate input
        let validator = Validator::new();
        validator
            .required("name", name)?
            .min_length("name", name, 2)?
            .max_length("name", name, 200)?;

        if let Some(email) = email {
            validator.email("email", email)?;
        }

        if let Some(phone) = phone {
            validator
                .min_length("phone", phone, 8)?
                .max_length("phone", phone, 20)?;
        }

        // Generate customer code
        let customer_code = Self::generate_customer_code(conn)?;

        // Create new customer
        let new_customer = NewCustomer {
            customer_code,
            name: name.to_string(),
            email: email.map(|s| s.to_string()),
            phone: phone.map(|s| s.to_string()),
            address: address.map(|s| s.to_string()),
            customer_type: customer_type.to_string(),
            company_name: company_name.map(|s| s.to_string()),
            tax_id: tax_id.map(|s| s.to_string()),
            credit_limit: credit_limit.unwrap_or(0),
            status: CustomerStatus::Active.to_string(),
            notes: notes.map(|s| s.to_string()),
        };

        diesel::insert_into(customers::table)
            .values(&new_customer)
            .returning(Customer::as_returning())
            .get_result(conn)
            .map_err(Into::into)
    }

    pub fn get_customer_by_id(conn: &mut DatabaseConnection, customer_id: i32) -> Result<Option<Customer>> {
        customers::table
            .find(customer_id)
            .first::<Customer>(conn)
            .optional()
            .map_err(Into::into)
    }

    pub fn get_customer_by_code(conn: &mut DatabaseConnection, customer_code: &str) -> Result<Option<Customer>> {
        customers::table
            .filter(customers::customer_code.eq(customer_code))
            .first::<Customer>(conn)
            .optional()
            .map_err(Into::into)
    }

    pub fn get_customer_with_stats(conn: &mut DatabaseConnection, customer_id: i32) -> Result<Option<CustomerWithStats>> {
        let customer = Self::get_customer_by_id(conn, customer_id)?;

        if let Some(customer) = customer {
            // Get total leads count
            let total_leads = leads::table
                .filter(leads::customer_id.eq(customer_id))
                .count()
                .get_result::<i64>(conn)?;

            // Get active deals count
            let active_deals = deals::table
                .inner_join(leads::table)
                .filter(leads::customer_id.eq(customer_id))
                .filter(deals::stage.ne("closed_won").and(deals::stage.ne("closed_lost")))
                .count()
                .get_result::<i64>(conn)?;

            // Get total deal value (active deals)
            let total_deal_value: Option<i64> = deals::table
                .inner_join(leads::table)
                .filter(leads::customer_id.eq(customer_id))
                .filter(deals::stage.ne("closed_won").and(deals::stage.ne("closed_lost")))
                .select(diesel::dsl::sum(deals::deal_value))
                .first(conn)?;

            // Get last activity (placeholder - would need activities table)
            let last_activity = None; // TODO: Implement when activities are ready

            Ok(Some(CustomerWithStats {
                customer,
                total_leads,
                active_deals,
                total_deal_value: total_deal_value.unwrap_or(0) as i32,
                last_activity,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_customers(
        conn: &mut DatabaseConnection,
        filters: &FilterOptions,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResult<Customer>> {
        let mut query = customers::table.into_boxed();

        // Apply filters
        if let Some(search) = &filters.search {
            query = query.filter(
                customers::name.like(format!("%{}%", search))
                    .or(customers::customer_code.like(format!("%{}%", search)))
                    .or(customers::email.like(format!("%{}%", search)))
                    .or(customers::company_name.like(format!("%{}%", search)))
            );
        }

        if let Some(status_filter) = &filters.status {
            query = query.filter(customers::status.eq(status_filter));
        }

        if let Some(type_filter) = &filters.filter_type {
            query = query.filter(customers::customer_type.eq(type_filter));
        }

        // Apply sorting
        query = match filters.sort_by.as_deref() {
            Some("name") => {
                if filters.sort_desc {
                    query.order(customers::name.desc())
                } else {
                    query.order(customers::name.asc())
                }
            }
            Some("code") => {
                if filters.sort_desc {
                    query.order(customers::customer_code.desc())
                } else {
                    query.order(customers::customer_code.asc())
                }
            }
            Some("type") => {
                if filters.sort_desc {
                    query.order(customers::customer_type.desc())
                } else {
                    query.order(customers::customer_type.asc())
                }
            }
            Some("created_at") => {
                if filters.sort_desc {
                    query.order(customers::created_at.desc())
                } else {
                    query.order(customers::created_at.asc())
                }
            }
            _ => query.order(customers::created_at.desc()),
        };

        query.paginate_result(pagination, conn)
    }

    pub fn get_customer_summaries(
        conn: &mut DatabaseConnection,
        filters: &FilterOptions,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResult<CustomerSummary>> {
        let customer_query = customers::table
            .left_join(leads::table)
            .left_join(deals::table.on(deals::lead_id.eq(leads::id.nullable())))
            .group_by((
                customers::id,
                customers::customer_code,
                customers::name,
                customers::customer_type,
                customers::status,
            ))
            .select((
                customers::id,
                customers::customer_code,
                customers::name,
                customers::customer_type,
                diesel::dsl::count(deals::id.nullable()),
                diesel::dsl::sum(deals::deal_value.nullable()).nullable(),
                customers::status,
            ));

        let results: Vec<(i32, String, String, String, i64, Option<i64>, String)> = customer_query
            .offset(pagination.offset())
            .limit(pagination.limit)
            .load(conn)?;

        let total_items = customers::table.count().get_result::<i64>(conn)?;

        let summaries: Vec<CustomerSummary> = results
            .into_iter()
            .map(|(id, code, name, customer_type, deal_count, total_value, status)| {
                CustomerSummary {
                    id,
                    customer_code: code,
                    name,
                    customer_type,
                    total_deals: deal_count,
                    total_value: total_value.unwrap_or(0) as i32,
                    status,
                }
            })
            .collect();

        Ok(PaginatedResult {
            items: summaries,
            total_items,
            page: pagination.page,
            per_page: pagination.per_page,
            total_pages: (total_items as f64 / pagination.per_page as f64).ceil() as i64,
        })
    }

    pub fn update_customer(
        conn: &mut DatabaseConnection,
        customer_id: i32,
        name: Option<&str>,
        email: Option<Option<&str>>,
        phone: Option<Option<&str>>,
        address: Option<Option<&str>>,
        company_name: Option<Option<&str>>,
        tax_id: Option<Option<&str>>,
        credit_limit: Option<i32>,
        status: Option<CustomerStatus>,
        notes: Option<Option<&str>>,
    ) -> Result<Customer> {
        // Check if customer exists
        let _customer = Self::get_customer_by_id(conn, customer_id)?
            .ok_or_else(|| crate::core::error::CLIERPError::NotFound(
                format!("Customer with ID {} not found", customer_id)
            ))?;

        // Validate input
        let validator = Validator::new();
        if let Some(name) = name {
            validator
                .required("name", name)?
                .min_length("name", name, 2)?
                .max_length("name", name, 200)?;
        }

        if let Some(Some(email)) = email {
            validator.email("email", email)?;
        }

        // Build update query
        use crate::database::schema::customers::dsl::*;
        let mut update_query = diesel::update(customers.find(customer_id));

        if let Some(name_val) = name {
            update_query = update_query.set(name.eq(name_val));
        }
        if let Some(email_val) = email {
            update_query = update_query.set(email.eq(email_val.map(|s| s.to_string())));
        }
        if let Some(phone_val) = phone {
            update_query = update_query.set(phone.eq(phone_val.map(|s| s.to_string())));
        }
        if let Some(address_val) = address {
            update_query = update_query.set(address.eq(address_val.map(|s| s.to_string())));
        }
        if let Some(company_val) = company_name {
            update_query = update_query.set(company_name.eq(company_val.map(|s| s.to_string())));
        }
        if let Some(tax_val) = tax_id {
            update_query = update_query.set(tax_id.eq(tax_val.map(|s| s.to_string())));
        }
        if let Some(limit_val) = credit_limit {
            update_query = update_query.set(credit_limit.eq(limit_val));
        }
        if let Some(status_val) = status {
            update_query = update_query.set(status.eq(status_val.to_string()));
        }
        if let Some(notes_val) = notes {
            update_query = update_query.set(notes.eq(notes_val.map(|s| s.to_string())));
        }

        // Always update the updated_at timestamp
        update_query = update_query.set(updated_at.eq(Utc::now().naive_utc()));

        update_query
            .returning(Customer::as_returning())
            .get_result(conn)
            .map_err(Into::into)
    }

    pub fn delete_customer(conn: &mut DatabaseConnection, customer_id: i32) -> Result<bool> {
        // Check if customer has any leads or deals
        let has_leads = leads::table
            .filter(leads::customer_id.eq(customer_id))
            .first::<crate::database::Lead>(conn)
            .optional()?
            .is_some();

        if has_leads {
            return Err(crate::core::error::CLIERPError::BusinessLogic(
                "Cannot delete customer with existing leads or deals. Set status to inactive instead.".to_string()
            ));
        }

        let deleted_rows = diesel::delete(customers::table.find(customer_id))
            .execute(conn)?;

        Ok(deleted_rows > 0)
    }

    pub fn search_customers(conn: &mut DatabaseConnection, query: &str) -> Result<Vec<Customer>> {
        customers::table
            .filter(
                customers::name.like(format!("%{}%", query))
                    .or(customers::customer_code.like(format!("%{}%", query)))
                    .or(customers::email.like(format!("%{}%", query)))
                    .or(customers::company_name.like(format!("%{}%", query)))
            )
            .filter(customers::status.eq(CustomerStatus::Active.to_string()))
            .order(customers::name.asc())
            .limit(10)
            .load::<Customer>(conn)
            .map_err(Into::into)
    }

    pub fn get_customer_statistics(conn: &mut DatabaseConnection) -> Result<CustomerStatistics> {
        // Total customers count
        let total_customers = customers::table
            .count()
            .get_result::<i64>(conn)?;

        // Active customers count
        let active_customers = customers::table
            .filter(customers::status.eq(CustomerStatus::Active.to_string()))
            .count()
            .get_result::<i64>(conn)?;

        // Business vs Individual breakdown
        let business_customers = customers::table
            .filter(customers::customer_type.eq(CustomerType::Business.to_string()))
            .filter(customers::status.eq(CustomerStatus::Active.to_string()))
            .count()
            .get_result::<i64>(conn)?;

        let individual_customers = customers::table
            .filter(customers::customer_type.eq(CustomerType::Individual.to_string()))
            .filter(customers::status.eq(CustomerStatus::Active.to_string()))
            .count()
            .get_result::<i64>(conn)?;

        // Total credit limit
        let total_credit_limit: Option<i64> = customers::table
            .filter(customers::status.eq(CustomerStatus::Active.to_string()))
            .select(diesel::dsl::sum(customers::credit_limit))
            .first(conn)?;

        Ok(CustomerStatistics {
            total_customers,
            active_customers,
            business_customers,
            individual_customers,
            total_credit_limit: total_credit_limit.unwrap_or(0) as i32,
        })
    }

    fn generate_customer_code(conn: &mut DatabaseConnection) -> Result<String> {
        let count = customers::table
            .count()
            .get_result::<i64>(conn)?;

        Ok(format!("CUST{:06}", count + 1))
    }
}

#[derive(Debug, Serialize)]
pub struct CustomerStatistics {
    pub total_customers: i64,
    pub active_customers: i64,
    pub business_customers: i64,
    pub individual_customers: i64,
    pub total_credit_limit: i32,
}