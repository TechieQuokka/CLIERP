use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use chrono::{NaiveDateTime, NaiveDate};

use super::schema::{departments, employees, users, audit_logs};

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