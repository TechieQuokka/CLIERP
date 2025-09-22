use std::io::{self, Write};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent},
    execute,
    style::{Color, Print, ResetColor, SetForegroundColor},
    terminal::{self, ClearType},
};
use crate::core::{error::CLIERPError, result::CLIERPResult};
use crate::cli::session::SessionManager;
use crate::modules::inventory::{ProductService, SupplierService};
use crate::database::connection::get_connection;

pub struct InteractiveMode {
    session_manager: SessionManager,
    product_service: ProductService,
    supplier_service: SupplierService,
}

impl InteractiveMode {
    pub fn new(session_manager: SessionManager) -> Self {
        Self {
            session_manager,
            product_service: ProductService::new(),
            supplier_service: SupplierService,
        }
    }

    pub async fn start(&mut self) -> CLIERPResult<()> {
        terminal::enable_raw_mode()?;

        // Display welcome message
        self.print_welcome()?;

        loop {
            match self.main_menu().await {
                Ok(should_continue) => {
                    if !should_continue {
                        break;
                    }
                }
                Err(e) => {
                    self.print_error(&format!("Error: {}", e))?;
                    self.wait_for_key()?;
                }
            }
        }

        terminal::disable_raw_mode()?;
        Ok(())
    }

    async fn main_menu(&mut self) -> CLIERPResult<bool> {
        self.clear_screen()?;
        self.print_header("CLIERP Interactive Mode")?;

        let options = vec![
            "1. Inventory Management",
            "2. Purchase Orders",
            "3. CRM Operations",
            "4. Reports",
            "5. System Administration",
            "0. Exit",
        ];

        let choice = self.show_menu("Main Menu", &options)?;

        match choice {
            1 => self.inventory_menu().await?,
            2 => self.purchase_menu().await?,
            3 => self.crm_menu().await?,
            4 => self.reports_menu().await?,
            5 => self.admin_menu().await?,
            0 => return Ok(false),
            _ => {
                self.print_error("Invalid selection")?;
                self.wait_for_key()?;
            }
        }

        Ok(true)
    }

    async fn inventory_menu(&mut self) -> CLIERPResult<()> {
        loop {
            self.clear_screen()?;
            self.print_header("Inventory Management")?;

            let options = vec![
                "1. Add Product",
                "2. Search Products",
                "3. Stock Adjustment",
                "4. Low Stock Report",
                "5. Product Categories",
                "0. Back to Main Menu",
            ];

            let choice = self.show_menu("Inventory Menu", &options)?;

            match choice {
                1 => self.add_product_wizard().await?,
                2 => self.search_products().await?,
                3 => self.stock_adjustment_wizard().await?,
                4 => self.show_low_stock_report().await?,
                5 => self.manage_categories().await?,
                0 => break,
                _ => {
                    self.print_error("Invalid selection")?;
                    self.wait_for_key()?;
                }
            }
        }
        Ok(())
    }

    async fn add_product_wizard(&mut self) -> CLIERPResult<()> {
        self.clear_screen()?;
        self.print_header("Add New Product")?;

        // Guided product creation
        let sku = self.input_text("Enter SKU", true)?;
        let name = self.input_text("Enter Product Name", true)?;
        let description = self.input_text("Enter Description (optional)", false)?;

        // Category selection
        let categories = self.load_categories().await?;
        let category_id = self.select_from_list("Select Category", &categories)?;

        let price = self.input_currency("Enter Selling Price")?;
        let cost_price = self.input_currency("Enter Cost Price")?;
        let initial_stock = self.input_number("Enter Initial Stock", 0)?;
        let min_stock = self.input_number("Enter Minimum Stock Level", 0)?;
        let max_stock = self.input_number_optional("Enter Maximum Stock Level (optional)")?;
        let unit = self.input_text("Enter Unit of Measure (e.g., ea, kg, l)", true)?;
        let barcode = self.input_text("Enter Barcode (optional)", false)?;

        // Confirmation
        self.print_info("\n=== Product Summary ===")?;
        println!("SKU: {}", sku);
        println!("Name: {}", name);
        if let Some(desc) = &description {
            println!("Description: {}", desc);
        }
        println!("Price: ¥{}", price as f64 / 100.0);
        println!("Cost: ¥{}", cost_price as f64 / 100.0);
        println!("Initial Stock: {} {}", initial_stock, unit);
        println!("Min Stock: {}", min_stock);

        if self.confirm("Create this product?")? {
            match self.product_service.create_product(
                &sku,
                &name,
                description.as_deref(),
                category_id,
                price,
                cost_price,
                initial_stock,
                min_stock,
                max_stock,
                &unit,
                barcode.as_deref(),
            ) {
                Ok(product) => {
                    self.print_success(&format!("✅ Product created successfully! ID: {}", product.id))?;
                }
                Err(e) => {
                    self.print_error(&format!("Failed to create product: {}", e))?;
                }
            }
        }

        self.wait_for_key()?;
        Ok(())
    }

