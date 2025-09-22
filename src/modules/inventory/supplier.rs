use diesel::prelude::*;
use chrono::{Utc, NaiveDate};
use crate::core::result::CLIERPResult;

// Type alias for convenience
type Result<T> = CLIERPResult<T>;
use crate::database::{DatabaseConnection, Supplier, NewSupplier, SupplierStatus};
use crate::database::schema::suppliers;
use crate::utils::validation::{validate_email, validate_required_string};
use crate::utils::pagination::{Paginate, PaginationParams, PaginatedResult};
use crate::utils::filters::FilterOptions;

pub struct SupplierService;

impl SupplierService {
    pub fn create_supplier(
        conn: &mut DatabaseConnection,
        supplier_code: &str,
        name: &str,
        contact_person: Option<&str>,
        email: Option<&str>,
        phone: Option<&str>,
        address: Option<&str>,
        payment_terms: Option<&str>,
    ) -> Result<Supplier> {
        // Validate input
        validate_required_string(supplier_code, "supplier_code")?;
        validate_required_string(name, "name")?;
        if supplier_code.len() < 2 {
            return Err(crate::core::error::CLIERPError::Validation(
                "Supplier code must be at least 2 characters long".to_string()
            ));
        }
        if supplier_code.len() > 20 {
            return Err(crate::core::error::CLIERPError::Validation(
                "Supplier code cannot exceed 20 characters".to_string()
            ));
        }
        if name.len() < 2 {
            return Err(crate::core::error::CLIERPError::Validation(
                "Name must be at least 2 characters long".to_string()
            ));
        }
        if name.len() > 200 {
            return Err(crate::core::error::CLIERPError::Validation(
                "Name cannot exceed 200 characters".to_string()
            ));
        }

        if let Some(email) = email {
            validate_email(email)?;
        }

        // Check if supplier code already exists
        let existing = suppliers::table
            .filter(suppliers::supplier_code.eq(supplier_code))
            .first::<Supplier>(conn)
            .optional()?;

        if existing.is_some() {
            return Err(crate::core::error::CLIERPError::BusinessLogic(
                format!("Supplier with code '{}' already exists", supplier_code)
            ));
        }

        // Create new supplier
        let new_supplier = NewSupplier {
            supplier_code: supplier_code.to_string(),
            name: name.to_string(),
            contact_person: contact_person.map(|s| s.to_string()),
            email: email.map(|s| s.to_string()),
            phone: phone.map(|s| s.to_string()),
            address: address.map(|s| s.to_string()),
            payment_terms: payment_terms.map(|s| s.to_string()),
            status: SupplierStatus::Active.to_string(),
        };

        diesel::insert_into(suppliers::table)
            .values(&new_supplier)
            .execute(conn)?;

        // Get the inserted supplier by supplier code since SQLite doesn't support RETURNING
        suppliers::table
            .filter(suppliers::supplier_code.eq(&new_supplier.supplier_code))
            .first::<Supplier>(conn)
            .map_err(Into::into)
    }

    pub fn get_supplier_by_id(conn: &mut DatabaseConnection, supplier_id: i32) -> Result<Option<Supplier>> {
        suppliers::table
            .find(supplier_id)
            .first::<Supplier>(conn)
            .optional()
            .map_err(Into::into)
    }

    pub fn get_supplier_by_code(conn: &mut DatabaseConnection, supplier_code: &str) -> Result<Option<Supplier>> {
        suppliers::table
            .filter(suppliers::supplier_code.eq(supplier_code))
            .first::<Supplier>(conn)
            .optional()
            .map_err(Into::into)
    }

