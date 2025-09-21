use crate::core::error::CLIERPError;
use crate::core::result::CLIERPResult;
use crate::database::{
    connection::DatabaseConnection,
    models::{Department, Employee, NewEmployee},
    schema::{departments, employees},
};
use chrono::{NaiveDate, Utc};
use diesel::prelude::*;

#[derive(Debug)]
pub struct CreateEmployeeRequest {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department_id: i32,
    pub position: String,
    pub hire_date: NaiveDate,
    pub salary: i32,
}

#[derive(Debug)]
pub struct UpdateEmployeeRequest {
    pub id: i32,
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department_id: Option<i32>,
    pub position: Option<String>,
    pub salary: Option<i32>,
    pub status: Option<String>,
}

#[derive(Default)]
pub struct EmployeeService;

impl EmployeeService {
    pub fn new() -> Self {
        Self
    }

    /// Generate next employee code
    fn generate_employee_code(&self, conn: &mut DatabaseConnection) -> CLIERPResult<String> {
        use crate::database::schema::employees::dsl::*;

        let last_emp = employees
            .order(id.desc())
            .first::<Employee>(conn)
            .optional()?;

        let next_id = match last_emp {
            Some(emp) => emp.id + 1,
            None => 1,
        };

        Ok(format!("EMP{:06}", next_id))
    }

    /// Create a new employee
    pub fn create_employee(
        &self,
        conn: &mut DatabaseConnection,
        request: CreateEmployeeRequest,
    ) -> CLIERPResult<Employee> {
        use crate::database::schema::employees::dsl::*;

        // Validate department exists
        let _dept = departments::table
            .filter(departments::id.eq(request.department_id))
            .first::<Department>(conn)
            .optional()?
            .ok_or_else(|| {
                CLIERPError::ValidationError(format!(
                    "Department with ID {} not found",
                    request.department_id
                ))
            })?;

        // Check if email already exists (if provided)
        if let Some(ref email_val) = request.email {
            let existing = employees
                .filter(email.eq(email_val))
                .first::<Employee>(conn)
                .optional()?;

            if existing.is_some() {
                return Err(CLIERPError::ValidationError(format!(
                    "Employee with email '{}' already exists",
                    email_val
                )));
            }
        }

        // Generate employee code
        let employee_code_val = self.generate_employee_code(conn)?;

        let new_emp = NewEmployee {
            employee_code: employee_code_val.clone(),
            name: request.name,
            email: request.email,
            phone: request.phone,
            department_id: request.department_id,
            position: request.position,
            hire_date: request.hire_date,
            salary: request.salary,
            status: "active".to_string(),
        };

        diesel::insert_into(employees)
            .values(&new_emp)
            .execute(conn)?;

        // Get the inserted employee by employee code
        let employee = employees
            .filter(employee_code.eq(&employee_code_val))
            .first::<Employee>(conn)?;

        Ok(employee)
    }

    /// List all employees
    pub fn list_employees(
        &self,
        conn: &mut DatabaseConnection,
    ) -> CLIERPResult<Vec<EmployeeWithDepartment>> {
        use crate::database::schema::employees::dsl::*;

        let emp_list = employees
            .inner_join(departments::table)
            .select((Employee::as_select(), Department::as_select()))
            .order(name.asc())
            .load::<(Employee, Department)>(conn)?;

        let result = emp_list
            .into_iter()
            .map(|(emp, dept)| EmployeeWithDepartment {
                employee: emp,
                department: dept,
            })
            .collect();

        Ok(result)
    }

    /// List employees by department
    pub fn list_employees_by_department(
        &self,
        conn: &mut DatabaseConnection,
        dept_id: i32,
    ) -> CLIERPResult<Vec<EmployeeWithDepartment>> {
        use crate::database::schema::employees::dsl::*;

        let emp_list = employees
            .inner_join(departments::table)
            .filter(department_id.eq(dept_id))
            .select((Employee::as_select(), Department::as_select()))
            .order(name.asc())
            .load::<(Employee, Department)>(conn)?;

        let result = emp_list
            .into_iter()
            .map(|(emp, dept)| EmployeeWithDepartment {
                employee: emp,
                department: dept,
            })
            .collect();

        Ok(result)
    }

    /// Get employee by ID
    pub fn get_employee_by_id(
        &self,
        conn: &mut DatabaseConnection,
        emp_id: i32,
    ) -> CLIERPResult<Option<EmployeeWithDepartment>> {
        use crate::database::schema::employees::dsl::*;

        let result = employees
            .inner_join(departments::table)
            .filter(id.eq(emp_id))
            .select((Employee::as_select(), Department::as_select()))
            .first::<(Employee, Department)>(conn)
            .optional()?;

        match result {
            Some((emp, dept)) => Ok(Some(EmployeeWithDepartment {
                employee: emp,
                department: dept,
            })),
            None => Ok(None),
        }
    }

    /// Get employee by code
    pub fn get_employee_by_code(
        &self,
        conn: &mut DatabaseConnection,
        emp_code: &str,
    ) -> CLIERPResult<Option<EmployeeWithDepartment>> {
        use crate::database::schema::employees::dsl::*;

        let result = employees
            .inner_join(departments::table)
            .filter(employee_code.eq(emp_code))
            .select((Employee::as_select(), Department::as_select()))
            .first::<(Employee, Department)>(conn)
            .optional()?;

        match result {
            Some((emp, dept)) => Ok(Some(EmployeeWithDepartment {
                employee: emp,
                department: dept,
            })),
            None => Ok(None),
        }
    }

