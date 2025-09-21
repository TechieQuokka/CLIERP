use crate::core::{
    result::CLIERPResult,
    error::CLIERPError,
    config::CLIERPConfig,
    logging,
    auth::AuthService,
    command::{CLIArgs, CLICommands, CommandRegistry},
};
use crate::database::{connection::DatabaseManager, migrations};
use crate::cli::{commands::*, session::SessionManager};
use clap::Parser;

pub struct CLIApp {
    config: CLIERPConfig,
    auth_service: AuthService,
    command_registry: CommandRegistry,
    session_manager: SessionManager,
}

impl CLIApp {
    pub fn new() -> CLIERPResult<Self> {
        // Load configuration
        let config = CLIERPConfig::load()
            .map_err(CLIERPError::Configuration)?;

        config.validate()
            .map_err(CLIERPError::Configuration)?;

        // Initialize logging
        logging::init_logging(&config)?;

        // Initialize database
        DatabaseManager::initialize(&config)?;

        // Run migrations
        let mut conn = DatabaseManager::establish_connection(&config.database.url)?;
        migrations::run_migrations(&mut conn)?;

        // Initialize services
        let auth_service = AuthService::new(config.clone());
        let command_registry = CommandRegistry::new();
        let session_manager = SessionManager::new(config.clone());

        // Create default admin user if needed
        auth_service.create_default_admin()?;

        Ok(Self {
            config,
            auth_service,
            command_registry,
            session_manager,
        })
    }

    pub async fn run(&mut self) -> CLIERPResult<()> {
        let args = CLIArgs::parse();

        // Register all commands
        self.register_commands();

        // Handle global flags
        if args.verbose {
            tracing::info!("Verbose mode enabled");
        }

        // Execute command
        match args.command {
            Some(command) => self.execute_command(command).await,
            None => {
                // Interactive mode or help
                println!("CLIERP - CLI-based ERP System");
                println!("Use --help for more information");
                Ok(())
            }
        }
    }

    fn register_commands(&mut self) {
        // Register system commands
        self.command_registry.register(SystemInitCommand::new());
        self.command_registry.register(SystemStatusCommand::new());
        self.command_registry.register(SystemMigrateCommand::new());

        // Register auth commands
        self.command_registry.register(AuthLoginCommand::new(self.auth_service.clone()));
        self.command_registry.register(AuthLogoutCommand::new());
        self.command_registry.register(AuthWhoamiCommand::new());

        // Register HR commands
        self.command_registry.register(HrDeptListCommand::new());
        self.command_registry.register(HrEmployeeListCommand::new(None));

        // More commands will be registered as modules are implemented
    }

    async fn execute_command(&mut self, command: CLICommands) -> CLIERPResult<()> {
        match command {
            CLICommands::System { action } => {
                self.execute_system_command(action).await
            }
            CLICommands::Auth { action } => {
                self.execute_auth_command(action).await
            }
            CLICommands::Hr { action } => {
                self.execute_hr_command(action).await
            }
            CLICommands::Fin { action } => {
                self.execute_fin_command(action).await
            }
            CLICommands::Inv { action } => {
                self.execute_inv_command(action).await
            }
            CLICommands::Crm { action } => {
                self.execute_crm_command(action).await
            }
        }
    }

    async fn execute_system_command(&mut self, action: crate::core::command::SystemCommands) -> CLIERPResult<()> {
        use crate::core::command::SystemCommands;

        match action {
            SystemCommands::Init => {
                println!("Initializing CLIERP system...");

                // Initialize database
                let mut conn = DatabaseManager::establish_connection(&self.config.database.url)?;
                migrations::run_migrations(&mut conn)?;

                // Create default admin
                self.auth_service.create_default_admin()?;

                println!("✓ System initialized successfully!");
                println!("Default admin user created: username 'admin'");
                println!("Please login and change the default password.");
                Ok(())
            }
            SystemCommands::Status => {
                println!("CLIERP System Status");
                println!("===================");
                println!("Version: {}", crate::VERSION);
                println!("Database: Connected");

                // Check database connection
                let db_manager = DatabaseManager::new()?;
                match db_manager.get_connection() {
                    Ok(_) => println!("Database: ✓ Connected"),
                    Err(e) => println!("Database: ✗ Error - {}", e),
                }

                Ok(())
            }
            SystemCommands::Migrate => {
                println!("Running database migrations...");
                let mut conn = DatabaseManager::establish_connection(&self.config.database.url)?;
                migrations::run_migrations(&mut conn)?;
                println!("✓ Migrations completed successfully!");
                Ok(())
            }
            SystemCommands::CreateAdmin => {
                self.auth_service.create_default_admin()?;
                println!("✓ Default admin user created!");
                Ok(())
            }
        }
    }

