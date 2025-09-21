use crate::core::error::CLIERPError;
use crate::core::result::CLIERPResult;
use crate::database::{
    connection::DatabaseConnection,
    models::{Department, NewDepartment},
    schema::departments,
};
use chrono::Utc;
use diesel::prelude::*;

#[derive(Default)]
pub struct DepartmentService;

impl DepartmentService {
    pub fn new() -> Self {
        Self
    }

    /// Create a new department
    pub fn create_department(
        &self,
        conn: &mut DatabaseConnection,
        dept_name: String,
        dept_description: Option<String>,
        dept_manager_id: Option<i32>,
    ) -> CLIERPResult<Department> {
        use crate::database::schema::departments::dsl::*;

        // Check if department name already exists
        let existing = departments
            .filter(name.eq(&dept_name))
            .first::<Department>(conn)
            .optional()?;

        if existing.is_some() {
            return Err(CLIERPError::ValidationError(format!(
                "Department '{}' already exists",
                dept_name
            )));
        }

        let new_dept = NewDepartment {
            name: dept_name,
            description: dept_description,
            manager_id: dept_manager_id,
        };

        diesel::insert_into(departments)
            .values(&new_dept)
            .execute(conn)?;

        // Get the inserted department by name
        let department = departments
            .filter(name.eq(&new_dept.name))
            .first::<Department>(conn)?;

        Ok(department)
    }

    /// List all departments
    pub fn list_departments(&self, conn: &mut DatabaseConnection) -> CLIERPResult<Vec<Department>> {
        use crate::database::schema::departments::dsl::*;

        let dept_list = departments.order(name.asc()).load::<Department>(conn)?;

        Ok(dept_list)
    }

    /// Get department by ID
    pub fn get_department_by_id(
        &self,
        conn: &mut DatabaseConnection,
        dept_id: i32,
    ) -> CLIERPResult<Option<Department>> {
        use crate::database::schema::departments::dsl::*;

        let department = departments
            .filter(id.eq(dept_id))
            .first::<Department>(conn)
            .optional()?;

        Ok(department)
    }

    /// Get department by name
    pub fn get_department_by_name(
        &self,
        conn: &mut DatabaseConnection,
        dept_name: &str,
    ) -> CLIERPResult<Option<Department>> {
        use crate::database::schema::departments::dsl::*;

        let department = departments
            .filter(name.eq(dept_name))
            .first::<Department>(conn)
            .optional()?;

        Ok(department)
    }

    /// Update department
    pub fn update_department(
        &self,
        conn: &mut DatabaseConnection,
        dept_id: i32,
        new_name: Option<String>,
        new_description: Option<String>,
        new_manager_id: Option<i32>,
    ) -> CLIERPResult<Department> {
        use crate::database::schema::departments::dsl::*;

        // Check if department exists
        let dept = self.get_department_by_id(conn, dept_id)?.ok_or_else(|| {
            CLIERPError::NotFound(format!("Department with ID {} not found", dept_id))
        })?;

        // If updating name, check for conflicts
        if let Some(ref new_name_val) = new_name {
            if new_name_val != &dept.name {
                let existing = departments
                    .filter(name.eq(new_name_val))
                    .filter(id.ne(dept_id))
                    .first::<Department>(conn)
                    .optional()?;

                if existing.is_some() {
                    return Err(CLIERPError::ValidationError(format!(
                        "Department '{}' already exists",
                        new_name_val
                    )));
                }
            }
        }

        let mut changeset = DepartmentChangeset::default();

        if let Some(new_name_val) = new_name {
            changeset.name = Some(new_name_val);
        }
        if new_description.is_some() {
            changeset.description = new_description;
        }
        if new_manager_id.is_some() {
            changeset.manager_id = new_manager_id;
        }
        changeset.updated_at = Some(Utc::now().naive_utc());

        diesel::update(departments.filter(id.eq(dept_id)))
            .set(&changeset)
            .execute(conn)?;

        // Get the updated department
        let updated_dept = departments
            .filter(id.eq(dept_id))
            .first::<Department>(conn)?;

        Ok(updated_dept)
    }

    /// Delete department
    pub fn delete_department(
        &self,
        conn: &mut DatabaseConnection,
        dept_id: i32,
    ) -> CLIERPResult<()> {
        use crate::database::schema::departments::dsl::*;
        use crate::database::schema::employees::dsl as emp_dsl;

        // Check if department exists
        let _dept = self.get_department_by_id(conn, dept_id)?.ok_or_else(|| {
            CLIERPError::NotFound(format!("Department with ID {} not found", dept_id))
        })?;

        // Check if there are employees in this department
        let employee_count = emp_dsl::employees
            .filter(emp_dsl::department_id.eq(dept_id))
            .count()
            .get_result::<i64>(conn)?;

        if employee_count > 0 {
            return Err(CLIERPError::ValidationError(format!(
                "Cannot delete department: {} employees are still assigned to it",
                employee_count
            )));
        }

        diesel::delete(departments.filter(id.eq(dept_id))).execute(conn)?;

        Ok(())
    }

    /// Get departments with employee count
    pub fn list_departments_with_employee_count(
        &self,
        conn: &mut DatabaseConnection,
    ) -> CLIERPResult<Vec<DepartmentWithEmployeeCount>> {
        use crate::database::schema::departments::dsl::*;
        use crate::database::schema::employees::dsl as emp_dsl;

        let dept_list = departments
            .left_join(emp_dsl::employees)
            .group_by((id, name, description, manager_id, created_at, updated_at))
            .select((
                (id, name, description, manager_id, created_at, updated_at),
                diesel::dsl::count(emp_dsl::id).nullable(),
            ))
            .load::<(
                (
                    i32,
                    String,
                    Option<String>,
                    Option<i32>,
                    chrono::NaiveDateTime,
                    chrono::NaiveDateTime,
                ),
                Option<i64>,
            )>(conn)?;

        let result = dept_list
            .into_iter()
            .map(
                |((dept_id, dept_name, dept_desc, mgr_id, created, updated), emp_count)| {
                    DepartmentWithEmployeeCount {
                        department: Department {
                            id: dept_id,
                            name: dept_name,
                            description: dept_desc,
                            manager_id: mgr_id,
                            created_at: created,
                            updated_at: updated,
                        },
                        employee_count: emp_count.unwrap_or(0),
                    }
                },
            )
            .collect();

        Ok(result)
    }
}

#[derive(Debug, serde::Serialize)]
pub struct DepartmentWithEmployeeCount {
    pub department: Department,
    pub employee_count: i64,
}

impl crate::utils::export::CsvSerializable for DepartmentWithEmployeeCount {
    fn to_csv_row(&self) -> Vec<String> {
        use crate::utils::export::escape_csv_value;

        vec![
            self.department.id.to_string(),
            escape_csv_value(&self.department.name),
            escape_csv_value(&self.department.description.clone().unwrap_or_default()),
            self.department
                .manager_id
                .map(|id| id.to_string())
                .unwrap_or_default(),
            self.employee_count.to_string(),
            self.department
                .created_at
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
            self.department
                .updated_at
                .format("%Y-%m-%d %H:%M:%S")
                .to_string(),
        ]
    }
}

#[derive(AsChangeset, Default)]
#[diesel(table_name = departments)]
struct DepartmentChangeset {
    pub name: Option<String>,
    pub description: Option<String>,
    pub manager_id: Option<i32>,
    pub updated_at: Option<chrono::NaiveDateTime>,
}
