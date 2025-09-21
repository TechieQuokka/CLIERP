use chrono::{NaiveDate, Utc};
use diesel::prelude::*;

use crate::core::result::CLIERPResult;
use crate::database::connection::get_connection;
use crate::database::models::{StockAudit, NewStockAudit, StockAuditItem, NewStockAuditItem, Product};
use crate::database::schema::{stock_audits, stock_audit_items, products, categories};
use crate::utils::pagination::{PaginationParams, PaginationResult};
use crate::utils::validation::{validate_required_string, ValidationResult};

#[derive(Debug, Clone)]
pub struct StockAuditService;

impl StockAuditService {
    pub fn new() -> Self {
        Self
    }

    pub fn create_audit(
        &self,
        audit_name: &str,
        audit_date: NaiveDate,
        conducted_by: Option<i32>,
        notes: Option<&str>,
    ) -> CLIERPResult<StockAudit> {
        validate_required_string(audit_name, "Audit name")?;

        let mut connection = get_connection()?;

        let new_audit = NewStockAudit {
            audit_name: audit_name.to_string(),
            audit_date,
            status: "pending".to_string(),
            conducted_by,
            notes: notes.map(|s| s.to_string()),
        };

        diesel::insert_into(stock_audits::table)
            .values(&new_audit)
            .execute(&mut connection)?;

        let audit = stock_audits::table
            .order(stock_audits::id.desc())
            .first::<StockAudit>(&mut connection)?;

        tracing::info!("Created stock audit: {} (ID: {})", audit.audit_name, audit.id);
        Ok(audit)
    }

    pub fn list_audits(
        &self,
        pagination: &PaginationParams,
        status_filter: Option<&str>,
    ) -> CLIERPResult<PaginationResult<StockAudit>> {
        let mut connection = get_connection()?;

        let mut query = stock_audits::table.into_boxed();

        if let Some(status) = status_filter {
            query = query.filter(stock_audits::status.eq(status));
        }

        // Get total count
        let total_count = {
            let mut count_query = stock_audits::table.into_boxed();
            if let Some(status) = status_filter {
                count_query = count_query.filter(stock_audits::status.eq(status));
            }
            count_query.count().get_result::<i64>(&mut connection)? as usize
        };

        // Get audits with pagination
        let audits = query
            .order_by(stock_audits::audit_date.desc())
            .offset(pagination.offset())
            .limit(pagination.limit())
            .load::<StockAudit>(&mut connection)?;

        Ok(PaginationResult::new_simple(audits, total_count, pagination))
    }

    pub fn get_audit(&self, id: i32) -> CLIERPResult<StockAudit> {
        let mut connection = get_connection()?;

        let audit = stock_audits::table
            .find(id)
            .first::<StockAudit>(&mut connection)?;

        Ok(audit)
    }

