use crate::core::{auth::AuthenticatedUser, command::Command, result::CLIERPResult};
use crate::database::connection::DatabaseManager;
use crate::modules::hr::department::{DepartmentService, DepartmentWithEmployeeCount};
use crate::utils::formatting::format_table;
use chrono::NaiveDate;

// Department Commands

pub struct HrDeptListCommand;

impl Default for HrDeptListCommand {
    fn default() -> Self {
        Self::new()
    }
}

impl HrDeptListCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for HrDeptListCommand {
    fn execute(
        &self,
        _args: &dyn std::any::Any,
        user: Option<&AuthenticatedUser>,
    ) -> CLIERPResult<()> {
        let _user = user.ok_or_else(|| crate::core::error::CLIERPError::AuthenticationRequired)?;

        let db_manager = DatabaseManager::new()?;
        let mut conn = db_manager.get_connection()?;
        let dept_service = DepartmentService::new();

        let departments = dept_service.list_departments_with_employee_count(&mut conn)?;

        if departments.is_empty() {
            println!("No departments found.");
            return Ok(());
        }

        display_departments_table(&departments);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "hr-dept-list"
    }

    fn description(&self) -> &'static str {
        "List all departments with employee count"
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

pub struct HrDeptAddCommand {
    pub name: String,
    pub description: Option<String>,
    pub manager_id: Option<i32>,
}

impl HrDeptAddCommand {
    pub fn new(name: String, description: Option<String>, manager_id: Option<i32>) -> Self {
        Self {
            name,
            description,
            manager_id,
        }
    }
}

impl Command for HrDeptAddCommand {
    fn execute(
        &self,
        _args: &dyn std::any::Any,
        user: Option<&AuthenticatedUser>,
    ) -> CLIERPResult<()> {
        let _user = user.ok_or_else(|| crate::core::error::CLIERPError::AuthenticationRequired)?;

        let db_manager = DatabaseManager::new()?;
        let mut conn = db_manager.get_connection()?;
        let dept_service = DepartmentService::new();

        let department = dept_service.create_department(
            &mut conn,
            self.name.clone(),
            self.description.clone(),
            self.manager_id,
        )?;

        println!("✅ Department created successfully!");
        println!("ID: {}", department.id);
        println!("Name: {}", department.name);
        if let Some(desc) = &department.description {
            println!("Description: {}", desc);
        }
        println!(
            "Created: {}",
            department.created_at.format("%Y-%m-%d %H:%M:%S")
        );

        Ok(())
    }

    fn name(&self) -> &'static str {
        "hr-dept-add"
    }

    fn description(&self) -> &'static str {
        "Create a new department"
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

pub struct HrDeptUpdateCommand {
    pub id: i32,
    pub name: Option<String>,
    pub description: Option<String>,
    pub manager_id: Option<i32>,
}

impl HrDeptUpdateCommand {
    pub fn new(
        id: i32,
        name: Option<String>,
        description: Option<String>,
        manager_id: Option<i32>,
    ) -> Self {
        Self {
            id,
            name,
            description,
            manager_id,
        }
    }
}

impl Command for HrDeptUpdateCommand {
    fn execute(
        &self,
        _args: &dyn std::any::Any,
        user: Option<&AuthenticatedUser>,
    ) -> CLIERPResult<()> {
        let _user = user.ok_or_else(|| crate::core::error::CLIERPError::AuthenticationRequired)?;

        let db_manager = DatabaseManager::new()?;
        let mut conn = db_manager.get_connection()?;
        let dept_service = DepartmentService::new();

        let department = dept_service.update_department(
            &mut conn,
            self.id,
            self.name.clone(),
            self.description.clone(),
            self.manager_id,
        )?;

        println!("✅ Department updated successfully!");
        println!("ID: {}", department.id);
        println!("Name: {}", department.name);
        if let Some(desc) = &department.description {
            println!("Description: {}", desc);
        }
        println!(
            "Updated: {}",
            department.updated_at.format("%Y-%m-%d %H:%M:%S")
        );

        Ok(())
    }

    fn name(&self) -> &'static str {
        "hr-dept-update"
    }