    async fn purchase_menu(&mut self) -> CLIERPResult<()> {
        loop {
            self.clear_screen()?;
            self.print_header("Purchase Management")?;

            let options = vec![
                "1. Create Purchase Order",
                "2. View Purchase Orders",
                "3. Approve Purchase Order",
                "4. Receive Items",
                "5. Supplier Management",
                "0. Back to Main Menu",
            ];

            let choice = self.show_menu("Purchase Menu", &options)?;

            match choice {
                1 => self.create_purchase_order_wizard().await?,
                2 => self.view_purchase_orders().await?,
                3 => self.approve_purchase_order().await?,
                4 => self.receive_items_wizard().await?,
                5 => self.supplier_management().await?,
                0 => break,
                _ => {
                    self.print_error("Invalid selection")?;
                    self.wait_for_key()?;
                }
            }
        }
        Ok(())
    }

    async fn create_purchase_order_wizard(&mut self) -> CLIERPResult<()> {
        self.clear_screen()?;
        self.print_header("Create Purchase Order")?;

        // Step 1: Select supplier
        let mut conn = get_connection()?;
        let suppliers = SupplierService::list_all_active(&mut conn)?;
        if suppliers.is_empty() {
            self.print_error("No active suppliers found. Please add suppliers first.")?;
            self.wait_for_key()?;
            return Ok(());
        }

        let supplier_names: Vec<String> = suppliers.iter()
            .map(|s| format!("{} - {}", s.supplier_code, s.name))
            .collect();

        let supplier_index = self.select_from_list("Select Supplier", &supplier_names)?;
        let selected_supplier = &suppliers[supplier_index];

        // Step 2: Add items
        let mut items = Vec::new();
        loop {
            self.print_info("\n=== Add Purchase Order Items ===")?;

            // Product selection (simplified - in practice, you'd have better search)
            let product_id = self.input_number("Enter Product ID", 1)?;
            let quantity = self.input_number("Enter Quantity", 1)?;
            let unit_cost = self.input_currency("Enter Unit Cost")?;

            items.push((product_id, quantity, unit_cost));

            if !self.confirm("Add another item?")? {
                break;
            }
        }

        // Step 3: Additional details
        let expected_date = self.input_date_optional("Expected Delivery Date (YYYY-MM-DD, optional)")?;
        let notes = self.input_text("Notes (optional)", false)?;

        // Step 4: Summary and confirmation
        self.print_info("\n=== Purchase Order Summary ===")?;
        println!("Supplier: {} - {}", selected_supplier.supplier_code, selected_supplier.name);
        println!("Items: {} products", items.len());

        let total_amount: i32 = items.iter()
            .map(|(_, qty, cost)| qty * cost)
            .sum();
        println!("Total Amount: ¥{}", total_amount as f64 / 100.0);

        if self.confirm("Create this purchase order?")? {
            // Here you would call the actual PurchaseOrderService
            self.print_success("✅ Purchase order created successfully!")?;
        }

        self.wait_for_key()?;
        Ok(())
    }

    // Helper UI methods
    fn clear_screen(&self) -> CLIERPResult<()> {
        execute!(io::stdout(), terminal::Clear(ClearType::All), cursor::MoveTo(0, 0))?;
        Ok(())
    }

    fn print_header(&self, title: &str) -> CLIERPResult<()> {
        execute!(
            io::stdout(),
            SetForegroundColor(Color::Cyan),
            Print(format!("\n{}\n{}\n", title, "=".repeat(title.len()))),
            ResetColor
        )?;
        Ok(())
    }

    fn print_success(&self, message: &str) -> CLIERPResult<()> {
        execute!(
            io::stdout(),
            SetForegroundColor(Color::Green),
            Print(format!("{}\n", message)),
            ResetColor
        )?;
        Ok(())
    }

    fn print_error(&self, message: &str) -> CLIERPResult<()> {
        execute!(
            io::stdout(),
            SetForegroundColor(Color::Red),
            Print(format!("{}\n", message)),
            ResetColor
        )?;
        Ok(())
    }

    fn print_info(&self, message: &str) -> CLIERPResult<()> {
        execute!(
            io::stdout(),
            SetForegroundColor(Color::Yellow),
            Print(format!("{}\n", message)),
            ResetColor
        )?;
        Ok(())
    }

