use chrono::Utc;
use diesel::prelude::*;

use crate::core::result::CLIERPResult;
use crate::database::connection::get_connection;
use crate::database::models::{Product, NewProduct, StockMovement, NewStockMovement, Category};
use crate::database::schema::{products, stock_movements, categories};
use crate::utils::pagination::{PaginationParams, PaginationResult};
use crate::utils::validation::{validate_required_string, ValidationResult};

#[derive(Debug, Clone)]
pub struct ProductService;

impl ProductService {
    pub fn new() -> Self {
        Self
    }

    pub fn create_product(
        &self,
        sku: &str,
        name: &str,
        description: Option<&str>,
        category_id: i32,
        price: i32,
        cost_price: i32,
        initial_stock: i32,
        min_stock_level: i32,
        max_stock_level: Option<i32>,
        unit: &str,
        barcode: Option<&str>,
    ) -> CLIERPResult<Product> {
        // Validate inputs
        validate_required_string(sku, "SKU")?;
        validate_required_string(name, "Product name")?;
        validate_required_string(unit, "Unit")?;

        if price < 0 {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Price cannot be negative".to_string(),
            ));
        }

        if cost_price < 0 {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Cost price cannot be negative".to_string(),
            ));
        }

        if initial_stock < 0 {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Initial stock cannot be negative".to_string(),
            ));
        }

        if min_stock_level < 0 {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Minimum stock level cannot be negative".to_string(),
            ));
        }

        if let Some(max_level) = max_stock_level {
            if max_level < min_stock_level {
                return Err(crate::core::error::CLIERPError::ValidationError(
                    "Maximum stock level cannot be less than minimum stock level".to_string(),
                ));
            }
        }

        let mut connection = get_connection()?;

        // Check if category exists
        categories::table
            .find(category_id)
            .first::<Category>(&mut connection)?;

        // Check for duplicate SKU
        let existing = products::table
            .filter(products::sku.eq(sku))
            .first::<Product>(&mut connection)
            .optional()?;

        if existing.is_some() {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "SKU already exists".to_string(),
            ));
        }

        let new_product = NewProduct {
            sku: sku.to_string(),
            name: name.to_string(),
            description: description.map(|s| s.to_string()),
            category_id,
            price,
            cost_price,
            current_stock: initial_stock,
            min_stock_level,
            max_stock_level,
            unit: unit.to_string(),
            barcode: barcode.map(|s| s.to_string()),
            is_active: true,
        };

        diesel::insert_into(products::table)
            .values(&new_product)
            .execute(&mut connection)?;

        let product = products::table
            .order(products::id.desc())
            .first::<Product>(&mut connection)?;

        // Create initial stock movement if stock > 0
        if initial_stock > 0 {
            let stock_movement = NewStockMovement {
                product_id: product.id,
                movement_type: "in".to_string(),
                quantity: initial_stock,
                unit_cost: Some(cost_price),
                reference_type: Some("initial_stock".to_string()),
                reference_id: None,
                notes: Some("Initial stock entry".to_string()),
                moved_by: None, // TODO: Add user context
            };

            diesel::insert_into(stock_movements::table)
                .values(&stock_movement)
                .execute(&mut connection)?;
        }

        tracing::info!("Created product: {} (SKU: {})", product.name, product.sku);
        Ok(product)
    }

    pub fn get_product_by_id(&self, id: i32) -> CLIERPResult<Product> {
        let mut connection = get_connection()?;

        let product = products::table
            .find(id)
            .first::<Product>(&mut connection)?;

        Ok(product)
    }

    pub fn get_product_by_sku(&self, sku: &str) -> CLIERPResult<Option<Product>> {
        let mut connection = get_connection()?;

        let product = products::table
            .filter(products::sku.eq(sku))
            .first::<Product>(&mut connection)
            .optional()?;

        Ok(product)
    }

    pub fn list_products(
        &self,
        pagination: &PaginationParams,
        category_id: Option<i32>,
        active_only: bool,
        search_term: Option<&str>,
        low_stock_only: bool,
    ) -> CLIERPResult<PaginationResult<ProductWithCategory>> {
        let mut connection = get_connection()?;

        let mut query = products::table
            .inner_join(categories::table)
            .into_boxed();

        // Filter by category
        if let Some(category_id) = category_id {
            query = query.filter(products::category_id.eq(category_id));
        }

        // Filter by active status
        if active_only {
            query = query.filter(products::is_active.eq(true));
        }

        // Search by name or SKU
        let search_pattern = if let Some(search_term) = search_term {
            let pattern = format!("%{}%", search_term);
            query = query.filter(
                products::name.like(pattern.clone())
                    .or(products::sku.like(pattern.clone()))
            );
            Some(pattern)
        } else {
            None
        };

        // Filter low stock items
        if low_stock_only {
            query = query.filter(products::current_stock.le(products::min_stock_level));
        }

        // Get total count
        let total_count = {
            let mut count_query = products::table
                .inner_join(categories::table)
                .into_boxed();

            if let Some(category_id) = category_id {
                count_query = count_query.filter(products::category_id.eq(category_id));
            }
            if active_only {
                count_query = count_query.filter(products::is_active.eq(true));
            }
            if let Some(ref pattern) = search_pattern {
                count_query = count_query.filter(
                    products::name.like(pattern.clone())
                        .or(products::sku.like(pattern.clone()))
                );
            }
            if low_stock_only {
                count_query = count_query.filter(products::current_stock.le(products::min_stock_level));
            }

            count_query.count().get_result::<i64>(&mut connection)? as usize
        };

        // Apply pagination and ordering
        let results = query
            .order_by(products::name.asc())
            .offset(pagination.offset())
            .limit(pagination.limit())
            .load::<(Product, Category)>(&mut connection)?;

        let products_with_category = results
            .into_iter()
            .map(|(product, category)| ProductWithCategory {
                product,
                category,
            })
            .collect();

        Ok(PaginationResult::new_simple(products_with_category, total_count, pagination))
    }

    pub fn update_product(
        &self,
        id: i32,
        name: Option<&str>,
        description: Option<Option<&str>>,
        category_id: Option<i32>,
        price: Option<i32>,
        cost_price: Option<i32>,
        min_stock_level: Option<i32>,
        max_stock_level: Option<Option<i32>>,
        unit: Option<&str>,
        barcode: Option<Option<&str>>,
        is_active: Option<bool>,
    ) -> CLIERPResult<Product> {
        let mut connection = get_connection()?;

        // Check if product exists
        let existing_product = self.get_product_by_id(id)?;

        // Validate inputs
        if let Some(name) = name {
            validate_required_string(name, "Product name")?;
        }

        if let Some(unit) = unit {
            validate_required_string(unit, "Unit")?;
        }

        if let Some(price) = price {
            if price < 0 {
                return Err(crate::core::error::CLIERPError::ValidationError(
                    "Price cannot be negative".to_string(),
                ));
            }
        }

        if let Some(cost_price) = cost_price {
            if cost_price < 0 {
                return Err(crate::core::error::CLIERPError::ValidationError(
                    "Cost price cannot be negative".to_string(),
                ));
            }
        }

        if let Some(min_level) = min_stock_level {
            if min_level < 0 {
                return Err(crate::core::error::CLIERPError::ValidationError(
                    "Minimum stock level cannot be negative".to_string(),
                ));
            }
        }

        // Check if category exists
        if let Some(category_id) = category_id {
            categories::table
                .find(category_id)
                .first::<Category>(&mut connection)?;
        }

        // Build update changeset
        let mut changeset = ProductUpdateChangeset::default();

        if let Some(name) = name {
            changeset.name = Some(name.to_string());
        }
        if let Some(description) = description {
            changeset.description = Some(description.map(|s| s.to_string()));
        }
        if let Some(category_id) = category_id {
            changeset.category_id = Some(category_id);
        }
        if let Some(price) = price {
            changeset.price = Some(price);
        }
        if let Some(cost_price) = cost_price {
            changeset.cost_price = Some(cost_price);
        }
        if let Some(min_stock_level) = min_stock_level {
            changeset.min_stock_level = Some(min_stock_level);
        }
        if let Some(max_stock_level) = max_stock_level {
            changeset.max_stock_level = Some(max_stock_level);
        }
        if let Some(unit) = unit {
            changeset.unit = Some(unit.to_string());
        }
        if let Some(barcode) = barcode {
            changeset.barcode = Some(barcode.map(|s| s.to_string()));
        }
        if let Some(is_active) = is_active {
            changeset.is_active = Some(is_active);
        }
        changeset.updated_at = Some(Utc::now().naive_utc());

        diesel::update(products::table.find(id))
            .set(&changeset)
            .execute(&mut connection)?;

        let updated_product = self.get_product_by_id(id)?;

        tracing::info!("Updated product: {} (SKU: {})", updated_product.name, updated_product.sku);
        Ok(updated_product)
    }

    pub fn update_stock(
        &self,
        product_id: i32,
        quantity_change: i32,
        movement_type: &str,
        unit_cost: Option<i32>,
        reference_type: Option<&str>,
        reference_id: Option<i32>,
        notes: Option<&str>,
        moved_by: Option<i32>,
    ) -> CLIERPResult<Product> {
        let mut connection = get_connection()?;

        // Check if product exists
        let mut product = self.get_product_by_id(product_id)?;

        // Validate movement type
        if !["in", "out", "adjustment"].contains(&movement_type) {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Invalid movement type. Must be 'in', 'out', or 'adjustment'".to_string(),
            ));
        }

        // Calculate new stock level
        let new_stock = match movement_type {
            "in" | "adjustment" if quantity_change > 0 => product.current_stock + quantity_change.abs(),
            "out" | "adjustment" if quantity_change < 0 => product.current_stock - quantity_change.abs(),
            "adjustment" => quantity_change, // Direct assignment for adjustment
            _ => {
                return Err(crate::core::error::CLIERPError::ValidationError(
                    "Invalid quantity change for movement type".to_string(),
                ));
            }
        };

        if new_stock < 0 {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Resulting stock cannot be negative".to_string(),
            ));
        }

        // Create stock movement record
        let stock_movement = NewStockMovement {
            product_id,
            movement_type: movement_type.to_string(),
            quantity: quantity_change,
            unit_cost,
            reference_type: reference_type.map(|s| s.to_string()),
            reference_id,
            notes: notes.map(|s| s.to_string()),
            moved_by,
        };

        // Update product stock
        product.current_stock = new_stock;

        // Execute in transaction
        connection.transaction::<_, diesel::result::Error, _>(|conn| {
            // Insert stock movement
            diesel::insert_into(stock_movements::table)
                .values(&stock_movement)
                .execute(conn)?;

            // Update product stock
            diesel::update(products::table.find(product_id))
                .set((
                    products::current_stock.eq(new_stock),
                    products::updated_at.eq(Utc::now().naive_utc()),
                ))
                .execute(conn)?;

            Ok(())
        })?;

        // Reload product to get updated data
        product = self.get_product_by_id(product_id)?;

        tracing::info!(
            "Updated stock for product {} ({}): {} -> {}",
            product.name,
            product.sku,
            product.current_stock - quantity_change,
            product.current_stock
        );

        Ok(product)
    }

    pub fn get_stock_movements(
        &self,
        product_id: i32,
        pagination: &PaginationParams,
    ) -> CLIERPResult<PaginationResult<StockMovement>> {
        let mut connection = get_connection()?;

        // Get total count
        let total_count = stock_movements::table
            .filter(stock_movements::product_id.eq(product_id))
            .count()
            .get_result::<i64>(&mut connection)? as usize;

        // Get movements with pagination
        let movements = stock_movements::table
            .filter(stock_movements::product_id.eq(product_id))
            .order_by(stock_movements::movement_date.desc())
            .offset(pagination.offset())
            .limit(pagination.limit())
            .load::<StockMovement>(&mut connection)?;

        Ok(PaginationResult::new_simple(movements, total_count, pagination))
    }

    pub fn get_low_stock_products(&self) -> CLIERPResult<Vec<ProductWithCategory>> {
        let mut connection = get_connection()?;

        let results = products::table
            .inner_join(categories::table)
            .filter(products::is_active.eq(true))
            .filter(products::current_stock.le(products::min_stock_level))
            .order_by(products::name.asc())
            .load::<(Product, Category)>(&mut connection)?;

        let products_with_category = results
            .into_iter()
            .map(|(product, category)| ProductWithCategory {
                product,
                category,
            })
            .collect();

        Ok(products_with_category)
    }

    pub fn delete_product(&self, id: i32, force: bool) -> CLIERPResult<()> {
        let mut connection = get_connection()?;

        // Check if product exists
        let product = self.get_product_by_id(id)?;

        // Check if product has stock movements
        let movement_count = stock_movements::table
            .filter(stock_movements::product_id.eq(id))
            .count()
            .get_result::<i64>(&mut connection)?;

        if movement_count > 0 && !force {
            return Err(crate::core::error::CLIERPError::ValidationError(
                format!("Product has {} stock movements. Use --force to delete anyway.", movement_count),
            ));
        }

        // If force delete, remove stock movements first
        if force {
            diesel::delete(stock_movements::table.filter(stock_movements::product_id.eq(id)))
                .execute(&mut connection)?;
        }

        // Delete the product
        diesel::delete(products::table.find(id)).execute(&mut connection)?;

        tracing::info!("Deleted product: {} (SKU: {})", product.name, product.sku);
        Ok(())
    }
}

#[derive(Debug, Clone)]
pub struct ProductWithCategory {
    pub product: Product,
    pub category: Category,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = products)]
struct ProductUpdateChangeset {
    name: Option<String>,
    description: Option<Option<String>>,
    category_id: Option<i32>,
    price: Option<i32>,
    cost_price: Option<i32>,
    min_stock_level: Option<i32>,
    max_stock_level: Option<Option<i32>>,
    unit: Option<String>,
    barcode: Option<Option<String>>,
    is_active: Option<bool>,
    updated_at: Option<chrono::NaiveDateTime>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_product_service_creation() {
        let service = ProductService::new();
        // Basic instantiation test
        assert!(true);
    }
}