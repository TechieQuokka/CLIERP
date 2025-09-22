use chrono::{NaiveDate, NaiveDateTime, NaiveTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};

use super::schema::{
    accounts, attendances, audit_logs, categories, departments, employees, payrolls, products,
    product_attachments, stock_movements, stock_audits, stock_audit_items, transactions, users,
};

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = departments)]
pub struct Department {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub manager_id: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = departments)]
pub struct NewDepartment {
    pub name: String,
    pub description: Option<String>,
    pub manager_id: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = employees)]
pub struct Employee {
    pub id: i32,
    pub employee_code: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department_id: i32,
    pub position: String,
    pub hire_date: NaiveDate,
    pub salary: i32,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = employees)]
pub struct NewEmployee {
    pub employee_code: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department_id: i32,
    pub position: String,
    pub hire_date: NaiveDate,
    pub salary: i32,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub employee_id: Option<i32>,
    pub role: String,
    pub is_active: bool,
    pub last_login: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub employee_id: Option<i32>,
    pub role: String,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = audit_logs)]
pub struct AuditLog {
    pub id: i32,
    pub user_id: Option<i32>,
    pub table_name: String,
    pub record_id: i32,
    pub action: String,
    pub old_values: Option<String>,
    pub new_values: Option<String>,
    pub changed_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = audit_logs)]
pub struct NewAuditLog {
    pub user_id: Option<i32>,
    pub table_name: String,
    pub record_id: i32,
    pub action: String,
    pub old_values: Option<String>,
    pub new_values: Option<String>,
}

// Enums for better type safety
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum EmployeeStatus {
    Active,
    Inactive,
    Terminated,
}

impl std::fmt::Display for EmployeeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            EmployeeStatus::Active => write!(f, "active"),
            EmployeeStatus::Inactive => write!(f, "inactive"),
            EmployeeStatus::Terminated => write!(f, "terminated"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UserRole {
    Admin,
    Manager,
    Supervisor,
    Employee,
    Auditor,
}

impl std::fmt::Display for UserRole {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            UserRole::Admin => write!(f, "admin"),
            UserRole::Manager => write!(f, "manager"),
            UserRole::Supervisor => write!(f, "supervisor"),
            UserRole::Employee => write!(f, "employee"),
            UserRole::Auditor => write!(f, "auditor"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditAction {
    Insert,
    Update,
    Delete,
}

impl std::fmt::Display for AuditAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditAction::Insert => write!(f, "INSERT"),
            AuditAction::Update => write!(f, "UPDATE"),
            AuditAction::Delete => write!(f, "DELETE"),
        }
    }
}

// Attendance models
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = attendances)]
pub struct Attendance {
    pub id: i32,
    pub employee_id: i32,
    pub date: NaiveDate,
    pub check_in: Option<NaiveTime>,
    pub check_out: Option<NaiveTime>,
    pub break_time: Option<i32>, // in minutes
    pub overtime_hours: Option<f32>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = attendances)]
pub struct NewAttendance {
    pub employee_id: i32,
    pub date: NaiveDate,
    pub check_in: Option<NaiveTime>,
    pub check_out: Option<NaiveTime>,
    pub break_time: Option<i32>,
    pub overtime_hours: Option<f32>,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttendanceStatus {
    Present,
    Absent,
    Late,
    EarlyLeave,
    Holiday,
}

impl std::fmt::Display for AttendanceStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttendanceStatus::Present => write!(f, "present"),
            AttendanceStatus::Absent => write!(f, "absent"),
            AttendanceStatus::Late => write!(f, "late"),
            AttendanceStatus::EarlyLeave => write!(f, "early_leave"),
            AttendanceStatus::Holiday => write!(f, "holiday"),
        }
    }
}

// Payroll models
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = payrolls)]
pub struct Payroll {
    pub id: i32,
    pub employee_id: i32,
    pub period: String, // YYYY-MM format
    pub base_salary: i32,
    pub overtime_pay: Option<i32>,
    pub bonuses: Option<i32>,
    pub deductions: Option<i32>,
    pub net_salary: i32,
    pub payment_date: Option<NaiveDate>,
    pub status: String,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = payrolls)]
pub struct NewPayroll {
    pub employee_id: i32,
    pub period: String,
    pub base_salary: i32,
    pub overtime_pay: Option<i32>,
    pub bonuses: Option<i32>,
    pub deductions: Option<i32>,
    pub net_salary: i32,
    pub payment_date: Option<NaiveDate>,
    pub status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PayrollStatus {
    Pending,
    Processed,
    Paid,
}

impl std::fmt::Display for PayrollStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PayrollStatus::Pending => write!(f, "pending"),
            PayrollStatus::Processed => write!(f, "processed"),
            PayrollStatus::Paid => write!(f, "paid"),
        }
    }
}

// Account models for finance
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = accounts)]
pub struct Account {
    pub id: i32,
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub parent_id: Option<i32>,
    pub balance: i32,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = accounts)]
pub struct NewAccount {
    pub account_code: String,
    pub account_name: String,
    pub account_type: String,
    pub parent_id: Option<i32>,
    pub balance: i32,
    pub is_active: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AccountType {
    Asset,
    Liability,
    Equity,
    Revenue,
    Expense,
}

impl std::fmt::Display for AccountType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AccountType::Asset => write!(f, "asset"),
            AccountType::Liability => write!(f, "liability"),
            AccountType::Equity => write!(f, "equity"),
            AccountType::Revenue => write!(f, "revenue"),
            AccountType::Expense => write!(f, "expense"),
        }
    }
}

