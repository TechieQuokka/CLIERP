use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{NaiveDateTime, NaiveDate, NaiveTime};

use super::schema::{departments, employees, users, audit_logs, attendances, payrolls, accounts, transactions};

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
    pub break_time: i32, // in minutes
    pub overtime_hours: f32,
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
    pub break_time: i32,
    pub overtime_hours: f32,
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
    pub overtime_pay: i32,
    pub bonuses: i32,
    pub deductions: i32,
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
    pub overtime_pay: i32,
    pub bonuses: i32,
    pub deductions: i32,
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