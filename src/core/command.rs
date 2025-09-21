use crate::core::{auth::AuthenticatedUser, error::CLIERPError, result::CLIERPResult};
use clap::{Parser, Subcommand};
use std::collections::HashMap;

/// Trait for implementing CLI commands
pub trait Command {
    /// Execute the command with the given arguments
    fn execute(
        &self,
        args: &dyn std::any::Any,
        user: Option<&AuthenticatedUser>,
    ) -> CLIERPResult<()>;

    /// Get command name
    fn name(&self) -> &'static str;

    /// Get command description
    fn description(&self) -> &'static str;

    /// Check if command requires authentication
    fn requires_auth(&self) -> bool {
        true
    }

    /// Get required minimum role for this command
    fn required_role(&self) -> Option<&'static str> {
        None
    }
}

/// Command registry for managing available commands
pub struct CommandRegistry {
    commands: HashMap<String, Box<dyn Command>>,
}

impl CommandRegistry {
    pub fn new() -> Self {
        Self {
            commands: HashMap::new(),
        }
    }

    /// Register a new command
    pub fn register<C: Command + 'static>(&mut self, command: C) {
        self.commands
            .insert(command.name().to_string(), Box::new(command));
    }

    /// Get a command by name
    pub fn get(&self, name: &str) -> Option<&dyn Command> {
        self.commands.get(name).map(|boxed| boxed.as_ref())
    }

    /// Get all available commands
    pub fn get_all(&self) -> &HashMap<String, Box<dyn Command>> {
        &self.commands
    }

    /// Execute a command
    pub fn execute(
        &self,
        name: &str,
        args: &dyn std::any::Any,
        user: Option<&AuthenticatedUser>,
    ) -> CLIERPResult<()> {
        let command = self
            .get(name)
            .ok_or_else(|| CLIERPError::NotFound(format!("Command '{}' not found", name)))?;

        // Check authentication requirements
        if command.requires_auth() && user.is_none() {
            return Err(CLIERPError::Authentication(
                "Authentication required".to_string(),
            ));
        }

        // Check role requirements
        if let (Some(required_role), Some(user)) = (command.required_role(), user) {
            // Implementation of role checking logic would go here
            tracing::debug!(
                "Checking role requirement: {} for user: {}",
                required_role,
                user.username
            );
        }

        command.execute(args, user)
    }
}

impl Default for CommandRegistry {
    fn default() -> Self {
        Self::new()
    }
}

/// Base CLI application structure
#[derive(Parser)]
#[command(name = "clierp")]
#[command(about = "CLI-based ERP System")]
#[command(version = env!("CARGO_PKG_VERSION"))]
pub struct CLIArgs {
    #[command(subcommand)]
    pub command: Option<CLICommands>,

    /// Enable verbose logging
    #[arg(short, long)]
    pub verbose: bool,

    /// Configuration file path
    #[arg(short, long)]
    pub config: Option<String>,
}

#[derive(Subcommand)]
pub enum CLICommands {
    /// Authentication commands
    Auth {
        #[command(subcommand)]
        action: AuthCommands,
    },
    /// Human Resources commands
    Hr {
        #[command(subcommand)]
        action: HrCommands,
    },
    /// Finance commands
    Fin {
        #[command(subcommand)]
        action: FinCommands,
    },
    /// Inventory commands
    Inv {
        #[command(subcommand)]
        action: InvCommands,
    },
    /// CRM commands
    Crm {
        #[command(subcommand)]
        action: CrmCommands,
    },
    /// Sales management commands (Extended CRM)
    Sales {
        #[command(subcommand)]
        action: SalesCommands,
    },
    /// Purchase management commands
    Purchase {
        #[command(subcommand)]
        action: PurchaseCommands,
    },
    /// System commands
    System {
        #[command(subcommand)]
        action: SystemCommands,
    },
}