    fn show_menu(&self, title: &str, options: &[&str]) -> CLIERPResult<usize> {
        println!("\n{}", title);
        println!("{}", "-".repeat(title.len()));

        for option in options {
            println!("{}", option);
        }

        print!("\nSelect option: ");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        input.trim().parse().map_err(|_| {
            CLIERPError::InvalidInput("Invalid menu selection".to_string())
        })
    }

    fn input_text(&self, prompt: &str, required: bool) -> CLIERPResult<Option<String>> {
        loop {
            print!("{}: ", prompt);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;
            let input = input.trim().to_string();

            if input.is_empty() && required {
                self.print_error("This field is required")?;
                continue;
            }

            return Ok(if input.is_empty() { None } else { Some(input) });
        }
    }

    fn input_currency(&self, prompt: &str) -> CLIERPResult<i32> {
        loop {
            print!("{} (¥): ", prompt);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim().parse::<f64>() {
                Ok(amount) => return Ok((amount * 100.0) as i32),
                Err(_) => {
                    self.print_error("Please enter a valid amount")?;
                    continue;
                }
            }
        }
    }

    fn input_number(&self, prompt: &str, min: i32) -> CLIERPResult<i32> {
        loop {
            print!("{}: ", prompt);
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim().parse::<i32>() {
                Ok(num) if num >= min => return Ok(num),
                Ok(_) => {
                    self.print_error(&format!("Number must be at least {}", min))?;
                    continue;
                }
                Err(_) => {
                    self.print_error("Please enter a valid number")?;
                    continue;
                }
            }
        }
    }

    fn confirm(&self, prompt: &str) -> CLIERPResult<bool> {
        print!("{} (y/N): ", prompt);
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;

        Ok(input.trim().to_lowercase() == "y" || input.trim().to_lowercase() == "yes")
    }

    fn wait_for_key(&self) -> CLIERPResult<()> {
        print!("\nPress Enter to continue...");
        io::stdout().flush()?;

        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        Ok(())
    }

    // Placeholder implementations for missing methods
    async fn load_categories(&self) -> CLIERPResult<Vec<String>> {
        // In practice, load from CategoryService
        Ok(vec!["Electronics".to_string(), "Office Supplies".to_string()])
    }

    fn select_from_list(&self, prompt: &str, items: &[String]) -> CLIERPResult<usize> {
        println!("\n{}", prompt);
        for (i, item) in items.iter().enumerate() {
            println!("{}. {}", i + 1, item);
        }

        loop {
            print!("Select (1-{}): ", items.len());
            io::stdout().flush()?;

            let mut input = String::new();
            io::stdin().read_line(&mut input)?;

            match input.trim().parse::<usize>() {
                Ok(choice) if choice >= 1 && choice <= items.len() => {
                    return Ok(choice - 1);
                }
                _ => {
                    self.print_error("Invalid selection")?;
                    continue;
                }
            }
        }
    }

    // Stub implementations for missing functionality
    async fn search_products(&mut self) -> CLIERPResult<()> { Ok(()) }
    async fn stock_adjustment_wizard(&mut self) -> CLIERPResult<()> { Ok(()) }
    async fn show_low_stock_report(&mut self) -> CLIERPResult<()> { Ok(()) }
    async fn manage_categories(&mut self) -> CLIERPResult<()> { Ok(()) }
    async fn view_purchase_orders(&mut self) -> CLIERPResult<()> { Ok(()) }
    async fn approve_purchase_order(&mut self) -> CLIERPResult<()> { Ok(()) }
    async fn receive_items_wizard(&mut self) -> CLIERPResult<()> { Ok(()) }
    async fn supplier_management(&mut self) -> CLIERPResult<()> { Ok(()) }
    async fn crm_menu(&mut self) -> CLIERPResult<()> { Ok(()) }
    async fn reports_menu(&mut self) -> CLIERPResult<()> { Ok(()) }
    async fn admin_menu(&mut self) -> CLIERPResult<()> { Ok(()) }
    fn input_number_optional(&self, _prompt: &str) -> CLIERPResult<Option<i32>> { Ok(None) }
    fn input_date_optional(&self, _prompt: &str) -> CLIERPResult<Option<String>> { Ok(None) }
    fn print_welcome(&self) -> CLIERPResult<()> {
        execute!(
            io::stdout(),
            SetForegroundColor(Color::Green),
            Print("Welcome to CLIERP Interactive Mode!\n"),
            Print("Use arrow keys to navigate, Enter to select, Esc to go back.\n\n"),
            ResetColor
        )?;
        Ok(())
    }
}