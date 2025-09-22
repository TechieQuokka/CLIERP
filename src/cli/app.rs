use crate::cli::{commands::*, session::SessionManager};
use crate::core::{
    auth::AuthService,
    command::{CLIArgs, CLICommands, CommandRegistry},
    config::CLIERPConfig,
    error::CLIERPError,
    logging,
    result::CLIERPResult,
};
use crate::database::{connection::{DatabaseManager, get_connection}, migrations};
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
            CLICommands::Crm { action } => self.handle_crm_command(action).await,
            CLICommands::Sales { action } => self.execute_sales_command(action).await,
            CLICommands::Purchase { action } => self.execute_purchase_command(action).await,
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
                let mut conn = get_connection()?;
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
                let mut conn = get_connection()?;
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
                    result.current_page(), result.pagination.total_pages, result.pagination.total_count
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

    async fn handle_crm_command(
        &mut self,
        action: crate::core::command::CrmCommands,
    ) -> CLIERPResult<()> {
        // Check authentication for CRM commands
        let _user = self.session_manager.get_current_user()?.ok_or_else(|| {
            CLIERPError::Authentication("Login required for CRM commands".to_string())
        })?;

        let mut conn = get_connection()?;

        match action {
            crate::core::command::CrmCommands::Customer { action } => {
                println!("Customer command: {:?}", action);
                println!("Full CRM functionality available through interactive mode");
                Ok(())
            }
            crate::core::command::CrmCommands::Lead { action } => {
                println!("Lead command: {:?}", action);
                println!("Full CRM functionality available through interactive mode");
                Ok(())
            }
        }
    }

    async fn execute_sales_command(
        &mut self,
        action: crate::core::command::SalesCommands,
    ) -> CLIERPResult<()> {
        // Check authentication for sales commands
        let _user = self.session_manager.get_current_user()?.ok_or_else(|| {
            CLIERPError::Authentication("Login required for sales commands".to_string())
        })?;

        let mut conn = get_connection()?;

        // Convert from simple command enum to the extended command structure
        use crate::cli::commands::crm_extended::{execute_crm_extended_command, CrmExtendedCommands, CrmExtendedAction};

        let extended_action = match action {
            crate::core::command::SalesCommands::Dashboard => CrmExtendedAction::Dashboard,
            crate::core::command::SalesCommands::Pipeline => CrmExtendedAction::Pipeline,
            crate::core::command::SalesCommands::Performance => CrmExtendedAction::Performance,
            _ => {
                println!("Sales command: {:?}", action);
                println!("Full sales functionality available through interactive mode");
                return Ok(());
            }
        };

        let extended_cmd = CrmExtendedCommands {
            action: extended_action,
        };

        match execute_crm_extended_command(&mut conn, extended_cmd) {
            Ok(_) => Ok(()),
            Err(e) => {
                eprintln!("Sales command failed: {}", e);
                Err(CLIERPError::Internal(format!("Sales command error: {}", e)))
            }
        }
    }

    async fn execute_purchase_command(
        &mut self,
        action: crate::core::command::PurchaseCommands,
    ) -> CLIERPResult<()> {
        use crate::core::command::{PurchaseCommands, SupplierCommands, PurchaseOrderCommands};
        use crate::modules::inventory::{SupplierService, PurchaseOrderService, PurchaseOrderItem, ReceiveItemData};
        use crate::utils::filters::FilterOptions;
        use crate::utils::pagination::PaginationParams;

        // Check authentication for purchase commands
        let _user = self.session_manager.get_current_user()?.ok_or_else(|| {
            CLIERPError::Authentication("Login required for purchase commands".to_string())
        })?;

        let mut conn = get_connection()?;

        match action {
            PurchaseCommands::Supplier { action } => {
                match action {
                    SupplierCommands::Add {
                        code,
                        name,
                        contact,
                        email,
                        phone,
                        address,
                        payment_terms,
                    } => {
                        let supplier = SupplierService::create_supplier(
                            &mut conn,
                            &code,
                            &name,
                            contact.as_deref(),
                            email.as_deref(),
                            phone.as_deref(),
                            address.as_deref(),
                            payment_terms.as_deref(),
                        )?;

                        println!("✅ Supplier created successfully!");
                        println!("ID: {}", supplier.id);
                        println!("Code: {}", supplier.supplier_code);
                        println!("Name: {}", supplier.name);
                    }
                    SupplierCommands::List {
                        search,
                        status,
                        page,
                        per_page,
                    } => {
                        let filters = FilterOptions {
                            search,
                            status,
                            ..Default::default()
                        };

                        let pagination = PaginationParams::new(page as usize, per_page as i64);
                        let result = SupplierService::list_suppliers(&mut conn, &filters, &pagination)?;

                        if result.data.is_empty() {
                            println!("No suppliers found.");
                            return Ok(());
                        }

                        println!("Suppliers:");
                        for (i, supplier) in result.data.iter().enumerate() {
                            println!(
                                "  {}. {} ({}) - {} - {}",
                                (page - 1) * per_page + i as u32 + 1,
                                supplier.name,
                                supplier.supplier_code,
                                supplier.contact_person.as_deref().unwrap_or("-"),
                                supplier.status
                            );
                        }
                        println!("Page {} of {} ({} total)", result.pagination.current_page, result.pagination.total_pages, result.pagination.total_count);
                    }
                    SupplierCommands::Show { supplier_id } => {
                        let supplier = SupplierService::get_supplier_by_id(&mut conn, supplier_id)?
                            .ok_or_else(|| CLIERPError::NotFound(format!("Supplier with ID {} not found", supplier_id)))?;

                        let stats = SupplierService::get_supplier_statistics(&mut conn, supplier_id)?;

                        println!("Supplier Details:");
                        println!("ID: {}", supplier.id);
                        println!("Code: {}", supplier.supplier_code);
                        println!("Name: {}", supplier.name);
                        println!("Contact Person: {}", supplier.contact_person.unwrap_or_else(|| "-".to_string()));
                        println!("Email: {}", supplier.email.unwrap_or_else(|| "-".to_string()));
                        println!("Phone: {}", supplier.phone.unwrap_or_else(|| "-".to_string()));
                        println!("Address: {}", supplier.address.unwrap_or_else(|| "-".to_string()));
                        println!("Payment Terms: {}", supplier.payment_terms.unwrap_or_else(|| "-".to_string()));
                        println!("Status: {}", supplier.status);
                        println!();
                        println!("Statistics:");
                        println!("Total Orders: {}", stats.total_orders);
                        println!("Pending Orders: {}", stats.pending_orders);
                        println!("Total Amount: ₩{}", stats.total_amount);
                    }
                    SupplierCommands::Update {
                        supplier_id,
                        name,
                        contact,
                        email,
                        phone,
                        address,
                        payment_terms,
                        status,
                    } => {
                        let status_enum = status.map(|s| match s.as_str() {
                            "active" => crate::database::SupplierStatus::Active,
                            "inactive" => crate::database::SupplierStatus::Inactive,
                            "blacklisted" => crate::database::SupplierStatus::Blacklisted,
                            _ => crate::database::SupplierStatus::Active,
                        });

                        let supplier = SupplierService::update_supplier(
                            &mut conn,
                            supplier_id,
                            name.as_deref(),
                            contact.as_deref(),
                            email.as_deref(),
                            phone.as_deref(),
                            address.as_deref(),
                            Some(payment_terms.as_deref()),
                            status_enum,
                        )?;

                        println!("✅ Supplier updated successfully!");
                        println!("ID: {}", supplier.id);
                        println!("Name: {}", supplier.name);
                        println!("Status: {}", supplier.status);
                    }
                }
            }
            PurchaseCommands::Order { action } => {
                match action {
                    PurchaseOrderCommands::Create {
                        supplier_id,
                        expected_date,
                        notes,
                        items,
                    } => {
                        let expected_date = expected_date.map(|s| s.parse().unwrap());

                        // Parse items string
                        let items: Result<Vec<PurchaseOrderItem>, _> = items
                            .split(',')
                            .map(|item| {
                                let parts: Vec<&str> = item.split(':').collect();
                                if parts.len() != 3 {
                                    return Err(CLIERPError::InvalidInput(
                                        "Items format should be: product_id:quantity:unit_cost".to_string()
                                    ));
                                }
                                Ok(PurchaseOrderItem {
                                    product_id: parts[0].parse().map_err(|_| CLIERPError::InvalidInput("Invalid product ID".to_string()))?,
                                    quantity: parts[1].parse().map_err(|_| CLIERPError::InvalidInput("Invalid quantity".to_string()))?,
                                    unit_cost: parts[2].parse().map_err(|_| CLIERPError::InvalidInput("Invalid unit cost".to_string()))?,
                                })
                            })
                            .collect();

                        let items = items?;
                        let current_user_id = Some(1); // TODO: Get from session

                        let po_with_details = PurchaseOrderService::create_purchase_order(
                            &mut conn,
                            supplier_id,
                            expected_date,
                            notes.as_deref(),
                            items,
                            current_user_id,
                        )?;

                        println!("✅ Purchase order created successfully!");
                        println!("PO Number: {}", po_with_details.purchase_order.po_number);
                        println!("Supplier: {}", po_with_details.supplier.name);
                        println!("Total Amount: ₩{}", po_with_details.purchase_order.total_amount);
                        println!("Items: {} products", po_with_details.items.len());
                    }
                    PurchaseOrderCommands::List {
                        search,
                        status,
                        date_from,
                        date_to,
                        page,
                        per_page,
                    } => {
                        let filters = FilterOptions {
                            search,
                            status,
                            date_from: date_from.map(|s| s.parse().unwrap()),
                            date_to: date_to.map(|s| s.parse().unwrap()),
                            ..Default::default()
                        };

                        let pagination = PaginationParams::new(page as usize, per_page as i64);
                        let result = PurchaseOrderService::list_purchase_orders(&mut conn, &filters, &pagination)?;

                        if result.data.is_empty() {
                            println!("No purchase orders found.");
                            return Ok(());
                        }

                        println!("Purchase Orders:");
                        for (i, po) in result.data.iter().enumerate() {
                            println!(
                                "  {}. {} - {} - {} - {} items - ₩{}",
                                (page - 1) * per_page + i as u32 + 1,
                                po.po_number,
                                po.supplier_name,
                                po.status,
                                po.items_count,
                                po.total_amount
                            );
                        }
                        println!("Page {} of {} ({} total)", result.pagination.current_page, result.pagination.total_pages, result.pagination.total_count);
                    }
                    PurchaseOrderCommands::Show { po_id } => {
                        let po_details = PurchaseOrderService::get_purchase_order_with_details(&mut conn, po_id)?;

                        println!("Purchase Order Details:");
                        println!("PO Number: {}", po_details.purchase_order.po_number);
                        println!("Supplier: {} ({})", po_details.supplier.name, po_details.supplier.supplier_code);
                        println!("Order Date: {}", po_details.purchase_order.order_date);
                        println!("Expected Date: {}", po_details.purchase_order.expected_date.map(|d| d.to_string()).unwrap_or_else(|| "-".to_string()));
                        println!("Status: {}", po_details.purchase_order.status);
                        println!("Total Amount: ₩{}", po_details.purchase_order.total_amount);
                        if let Some(notes) = &po_details.purchase_order.notes {
                            println!("Notes: {}", notes);
                        }
                        println!();

                        println!("Items:");
                        for (i, item) in po_details.items.iter().enumerate() {
                            println!(
                                "  {}. {} ({}) - Qty: {} - Cost: ₩{} each - Total: ₩{} - Received: {} - Status: {}",
                                i + 1,
                                item.product_name,
                                item.product_sku,
                                item.purchase_item.quantity,
                                item.purchase_item.unit_cost,
                                item.purchase_item.total_cost,
                                item.purchase_item.received_quantity,
                                item.purchase_item.status
                            );
                        }
                    }
                    PurchaseOrderCommands::Approve { po_id } => {
                        let current_user_id = 1; // TODO: Get from session

                        let purchase_order = PurchaseOrderService::approve_purchase_order(&mut conn, po_id, current_user_id)?;

                        println!("✅ Purchase order approved successfully!");
                        println!("PO Number: {}", purchase_order.po_number);
                        println!("Status: {}", purchase_order.status);
                    }
                    PurchaseOrderCommands::Receive { po_id, items } => {
                        let current_user_id = Some(1); // TODO: Get from session

                        // Parse received items string
                        let received_items: Result<Vec<ReceiveItemData>, _> = items
                            .split(',')
                            .map(|item| {
                                let parts: Vec<&str> = item.split(':').collect();
                                if parts.len() != 2 {
                                    return Err(CLIERPError::InvalidInput(
                                        "Items format should be: item_id:quantity".to_string()
                                    ));
                                }
                                Ok(ReceiveItemData {
                                    item_id: parts[0].parse().map_err(|_| CLIERPError::InvalidInput("Invalid item ID".to_string()))?,
                                    quantity: parts[1].parse().map_err(|_| CLIERPError::InvalidInput("Invalid quantity".to_string()))?,
                                })
                            })
                            .collect();

                        let received_items = received_items?;

                        let purchase_order = PurchaseOrderService::receive_purchase_items(
                            &mut conn,
                            po_id,
                            received_items,
                            current_user_id,
                        )?;

                        println!("✅ Purchase order items received successfully!");
                        println!("PO Number: {}", purchase_order.po_number);
                        println!("Status: {}", purchase_order.status);
                    }
                }
            }
        }

        Ok(())
    }
}