    fn description(&self) -> &'static str {
        "Update an existing department"
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

pub struct HrDeptDeleteCommand {
    pub id: i32,
}

impl HrDeptDeleteCommand {
    pub fn new(id: i32) -> Self {
        Self { id }
    }
}

impl Command for HrDeptDeleteCommand {
    fn execute(
        &self,
        _args: &dyn std::any::Any,
        user: Option<&AuthenticatedUser>,
    ) -> CLIERPResult<()> {
        let _user = user.ok_or_else(|| crate::core::error::CLIERPError::AuthenticationRequired)?;

        let db_manager = DatabaseManager::new()?;
        let mut conn = db_manager.get_connection()?;
        let dept_service = DepartmentService::new();

        dept_service.delete_department(&mut conn, self.id)?;

        println!("✅ Department deleted successfully!");

        Ok(())
    }

    fn name(&self) -> &'static str {
        "hr-dept-delete"
    }

    fn description(&self) -> &'static str {
        "Delete a department"
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

// Employee Commands

pub struct HrEmployeeListCommand {
    pub department_id: Option<i32>,
}

impl HrEmployeeListCommand {
    pub fn new(department_id: Option<i32>) -> Self {
        Self { department_id }
    }
}

impl Command for HrEmployeeListCommand {
    fn execute(
        &self,
        _args: &dyn std::any::Any,
        user: Option<&AuthenticatedUser>,
    ) -> CLIERPResult<()> {
        use crate::modules::hr::employee::EmployeeService;

        let _user = user.ok_or_else(|| crate::core::error::CLIERPError::AuthenticationRequired)?;

        let db_manager = DatabaseManager::new()?;
        let mut conn = db_manager.get_connection()?;
        let emp_service = EmployeeService::new();

        let employees = match self.department_id {
            Some(dept_id) => emp_service.list_employees_by_department(&mut conn, dept_id)?,
            None => emp_service.list_employees(&mut conn)?,
        };

        if employees.is_empty() {
            println!("No employees found.");
            return Ok(());
        }

        display_employees_table(&employees);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "hr-employee-list"
    }

    fn description(&self) -> &'static str {
        "List all employees"
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

pub struct HrEmployeeAddCommand {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department_id: i32,
    pub position: String,
    pub hire_date: NaiveDate,
    pub salary: i32,
}

impl HrEmployeeAddCommand {
    pub fn new(
        name: String,
        email: Option<String>,
        phone: Option<String>,
        department_id: i32,
        position: String,
        hire_date: NaiveDate,
        salary: i32,
    ) -> Self {
        Self {
            name,
            email,
            phone,
            department_id,
            position,
            hire_date,
            salary,
        }
    }
}

impl Command for HrEmployeeAddCommand {
    fn execute(
        &self,
        _args: &dyn std::any::Any,
        user: Option<&AuthenticatedUser>,
    ) -> CLIERPResult<()> {
        use crate::modules::hr::employee::EmployeeService;

        let _user = user.ok_or_else(|| crate::core::error::CLIERPError::AuthenticationRequired)?;

        let db_manager = DatabaseManager::new()?;
        let mut conn = db_manager.get_connection()?;
        let emp_service = EmployeeService::new();

        let request = crate::modules::hr::employee::CreateEmployeeRequest {
            name: self.name.clone(),
            email: self.email.clone(),
            phone: self.phone.clone(),
            department_id: self.department_id,
            position: self.position.clone(),
            hire_date: self.hire_date,
            salary: self.salary,
        };

        let employee = emp_service.create_employee(&mut conn, request)?;

        println!("✅ Employee created successfully!");
        println!("ID: {}", employee.id);
        println!("Code: {}", employee.employee_code);
        println!("Name: {}", employee.name);
        if let Some(email) = &employee.email {
            println!("Email: {}", email);
        }
        println!("Position: {}", employee.position);
        println!(
            "Salary: {}",
            crate::utils::formatting::format_currency(employee.salary)
        );
        println!("Hire Date: {}", employee.hire_date);
        println!(
            "Created: {}",
            employee.created_at.format("%Y-%m-%d %H:%M:%S")
        );

        Ok(())
    }

    fn name(&self) -> &'static str {
        "hr-employee-add"
    }

