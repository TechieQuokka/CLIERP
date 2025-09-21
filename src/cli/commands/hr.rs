use crate::core::{result::CLIERPResult, command::Command, auth::AuthenticatedUser};

pub struct HrDeptListCommand;

impl HrDeptListCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for HrDeptListCommand {
    fn execute(&self, _args: &dyn std::any::Any, _user: Option<&AuthenticatedUser>) -> CLIERPResult<()> {
        println!("Department list command executed!");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "hr-dept-list"
    }

    fn description(&self) -> &'static str {
        "List all departments"
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

pub struct HrEmployeeListCommand;

impl HrEmployeeListCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for HrEmployeeListCommand {
    fn execute(&self, _args: &dyn std::any::Any, _user: Option<&AuthenticatedUser>) -> CLIERPResult<()> {
        println!("Employee list command executed!");
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