// Transaction models for finance
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = transactions)]
pub struct Transaction {
    pub id: i32,
    pub account_id: i32,
    pub transaction_date: NaiveDate,
    pub amount: i32,
    pub debit_credit: String,
    pub description: String,
    pub reference: Option<String>,
    pub created_by: Option<i32>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = transactions)]
pub struct NewTransaction {
    pub account_id: i32,
    pub transaction_date: NaiveDate,
    pub amount: i32,
    pub debit_credit: String,
    pub description: String,
    pub reference: Option<String>,
    pub created_by: Option<i32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Debit,
    Credit,
}

impl std::fmt::Display for TransactionType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TransactionType::Debit => write!(f, "debit"),
            TransactionType::Credit => write!(f, "credit"),
        }
    }
}

// Category models for inventory
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = categories)]
pub struct Category {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = categories)]
pub struct NewCategory {
    pub name: String,
    pub description: Option<String>,
    pub parent_id: Option<i32>,
    pub is_active: bool,
}

// Product models for inventory
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = products)]
pub struct Product {
    pub id: i32,
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: i32,
    pub price: i32,
    pub cost_price: i32,
    pub current_stock: i32,
    pub min_stock_level: i32,
    pub max_stock_level: Option<i32>,
    pub unit: String,
    pub barcode: Option<String>,
    pub is_active: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = products)]
pub struct NewProduct {
    pub sku: String,
    pub name: String,
    pub description: Option<String>,
    pub category_id: i32,
    pub price: i32,
    pub cost_price: i32,
    pub current_stock: i32,
    pub min_stock_level: i32,
    pub max_stock_level: Option<i32>,
    pub unit: String,
    pub barcode: Option<String>,
    pub is_active: bool,
}

// Stock movement models for inventory tracking
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = stock_movements)]
pub struct StockMovement {
    pub id: i32,
    pub product_id: i32,
    pub movement_type: String,
    pub quantity: i32,
    pub unit_cost: Option<i32>,
    pub reference_type: Option<String>,
    pub reference_id: Option<i32>,
    pub notes: Option<String>,
    pub moved_by: Option<i32>,
    pub movement_date: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = stock_movements)]
pub struct NewStockMovement {
    pub product_id: i32,
    pub movement_type: String,
    pub quantity: i32,
    pub unit_cost: Option<i32>,
    pub reference_type: Option<String>,
    pub reference_id: Option<i32>,
    pub notes: Option<String>,
    pub moved_by: Option<i32>,
}

// Enums for inventory management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StockMovementType {
    In,
    Out,
    Adjustment,
}

impl std::fmt::Display for StockMovementType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StockMovementType::In => write!(f, "in"),
            StockMovementType::Out => write!(f, "out"),
            StockMovementType::Adjustment => write!(f, "adjustment"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProductUnit {
    Each,
    Kilogram,
    Liter,
    Meter,
    Piece,
    Box,
    Pack,
}

impl std::fmt::Display for ProductUnit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductUnit::Each => write!(f, "ea"),
            ProductUnit::Kilogram => write!(f, "kg"),
            ProductUnit::Liter => write!(f, "l"),
            ProductUnit::Meter => write!(f, "m"),
            ProductUnit::Piece => write!(f, "pc"),
            ProductUnit::Box => write!(f, "box"),
            ProductUnit::Pack => write!(f, "pack"),
        }
    }
}

// Product attachment models
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = product_attachments)]
pub struct ProductAttachment {
    pub id: i32,
    pub product_id: i32,
    pub attachment_type: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i32,
    pub mime_type: Option<String>,
    pub is_primary: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = product_attachments)]
pub struct NewProductAttachment {
    pub product_id: i32,
    pub attachment_type: String,
    pub file_name: String,
    pub file_path: String,
    pub file_size: i32,
    pub mime_type: Option<String>,
    pub is_primary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AttachmentType {
    Image,
    Document,
    Manual,
    Certificate,
}

impl std::fmt::Display for AttachmentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AttachmentType::Image => write!(f, "image"),
            AttachmentType::Document => write!(f, "document"),
            AttachmentType::Manual => write!(f, "manual"),
            AttachmentType::Certificate => write!(f, "certificate"),
        }
    }
}

// Stock audit models
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = stock_audits)]
pub struct StockAudit {
    pub id: i32,
    pub audit_name: String,
    pub audit_date: NaiveDate,
    pub status: String,
    pub conducted_by: Option<i32>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = stock_audits)]
pub struct NewStockAudit {
    pub audit_name: String,
    pub audit_date: NaiveDate,
    pub status: String,
    pub conducted_by: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = stock_audit_items)]
pub struct StockAuditItem {
    pub id: i32,
    pub audit_id: i32,
    pub product_id: i32,
    pub expected_quantity: i32,
    pub actual_quantity: Option<i32>,
    pub variance: Option<i32>,
    pub notes: Option<String>,
    pub audited_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = stock_audit_items)]
pub struct NewStockAuditItem {
    pub audit_id: i32,
    pub product_id: i32,
    pub expected_quantity: i32,
    pub actual_quantity: Option<i32>,
    pub variance: Option<i32>,
    pub notes: Option<String>,
    pub audited_at: Option<NaiveDateTime>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AuditStatus {
    Pending,
    InProgress,
    Completed,
    Cancelled,
}

impl std::fmt::Display for AuditStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AuditStatus::Pending => write!(f, "pending"),
            AuditStatus::InProgress => write!(f, "in_progress"),
            AuditStatus::Completed => write!(f, "completed"),
            AuditStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}
