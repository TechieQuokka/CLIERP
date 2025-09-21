use crate::cli::{commands::*, session::SessionManager};
use crate::core::{
    auth::AuthService,
    command::{CLIArgs, CLICommands, CommandRegistry},
    config::CLIERPConfig,
    error::CLIERPError,
    logging,
    result::CLIERPResult,
};
use crate::database::{connection::DatabaseManager, migrations};
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
        let config = CLIERPConfig::load().map_err(CLIERPError::Configuration)?;

        config.validate().map_err(CLIERPError::Configuration)?;

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
        self.command_registry
            .register(AuthLoginCommand::new(self.auth_service.clone()));
        self.command_registry.register(AuthLogoutCommand::new());
        self.command_registry.register(AuthWhoamiCommand::new());

        // Register HR commands
        self.command_registry.register(HrDeptListCommand::new());
        self.command_registry
            .register(HrEmployeeListCommand::new(None));

        // More commands will be registered as modules are implemented
    }

    async fn execute_command(&mut self, command: CLICommands) -> CLIERPResult<()> {
        match command {
            CLICommands::System { action } => self.execute_system_command(action).await,
            CLICommands::Auth { action } => self.execute_auth_command(action).await,
            CLICommands::Hr { action } => self.execute_hr_command(action).await,
            CLICommands::Fin { action } => self.execute_fin_command(action).await,
            CLICommands::Inv { action } => self.execute_inv_command(action).await,
            CLICommands::Crm { action } => self.execute_crm_command(action).await,
        }
    }

    async fn execute_system_command(
        &mut self,
        action: crate::core::command::SystemCommands,
    ) -> CLIERPResult<()> {
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

    async fn execute_auth_command(
        &mut self,
        action: crate::core::command::AuthCommands,
    ) -> CLIERPResult<()> {
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
            AuthCommands::CreateUser {
                username,
                email,
                role,
                employee_id,
            } => {
                // Check if current user is admin
                if let Some(current_user) = self.session_manager.get_current_user()? {
                    if !matches!(current_user.role, crate::database::models::UserRole::Admin) {
                        return Err(CLIERPError::Authorization(
                            "Admin role required".to_string(),
                        ));
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

                let user = self.auth_service.create_user(
                    username,
                    email,
                    password,
                    user_role,
                    employee_id,
                )?;
                println!("✓ User created successfully: {}", user.username);
                Ok(())
            }
        }
    }

    async fn execute_hr_command(
        &mut self,
        action: crate::core::command::HrCommands,
    ) -> CLIERPResult<()> {
        // Check authentication for HR commands
        let _user = self.session_manager.get_current_user()?.ok_or_else(|| {
            CLIERPError::Authentication("Login required for HR commands".to_string())
        })?;

        println!("HR command executed: {:?}", action);
        // HR command implementation will be added in Phase 2
        Ok(())
    }

    async fn execute_fin_command(
        &mut self,
        action: crate::core::command::FinCommands,
    ) -> CLIERPResult<()> {
        // Check authentication for Finance commands
        let _user = self.session_manager.get_current_user()?.ok_or_else(|| {
            CLIERPError::Authentication("Login required for Finance commands".to_string())
        })?;

        println!("Finance command executed: {:?}", action);
        // Finance command implementation will be added in Phase 2
        Ok(())
    }

    async fn execute_inv_command(
        &mut self,
        action: crate::core::command::InvCommands,
    ) -> CLIERPResult<()> {
        // Check authentication for Inventory commands
        let _user = self.session_manager.get_current_user()?.ok_or_else(|| {
            CLIERPError::Authentication("Login required for Inventory commands".to_string())
        })?;

        use crate::core::command::{InvCommands, ProductCommands, StockCommands};
        use crate::modules::inventory::{CategoryService, ProductService};

        match action {
            InvCommands::Product { action } => {
                self.execute_product_command(action).await
            }
            InvCommands::Stock { action } => {
                self.execute_stock_command(action).await
            }
        }
    }

    async fn execute_product_command(
        &mut self,
        action: crate::core::command::ProductCommands,
    ) -> CLIERPResult<()> {
        use crate::core::command::ProductCommands;
        use crate::modules::inventory::ProductService;
        use crate::utils::pagination::PaginationParams;

        let service = ProductService::new();

        match action {
            ProductCommands::Add {
                sku,
                name,
                category_id,
                price,
                cost_price,
                stock,
                min_stock,
                max_stock,
                unit,
                description,
                barcode,
            } => {
                let product = service.create_product(
                    &sku,
                    &name,
                    description.as_deref(),
                    category_id,
                    price,
                    cost_price.unwrap_or(0),
                    stock.unwrap_or(0),
                    min_stock.unwrap_or(0),
                    max_stock,
                    &unit.unwrap_or_else(|| "ea".to_string()),
                    barcode.as_deref(),
                )?;

                println!("✅ Product created:");
                println!("  ID: {}", product.id);
                println!("  SKU: {}", product.sku);
                println!("  Name: {}", product.name);
                println!("  Category ID: {}", product.category_id);
                println!("  Price: ¥{}", product.price as f64 / 100.0);
                println!("  Stock: {} {}", product.current_stock, product.unit);
            }
            ProductCommands::List {
                category_id,
                search,
                low_stock,
                active,
                page,
                per_page,
            } => {
                let pagination = PaginationParams::new(page.unwrap_or(1), per_page.unwrap_or(20));
                let result = service.list_products(
                    &pagination,
                    category_id,
                    active.unwrap_or(true),
                    search.as_deref(),
                    low_stock.unwrap_or(false),
                )?;

                if result.data.is_empty() {
                    println!("No products found.");
                    return Ok(());
                }

                println!("Products:");
                for (i, prod_with_cat) in result.data.iter().enumerate() {
                    let status = if prod_with_cat.product.current_stock <= prod_with_cat.product.min_stock_level {
                        "[LOW STOCK]"
                    } else if prod_with_cat.product.is_active {
                        "[ACTIVE]"
                    } else {
                        "[INACTIVE]"
                    };

                    println!(
                        "  {}. {} ({}) - {} - ¥{} - {} {} {}",
                        i + 1,
                        prod_with_cat.product.name,
                        prod_with_cat.product.sku,
                        prod_with_cat.category.name,
                        prod_with_cat.product.price as f64 / 100.0,
                        prod_with_cat.product.current_stock,
                        prod_with_cat.product.unit,
                        status
                    );
                }

                println!(
                    "\nPage {} of {} (Total: {} products)",
                    result.current_page(), result.total_pages(), result.total_items()
                );
            }
            ProductCommands::Show { id, sku } => {
                let product = if let Some(id) = id {
                    service.get_product_by_id(id)?
                } else if let Some(sku) = sku {
                    service.get_product_by_sku(&sku)?
                        .ok_or_else(|| CLIERPError::NotFound(format!("Product with SKU '{}' not found", sku)))?
                } else {
                    return Err(CLIERPError::InvalidInput("Either --id or --sku must be provided".to_string()));
                };

                println!("Product Details:");
                println!("  ID: {}", product.id);
                println!("  SKU: {}", product.sku);
                println!("  Name: {}", product.name);
                println!("  Category ID: {}", product.category_id);
                println!("  Price: ¥{}", product.price as f64 / 100.0);
                println!("  Cost Price: ¥{}", product.cost_price as f64 / 100.0);
                println!("  Current Stock: {} {}", product.current_stock, product.unit);
                println!("  Min Stock Level: {}", product.min_stock_level);
                if let Some(max_level) = product.max_stock_level {
                    println!("  Max Stock Level: {}", max_level);
                }
                if let Some(desc) = &product.description {
                    println!("  Description: {}", desc);
                }
                if let Some(barcode) = &product.barcode {
                    println!("  Barcode: {}", barcode);
                }
                println!("  Active: {}", if product.is_active { "Yes" } else { "No" });
                println!("  Created: {}", product.created_at.format("%Y-%m-%d %H:%M:%S"));
                println!("  Updated: {}", product.updated_at.format("%Y-%m-%d %H:%M:%S"));
            }
            _ => {
                println!("Product command not yet implemented: {:?}", action);
            }
        }

        Ok(())
    }

    async fn execute_stock_command(
        &mut self,
        action: crate::core::command::StockCommands,
    ) -> CLIERPResult<()> {
        use crate::core::command::StockCommands;
        use crate::modules::inventory::ProductService;

        let service = ProductService::new();

        match action {
            StockCommands::In {
                product_id,
                sku,
                quantity,
                unit_cost,
                reference,
                notes,
            } => {
                let product_id = if let Some(id) = product_id {
                    id
                } else if let Some(sku) = sku {
                    let product = service.get_product_by_sku(&sku)?
                        .ok_or_else(|| CLIERPError::NotFound(format!("Product with SKU '{}' not found", sku)))?;
                    product.id
                } else {
                    return Err(CLIERPError::InvalidInput("Either --product-id or --sku must be provided".to_string()));
                };

                let updated_product = service.update_stock(
                    product_id,
                    quantity,
                    "in",
                    unit_cost,
                    reference.as_deref(),
                    None,
                    notes.as_deref(),
                    None, // TODO: Add user context
                )?;

                println!("✅ Stock added:");
                println!("  Product: {} ({})", updated_product.name, updated_product.sku);
                println!("  Quantity Added: {} {}", quantity, updated_product.unit);
                println!("  New Stock Level: {} {}", updated_product.current_stock, updated_product.unit);
            }
            StockCommands::Out {
                product_id,
                sku,
                quantity,
                reference,
                notes,
            } => {
                let product_id = if let Some(id) = product_id {
                    id
                } else if let Some(sku) = sku {
                    let product = service.get_product_by_sku(&sku)?
                        .ok_or_else(|| CLIERPError::NotFound(format!("Product with SKU '{}' not found", sku)))?;
                    product.id
                } else {
                    return Err(CLIERPError::InvalidInput("Either --product-id or --sku must be provided".to_string()));
                };

                let updated_product = service.update_stock(
                    product_id,
                    -quantity.abs(),
                    "out",
                    None,
                    reference.as_deref(),
                    None,
                    notes.as_deref(),
                    None, // TODO: Add user context
                )?;

                println!("✅ Stock removed:");
                println!("  Product: {} ({})", updated_product.name, updated_product.sku);
                println!("  Quantity Removed: {} {}", quantity, updated_product.unit);
                println!("  New Stock Level: {} {}", updated_product.current_stock, updated_product.unit);
            }
            StockCommands::Check { low_stock } => {
                if low_stock {
                    let low_stock_products = service.get_low_stock_products()?;

                    if low_stock_products.is_empty() {
                        println!("No low stock products found.");
                        return Ok(());
                    }

                    println!("Low Stock Products:");
                    for (i, prod_with_cat) in low_stock_products.iter().enumerate() {
                        println!(
                            "  {}. {} ({}) - {} - Current: {} {} / Min: {}",
                            i + 1,
                            prod_with_cat.product.name,
                            prod_with_cat.product.sku,
                            prod_with_cat.category.name,
                            prod_with_cat.product.current_stock,
                            prod_with_cat.product.unit,
                            prod_with_cat.product.min_stock_level
                        );
                    }
                } else {
                    println!("General stock check not yet implemented");
                }
            }
            _ => {
                println!("Stock command not yet implemented: {:?}", action);
            }
        }

        Ok(())
    }

    async fn execute_crm_command(
        &mut self,
        action: crate::core::command::CrmCommands,
    ) -> CLIERPResult<()> {
        // Check authentication for CRM commands
        let _user = self.session_manager.get_current_user()?.ok_or_else(|| {
            CLIERPError::Authentication("Login required for CRM commands".to_string())
        })?;

        println!("CRM command executed: {:?}", action);
        // CRM command implementation will be added in Phase 3
        Ok(())
    }
}