    fn description(&self) -> &'static str {
        "Create a new employee"
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

pub struct HrEmployeeShowCommand {
    pub id: Option<i32>,
    pub code: Option<String>,
}

impl HrEmployeeShowCommand {
    pub fn new(id: Option<i32>, code: Option<String>) -> Self {
        Self { id, code }
    }
}

impl Command for HrEmployeeShowCommand {
    fn execute(
        &self,
        _args: &dyn std::any::Any,
        user: Option<&AuthenticatedUser>,
    ) -> CLIERPResult<()> {
        use crate::modules::hr::employee::EmployeeService;

        let _user = user.ok_or_else(|| crate::core::error::CLIERPError::AuthenticationRequired)?;

        let db_manager = DatabaseManager::new()?;
        let mut conn = db_manager.get_connection()?;
        let emp_service = EmployeeService::new();

        let employee = if let Some(emp_id) = self.id {
            emp_service.get_employee_by_id(&mut conn, emp_id)?
        } else if let Some(ref emp_code) = self.code {
            emp_service.get_employee_by_code(&mut conn, emp_code)?
        } else {
            return Err(crate::core::error::CLIERPError::ValidationError(
                "Either --id or --code must be provided".to_string(),
            ));
        };

        match employee {
            Some(emp_with_dept) => {
                display_employee_detail(&emp_with_dept);
            }
            None => {
                println!("Employee not found.");
            }
        }

        Ok(())
    }

    fn name(&self) -> &'static str {
        "hr-employee-show"
    }

    fn description(&self) -> &'static str {
        "Show detailed employee information"
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

pub struct HrEmployeeUpdateCommand {
    pub id: i32,
    pub name: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub department_id: Option<i32>,
    pub position: Option<String>,
    pub salary: Option<i32>,
    pub status: Option<String>,
}

impl HrEmployeeUpdateCommand {
    pub fn new(
        id: i32,
        name: Option<String>,
        email: Option<String>,
        phone: Option<String>,
        department_id: Option<i32>,
        position: Option<String>,
        salary: Option<i32>,
        status: Option<String>,
    ) -> Self {
        Self {
            id,
            name,
            email,
            phone,
            department_id,
            position,
            salary,
            status,
        }
    }
}

impl Command for HrEmployeeUpdateCommand {
    fn execute(
        &self,
        _args: &dyn std::any::Any,
        user: Option<&AuthenticatedUser>,
    ) -> CLIERPResult<()> {
        use crate::modules::hr::employee::EmployeeService;

        let _user = user.ok_or_else(|| crate::core::error::CLIERPError::AuthenticationRequired)?;

        let db_manager = DatabaseManager::new()?;
        let mut conn = db_manager.get_connection()?;
        let emp_service = EmployeeService::new();

        let request = crate::modules::hr::employee::UpdateEmployeeRequest {
            id: self.id,
            name: self.name.clone(),
            email: self.email.clone(),
            phone: self.phone.clone(),
            department_id: self.department_id,
            position: self.position.clone(),
            salary: self.salary,
            status: self.status.clone(),
        };

        let employee = emp_service.update_employee(&mut conn, request)?;

        println!("✅ Employee updated successfully!");
        println!("ID: {}", employee.id);
        println!("Code: {}", employee.employee_code);
        println!("Name: {}", employee.name);
        if let Some(email) = &employee.email {
            println!("Email: {}", email);
        }
        println!("Position: {}", employee.position);
        println!("Status: {}", employee.status);
        println!(
            "Updated: {}",
            employee.updated_at.format("%Y-%m-%d %H:%M:%S")
        );

        Ok(())
    }

    fn name(&self) -> &'static str {
        "hr-employee-update"
    }

    fn description(&self) -> &'static str {
        "Update an existing employee"
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

pub struct HrEmployeeSearchCommand {
    pub query: String,
}

impl HrEmployeeSearchCommand {
    pub fn new(query: String) -> Self {
        Self { query }
    }
}

impl Command for HrEmployeeSearchCommand {
    fn execute(
        &self,
        _args: &dyn std::any::Any,
        user: Option<&AuthenticatedUser>,
    ) -> CLIERPResult<()> {
        use crate::modules::hr::employee::EmployeeService;

        let _user = user.ok_or_else(|| crate::core::error::CLIERPError::AuthenticationRequired)?;

        let db_manager = DatabaseManager::new()?;
        let mut conn = db_manager.get_connection()?;
        let emp_service = EmployeeService::new();

        let employees = emp_service.search_employees(&mut conn, &self.query)?;

        if employees.is_empty() {
            println!("No employees found matching '{}'.", self.query);
            return Ok(());
        }

        println!("Search results for '{}':", self.query);
        display_employees_table(&employees);
        Ok(())
    }

    fn name(&self) -> &'static str {
        "hr-employee-search"
    }