    pub fn update_audit_status(
        &self,
        id: i32,
        new_status: &str,
    ) -> CLIERPResult<StockAudit> {
        if !["pending", "in_progress", "completed", "cancelled"].contains(&new_status) {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Invalid audit status".to_string(),
            ));
        }

        let mut connection = get_connection()?;

        diesel::update(stock_audits::table.find(id))
            .set((
                stock_audits::status.eq(new_status),
                stock_audits::updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(&mut connection)?;

        let audit = self.get_audit(id)?;

        tracing::info!("Updated audit {} status to: {}", audit.audit_name, new_status);
        Ok(audit)
    }

    pub fn start_audit(&self, id: i32) -> CLIERPResult<Vec<StockAuditItem>> {
        let mut connection = get_connection()?;

        // Update audit status to in_progress
        self.update_audit_status(id, "in_progress")?;

        // Get all active products to audit
        let products = products::table
            .filter(products::is_active.eq(true))
            .load::<Product>(&mut connection)?;

        let mut audit_items = Vec::new();

        // Create audit items for each product
        for product in products {
            let new_audit_item = NewStockAuditItem {
                audit_id: id,
                product_id: product.id,
                expected_quantity: product.current_stock,
                actual_quantity: None,
                variance: None,
                notes: None,
                audited_at: None,
            };

            diesel::insert_into(stock_audit_items::table)
                .values(&new_audit_item)
                .execute(&mut connection)?;

            let audit_item = stock_audit_items::table
                .filter(stock_audit_items::audit_id.eq(id))
                .filter(stock_audit_items::product_id.eq(product.id))
                .first::<StockAuditItem>(&mut connection)?;

            audit_items.push(audit_item);
        }

        tracing::info!("Started audit {} with {} items", id, audit_items.len());
        Ok(audit_items)
    }

    pub fn record_audit_count(
        &self,
        audit_id: i32,
        product_id: i32,
        actual_quantity: i32,
        notes: Option<&str>,
    ) -> CLIERPResult<StockAuditItem> {
        let mut connection = get_connection()?;

        // Get the audit item
        let audit_item = stock_audit_items::table
            .filter(stock_audit_items::audit_id.eq(audit_id))
            .filter(stock_audit_items::product_id.eq(product_id))
            .first::<StockAuditItem>(&mut connection)?;

        // Calculate variance
        let variance = actual_quantity - audit_item.expected_quantity;

        // Update audit item
        diesel::update(stock_audit_items::table.find(audit_item.id))
            .set((
                stock_audit_items::actual_quantity.eq(Some(actual_quantity)),
                stock_audit_items::variance.eq(Some(variance)),
                stock_audit_items::notes.eq(notes.map(|s| s.to_string())),
                stock_audit_items::audited_at.eq(Some(Utc::now().naive_utc())),
            ))
            .execute(&mut connection)?;

        let updated_item = stock_audit_items::table
            .find(audit_item.id)
            .first::<StockAuditItem>(&mut connection)?;

        tracing::info!(
            "Recorded audit count for product {}: expected {}, actual {}, variance {}",
            product_id,
            audit_item.expected_quantity,
            actual_quantity,
            variance
        );

        Ok(updated_item)
    }

    pub fn get_audit_items(
        &self,
        audit_id: i32,
        pagination: &PaginationParams,
        variance_only: bool,
    ) -> CLIERPResult<PaginationResult<StockAuditItemWithProduct>> {
        use crate::modules::inventory::ProductWithCategory;
        let mut connection = get_connection()?;

        let mut query = stock_audit_items::table
            .inner_join(products::table.inner_join(categories::table))
            .filter(stock_audit_items::audit_id.eq(audit_id))
            .into_boxed();

        if variance_only {
            query = query.filter(stock_audit_items::variance.ne(Some(0)));
        }

        // Get total count
        let total_count = {
            let mut count_query = stock_audit_items::table
                .filter(stock_audit_items::audit_id.eq(audit_id))
                .into_boxed();

            if variance_only {
                count_query = count_query.filter(stock_audit_items::variance.ne(Some(0)));
            }

            count_query.count().get_result::<i64>(&mut connection)? as usize
        };

        // Get items with pagination
        let results = query
            .order_by(products::name.asc())
            .offset(pagination.offset())
            .limit(pagination.limit())
            .load::<(StockAuditItem, (Product, crate::database::models::Category))>(&mut connection)?;

        let items_with_product = results
            .into_iter()
            .map(|(item, (product, category))| StockAuditItemWithProduct {
                audit_item: item,
                product_with_category: ProductWithCategory {
                    product,
                    category,
                },
            })
            .collect();

        Ok(PaginationResult::new_simple(items_with_product, total_count, pagination))
    }

    pub fn complete_audit(&self, audit_id: i32, apply_adjustments: bool) -> CLIERPResult<AuditSummary> {
        let mut connection = get_connection()?;

        // Get audit
        let audit = self.get_audit(audit_id)?;

        if audit.status != "in_progress" {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Audit must be in progress to complete".to_string(),
            ));
        }

        // Get all audit items
        let audit_items = stock_audit_items::table
            .filter(stock_audit_items::audit_id.eq(audit_id))
            .load::<StockAuditItem>(&mut connection)?;

        // Check if all items are audited
        let unaudited_count = audit_items
            .iter()
            .filter(|item| item.actual_quantity.is_none())
            .count();

        if unaudited_count > 0 {
            return Err(crate::core::error::CLIERPError::ValidationError(
                format!("{} items are not yet audited", unaudited_count),
            ));
        }

        // Calculate summary
        let total_items = audit_items.len();
        let items_with_variance = audit_items
            .iter()
            .filter(|item| item.variance.unwrap_or(0) != 0)
            .count();

        let total_variance = audit_items
            .iter()
            .map(|item| item.variance.unwrap_or(0))
            .sum::<i32>();

        // Apply stock adjustments if requested
        if apply_adjustments {
            for audit_item in &audit_items {
                if let Some(variance) = audit_item.variance {
                    if variance != 0 {
                        // Get product
                        let product = products::table
                            .find(audit_item.product_id)
                            .first::<Product>(&mut connection)?;

                        // Apply adjustment to actual stock
                        let new_stock = audit_item.actual_quantity.unwrap_or(product.current_stock);

                        diesel::update(products::table.find(audit_item.product_id))
                            .set((
                                products::current_stock.eq(new_stock),
                                products::updated_at.eq(Utc::now().naive_utc()),
                            ))
                            .execute(&mut connection)?;

                        // Create stock movement record
                        let movement_type = if variance > 0 { "in" } else { "out" };
                        let movement_quantity = variance.abs();

                        let stock_movement = crate::database::models::NewStockMovement {
                            product_id: audit_item.product_id,
                            movement_type: movement_type.to_string(),
                            quantity: movement_quantity,
                            unit_cost: None,
                            reference_type: Some("audit_adjustment".to_string()),
                            reference_id: Some(audit_id),
                            notes: Some(format!("Stock audit adjustment: {}", audit.audit_name)),
                            moved_by: audit.conducted_by,
                        };

                        diesel::insert_into(crate::database::schema::stock_movements::table)
                            .values(&stock_movement)
                            .execute(&mut connection)?;

                        tracing::info!(
                            "Applied stock adjustment for product {}: {} -> {} (variance: {})",
                            audit_item.product_id,
                            product.current_stock,
                            new_stock,
                            variance
                        );
                    }
                }
            }
        }

        // Update audit status to completed
        self.update_audit_status(audit_id, "completed")?;

        let summary = AuditSummary {
            audit_id,
            audit_name: audit.audit_name,
            total_items,
            items_with_variance,
            total_variance,
            adjustments_applied: apply_adjustments,
        };

        tracing::info!(
            "Completed audit {}: {} items, {} with variance, total variance: {}",
            summary.audit_name,
            summary.total_items,
            summary.items_with_variance,
            summary.total_variance
        );

        Ok(summary)
    }

    pub fn cancel_audit(&self, audit_id: i32) -> CLIERPResult<()> {
        let mut connection = get_connection()?;

        // Update audit status
        self.update_audit_status(audit_id, "cancelled")?;

        // Delete all audit items
        diesel::delete(
            stock_audit_items::table.filter(stock_audit_items::audit_id.eq(audit_id))
        )
        .execute(&mut connection)?;

        tracing::info!("Cancelled audit {}", audit_id);
        Ok(())
    }

    pub fn delete_audit(&self, audit_id: i32, force: bool) -> CLIERPResult<()> {
        let mut connection = get_connection()?;

        let audit = self.get_audit(audit_id)?;

        if audit.status == "in_progress" && !force {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Cannot delete audit in progress. Use --force to delete anyway".to_string(),
            ));
        }

        // Delete audit items first
        diesel::delete(
            stock_audit_items::table.filter(stock_audit_items::audit_id.eq(audit_id))
        )
        .execute(&mut connection)?;

        // Delete audit
        diesel::delete(stock_audits::table.find(audit_id))
            .execute(&mut connection)?;

        tracing::info!("Deleted audit: {}", audit.audit_name);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct StockAuditItemWithProduct {
    pub audit_item: StockAuditItem,
    pub product_with_category: crate::modules::inventory::ProductWithCategory,
}

#[derive(Debug, Clone)]
pub struct AuditSummary {
    pub audit_id: i32,
    pub audit_name: String,
    pub total_items: usize,
    pub items_with_variance: usize,
    pub total_variance: i32,
    pub adjustments_applied: bool,
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::NaiveDate;

    #[test]
    fn test_audit_service_creation() {
        let service = StockAuditService::new();
        // Basic instantiation test
        assert!(true);
    }

    #[test]
    fn test_audit_summary_creation() {
        let summary = AuditSummary {
            audit_id: 1,
            audit_name: "Test Audit".to_string(),
            total_items: 10,
            items_with_variance: 3,
            total_variance: -5,
            adjustments_applied: true,
        };

        assert_eq!(summary.audit_id, 1);
        assert_eq!(summary.total_items, 10);
        assert_eq!(summary.items_with_variance, 3);
        assert_eq!(summary.total_variance, -5);
        assert!(summary.adjustments_applied);
    }
}