    pub fn list_suppliers(
        conn: &mut DatabaseConnection,
        filters: &FilterOptions,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResult<Supplier>> {
        let mut query = suppliers::table.into_boxed();

        // Apply filters
        if let Some(search) = &filters.search {
            query = query.filter(
                suppliers::name.like(format!("%{}%", search))
                    .or(suppliers::supplier_code.like(format!("%{}%", search)))
                    .or(suppliers::contact_person.like(format!("%{}%", search)))
            );
        }

        if let Some(status_filter) = &filters.status {
            query = query.filter(suppliers::status.eq(status_filter));
        }

        // Apply sorting
        query = match filters.sort_by.as_deref() {
            Some("name") => {
                if filters.sort_desc {
                    query.order(suppliers::name.desc())
                } else {
                    query.order(suppliers::name.asc())
                }
            }
            Some("code") => {
                if filters.sort_desc {
                    query.order(suppliers::supplier_code.desc())
                } else {
                    query.order(suppliers::supplier_code.asc())
                }
            }
            Some("created_at") => {
                if filters.sort_desc {
                    query.order(suppliers::created_at.desc())
                } else {
                    query.order(suppliers::created_at.asc())
                }
            }
            _ => query.order(suppliers::created_at.desc()),
        };

        query.paginate_result(pagination, conn)
    }

    pub fn update_supplier(
        conn: &mut DatabaseConnection,
        supplier_id: i32,
        name: Option<&str>,
        contact_person: Option<Option<&str>>,
        email: Option<Option<&str>>,
        phone: Option<Option<&str>>,
        address: Option<Option<&str>>,
        payment_terms: Option<Option<&str>>,
        status: Option<SupplierStatus>,
    ) -> Result<Supplier> {
        // Check if supplier exists
        let supplier = Self::get_supplier_by_id(conn, supplier_id)?
            .ok_or_else(|| crate::core::error::CLIERPError::NotFound(
                format!("Supplier with ID {} not found", supplier_id)
            ))?;

        // Validate input
        if let Some(name) = name {
            validate_required_string(name, "name")?;
            if name.len() < 2 {
                return Err(crate::core::error::CLIERPError::Validation(
                    "Name must be at least 2 characters long".to_string()
                ));
            }
            if name.len() > 200 {
                return Err(crate::core::error::CLIERPError::Validation(
                    "Name cannot exceed 200 characters".to_string()
                ));
            }
        }

        if let Some(Some(email)) = email {
            validate_email(email)?;
        }

        // Build update query dynamically
        use crate::database::schema::suppliers::dsl::*;

        let current_time = Utc::now().naive_utc();

        // First, update all non-None fields in separate statements if needed
        // Or create a single update with all fields including timestamp

        // For simplicity, let's update each field individually when provided
        if let Some(name_val) = name {
            diesel::update(suppliers.find(supplier_id))
                .set(name.eq(name_val))
                .execute(conn)?;
        }

        if let Some(contact_val) = contact_person {
            diesel::update(suppliers.find(supplier_id))
                .set(contact_person.eq(contact_val.map(|s| s.to_string())))
                .execute(conn)?;
        }

        if let Some(email_val) = email {
            diesel::update(suppliers.find(supplier_id))
                .set(email.eq(email_val.map(|s| s.to_string())))
                .execute(conn)?;
        }

        if let Some(phone_val) = phone {
            diesel::update(suppliers.find(supplier_id))
                .set(phone.eq(phone_val.map(|s| s.to_string())))
                .execute(conn)?;
        }

        if let Some(address_val) = address {
            diesel::update(suppliers.find(supplier_id))
                .set(address.eq(address_val.map(|s| s.to_string())))
                .execute(conn)?;
        }

        if let Some(payment_val) = payment_terms {
            diesel::update(suppliers.find(supplier_id))
                .set(payment_terms.eq(payment_val.map(|s| s.to_string())))
                .execute(conn)?;
        }

        if let Some(status_val) = status {
            diesel::update(suppliers.find(supplier_id))
                .set(status.eq(status_val.to_string()))
                .execute(conn)?;
        }

        // Always update the timestamp
        diesel::update(suppliers.find(supplier_id))
            .set(updated_at.eq(current_time))
            .execute(conn)?;

        // Get the updated supplier
        crate::database::schema::suppliers::table
            .find(supplier_id)
            .first::<Supplier>(conn)
            .map_err(Into::into)
    }

    pub fn delete_supplier(conn: &mut DatabaseConnection, supplier_id: i32) -> Result<bool> {
        // Check if supplier exists
        let supplier = Self::get_supplier_by_id(conn, supplier_id)?
            .ok_or_else(|| crate::core::error::CLIERPError::NotFound(
                format!("Supplier with ID {} not found", supplier_id)
            ))?;

        // Check if supplier has any purchase orders
        use crate::database::schema::purchase_orders;
        let has_orders = purchase_orders::table
            .filter(purchase_orders::supplier_id.eq(supplier_id))
            .first::<crate::database::PurchaseOrder>(conn)
            .optional()?
            .is_some();

        if has_orders {
            return Err(crate::core::error::CLIERPError::BusinessLogic(
                "Cannot delete supplier with existing purchase orders. Set status to inactive instead.".to_string()
            ));
        }

        let deleted_rows = diesel::delete(suppliers::table.find(supplier_id))
            .execute(conn)?;

        Ok(deleted_rows > 0)
    }

    pub fn get_active_suppliers(conn: &mut DatabaseConnection) -> Result<Vec<Supplier>> {
        suppliers::table
            .filter(suppliers::status.eq(SupplierStatus::Active.to_string()))
            .order(suppliers::name.asc())
            .load::<Supplier>(conn)
            .map_err(Into::into)
    }

    pub fn search_suppliers(conn: &mut DatabaseConnection, query: &str) -> Result<Vec<Supplier>> {
        suppliers::table
            .filter(
                suppliers::name.like(format!("%{}%", query))
                    .or(suppliers::supplier_code.like(format!("%{}%", query)))
                    .or(suppliers::contact_person.like(format!("%{}%", query)))
            )
            .filter(suppliers::status.eq(SupplierStatus::Active.to_string()))
            .order(suppliers::name.asc())
            .limit(10)
            .load::<Supplier>(conn)
            .map_err(Into::into)
    }

    pub fn get_supplier_statistics(conn: &mut DatabaseConnection, supplier_id: i32) -> Result<SupplierStatistics> {
        use crate::database::schema::{purchase_orders, purchase_items};

        // Get total purchase orders count
        let total_orders = purchase_orders::table
            .filter(purchase_orders::supplier_id.eq(supplier_id))
            .count()
            .get_result::<i64>(conn)?;

        // Get pending orders count
        let pending_orders = purchase_orders::table
            .filter(purchase_orders::supplier_id.eq(supplier_id))
            .filter(purchase_orders::status.eq("pending"))
            .count()
            .get_result::<i64>(conn)?;

        // Get total purchase amount
        let total_amount: Option<i64> = purchase_orders::table
            .filter(purchase_orders::supplier_id.eq(supplier_id))
            .select(diesel::dsl::sum(purchase_orders::total_amount))
            .first(conn)?;

        Ok(SupplierStatistics {
            total_orders,
            pending_orders,
            total_amount: total_amount.unwrap_or(0) as i32,
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct SupplierStatistics {
    pub total_orders: i64,
    pub pending_orders: i64,
    pub total_amount: i32,
}