    fn description(&self) -> &'static str {
        "Search employees by name, email, or code"
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

pub struct HrEmployeeDeleteCommand {
    pub id: i32,
}

impl HrEmployeeDeleteCommand {
    pub fn new(id: i32) -> Self {
        Self { id }
    }
}

impl Command for HrEmployeeDeleteCommand {
    fn execute(
        &self,
        _args: &dyn std::any::Any,
        user: Option<&AuthenticatedUser>,
    ) -> CLIERPResult<()> {
        use crate::modules::hr::employee::EmployeeService;

        let _user = user.ok_or_else(|| crate::core::error::CLIERPError::AuthenticationRequired)?;

        let db_manager = DatabaseManager::new()?;
        let mut conn = db_manager.get_connection()?;
        let emp_service = EmployeeService::new();

        emp_service.delete_employee(&mut conn, self.id)?;

        println!("✅ Employee terminated successfully!");

        Ok(())
    }

    fn name(&self) -> &'static str {
        "hr-employee-delete"
    }

    fn description(&self) -> &'static str {
        "Terminate an employee (soft delete)"
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

// Export Commands

pub struct HrDeptExportCommand {
    pub format: String,
    pub output: Option<String>,
}

impl HrDeptExportCommand {
    pub fn new(format: String, output: Option<String>) -> Self {
        Self { format, output }
    }
}

impl Command for HrDeptExportCommand {
    fn execute(
        &self,
        _args: &dyn std::any::Any,
        user: Option<&AuthenticatedUser>,
    ) -> CLIERPResult<()> {
        use crate::utils::export::ExportService;

        let _user = user.ok_or_else(|| crate::core::error::CLIERPError::AuthenticationRequired)?;

        let db_manager = DatabaseManager::new()?;
        let mut conn = db_manager.get_connection()?;
        let dept_service = DepartmentService::new();

        let departments = dept_service.list_departments_with_employee_count(&mut conn)?;

        if departments.is_empty() {
            println!("No departments to export.");
            return Ok(());
        }

        let export_service = ExportService::new();
        let file_path = match &self.output {
            Some(path) => path.clone(),
            None => ExportService::generate_filename("departments", &self.format),
        };

        ExportService::prepare_file_path(&file_path)?;

        match self.format.to_lowercase().as_str() {
            "csv" => {
                let headers = &[
                    "ID",
                    "Name",
                    "Description",
                    "Manager ID",
                    "Employee Count",
                    "Created",
                    "Updated",
                ];
                export_service.export_to_csv(&departments, headers, &file_path)?;
            }
            "json" => {
                export_service.export_to_json(&departments, &file_path)?;
            }
            _ => {
                return Err(crate::core::error::CLIERPError::ValidationError(
                    "Unsupported format. Use 'csv' or 'json'.".to_string(),
                ));
            }
        }

        println!("✅ Departments exported successfully to: {}", file_path);
        println!("📊 Exported {} departments", departments.len());

        Ok(())
    }

    fn name(&self) -> &'static str {
        "hr-dept-export"
    }

    fn description(&self) -> &'static str {
        "Export departments to CSV or JSON"
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

pub struct HrEmployeeExportCommand {
    pub format: String,
    pub output: Option<String>,
    pub department_id: Option<i32>,
}

impl HrEmployeeExportCommand {
    pub fn new(format: String, output: Option<String>, department_id: Option<i32>) -> Self {
        Self {
            format,
            output,
            department_id,
        }
    }
}

impl Command for HrEmployeeExportCommand {
    fn execute(
        &self,
        _args: &dyn std::any::Any,
        user: Option<&AuthenticatedUser>,
    ) -> CLIERPResult<()> {
        use crate::modules::hr::employee::EmployeeService;
        use crate::utils::export::ExportService;

        let _user = user.ok_or_else(|| crate::core::error::CLIERPError::AuthenticationRequired)?;

        let db_manager = DatabaseManager::new()?;
        let mut conn = db_manager.get_connection()?;
        let emp_service = EmployeeService::new();

        let employees = match self.department_id {
            Some(dept_id) => emp_service.list_employees_by_department(&mut conn, dept_id)?,
            None => emp_service.list_employees(&mut conn)?,
        };

        if employees.is_empty() {
            println!("No employees to export.");
            return Ok(());
        }

        let export_service = ExportService::new();
        let file_path = match &self.output {
            Some(path) => path.clone(),
            None => {
                let prefix = match self.department_id {
                    Some(dept_id) => format!("employees_dept_{}", dept_id),
                    None => "employees".to_string(),
                };
                ExportService::generate_filename(&prefix, &self.format)
            }
        };

        ExportService::prepare_file_path(&file_path)?;

        match self.format.to_lowercase().as_str() {
            "csv" => {
                let headers = &[
                    "ID",
                    "Code",
                    "Name",
                    "Email",
                    "Phone",
                    "Department",
                    "Position",
                    "Salary",
                    "Status",
                    "Hire Date",
                    "Created",
                    "Updated",
                ];
                export_service.export_to_csv(&employees, headers, &file_path)?;
            }
            "json" => {
                export_service.export_to_json(&employees, &file_path)?;
            }
            _ => {
                return Err(crate::core::error::CLIERPError::ValidationError(
                    "Unsupported format. Use 'csv' or 'json'.".to_string(),
                ));
            }
        }

        println!("✅ Employees exported successfully to: {}", file_path);
        println!("📊 Exported {} employees", employees.len());

        Ok(())
    }

