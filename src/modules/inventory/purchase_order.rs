use diesel::prelude::*;
use chrono::{Utc, NaiveDate};
use crate::core::result::Result;
use crate::database::{
    DbConnection, PurchaseOrder, NewPurchaseOrder, PurchaseItem, NewPurchaseItem,
    PurchaseOrderStatus, PurchaseItemStatus, PurchaseOrderWithItems, PurchaseItemWithProduct,
    PurchaseOrderSummary, Supplier, Product
};
use crate::database::schema::{purchase_orders, purchase_items, suppliers, products};
use crate::utils::validation::Validator;
use crate::utils::pagination::{Paginate, PaginationParams, PaginatedResult};
use crate::utils::filters::FilterOptions;

pub struct PurchaseOrderService;

impl PurchaseOrderService {
    pub fn create_purchase_order(
        conn: &mut DbConnection,
        supplier_id: i32,
        expected_date: Option<NaiveDate>,
        notes: Option<&str>,
        items: Vec<PurchaseOrderItem>,
        created_by: Option<i32>,
    ) -> Result<PurchaseOrderWithItems> {
        // Validate input
        let validator = Validator::new();
        validator.positive("supplier_id", supplier_id as f64)?;

        if items.is_empty() {
            return Err(crate::core::error::AppError::ValidationError(
                "Purchase order must have at least one item".to_string()
            ));
        }

        // Check if supplier exists and is active
        let supplier = suppliers::table
            .find(supplier_id)
            .first::<Supplier>(conn)?;

        if supplier.status != "active" {
            return Err(crate::core::error::AppError::BusinessLogic(
                "Cannot create purchase order for inactive supplier".to_string()
            ));
        }

        // Generate PO number
        let po_number = Self::generate_po_number(conn)?;

        // Calculate total amount
        let mut total_amount = 0i32;
        for item in &items {
            validator
                .positive("quantity", item.quantity as f64)?
                .positive("unit_cost", item.unit_cost as f64)?;

            // Verify product exists
            let _product = products::table
                .find(item.product_id)
                .first::<Product>(conn)?;

            total_amount += item.quantity * item.unit_cost;
        }

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            // Create purchase order
            let new_po = NewPurchaseOrder {
                po_number: po_number.clone(),
                supplier_id,
                order_date: Utc::now().naive_utc().date(),
                expected_date,
                status: PurchaseOrderStatus::Pending.to_string(),
                total_amount,
                notes: notes.map(|s| s.to_string()),
                created_by,
            };

            let purchase_order = diesel::insert_into(purchase_orders::table)
                .values(&new_po)
                .returning(PurchaseOrder::as_returning())
                .get_result::<PurchaseOrder>(conn)?;

            // Create purchase order items
            let mut created_items = Vec::new();
            for item in items {
                let total_cost = item.quantity * item.unit_cost;
                let new_item = NewPurchaseItem {
                    po_id: purchase_order.id,
                    product_id: item.product_id,
                    quantity: item.quantity,
                    unit_cost: item.unit_cost,
                    total_cost,
                    received_quantity: 0,
                    status: PurchaseItemStatus::Pending.to_string(),
                };

                let created_item = diesel::insert_into(purchase_items::table)
                    .values(&new_item)
                    .returning(PurchaseItem::as_returning())
                    .get_result::<PurchaseItem>(conn)?;

                created_items.push(created_item);
            }

            Ok((purchase_order, created_items))
        })
        .map_err(|e| crate::core::error::AppError::DatabaseError(e.to_string()))
        .and_then(|(po, items)| {
            Self::get_purchase_order_with_details(conn, po.id)
        })
    }

    pub fn get_purchase_order_by_id(conn: &mut DbConnection, po_id: i32) -> Result<Option<PurchaseOrder>> {
        purchase_orders::table
            .find(po_id)
            .first::<PurchaseOrder>(conn)
            .optional()
            .map_err(Into::into)
    }

    pub fn get_purchase_order_by_number(conn: &mut DbConnection, po_number: &str) -> Result<Option<PurchaseOrder>> {
        purchase_orders::table
            .filter(purchase_orders::po_number.eq(po_number))
            .first::<PurchaseOrder>(conn)
            .optional()
            .map_err(Into::into)
    }

    pub fn get_purchase_order_with_details(
        conn: &mut DbConnection,
        po_id: i32,
    ) -> Result<PurchaseOrderWithItems> {
        let purchase_order = Self::get_purchase_order_by_id(conn, po_id)?
            .ok_or_else(|| crate::core::error::AppError::NotFound(
                format!("Purchase order with ID {} not found", po_id)
            ))?;

        let supplier = suppliers::table
            .find(purchase_order.supplier_id)
            .first::<Supplier>(conn)?;

        let items_with_products: Vec<PurchaseItemWithProduct> = purchase_items::table
            .inner_join(products::table)
            .filter(purchase_items::po_id.eq(po_id))
            .select((
                PurchaseItem::as_select(),
                products::name,
                products::sku,
                products::unit,
            ))
            .load::<(PurchaseItem, String, String, String)>(conn)?
            .into_iter()
            .map(|(item, product_name, product_sku, unit)| PurchaseItemWithProduct {
                purchase_item: item,
                product_name,
                product_sku,
                unit,
            })
            .collect();

        Ok(PurchaseOrderWithItems {
            purchase_order,
            items: items_with_products,
            supplier,
        })
    }

    pub fn list_purchase_orders(
        conn: &mut DbConnection,
        filters: &FilterOptions,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResult<PurchaseOrderSummary>> {
        let mut query = purchase_orders::table
            .inner_join(suppliers::table)
            .select((
                purchase_orders::id,
                purchase_orders::po_number,
                suppliers::name,
                purchase_orders::order_date,
                purchase_orders::status,
                purchase_orders::total_amount,
            ))
            .into_boxed();

        // Apply filters
        if let Some(search) = &filters.search {
            query = query.filter(
                purchase_orders::po_number.like(format!("%{}%", search))
                    .or(suppliers::name.like(format!("%{}%", search)))
            );
        }

        if let Some(status_filter) = &filters.status {
            query = query.filter(purchase_orders::status.eq(status_filter));
        }

        if let Some(date_from) = filters.date_from {
            query = query.filter(purchase_orders::order_date.ge(date_from));
        }

        if let Some(date_to) = filters.date_to {
            query = query.filter(purchase_orders::order_date.le(date_to));
        }

        // Apply sorting
        query = match filters.sort_by.as_deref() {
            Some("po_number") => {
                if filters.sort_desc {
                    query.order(purchase_orders::po_number.desc())
                } else {
                    query.order(purchase_orders::po_number.asc())
                }
            }
            Some("supplier") => {
                if filters.sort_desc {
                    query.order(suppliers::name.desc())
                } else {
                    query.order(suppliers::name.asc())
                }
            }
            Some("date") => {
                if filters.sort_desc {
                    query.order(purchase_orders::order_date.desc())
                } else {
                    query.order(purchase_orders::order_date.asc())
                }
            }
            Some("amount") => {
                if filters.sort_desc {
                    query.order(purchase_orders::total_amount.desc())
                } else {
                    query.order(purchase_orders::total_amount.asc())
                }
            }
            _ => query.order(purchase_orders::created_at.desc()),
        };

        let results: Vec<(i32, String, String, NaiveDate, String, i32)> = query
            .offset(pagination.offset())
            .limit(pagination.limit)
            .load(conn)?;

        let total_items = purchase_orders::table
            .inner_join(suppliers::table)
            .count()
            .get_result::<i64>(conn)?;

        // Count items for each PO
        let po_ids: Vec<i32> = results.iter().map(|(id, _, _, _, _, _)| *id).collect();
        let items_counts: Vec<(i32, i64)> = purchase_items::table
            .filter(purchase_items::po_id.eq_any(&po_ids))
            .group_by(purchase_items::po_id)
            .select((purchase_items::po_id, diesel::dsl::count(purchase_items::id)))
            .load(conn)?;

        let items_count_map: std::collections::HashMap<i32, i64> =
            items_counts.into_iter().collect();

        let summaries: Vec<PurchaseOrderSummary> = results
            .into_iter()
            .map(|(id, po_number, supplier_name, order_date, status, total_amount)| {
                PurchaseOrderSummary {
                    id,
                    po_number,
                    supplier_name,
                    order_date,
                    status,
                    total_amount,
                    items_count: *items_count_map.get(&id).unwrap_or(&0),
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

    pub fn approve_purchase_order(
        conn: &mut DbConnection,
        po_id: i32,
        approved_by: i32,
    ) -> Result<PurchaseOrder> {
        let purchase_order = Self::get_purchase_order_by_id(conn, po_id)?
            .ok_or_else(|| crate::core::error::AppError::NotFound(
                format!("Purchase order with ID {} not found", po_id)
            ))?;

        if purchase_order.status != PurchaseOrderStatus::Pending.to_string() {
            return Err(crate::core::error::AppError::BusinessLogic(
                "Only pending purchase orders can be approved".to_string()
            ));
        }

        diesel::update(purchase_orders::table.find(po_id))
            .set((
                purchase_orders::status.eq(PurchaseOrderStatus::Approved.to_string()),
                purchase_orders::approved_by.eq(Some(approved_by)),
                purchase_orders::approved_at.eq(Some(Utc::now().naive_utc())),
                purchase_orders::updated_at.eq(Utc::now().naive_utc()),
            ))
            .returning(PurchaseOrder::as_returning())
            .get_result(conn)
            .map_err(Into::into)
    }

    pub fn receive_purchase_items(
        conn: &mut DbConnection,
        po_id: i32,
        received_items: Vec<ReceiveItemData>,
        received_by: Option<i32>,
    ) -> Result<PurchaseOrder> {
        let purchase_order = Self::get_purchase_order_by_id(conn, po_id)?
            .ok_or_else(|| crate::core::error::AppError::NotFound(
                format!("Purchase order with ID {} not found", po_id)
            ))?;

        if purchase_order.status != PurchaseOrderStatus::Approved.to_string()
            && purchase_order.status != PurchaseOrderStatus::Sent.to_string() {
            return Err(crate::core::error::AppError::BusinessLogic(
                "Only approved or sent purchase orders can be received".to_string()
            ));
        }

        conn.transaction::<_, diesel::result::Error, _>(|conn| {
            for receive_data in received_items {
                // Get current item
                let current_item = purchase_items::table
                    .find(receive_data.item_id)
                    .first::<PurchaseItem>(conn)?;

                if current_item.po_id != po_id {
                    return Err(diesel::result::Error::RollbackTransaction);
                }

                let new_received = current_item.received_quantity + receive_data.quantity;
                if new_received > current_item.quantity {
                    return Err(diesel::result::Error::RollbackTransaction);
                }

                // Update item status
                let new_status = if new_received == current_item.quantity {
                    PurchaseItemStatus::Received.to_string()
                } else {
                    PurchaseItemStatus::Partial.to_string()
                };

                diesel::update(purchase_items::table.find(receive_data.item_id))
                    .set((
                        purchase_items::received_quantity.eq(new_received),
                        purchase_items::status.eq(new_status),
                    ))
                    .execute(conn)?;

                // Update product stock
                use crate::database::schema::products;
                diesel::update(products::table.find(current_item.product_id))
                    .set(products::current_stock.eq(products::current_stock + receive_data.quantity))
                    .execute(conn)?;

                // Create stock movement record
                use crate::database::schema::stock_movements;
                use crate::database::{NewStockMovement, StockMovementType};

                let stock_movement = NewStockMovement {
                    product_id: current_item.product_id,
                    movement_type: StockMovementType::In.to_string(),
                    quantity: receive_data.quantity,
                    unit_cost: Some(current_item.unit_cost),
                    reference_type: Some("purchase_order".to_string()),
                    reference_id: Some(po_id),
                    notes: Some(format!("Received from PO #{}", purchase_order.po_number)),
                    moved_by: received_by,
                };

                diesel::insert_into(stock_movements::table)
                    .values(&stock_movement)
                    .execute(conn)?;
            }

            // Check if all items are fully received
            let remaining_items = purchase_items::table
                .filter(purchase_items::po_id.eq(po_id))
                .filter(purchase_items::status.ne(PurchaseItemStatus::Received.to_string()))
                .count()
                .get_result::<i64>(conn)?;

            let new_po_status = if remaining_items == 0 {
                PurchaseOrderStatus::Received.to_string()
            } else {
                purchase_order.status // Keep current status
            };

            // Update purchase order status if needed
            if new_po_status != purchase_order.status {
                diesel::update(purchase_orders::table.find(po_id))
                    .set((
                        purchase_orders::status.eq(new_po_status),
                        purchase_orders::updated_at.eq(Utc::now().naive_utc()),
                    ))
                    .execute(conn)?;
            }

            Ok(())
        })
        .map_err(|e| crate::core::error::AppError::DatabaseError(e.to_string()))?;

        Self::get_purchase_order_by_id(conn, po_id)?
            .ok_or_else(|| crate::core::error::AppError::NotFound("Purchase order not found".to_string()))
    }

    fn generate_po_number(conn: &mut DbConnection) -> Result<String> {
        let count = purchase_orders::table
            .count()
            .get_result::<i64>(conn)?;

        let today = Utc::now().naive_utc().date();
        Ok(format!("PO{}{:06}", today.format("%Y%m%d"), count + 1))
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct PurchaseOrderItem {
    pub product_id: i32,
    pub quantity: i32,
    pub unit_cost: i32,
}

#[derive(Debug, serde::Deserialize)]
pub struct ReceiveItemData {
    pub item_id: i32,
    pub quantity: i32,
}