#[derive(Subcommand)]
pub enum AuthCommands {
    /// Login to the system
    Login {
        /// Username
        #[arg(short, long)]
        username: String,
        /// Password (will prompt if not provided)
        #[arg(short, long)]
        password: Option<String>,
    },
    /// Logout from the system
    Logout,
    /// Show current user information
    Whoami,
    /// Create a new user (admin only)
    CreateUser {
        /// Username
        #[arg(short, long)]
        username: String,
        /// Email
        #[arg(short, long)]
        email: String,
        /// Role
        #[arg(short, long)]
        role: String,
        /// Employee ID
        #[arg(long)]
        employee_id: Option<i32>,
    },
}

#[derive(Debug, Subcommand)]
pub enum HrCommands {
    /// Department management
    Dept {
        #[command(subcommand)]
        action: DeptCommands,
    },
    /// Employee management
    Employee {
        #[command(subcommand)]
        action: EmployeeCommands,
    },
    /// Attendance management
    Attendance {
        #[command(subcommand)]
        action: AttendanceCommands,
    },
    /// Payroll management
    Payroll {
        #[command(subcommand)]
        action: PayrollCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum DeptCommands {
    /// Add a new department
    Add {
        /// Department name
        #[arg(short, long)]
        name: String,
        /// Department description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// List all departments
    List,
    /// Show department details
    Show {
        /// Department ID
        id: i32,
    },
    /// Update department
    Update {
        /// Department ID
        id: i32,
        /// New name
        #[arg(short, long)]
        name: Option<String>,
        /// New description
        #[arg(short, long)]
        description: Option<String>,
    },
    /// Delete department
    Delete {
        /// Department ID
        id: i32,
    },
}

#[derive(Debug, Subcommand)]
pub enum EmployeeCommands {
    /// Add a new employee
    Add {
        /// Employee code
        #[arg(short, long)]
        code: String,
        /// Employee name
        #[arg(short, long)]
        name: String,
        /// Email
        #[arg(short, long)]
        email: Option<String>,
        /// Department ID
        #[arg(short, long)]
        department_id: i32,
        /// Position
        #[arg(short, long)]
        position: String,
        /// Salary
        #[arg(short, long)]
        salary: i32,
    },
    /// List employees
    List {
        /// Filter by department
        #[arg(short, long)]
        department: Option<i32>,
        /// Filter by status
        #[arg(short, long)]
        status: Option<String>,
    },
    /// Show employee details
    Show {
        /// Employee ID
        id: i32,
    },
    /// Update employee
    Update {
        /// Employee ID
        id: i32,
        /// New name
        #[arg(short, long)]
        name: Option<String>,
        /// New position
        #[arg(short, long)]
        position: Option<String>,
        /// New salary
        #[arg(short, long)]
        salary: Option<i32>,
    },
    /// Delete employee
    Delete {
        /// Employee ID
        id: i32,
    },
}

#[derive(Debug, Subcommand)]
pub enum AttendanceCommands {
    /// Check in
    Checkin {
        /// Employee ID
        #[arg(short, long)]
        employee_id: i32,
    },
    /// Check out
    Checkout {
        /// Employee ID
        #[arg(short, long)]
        employee_id: i32,
    },
    /// Show attendance status
    Status {
        /// Employee ID
        #[arg(short, long)]
        employee_id: Option<i32>,
        /// Date (YYYY-MM-DD)
        #[arg(short, long)]
        date: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum PayrollCommands {
    /// Calculate payroll
    Calculate {
        /// Period (YYYY-MM)
        #[arg(short, long)]
        period: String,
        /// Employee ID (optional, calculates for all if not provided)
        #[arg(short, long)]
        employee_id: Option<i32>,
    },
    /// Show payroll status
    Status {
        /// Period (YYYY-MM)
        #[arg(short, long)]
        period: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum FinCommands {
    /// Account management
    Account {
        #[command(subcommand)]
        action: AccountCommands,
    },
    /// Transaction management
    Transaction {
        #[command(subcommand)]
        action: TransactionCommands,
    },
    /// Reports
    Report {
        #[command(subcommand)]
        action: ReportCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum AccountCommands {
    /// Add account
    Add {
        /// Account code
        #[arg(short, long)]
        code: String,
        /// Account name
        #[arg(short, long)]
        name: String,
        /// Account type
        #[arg(short, long)]
        account_type: String,
    },
    /// List accounts
    List,
}

#[derive(Debug, Subcommand)]
pub enum TransactionCommands {
    /// Add transaction
    Add {
        /// Account ID
        #[arg(short, long)]
        account_id: i32,
        /// Amount
        #[arg(short, long)]
        amount: i32,
        /// Type (debit/credit)
        #[arg(short, long)]
        transaction_type: String,
        /// Description
        #[arg(short, long)]
        description: String,
    },
    /// List transactions
    List {
        /// Account ID filter
        #[arg(short, long)]
        account_id: Option<i32>,
    },
}

#[derive(Debug, Subcommand)]
pub enum ReportCommands {
    /// Balance sheet
    Balance,
    /// Income statement
    Income,
}

#[derive(Debug, Subcommand)]
pub enum InvCommands {
    /// Product management
    Product {
        #[command(subcommand)]
        action: ProductCommands,
    },
    /// Stock management
    Stock {
        #[command(subcommand)]
        action: StockCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum ProductCommands {
    /// Add product
    Add {
        /// Product SKU
        #[arg(short, long)]
        sku: String,
        /// Product name
        #[arg(short, long)]
        name: String,
        /// Category ID
        #[arg(short, long)]
        category_id: i32,
        /// Price (in cents)
        #[arg(short, long)]
        price: i32,
        /// Cost price (in cents)
        #[arg(long)]
        cost_price: Option<i32>,
        /// Initial stock
        #[arg(long)]
        stock: Option<i32>,
        /// Minimum stock level
        #[arg(long)]
        min_stock: Option<i32>,
        /// Maximum stock level
        #[arg(long)]
        max_stock: Option<i32>,
        /// Unit of measurement
        #[arg(short, long)]
        unit: Option<String>,
        /// Description
        #[arg(short, long)]
        description: Option<String>,
        /// Barcode
        #[arg(short, long)]
        barcode: Option<String>,
    },
    /// List products
    List {
        /// Category ID filter
        #[arg(long)]
        category_id: Option<i32>,
        /// Search term
        #[arg(short, long)]
        search: Option<String>,
        /// Show only low stock
        #[arg(long)]
        low_stock: Option<bool>,
        /// Include inactive products
        #[arg(long)]
        active: Option<bool>,
        /// Page number
        #[arg(long)]
        page: Option<usize>,
        /// Items per page
        #[arg(long)]
        per_page: Option<i64>,
    },
    /// Show product details
    Show {
        /// Product ID
        #[arg(short, long)]
        id: Option<i32>,
        /// Product SKU
        #[arg(short, long)]
        sku: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum StockCommands {
    /// Add stock (stock in)
    In {
        /// Product ID
        #[arg(long)]
        product_id: Option<i32>,
        /// Product SKU
        #[arg(short, long)]
        sku: Option<String>,
        /// Quantity to add
        #[arg(short, long)]
        quantity: i32,
        /// Unit cost
        #[arg(long)]
        unit_cost: Option<i32>,
        /// Reference information
        #[arg(short, long)]
        reference: Option<String>,
        /// Notes
        #[arg(long)]
        notes: Option<String>,
    },
    /// Remove stock (stock out)
    Out {
        /// Product ID
        #[arg(long)]
        product_id: Option<i32>,
        /// Product SKU
        #[arg(short, long)]
        sku: Option<String>,
        /// Quantity to remove
        #[arg(short, long)]
        quantity: i32,
        /// Reference information
        #[arg(short, long)]
        reference: Option<String>,
        /// Notes
        #[arg(long)]
        notes: Option<String>,
    },
    /// Check stock status
    Check {
        /// Show only low stock products
        #[arg(long)]
        low_stock: bool,
    },
    /// Update stock
    Update {
        /// Product ID
        #[arg(short, long)]
        product_id: i32,
        /// Quantity
        #[arg(short, long)]
        quantity: i32,
    },
}

#[derive(Debug, Subcommand)]
pub enum CrmCommands {
    /// Customer management
    Customer {
        #[command(subcommand)]
        action: CustomerCommands,
    },
    /// Lead management
    Lead {
        #[command(subcommand)]
        action: LeadCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum CustomerCommands {
    /// Add customer
    Add {
        /// Customer name
        #[arg(short, long)]
        name: String,
        /// Email
        #[arg(short, long)]
        email: String,
    },
    /// List customers
    List,
}

#[derive(Debug, Subcommand)]
pub enum LeadCommands {
    /// Add lead
    Add {
        /// Customer ID
        #[arg(short, long)]
        customer_id: i32,
        /// Source
        #[arg(short, long)]
        source: String,
    },
    /// List leads
    List,
}

#[derive(Debug, Subcommand)]
pub enum SalesCommands {
    /// Customer management
    Customer {
        #[command(subcommand)]
        action: SalesCustomerCommands,
    },
    /// Lead management
    Lead {
        #[command(subcommand)]
        action: SalesLeadCommands,
    },
    /// Deal management
    Deal {
        #[command(subcommand)]
        action: DealCommands,
    },
    /// Campaign management
    Campaign {
        #[command(subcommand)]
        action: CampaignCommands,
    },
    /// Activity management
    Activity {
        #[command(subcommand)]
        action: ActivityCommands,
    },
    /// CRM Dashboard
    Dashboard,
    /// Sales Pipeline
    Pipeline,
    /// Performance Overview
    Performance,
}

#[derive(Debug, Subcommand)]
pub enum SalesCustomerCommands {
    /// Create customer
    Create,
    /// List customers
    List,
    /// Show customer details
    Show,
    /// Update customer
    Update,
    /// Delete customer
    Delete,
    /// Search customers
    Search,
    /// Customer statistics
    Stats,
}

#[derive(Debug, Subcommand)]
pub enum SalesLeadCommands {
    /// Create lead
    Create,
    /// List leads
    List,
    /// Show lead details
    Show,
    /// Update lead status
    UpdateStatus,
    /// Update lead
    Update,
    /// Assign lead
    Assign,
    /// Delete lead
    Delete,
    /// Leads by status
    ByStatus,
    /// Lead statistics
    Stats,
}

#[derive(Debug, Subcommand)]
pub enum DealCommands {
    /// Create deal
    Create,
    /// List deals
    List,
    /// Show deal details
    Show,
    /// Update deal stage
    UpdateStage,
    /// Update deal
    Update,
    /// Delete deal
    Delete,
    /// Deals by stage
    ByStage,
    /// Deal statistics
    Stats,
}

#[derive(Debug, Subcommand)]
pub enum CampaignCommands {
    /// Create campaign
    Create,
    /// List campaigns
    List,
    /// Show campaign details
    Show,
    /// Update campaign
    Update,
    /// Update campaign status
    UpdateStatus,
    /// Delete campaign
    Delete,
    /// Campaigns by status
    ByStatus,
    /// Active campaigns
    Active,
    /// Campaign performance
    Performance,
    /// Campaign statistics
    Stats,
}

#[derive(Debug, Subcommand)]
pub enum ActivityCommands {
    /// Create activity
    Create,
    /// List activities
    List,
    /// Show activity details
    Show,
    /// Update activity
    Update,
    /// Complete activity
    Complete,
    /// Reopen activity
    Reopen,
    /// Delete activity
    Delete,
    /// Activities by customer
    ByCustomer,
    /// Activities by lead
    ByLead,
    /// Activities by employee
    ByEmployee,
    /// Overdue activities
    Overdue,
    /// Activity statistics
    Stats,
}

#[derive(Debug, Subcommand)]
pub enum PurchaseCommands {
    /// Supplier management
    Supplier {
        #[command(subcommand)]
        action: SupplierCommands,
    },
    /// Purchase order management
    Order {
        #[command(subcommand)]
        action: PurchaseOrderCommands,
    },
}

#[derive(Debug, Subcommand)]
pub enum SupplierCommands {
    /// Add supplier
    Add {
        /// Supplier code
        #[arg(short, long)]
        code: String,
        /// Supplier name
        #[arg(short, long)]
        name: String,
        /// Contact person
        #[arg(long)]
        contact: Option<String>,
        /// Email
        #[arg(short, long)]
        email: Option<String>,
        /// Phone
        #[arg(short, long)]
        phone: Option<String>,
        /// Address
        #[arg(long)]
        address: Option<String>,
        /// Payment terms
        #[arg(long)]
        payment_terms: Option<String>,
    },
    /// List suppliers
    List {
        /// Search term
        #[arg(short, long)]
        search: Option<String>,
        /// Status filter
        #[arg(long)]
        status: Option<String>,
        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,
        /// Items per page
        #[arg(long, default_value = "20")]
        per_page: u32,
    },
    /// Show supplier details
    Show {
        /// Supplier ID
        supplier_id: i32,
    },
    /// Update supplier
    Update {
        /// Supplier ID
        supplier_id: i32,
        /// Supplier name
        #[arg(short, long)]
        name: Option<String>,
        /// Contact person
        #[arg(long)]
        contact: Option<String>,
        /// Email
        #[arg(short, long)]
        email: Option<String>,
        /// Phone
        #[arg(short, long)]
        phone: Option<String>,
        /// Address
        #[arg(long)]
        address: Option<String>,
        /// Payment terms
        #[arg(long)]
        payment_terms: Option<String>,
        /// Status
        #[arg(long)]
        status: Option<String>,
    },
}

#[derive(Debug, Subcommand)]
pub enum PurchaseOrderCommands {
    /// Create purchase order
    Create {
        /// Supplier ID
        #[arg(short, long)]
        supplier_id: i32,
        /// Expected delivery date
        #[arg(long)]
        expected_date: Option<String>,
        /// Order notes
        #[arg(short, long)]
        notes: Option<String>,
        /// Items (format: product_id:quantity:unit_cost,...)
        #[arg(long)]
        items: String,
    },
    /// List purchase orders
    List {
        /// Search term
        #[arg(short, long)]
        search: Option<String>,
        /// Status filter
        #[arg(long)]
        status: Option<String>,
        /// Date from filter (YYYY-MM-DD)
        #[arg(long)]
        date_from: Option<String>,
        /// Date to filter (YYYY-MM-DD)
        #[arg(long)]
        date_to: Option<String>,
        /// Page number
        #[arg(long, default_value = "1")]
        page: u32,
        /// Items per page
        #[arg(long, default_value = "20")]
        per_page: u32,
    },
    /// Show purchase order details
    Show {
        /// Purchase order ID
        po_id: i32,
    },
    /// Approve purchase order
    Approve {
        /// Purchase order ID
        po_id: i32,
    },
    /// Receive purchase order items
    Receive {
        /// Purchase order ID
        po_id: i32,
        /// Received items (format: item_id:quantity,...)
        #[arg(long)]
        items: String,
    },
}

#[derive(Subcommand)]
pub enum SystemCommands {
    /// Initialize database
    Init,
    /// Show system status
    Status,
    /// Run database migrations
    Migrate,
    /// Create default admin user
    CreateAdmin,
}
