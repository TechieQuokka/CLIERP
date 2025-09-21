use crate::core::{result::CLIERPResult, command::Command, auth::{AuthenticatedUser, AuthService}};

pub struct AuthLoginCommand {
    auth_service: AuthService,
}

impl AuthLoginCommand {
    pub fn new(auth_service: AuthService) -> Self {
        Self { auth_service }
    }
}

impl Command for AuthLoginCommand {
    fn execute(&self, _args: &dyn std::any::Any, _user: Option<&AuthenticatedUser>) -> CLIERPResult<()> {
        println!("Login command executed!");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "auth-login"
    }

    fn description(&self) -> &'static str {
        "Login to the system"
    }

    fn requires_auth(&self) -> bool {
        false
    }
}

pub struct AuthLogoutCommand;

impl AuthLogoutCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for AuthLogoutCommand {
    fn execute(&self, _args: &dyn std::any::Any, _user: Option<&AuthenticatedUser>) -> CLIERPResult<()> {
        println!("Logout command executed!");
        Ok(())
    }

    fn name(&self) -> &'static str {
        "auth-logout"
    }

    fn description(&self) -> &'static str {
        "Logout from the system"
    }

    fn requires_auth(&self) -> bool {
        true
    }
}

pub struct AuthWhoamiCommand;

impl AuthWhoamiCommand {
    pub fn new() -> Self {
        Self
    }
}

impl Command for AuthWhoamiCommand {
    fn execute(&self, _args: &dyn std::any::Any, user: Option<&AuthenticatedUser>) -> CLIERPResult<()> {
        if let Some(user) = user {
            println!("Current user: {}", user.username);
        } else {
            println!("Not logged in");
        }
        Ok(())
    }

    fn name(&self) -> &'static str {
        "auth-whoami"
    }

    fn description(&self) -> &'static str {
        "Show current user information"
    }

    fn requires_auth(&self) -> bool {
        false
    }
}