    /// Search employees by name or email
    pub fn search_employees(
        &self,
        conn: &mut DatabaseConnection,
        query: &str,
    ) -> CLIERPResult<Vec<EmployeeWithDepartment>> {
        use crate::database::schema::employees::dsl::*;

        let search_pattern = format!("%{}%", query);

        let emp_list = employees
            .inner_join(departments::table)
            .filter(
                name.like(&search_pattern)
                    .or(email.like(&search_pattern))
                    .or(employee_code.like(&search_pattern)),
            )
            .select((Employee::as_select(), Department::as_select()))
            .order(name.asc())
            .load::<(Employee, Department)>(conn)?;

        let result = emp_list
            .into_iter()
            .map(|(emp, dept)| EmployeeWithDepartment {
                employee: emp,
                department: dept,
            })
            .collect();

        Ok(result)
    }

    /// Update employee
    pub fn update_employee(
        &self,
        conn: &mut DatabaseConnection,
        request: UpdateEmployeeRequest,
    ) -> CLIERPResult<Employee> {
        use crate::database::schema::employees::dsl::*;

        // Check if employee exists
        let emp = self.get_employee_by_id(conn, request.id)?.ok_or_else(|| {
            CLIERPError::NotFound(format!("Employee with ID {} not found", request.id))
        })?;

        // Validate department if changing
        if let Some(dept_id) = request.department_id {
            let _dept = departments::table
                .filter(departments::id.eq(dept_id))
                .first::<Department>(conn)
                .optional()?
                .ok_or_else(|| {
                    CLIERPError::ValidationError(format!(
                        "Department with ID {} not found",
                        dept_id
                    ))
                })?;
        }

        // Check email conflicts if updating
        if let Some(ref new_email_val) = request.email {
            if Some(new_email_val) != emp.employee.email.as_ref() {
                let existing = employees
                    .filter(email.eq(new_email_val))
                    .filter(id.ne(request.id))
                    .first::<Employee>(conn)
                    .optional()?;

                if existing.is_some() {
                    return Err(CLIERPError::ValidationError(format!(
                        "Employee with email '{}' already exists",
                        new_email_val
                    )));
                }
            }
        }

        let mut changeset = EmployeeChangeset::default();

        if let Some(new_name_val) = request.name {
            changeset.name = Some(new_name_val);
        }
        if request.email.is_some() {
            changeset.email = request.email;
        }
        if request.phone.is_some() {
            changeset.phone = request.phone;
        }
        if let Some(new_dept_id) = request.department_id {
            changeset.department_id = Some(new_dept_id);
        }
        if let Some(new_pos) = request.position {
            changeset.position = Some(new_pos);
        }
        if let Some(new_sal) = request.salary {
            changeset.salary = Some(new_sal);
        }
        if let Some(new_st) = request.status {
            changeset.status = Some(new_st);
        }
        changeset.updated_at = Some(Utc::now().naive_utc());

        diesel::update(employees.filter(id.eq(request.id)))
            .set(&changeset)
            .execute(conn)?;

        // Get the updated employee
        let updated_emp = employees
            .filter(id.eq(request.id))
            .first::<Employee>(conn)?;

        Ok(updated_emp)
    }

    /// Delete employee (soft delete by setting status to terminated)
    pub fn delete_employee(&self, conn: &mut DatabaseConnection, emp_id: i32) -> CLIERPResult<()> {
        use crate::database::schema::employees::dsl::*;

        // Check if employee exists
        let _emp = self.get_employee_by_id(conn, emp_id)?.ok_or_else(|| {
            CLIERPError::NotFound(format!("Employee with ID {} not found", emp_id))
        })?;

        // Soft delete - set status to terminated
        diesel::update(employees.filter(id.eq(emp_id)))
            .set((
                status.eq("terminated"),
                updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(conn)?;

        Ok(())
    }

    /// Get employee count by status
    pub fn get_employee_count_by_status(
        &self,
        conn: &mut DatabaseConnection,
    ) -> CLIERPResult<EmployeeStatusCount> {
        use crate::database::schema::employees::dsl::*;

        let active_count = employees
            .filter(status.eq("active"))
            .count()
            .get_result::<i64>(conn)?;

        let inactive_count = employees
            .filter(status.eq("inactive"))
            .count()
            .get_result::<i64>(conn)?;

        let terminated_count = employees
            .filter(status.eq("terminated"))
            .count()
            .get_result::<i64>(conn)?;

        Ok(EmployeeStatusCount {
            active: active_count,
            inactive: inactive_count,
            terminated: terminated_count,
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct EmployeeWithDepartment {
    pub employee: Employee,
    pub department: Department,
}

impl crate::utils::export::CsvSerializable for EmployeeWithDepartment {
    fn to_csv_row(&self) -> Vec<String> {
        use crate::utils::export::escape_csv_value;

        vec![
            self.employee.id.to_string(),
            escape_csv_value(&self.employee.employee_code),
            escape_csv_value(&self.employee.name),
            escape_csv_value(&self.employee.email.clone().unwrap_or_default()),
            escape_csv_value(&self.employee.phone.clone().unwrap_or_default()),
            escape_csv_value(&self.department.name),
            escape_csv_value(&self.employee.position),
            self.employee.salary.to_string(),
            escape_csv_value(&self.employee.status),
            self.employee.hire_date.format("%Y-%m-%d").to_string(),
            self.employee
                .created_at
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            self.employee
                .updated_at
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        ]
    }
}

#[derive(Debug)]
pub struct EmployeeStatusCount {
    pub active: i64,
    pub inactive: i64,
    pub terminated: i64,
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = employees)]
struct EmployeeChangeset {
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department_id: Option<i32>,
    pub position: Option<String>,
    pub salary: Option<i32>,
    pub status: Option<String>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}