    async fn execute_auth_command(&mut self, action: crate::core::command::AuthCommands) -> CLIERPResult<()> {
        use crate::core::command::AuthCommands;

        match action {
            AuthCommands::Login { username, password } => {
                let password = if let Some(pwd) = password {
                    pwd
                } else {
                    // Prompt for password securely
                    use std::io::{self, Write};
                    print!("Password: ");
                    io::stdout().flush().unwrap();
                    let mut input = String::new();
                    io::stdin().read_line(&mut input).unwrap();
                    input.trim().to_string()
                };

                match self.auth_service.authenticate(&username, &password) {
                    Ok(user) => {
                        let token = self.auth_service.generate_token(&user)?;
                        self.session_manager.save_session(&token)?;
                        println!("✓ Login successful! Welcome, {}", user.username);
                    }
                    Err(e) => {
                        println!("✗ Login failed: {}", e);
                        return Err(e);
                    }
                }
                Ok(())
            }
            AuthCommands::Logout => {
                self.session_manager.clear_session()?;
                println!("✓ Logged out successfully!");
                Ok(())
            }
            AuthCommands::Whoami => {
                if let Some(user) = self.session_manager.get_current_user()? {
                    println!("Current User:");
                    println!("  Username: {}", user.username);
                    println!("  Email: {}", user.email);
                    println!("  Role: {}", user.role);
                    if let Some(emp_id) = user.employee_id {
                        println!("  Employee ID: {}", emp_id);
                    }
                } else {
                    println!("Not logged in");
                }
                Ok(())
            }
            AuthCommands::CreateUser { username, email, role, employee_id } => {
                // Check if current user is admin
                if let Some(current_user) = self.session_manager.get_current_user()? {
                    if !matches!(current_user.role, crate::database::models::UserRole::Admin) {
                        return Err(CLIERPError::Authorization("Admin role required".to_string()));
                    }
                } else {
                    return Err(CLIERPError::Authentication("Login required".to_string()));
                }

                // Parse role
                let user_role = match role.as_str() {
                    "admin" => crate::database::models::UserRole::Admin,
                    "manager" => crate::database::models::UserRole::Manager,
                    "supervisor" => crate::database::models::UserRole::Supervisor,
                    "employee" => crate::database::models::UserRole::Employee,
                    "auditor" => crate::database::models::UserRole::Auditor,
                    _ => return Err(CLIERPError::Validation("Invalid role".to_string())),
                };

                // Prompt for password
                use std::io::{self, Write};
                print!("Password for new user: ");
                io::stdout().flush().unwrap();
                let mut password = String::new();
                io::stdin().read_line(&mut password).unwrap();
                let password = password.trim().to_string();

                let user = self.auth_service.create_user(username, email, password, user_role, employee_id)?;
                println!("✓ User created successfully: {}", user.username);
                Ok(())
            }
        }
    }

    async fn execute_hr_command(&mut self, action: crate::core::command::HrCommands) -> CLIERPResult<()> {
        // Check authentication for HR commands
        let _user = self.session_manager.get_current_user()?
            .ok_or_else(|| CLIERPError::Authentication("Login required for HR commands".to_string()))?;

        println!("HR command executed: {:?}", action);
        // HR command implementation will be added in Phase 2
        Ok(())
    }

    async fn execute_fin_command(&mut self, action: crate::core::command::FinCommands) -> CLIERPResult<()> {
        // Check authentication for Finance commands
        let _user = self.session_manager.get_current_user()?
            .ok_or_else(|| CLIERPError::Authentication("Login required for Finance commands".to_string()))?;

        println!("Finance command executed: {:?}", action);
        // Finance command implementation will be added in Phase 2
        Ok(())
    }

    async fn execute_inv_command(&mut self, action: crate::core::command::InvCommands) -> CLIERPResult<()> {
        // Check authentication for Inventory commands
        let _user = self.session_manager.get_current_user()?
            .ok_or_else(|| CLIERPError::Authentication("Login required for Inventory commands".to_string()))?;

        println!("Inventory command executed: {:?}", action);
        // Inventory command implementation will be added in Phase 3
        Ok(())
    }

    async fn execute_crm_command(&mut self, action: crate::core::command::CrmCommands) -> CLIERPResult<()> {
        // Check authentication for CRM commands
        let _user = self.session_manager.get_current_user()?
            .ok_or_else(|| CLIERPError::Authentication("Login required for CRM commands".to_string()))?;

        println!("CRM command executed: {:?}", action);
        // CRM command implementation will be added in Phase 3
        Ok(())
    }
}