    fn name(&self) -> &'static str {
        "hr-employee-export"
    }

    fn description(&self) -> &'static str {
        "Export employees to CSV or JSON"
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

// Helper functions

fn display_departments_table(departments: &[DepartmentWithEmployeeCount]) {
    let headers = vec![
        "ID",
        "Name",
        "Description",
        "Manager ID",
        "Employees",
        "Created",
    ];
    let rows: Vec<Vec<String>> = departments
        .iter()
        .map(|dept_with_count| {
            vec![
                dept_with_count.department.id.to_string(),
                dept_with_count.department.name.clone(),
                dept_with_count
                    .department
                    .description
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
                dept_with_count
                    .department
                    .manager_id
                    .map(|id| id.to_string())
                    .unwrap_or_else(|| "-".to_string()),
                dept_with_count.employee_count.to_string(),
                dept_with_count
                    .department
                    .created_at
                    .format("%Y-%m-%d")
                    .to_string(),
            ]
        })
        .collect();

    format_table(&headers, &rows);
}

fn display_employees_table(employees: &[crate::modules::hr::employee::EmployeeWithDepartment]) {
    let headers = vec![
        "ID",
        "Code",
        "Name",
        "Email",
        "Department",
        "Position",
        "Salary",
        "Status",
        "Hire Date",
    ];
    let rows: Vec<Vec<String>> = employees
        .iter()
        .map(|emp_with_dept| {
            vec![
                emp_with_dept.employee.id.to_string(),
                emp_with_dept.employee.employee_code.clone(),
                emp_with_dept.employee.name.clone(),
                emp_with_dept
                    .employee
                    .email
                    .clone()
                    .unwrap_or_else(|| "-".to_string()),
                emp_with_dept.department.name.clone(),
                emp_with_dept.employee.position.clone(),
                crate::utils::formatting::format_currency(emp_with_dept.employee.salary),
                emp_with_dept.employee.status.clone(),
                emp_with_dept
                    .employee
                    .hire_date
                    .format("%Y-%m-%d")
                    .to_string(),
            ]
        })
        .collect();

    format_table(&headers, &rows);
}

fn display_employee_detail(emp_with_dept: &crate::modules::hr::employee::EmployeeWithDepartment) {
    println!("📋 Employee Details");
    println!("═══════════════════");
    println!("ID: {}", emp_with_dept.employee.id);
    println!("Code: {}", emp_with_dept.employee.employee_code);
    println!("Name: {}", emp_with_dept.employee.name);

    if let Some(email) = &emp_with_dept.employee.email {
        println!("Email: {}", email);
    }

    if let Some(phone) = &emp_with_dept.employee.phone {
        println!("Phone: {}", phone);
    }

    println!(
        "Department: {} (ID: {})",
        emp_with_dept.department.name, emp_with_dept.department.id
    );
    println!("Position: {}", emp_with_dept.employee.position);
    println!(
        "Salary: {}",
        crate::utils::formatting::format_currency(emp_with_dept.employee.salary)
    );
    println!("Status: {}", emp_with_dept.employee.status);
    println!(
        "Hire Date: {}",
        emp_with_dept.employee.hire_date.format("%Y-%m-%d")
    );
    println!(
        "Created: {}",
        emp_with_dept
            .employee
            .created_at
            .format("%Y-%m-%d %H:%M:%S")
    );
    println!(
        "Updated: {}",
        emp_with_dept
            .employee
            .updated_at
            .format("%Y-%m-%d %H:%M:%S")
    );
}
