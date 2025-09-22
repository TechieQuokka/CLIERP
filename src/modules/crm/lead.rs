use diesel::prelude::*;
use chrono::{Utc, NaiveDate};
use crate::core::result::CLIERPResult;

// Type alias for convenience
type Result<T> = CLIERPResult<T>;
use crate::database::{
    DatabaseConnection, Lead, NewLead, LeadStatus, LeadPriority, LeadWithCustomer, Customer
};
use crate::database::schema::{leads, customers, employees};
use crate::utils::validation::validate_required_string;
use crate::utils::pagination::{Paginate, PaginationParams, PaginatedResult};
use crate::utils::filters::FilterOptions;

pub struct LeadService;

impl LeadService {
    pub fn create_lead(
        conn: &mut DatabaseConnection,
        title: &str,
        customer_id: Option<i32>,
        lead_source: &str,
        estimated_value: i32,
        expected_close_date: Option<NaiveDate>,
        priority: LeadPriority,
        assigned_to: Option<i32>,
        description: Option<&str>,
        notes: Option<&str>,
    ) -> Result<Lead> {
        // Validate input
        validate_required_string(title, "title")?;
        if title.len() < 2 {
            return Err(crate::core::error::CLIERPError::Validation(
                "Title must be at least 2 characters long".to_string()
            ));
        }
        if title.len() > 200 {
            return Err(crate::core::error::CLIERPError::Validation(
                "Title cannot exceed 200 characters".to_string()
            ));
        }
        validate_required_string(lead_source, "lead_source")?;
        if lead_source.len() < 2 {
            return Err(crate::core::error::CLIERPError::Validation(
                "Lead source must be at least 2 characters long".to_string()
            ));
        }
        if estimated_value < 0 {
            return Err(crate::core::error::CLIERPError::Validation(
                "Estimated value cannot be negative".to_string()
            ));
        }

        // Verify customer exists if provided
        if let Some(customer_id) = customer_id {
            customers::table
                .find(customer_id)
                .first::<Customer>(conn)?;
        }

        // Verify assigned employee exists if provided
        if let Some(assigned_to) = assigned_to {
            employees::table
                .find(assigned_to)
                .first::<crate::database::Employee>(conn)?;
        }

        // Create new lead
        let new_lead = NewLead {
            customer_id,
            lead_source: lead_source.to_string(),
            status: LeadStatus::New.to_string(),
            priority: priority.to_string(),
            estimated_value,
            probability: Self::calculate_initial_probability(&LeadStatus::New),
            expected_close_date,
            assigned_to,
            title: title.to_string(),
            description: description.map(|s| s.to_string()),
            notes: notes.map(|s| s.to_string()),
        };

        let current_time = Utc::now().naive_utc();
        let new_lead_with_time = NewLead {
            customer_id: new_lead.customer_id,
            lead_source: new_lead.lead_source.clone(),
            status: new_lead.status.clone(),
            priority: new_lead.priority.clone(),
            estimated_value: new_lead.estimated_value,
            probability: new_lead.probability,
            expected_close_date: new_lead.expected_close_date,
            assigned_to: new_lead.assigned_to,
            title: new_lead.title.clone(),
            description: new_lead.description.clone(),
            notes: new_lead.notes.clone(),
        };

        diesel::insert_into(leads::table)
            .values(&new_lead_with_time)
            .execute(conn)?;

        // Get the inserted lead by searching for the most recent lead with matching criteria
        leads::table
            .filter(leads::title.eq(&new_lead.title))
            .filter(leads::lead_source.eq(&new_lead.lead_source))
            .filter(leads::customer_id.eq(&new_lead.customer_id))
            .order(leads::created_at.desc())
            .first::<Lead>(conn)
            .map_err(Into::into)
    }

    pub fn get_lead_by_id(conn: &mut DatabaseConnection, lead_id: i32) -> Result<Option<Lead>> {
        leads::table
            .find(lead_id)
            .first::<Lead>(conn)
            .optional()
            .map_err(Into::into)
    }

