use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::schema::{suppliers, purchase_orders, purchase_items};

// Supplier models
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = suppliers)]
pub struct Supplier {
    pub id: i32,
    pub supplier_code: String,
    pub name: String,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub payment_terms: Option<String>,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = suppliers)]
pub struct NewSupplier {
    pub supplier_code: String,
    pub name: String,
    pub contact_person: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub payment_terms: Option<String>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SupplierStatus {
    Active,
    Inactive,
    Blacklisted,
}

impl std::fmt::Display for SupplierStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SupplierStatus::Active => write!(f, "active"),
            SupplierStatus::Inactive => write!(f, "inactive"),
            SupplierStatus::Blacklisted => write!(f, "blacklisted"),
        }
    }
}

// Purchase Order models
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = purchase_orders)]
pub struct PurchaseOrder {
    pub id: i32,
    pub po_number: String,
    pub supplier_id: i32,
    pub order_date: NaiveDate,
    pub expected_date: Option<NaiveDate>,
    pub status: String,
    pub total_amount: i32,
    pub notes: Option<String>,
    pub created_by: Option<i32>,
    pub approved_by: Option<i32>,
    pub approved_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = purchase_orders)]
pub struct NewPurchaseOrder {
    pub po_number: String,
    pub supplier_id: i32,
    pub order_date: NaiveDate,
    pub expected_date: Option<NaiveDate>,
    pub status: String,
    pub total_amount: i32,
    pub notes: Option<String>,
    pub created_by: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PurchaseOrderStatus {
    Pending,
    Approved,
    Sent,
    Received,
    Cancelled,
}

impl std::fmt::Display for PurchaseOrderStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PurchaseOrderStatus::Pending => write!(f, "pending"),
            PurchaseOrderStatus::Approved => write!(f, "approved"),
            PurchaseOrderStatus::Sent => write!(f, "sent"),
            PurchaseOrderStatus::Received => write!(f, "received"),
            PurchaseOrderStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

// Purchase Item models
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = purchase_items)]
pub struct PurchaseItem {
    pub id: i32,
    pub po_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub unit_cost: i32,
    pub total_cost: i32,
    pub received_quantity: i32,
    pub status: String,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = purchase_items)]
pub struct NewPurchaseItem {
    pub po_id: i32,
    pub product_id: i32,
    pub quantity: i32,
    pub unit_cost: i32,
    pub total_cost: i32,
    pub received_quantity: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PurchaseItemStatus {
    Pending,
    Partial,
    Received,
    Cancelled,
}

impl std::fmt::Display for PurchaseItemStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PurchaseItemStatus::Pending => write!(f, "pending"),
            PurchaseItemStatus::Partial => write!(f, "partial"),
            PurchaseItemStatus::Received => write!(f, "received"),
            PurchaseItemStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

// DTOs for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseOrderWithItems {
    pub purchase_order: PurchaseOrder,
    pub items: Vec<PurchaseItemWithProduct>,
    pub supplier: Supplier,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseItemWithProduct {
    pub purchase_item: PurchaseItem,
    pub product_name: String,
    pub product_sku: String,
    pub unit: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PurchaseOrderSummary {
    pub id: i32,
    pub po_number: String,
    pub supplier_name: String,
    pub order_date: NaiveDate,
    pub status: String,
    pub total_amount: i32,
    pub items_count: i64,
}