    pub fn get_lead_with_customer(conn: &mut DatabaseConnection, lead_id: i32) -> Result<Option<LeadWithCustomer>> {
        let lead = Self::get_lead_by_id(conn, lead_id)?;

        if let Some(lead) = lead {
            // Get customer info if available
            let customer = if let Some(customer_id) = lead.customer_id {
                customers::table
                    .find(customer_id)
                    .first::<Customer>(conn)
                    .optional()?
            } else {
                None
            };

            // Get assigned employee name if available
            let assigned_employee = if let Some(assigned_to) = lead.assigned_to {
                employees::table
                    .find(assigned_to)
                    .select(employees::name)
                    .first::<String>(conn)
                    .optional()?
            } else {
                None
            };

            Ok(Some(LeadWithCustomer {
                lead,
                customer,
                assigned_employee,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_leads(
        conn: &mut DatabaseConnection,
        filters: &FilterOptions,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResult<LeadWithCustomer>> {
        let mut query = leads::table
            .left_join(customers::table)
            .left_join(employees::table.on(employees::id.eq(leads::assigned_to.nullable())))
            .select((
                Lead::as_select(),
                customers::all_columns.nullable(),
                employees::name.nullable(),
            ))
            .into_boxed();

        // Apply filters
        if let Some(search) = &filters.search {
            query = query.filter(
                leads::title.like(format!("%{}%", search))
                    .or(customers::name.like(format!("%{}%", search)))
                    .or(leads::lead_source.like(format!("%{}%", search)))
            );
        }

        if let Some(status_filter) = &filters.status {
            query = query.filter(leads::status.eq(status_filter));
        }

        if let Some(priority_filter) = &filters.priority {
            query = query.filter(leads::priority.eq(priority_filter));
        }

        if let Some(assigned_to) = filters.assigned_to {
            query = query.filter(leads::assigned_to.eq(assigned_to));
        }

        if let Some(date_from) = filters.date_from {
            query = query.filter(leads::expected_close_date.ge(date_from));
        }

        if let Some(date_to) = filters.date_to {
            query = query.filter(leads::expected_close_date.le(date_to));
        }

        // Apply sorting
        query = match filters.sort_by.as_deref() {
            Some("title") => {
                if filters.sort_desc {
                    query.order(leads::title.desc())
                } else {
                    query.order(leads::title.asc())
                }
            }
            Some("status") => {
                if filters.sort_desc {
                    query.order(leads::status.desc())
                } else {
                    query.order(leads::status.asc())
                }
            }
            Some("priority") => {
                if filters.sort_desc {
                    query.order(leads::priority.desc())
                } else {
                    query.order(leads::priority.asc())
                }
            }
            Some("value") => {
                if filters.sort_desc {
                    query.order(leads::estimated_value.desc())
                } else {
                    query.order(leads::estimated_value.asc())
                }
            }
            Some("close_date") => {
                if filters.sort_desc {
                    query.order(leads::expected_close_date.desc())
                } else {
                    query.order(leads::expected_close_date.asc())
                }
            }
            Some("created_at") => {
                if filters.sort_desc {
                    query.order(leads::created_at.desc())
                } else {
                    query.order(leads::created_at.asc())
                }
            }
            _ => query.order(leads::created_at.desc()),
        };

        let results: Vec<(Lead, Option<Customer>, Option<String>)> = query
            .offset(pagination.offset())
            .limit(pagination.limit)
            .load(conn)?;

        let total_items = leads::table.count().get_result::<i64>(conn)?;

        let leads_with_customer: Vec<LeadWithCustomer> = results
            .into_iter()
            .map(|(lead, customer, assigned_employee)| LeadWithCustomer {
                lead,
                customer,
                assigned_employee,
            })
            .collect();

        Ok(PaginatedResult {
            items: leads_with_customer,
            total_items,
            page: pagination.page,
            per_page: pagination.per_page,
            total_pages: (total_items as f64 / pagination.per_page as f64).ceil() as i64,
        })
    }

    pub fn update_lead_status(
        conn: &mut DatabaseConnection,
        lead_id: i32,
        new_status: LeadStatus,
        notes: Option<&str>,
    ) -> Result<Lead> {
        let lead = Self::get_lead_by_id(conn, lead_id)?
            .ok_or_else(|| crate::core::error::CLIERPError::NotFound(
                format!("Lead with ID {} not found", lead_id)
            ))?;

        // Calculate new probability based on status
        let new_probability = Self::calculate_probability_for_status(&new_status);

        let updated_notes = if let Some(new_notes) = notes {
            if let Some(existing_notes) = &lead.notes {
                Some(format!("{}\n---\n{}", existing_notes, new_notes))
            } else {
                Some(new_notes.to_string())
            }
        } else {
            lead.notes
        };

        diesel::update(leads::table.find(lead_id))
            .set((
                leads::status.eq(new_status.to_string()),
                leads::probability.eq(new_probability),
                leads::notes.eq(updated_notes),
                leads::updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(conn)?;

        // Get the updated lead
        leads::table
            .find(lead_id)
            .first::<Lead>(conn)
            .map_err(Into::into)
    }

    pub fn update_lead(
        conn: &mut DatabaseConnection,
        lead_id: i32,
        title: Option<&str>,
        customer_id: Option<Option<i32>>,
        lead_source: Option<&str>,
        estimated_value: Option<i32>,
        expected_close_date: Option<Option<NaiveDate>>,
        priority: Option<LeadPriority>,
        assigned_to: Option<Option<i32>>,
        description: Option<Option<&str>>,
        notes: Option<Option<&str>>,
    ) -> Result<Lead> {
        // Check if lead exists
        let _lead = Self::get_lead_by_id(conn, lead_id)?
            .ok_or_else(|| crate::core::error::CLIERPError::NotFound(
                format!("Lead with ID {} not found", lead_id)
            ))?;

        // Validate input
        if let Some(title) = title {
            validate_required_string(title, "title")?;
            if title.len() < 2 {
                return Err(crate::core::error::CLIERPError::Validation(
                    "Title must be at least 2 characters long".to_string()
                ));
            }
            if title.len() > 200 {
                return Err(crate::core::error::CLIERPError::Validation(
                    "Title cannot exceed 200 characters".to_string()
                ));
            }
        }

        if let Some(lead_source) = lead_source {
            validate_required_string(lead_source, "lead_source")?;
            if lead_source.len() < 2 {
                return Err(crate::core::error::CLIERPError::Validation(
                    "Lead source must be at least 2 characters long".to_string()
                ));
            }
        }

        if let Some(estimated_value) = estimated_value {
            if *estimated_value < 0 {
                return Err(crate::core::error::CLIERPError::Validation(
                    "Estimated value cannot be negative".to_string()
                ));
            }
        }

        // Build update query - update each field individually
        use crate::database::schema::leads::dsl::*;

        let current_time = Utc::now().naive_utc();

        if let Some(title_val) = title {
            diesel::update(leads.find(lead_id))
                .set(title.eq(title_val))
                .execute(conn)?;
        }

        if let Some(customer_val) = customer_id {
            diesel::update(leads.find(lead_id))
                .set(customer_id.eq(*customer_val))
                .execute(conn)?;
        }

        if let Some(source_val) = lead_source {
            diesel::update(leads.find(lead_id))
                .set(lead_source.eq(source_val))
                .execute(conn)?;
        }

        if let Some(value_val) = estimated_value {
            diesel::update(leads.find(lead_id))
                .set(estimated_value.eq(*value_val))
                .execute(conn)?;
        }

        if let Some(date_val) = expected_close_date {
            diesel::update(leads.find(lead_id))
                .set(expected_close_date.eq(*date_val))
                .execute(conn)?;
        }

        if let Some(priority_val) = priority {
            diesel::update(leads.find(lead_id))
                .set(priority.eq(priority_val.to_string()))
                .execute(conn)?;
        }

        if let Some(assigned_val) = assigned_to {
            diesel::update(leads.find(lead_id))
                .set(assigned_to.eq(*assigned_val))
                .execute(conn)?;
        }

        if let Some(desc_val) = description {
            diesel::update(leads.find(lead_id))
                .set(description.eq(desc_val.map(|s| s.to_string())))
                .execute(conn)?;
        }

        if let Some(notes_val) = notes {
            diesel::update(leads.find(lead_id))
                .set(notes.eq(notes_val.map(|s| s.to_string())))
                .execute(conn)?;
        }

        // Always update the updated_at timestamp
        diesel::update(leads.find(lead_id))
            .set(updated_at.eq(current_time))
            .execute(conn)?;

        // Get the updated lead
        leads::table
            .find(lead_id)
            .first::<Lead>(conn)
            .map_err(Into::into)
    }

    pub fn assign_lead(
        conn: &mut DatabaseConnection,
        lead_id: i32,
        assigned_to: i32,
    ) -> Result<Lead> {
        // Verify employee exists
        employees::table
            .find(assigned_to)
            .first::<crate::database::Employee>(conn)?;

        diesel::update(leads::table.find(lead_id))
            .set((
                leads::assigned_to.eq(Some(assigned_to)),
                leads::updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(conn)?;

        // Get the updated lead
        leads::table
            .find(lead_id)
            .first::<Lead>(conn)
            .map_err(Into::into)
    }

    pub fn delete_lead(conn: &mut DatabaseConnection, lead_id: i32) -> Result<bool> {
        // Check if lead has any deals
        use crate::database::schema::deals;
        let has_deals = deals::table
            .filter(deals::lead_id.eq(lead_id))
            .first::<crate::database::Deal>(conn)
            .optional()?
            .is_some();

        if has_deals {
            return Err(crate::core::error::CLIERPError::BusinessLogic(
                "Cannot delete lead with existing deals.".to_string()
            ));
        }

        let deleted_rows = diesel::delete(leads::table.find(lead_id))
            .execute(conn)?;

        Ok(deleted_rows > 0)
    }

    pub fn get_leads_by_status(
        conn: &mut DatabaseConnection,
        status: LeadStatus,
    ) -> Result<Vec<LeadWithCustomer>> {
        let results: Vec<(Lead, Option<Customer>, Option<String>)> = leads::table
            .left_join(customers::table)
            .left_join(employees::table.on(employees::id.eq(leads::assigned_to.nullable())))
            .filter(leads::status.eq(status.to_string()))
            .select((
                Lead::as_select(),
                customers::all_columns.nullable(),
                employees::name.nullable(),
            ))
            .order(leads::created_at.desc())
            .load(conn)?;

        let leads_with_customer: Vec<LeadWithCustomer> = results
            .into_iter()
            .map(|(lead, customer, assigned_employee)| LeadWithCustomer {
                lead,
                customer,
                assigned_employee,
            })
            .collect();

        Ok(leads_with_customer)
    }

    pub fn get_lead_statistics(conn: &mut DatabaseConnection) -> Result<LeadStatistics> {
        // Total leads count
        let total_leads = leads::table
            .count()
            .get_result::<i64>(conn)?;

        // Leads by status
        let new_leads = leads::table
            .filter(leads::status.eq(LeadStatus::New.to_string()))
            .count()
            .get_result::<i64>(conn)?;

        let qualified_leads = leads::table
            .filter(leads::status.eq(LeadStatus::Qualified.to_string()))
            .count()
            .get_result::<i64>(conn)?;

        let closed_won = leads::table
            .filter(leads::status.eq(LeadStatus::ClosedWon.to_string()))
            .count()
            .get_result::<i64>(conn)?;

        let closed_lost = leads::table
            .filter(leads::status.eq(LeadStatus::ClosedLost.to_string()))
            .count()
            .get_result::<i64>(conn)?;

        // Total estimated value
        let total_estimated_value: Option<i64> = leads::table
            .filter(leads::status.ne(LeadStatus::ClosedLost.to_string()))
            .select(diesel::dsl::sum(leads::estimated_value))
            .first(conn)?;

        // Average deal size
        let average_deal_size = if total_leads > 0 {
            total_estimated_value.unwrap_or(0) as f64 / total_leads as f64
        } else {
            0.0
        };

        // Conversion rate
        let conversion_rate = if total_leads > 0 {
            (closed_won as f64 / total_leads as f64) * 100.0
        } else {
            0.0
        };

        Ok(LeadStatistics {
            total_leads,
            new_leads,
            qualified_leads,
            closed_won,
            closed_lost,
            total_estimated_value: total_estimated_value.unwrap_or(0) as i32,
            average_deal_size,
            conversion_rate,
        })
    }

    fn calculate_initial_probability(status: &LeadStatus) -> i32 {
        Self::calculate_probability_for_status(status)
    }

    fn calculate_probability_for_status(status: &LeadStatus) -> i32 {
        match status {
            LeadStatus::New => 10,
            LeadStatus::Contacted => 25,
            LeadStatus::Qualified => 50,
            LeadStatus::Proposal => 70,
            LeadStatus::Negotiation => 85,
            LeadStatus::ClosedWon => 100,
            LeadStatus::ClosedLost => 0,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct LeadStatistics {
    pub total_leads: i64,
    pub new_leads: i64,
    pub qualified_leads: i64,
    pub closed_won: i64,
    pub closed_lost: i64,
    pub total_estimated_value: i32,
    pub average_deal_size: f64,
    pub conversion_rate: f64,
}