use clap::{Arg, ArgMatches, Command};
use tabled::{Table, Tabled};

use crate::core::result::CLIERPResult;
use crate::modules::inventory::{CategoryService, ProductService, StockAuditService, SupplierService, PurchaseOrderService, CategoryTreeNode, ProductWithCategory, PurchaseOrderItem, ReceiveItemData};
use crate::utils::formatting::{format_currency, format_datetime};
use crate::utils::pagination::PaginationParams;

pub fn inventory_command() -> Command {
    Command::new("inventory")
        .alias("inv")
        .about("Inventory management commands")
        .subcommand_required(true)
        .subcommands([
            category_commands(),
            product_commands(),
            stock_commands(),
            audit_commands(),
            supplier_commands(),
            purchase_commands(),
        ])
}

fn category_commands() -> Command {
    Command::new("category")
        .alias("cat")
        .about("Category management")
        .subcommand_required(true)
        .subcommands([
            Command::new("add")
                .about("Add a new category")
                .args([
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .required(true)
                        .help("Category name"),
                    Arg::new("description")
                        .long("description")
                        .short('d')
                        .help("Category description"),
                    Arg::new("parent")
                        .long("parent")
                        .short('p')
                        .value_parser(clap::value_parser!(i32))
                        .help("Parent category ID"),
                ]),
            Command::new("list")
                .about("List categories")
                .args([
                    Arg::new("parent")
                        .long("parent")
                        .short('p')
                        .value_parser(clap::value_parser!(i32))
                        .help("Filter by parent category ID"),
                    Arg::new("all")
                        .long("all")
                        .short('a')
                        .action(clap::ArgAction::SetTrue)
                        .help("Include inactive categories"),
                    Arg::new("page")
                        .long("page")
                        .value_parser(clap::value_parser!(usize))
                        .default_value("1")
                        .help("Page number"),
                    Arg::new("per_page")
                        .long("per-page")
                        .value_parser(clap::value_parser!(i64))
                        .default_value("20")
                        .help("Items per page"),
                ]),
            Command::new("tree")
                .about("Show category tree structure"),
            Command::new("update")
                .about("Update a category")
                .args([
                    Arg::new("id")
                        .long("id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Category ID"),
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .help("New category name"),
                    Arg::new("description")
                        .long("description")
                        .short('d')
                        .help("New description"),
                    Arg::new("parent")
                        .long("parent")
                        .short('p')
                        .value_parser(clap::value_parser!(i32))
                        .help("New parent category ID"),
                    Arg::new("activate")
                        .long("activate")
                        .action(clap::ArgAction::SetTrue)
                        .help("Activate category"),
                    Arg::new("deactivate")
                        .long("deactivate")
                        .action(clap::ArgAction::SetTrue)
                        .help("Deactivate category"),
                ]),
            Command::new("delete")
                .about("Delete a category")
                .args([
                    Arg::new("id")
                        .long("id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Category ID"),
                    Arg::new("force")
                        .long("force")
                        .short('f')
                        .action(clap::ArgAction::SetTrue)
                        .help("Force delete even if category has children or products"),
                ]),
        ])
}

fn product_commands() -> Command {
    Command::new("product")
        .alias("prod")
        .about("Product management")
        .subcommand_required(true)
        .subcommands([
            Command::new("add")
                .about("Add a new product")
                .args([
                    Arg::new("sku")
                        .long("sku")
                        .short('s')
                        .required(true)
                        .help("Product SKU"),
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .required(true)
                        .help("Product name"),
                    Arg::new("category")
                        .long("category")
                        .short('c')
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Category ID"),
                    Arg::new("price")
                        .long("price")
                        .short('p')
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Selling price (in cents)"),
                    Arg::new("cost_price")
                        .long("cost-price")
                        .value_parser(clap::value_parser!(i32))
                        .default_value("0")
                        .help("Cost price (in cents)"),
                    Arg::new("stock")
                        .long("stock")
                        .value_parser(clap::value_parser!(i32))
                        .default_value("0")
                        .help("Initial stock quantity"),
                    Arg::new("min_stock")
                        .long("min-stock")
                        .value_parser(clap::value_parser!(i32))
                        .default_value("0")
                        .help("Minimum stock level"),
                    Arg::new("max_stock")
                        .long("max-stock")
                        .value_parser(clap::value_parser!(i32))
                        .help("Maximum stock level"),
                    Arg::new("unit")
                        .long("unit")
                        .short('u')
                        .default_value("ea")
                        .help("Unit of measurement"),
                    Arg::new("description")
                        .long("description")
                        .short('d')
                        .help("Product description"),
                    Arg::new("barcode")
                        .long("barcode")
                        .short('b')
                        .help("Product barcode"),
                ]),
            Command::new("list")
                .about("List products")
                .args([
                    Arg::new("category")
                        .long("category")
                        .short('c')
                        .value_parser(clap::value_parser!(i32))
                        .help("Filter by category ID"),
                    Arg::new("search")
                        .long("search")
                        .short('s')
                        .help("Search by name or SKU"),
                    Arg::new("low_stock")
                        .long("low-stock")
                        .action(clap::ArgAction::SetTrue)
                        .help("Show only low stock items"),
                    Arg::new("all")
                        .long("all")
                        .short('a')
                        .action(clap::ArgAction::SetTrue)
                        .help("Include inactive products"),
                    Arg::new("page")
                        .long("page")
                        .value_parser(clap::value_parser!(usize))
                        .default_value("1")
                        .help("Page number"),
                    Arg::new("per_page")
                        .long("per-page")
                        .value_parser(clap::value_parser!(i64))
                        .default_value("20")
                        .help("Items per page"),
                ]),
            Command::new("show")
                .about("Show product details")
                .args([
                    Arg::new("sku")
                        .long("sku")
                        .short('s')
                        .help("Product SKU"),
                    Arg::new("id")
                        .long("id")
                        .value_parser(clap::value_parser!(i32))
                        .help("Product ID"),
                ]),
            Command::new("update")
                .about("Update a product")
                .args([
                    Arg::new("id")
                        .long("id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Product ID"),
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .help("New product name"),
                    Arg::new("category")
                        .long("category")
                        .short('c')
                        .value_parser(clap::value_parser!(i32))
                        .help("New category ID"),
                    Arg::new("price")
                        .long("price")
                        .short('p')
                        .value_parser(clap::value_parser!(i32))
                        .help("New selling price (in cents)"),
                    Arg::new("cost_price")
                        .long("cost-price")
                        .value_parser(clap::value_parser!(i32))
                        .help("New cost price (in cents)"),
                    Arg::new("min_stock")
                        .long("min-stock")
                        .value_parser(clap::value_parser!(i32))
                        .help("New minimum stock level"),
                    Arg::new("max_stock")
                        .long("max-stock")
                        .value_parser(clap::value_parser!(i32))
                        .help("New maximum stock level"),
                    Arg::new("unit")
                        .long("unit")
                        .short('u')
                        .help("New unit of measurement"),
                    Arg::new("description")
                        .long("description")
                        .short('d')
                        .help("New product description"),
                    Arg::new("barcode")
                        .long("barcode")
                        .short('b')
                        .help("New product barcode"),
                    Arg::new("activate")
                        .long("activate")
                        .action(clap::ArgAction::SetTrue)
                        .help("Activate product"),
                    Arg::new("deactivate")
                        .long("deactivate")
                        .action(clap::ArgAction::SetTrue)
                        .help("Deactivate product"),
                ]),
        ])
}

fn stock_commands() -> Command {
    Command::new("stock")
        .about("Stock management")
        .subcommand_required(true)
        .subcommands([
            Command::new("status")
                .about("Show overall stock status")
                .args([
                    Arg::new("category")
                        .long("category")
                        .short('c')
                        .value_parser(clap::value_parser!(i32))
                        .help("Filter by category ID"),
                    Arg::new("low_stock_only")
                        .long("low-stock")
                        .action(clap::ArgAction::SetTrue)
                        .help("Show only low stock items"),
                    Arg::new("out_of_stock")
                        .long("out-of-stock")
                        .action(clap::ArgAction::SetTrue)
                        .help("Show only out of stock items"),
                ]),
            Command::new("in")
                .about("Record stock in (receipt)")
                .args([
                    Arg::new("sku")
                        .long("sku")
                        .short('s')
                        .required(true)
                        .help("Product SKU"),
                    Arg::new("quantity")
                        .long("quantity")
                        .short('q')
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Quantity to add"),
                    Arg::new("unit_cost")
                        .long("unit-cost")
                        .value_parser(clap::value_parser!(i32))
                        .help("Unit cost (in cents)"),
                    Arg::new("reference")
                        .long("reference")
                        .short('r')
                        .help("Reference (PO number, supplier, etc.)"),
                    Arg::new("notes")
                        .long("notes")
                        .short('n')
                        .help("Additional notes"),
                ]),
            Command::new("out")
                .about("Record stock out (issue/sale)")
                .args([
                    Arg::new("sku")
                        .long("sku")
                        .short('s')
                        .required(true)
                        .help("Product SKU"),
                    Arg::new("quantity")
                        .long("quantity")
                        .short('q')
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Quantity to remove"),
                    Arg::new("reference")
                        .long("reference")
                        .short('r')
                        .help("Reference (order number, customer, etc.)"),
                    Arg::new("notes")
                        .long("notes")
                        .short('n')
                        .help("Additional notes"),
                ]),
            Command::new("adjust")
                .about("Adjust stock level")
                .args([
                    Arg::new("sku")
                        .long("sku")
                        .short('s')
                        .required(true)
                        .help("Product SKU"),
                    Arg::new("new_quantity")
                        .long("quantity")
                        .short('q')
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("New stock quantity"),
                    Arg::new("reason")
                        .long("reason")
                        .short('r')
                        .required(true)
                        .help("Reason for adjustment"),
                    Arg::new("notes")
                        .long("notes")
                        .short('n')
                        .help("Additional notes"),
                ]),
            Command::new("check")
                .about("Check stock levels")
                .args([
                    Arg::new("low_stock")
                        .long("low-stock")
                        .action(clap::ArgAction::SetTrue)
                        .help("Show only low stock items"),
                ]),
            Command::new("history")
                .about("Show stock movement history")
                .args([
                    Arg::new("sku")
                        .long("sku")
                        .short('s')
                        .required(true)
                        .help("Product SKU"),
                    Arg::new("page")
                        .long("page")
                        .value_parser(clap::value_parser!(usize))
                        .default_value("1")
                        .help("Page number"),
                    Arg::new("per_page")
                        .long("per-page")
                        .value_parser(clap::value_parser!(i64))
                        .default_value("20")
                        .help("Items per page"),
                ]),
        ])
}

fn audit_commands() -> Command {
    Command::new("audit")
        .about("Stock audit management")
        .subcommand_required(true)
        .subcommands([
            Command::new("create")
                .about("Create a new stock audit")
                .args([
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .required(true)
                        .help("Audit name"),
                    Arg::new("date")
                        .long("date")
                        .short('d')
                        .help("Audit date (YYYY-MM-DD, defaults to today)"),
                    Arg::new("notes")
                        .long("notes")
                        .help("Audit notes"),
                ]),
            Command::new("list")
                .about("List stock audits")
                .args([
                    Arg::new("status")
                        .long("status")
                        .short('s')
                        .help("Filter by status (pending, in_progress, completed, cancelled)"),
                    Arg::new("page")
                        .long("page")
                        .value_parser(clap::value_parser!(usize))
                        .default_value("1")
                        .help("Page number"),
                    Arg::new("per_page")
                        .long("per-page")
                        .value_parser(clap::value_parser!(i64))
                        .default_value("10")
                        .help("Items per page"),
                ]),
            Command::new("start")
                .about("Start an audit (creates audit items)")
                .args([
                    Arg::new("id")
                        .long("id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Audit ID"),
                ]),
            Command::new("count")
                .about("Record actual count for a product")
                .args([
                    Arg::new("audit_id")
                        .long("audit-id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Audit ID"),
                    Arg::new("sku")
                        .long("sku")
                        .short('s')
                        .required(true)
                        .help("Product SKU"),
                    Arg::new("quantity")
                        .long("quantity")
                        .short('q')
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Actual quantity counted"),
                    Arg::new("notes")
                        .long("notes")
                        .short('n')
                        .help("Count notes"),
                ]),
            Command::new("items")
                .about("Show audit items")
                .args([
                    Arg::new("id")
                        .long("id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Audit ID"),
                    Arg::new("variance_only")
                        .long("variance-only")
                        .action(clap::ArgAction::SetTrue)
                        .help("Show only items with variance"),
                    Arg::new("page")
                        .long("page")
                        .value_parser(clap::value_parser!(usize))
                        .default_value("1")
                        .help("Page number"),
                    Arg::new("per_page")
                        .long("per-page")
                        .value_parser(clap::value_parser!(i64))
                        .default_value("20")
                        .help("Items per page"),
                ]),
            Command::new("complete")
                .about("Complete an audit")
                .args([
                    Arg::new("id")
                        .long("id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Audit ID"),
                    Arg::new("apply_adjustments")
                        .long("apply-adjustments")
                        .action(clap::ArgAction::SetTrue)
                        .help("Apply stock adjustments based on audit results"),
                ]),
            Command::new("cancel")
                .about("Cancel an audit")
                .args([
                    Arg::new("id")
                        .long("id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Audit ID"),
                ]),
        ])
}

pub fn handle_inventory_command(matches: &ArgMatches) -> CLIERPResult<()> {
    match matches.subcommand() {
        Some(("category", sub_matches)) => handle_category_command(sub_matches),
        Some(("product", sub_matches)) => handle_product_command(sub_matches),
        Some(("stock", sub_matches)) => handle_stock_command(sub_matches),
        Some(("audit", sub_matches)) => handle_audit_command(sub_matches),
        _ => {
            eprintln!("Unknown inventory subcommand");
            Ok(())
        }
    }
}

fn handle_category_command(matches: &ArgMatches) -> CLIERPResult<()> {
    let service = CategoryService::new();

    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            let description = sub_matches.get_one::<String>("description");
            let parent_id = sub_matches.get_one::<i32>("parent").copied();

            let category = service.create_category(
                name,
                description.map(|s| s.as_str()),
                parent_id,
            )?;

            println!("✅ Category created:");
            println!("  ID: {}", category.id);
            println!("  Name: {}", category.name);
            if let Some(desc) = &category.description {
                println!("  Description: {}", desc);
            }
            if let Some(parent_id) = category.parent_id {
                println!("  Parent ID: {}", parent_id);
            }
        }
        Some(("list", sub_matches)) => {
            let parent_id = sub_matches.get_one::<i32>("parent").copied();
            let active_only = !sub_matches.get_flag("all");
            let page = *sub_matches.get_one::<usize>("page").unwrap();
            let per_page = *sub_matches.get_one::<i64>("per_page").unwrap();

            let pagination = PaginationParams::new(page, per_page);
            let result = service.list_categories(&pagination, parent_id, active_only)?;

            if result.data.is_empty() {
                println!("No categories found.");
                return Ok(());
            }

            let pagination_info = (result.current_page(), result.total_pages(), result.total_items());

            let table_data: Vec<CategoryTableRow> = result.data
                .into_iter()
                .map(|cat| CategoryTableRow {
                    id: cat.id,
                    name: cat.name,
                    description: cat.description.unwrap_or_else(|| "-".to_string()),
                    parent_id: cat.parent_id.map_or_else(|| "-".to_string(), |id| id.to_string()),
                    active: if cat.is_active { "Yes" } else { "No" }.to_string(),
                    created_at: format_datetime(&cat.created_at),
                })
                .collect();

            let table = Table::new(table_data);
            println!("{}", table);
            println!("\nPage {} of {} (Total: {} categories)",
                pagination_info.0, pagination_info.1, pagination_info.2);
        }
        Some(("tree", _)) => {
            let tree = service.get_category_tree()?;
            println!("Category Tree:");
            print_category_tree(&tree, 0);
        }
        Some(("update", sub_matches)) => {
            let id = *sub_matches.get_one::<i32>("id").unwrap();
            let name = sub_matches.get_one::<String>("name");
            let description = sub_matches.get_one::<String>("description");
            let parent_id = sub_matches.get_one::<i32>("parent").copied();
            let activate = sub_matches.get_flag("activate");
            let deactivate = sub_matches.get_flag("deactivate");

            let is_active = if activate {
                Some(true)
            } else if deactivate {
                Some(false)
            } else {
                None
            };

            let category = service.update_category(
                id,
                name.map(|s| s.as_str()),
                description.map(|s| Some(s.as_str())),
                Some(parent_id),
                is_active,
            )?;

            println!("✅ Category updated:");
            println!("  ID: {}", category.id);
            println!("  Name: {}", category.name);
            if let Some(desc) = &category.description {
                println!("  Description: {}", desc);
            }
            println!("  Active: {}", if category.is_active { "Yes" } else { "No" });
        }
        Some(("delete", sub_matches)) => {
            let id = *sub_matches.get_one::<i32>("id").unwrap();
            let force = sub_matches.get_flag("force");

            service.delete_category(id, force)?;
            println!("✅ Category deleted successfully");
        }
        _ => {
            eprintln!("Unknown category subcommand");
        }
    }

    Ok(())
}

fn handle_product_command(matches: &ArgMatches) -> CLIERPResult<()> {
    let service = ProductService::new();

    match matches.subcommand() {
        Some(("add", sub_matches)) => {
            let sku = sub_matches.get_one::<String>("sku").unwrap();
            let name = sub_matches.get_one::<String>("name").unwrap();
            let category_id = *sub_matches.get_one::<i32>("category").unwrap();
            let price = *sub_matches.get_one::<i32>("price").unwrap();
            let cost_price = *sub_matches.get_one::<i32>("cost_price").unwrap();
            let initial_stock = *sub_matches.get_one::<i32>("stock").unwrap();
            let min_stock = *sub_matches.get_one::<i32>("min_stock").unwrap();
            let max_stock = sub_matches.get_one::<i32>("max_stock").copied();
            let unit = sub_matches.get_one::<String>("unit").unwrap();
            let description = sub_matches.get_one::<String>("description");
            let barcode = sub_matches.get_one::<String>("barcode");

            let product = service.create_product(
                sku,
                name,
                description.map(|s| s.as_str()),
                category_id,
                price,
                cost_price,
                initial_stock,
                min_stock,
                max_stock,
                unit,
                barcode.map(|s| s.as_str()),
            )?;

            println!("✅ Product created:");
            println!("  ID: {}", product.id);
            println!("  SKU: {}", product.sku);
            println!("  Name: {}", product.name);
            println!("  Category ID: {}", product.category_id);
            println!("  Price: {}", format_currency(product.price));
            println!("  Stock: {} {}", product.current_stock, product.unit);
        }
        Some(("list", sub_matches)) => {
            let category_id = sub_matches.get_one::<i32>("category").copied();
            let search_term = sub_matches.get_one::<String>("search");
            let low_stock_only = sub_matches.get_flag("low_stock");
            let active_only = !sub_matches.get_flag("all");
            let page = *sub_matches.get_one::<usize>("page").unwrap();
            let per_page = *sub_matches.get_one::<i64>("per_page").unwrap();

            let pagination = PaginationParams::new(page, per_page);
            let result = service.list_products(
                &pagination,
                category_id,
                active_only,
                search_term.map(|s| s.as_str()),
                low_stock_only,
            )?;

            if result.data.is_empty() {
                println!("No products found.");
                return Ok(());
            }

            let pagination_info = (result.current_page(), result.total_pages(), result.total_items());

            let table_data: Vec<ProductTableRow> = result.data
                .into_iter()
                .map(|prod_with_cat| ProductTableRow {
                    id: prod_with_cat.product.id,
                    sku: prod_with_cat.product.sku,
                    name: prod_with_cat.product.name,
                    category: prod_with_cat.category.name,
                    price: format_currency(prod_with_cat.product.price),
                    stock: format!("{} {}", prod_with_cat.product.current_stock, prod_with_cat.product.unit),
                    status: if prod_with_cat.product.current_stock <= prod_with_cat.product.min_stock_level {
                        "LOW".to_string()
                    } else if prod_with_cat.product.is_active {
                        "Active".to_string()
                    } else {
                        "Inactive".to_string()
                    },
                })
                .collect();

            let table = Table::new(table_data);
            println!("{}", table);
            println!("\nPage {} of {} (Total: {} products)",
                pagination_info.0, pagination_info.1, pagination_info.2);
        }
        _ => {
            eprintln!("Unknown product subcommand");
        }
    }

    Ok(())
}

#[derive(Tabled)]
struct CategoryTableRow {
    #[tabled(rename = "ID")]
    id: i32,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Description")]
    description: String,
    #[tabled(rename = "Parent ID")]
    parent_id: String,
    #[tabled(rename = "Active")]
    active: String,
    #[tabled(rename = "Created")]
    created_at: String,
}

#[derive(Tabled)]
struct ProductTableRow {
    #[tabled(rename = "ID")]
    id: i32,
    #[tabled(rename = "SKU")]
    sku: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Category")]
    category: String,
    #[tabled(rename = "Price")]
    price: String,
    #[tabled(rename = "Stock")]
    stock: String,
    #[tabled(rename = "Status")]
    status: String,
}

fn print_category_tree(nodes: &[CategoryTreeNode], depth: usize) {
    let indent = "  ".repeat(depth);
    for node in nodes {
        println!("{}├─ {} (ID: {})", indent, node.category.name, node.category.id);
        if !node.children.is_empty() {
            print_category_tree(&node.children, depth + 1);
        }
    }
}

fn handle_stock_command(matches: &ArgMatches) -> CLIERPResult<()> {
    let product_service = ProductService::new();

    match matches.subcommand() {
        Some(("status", sub_matches)) => {
            let category_id = sub_matches.get_one::<i32>("category").copied();
            let low_stock_only = sub_matches.get_flag("low_stock_only");
            let out_of_stock = sub_matches.get_flag("out_of_stock");

            let pagination = PaginationParams::new(1, 100); // Show more items for status
            let result = product_service.list_products(
                &pagination,
                category_id,
                true, // active only
                None,
                low_stock_only || out_of_stock,
            )?;

            if result.data.is_empty() {
                println!("No products found matching criteria.");
                return Ok(());
            }

            let mut filtered_products = result.data;
            if out_of_stock {
                filtered_products.retain(|p| p.product.current_stock == 0);
            }

            if filtered_products.is_empty() {
                println!("No products found matching criteria.");
                return Ok(());
            }

            let table_data: Vec<StockStatusRow> = filtered_products
                .into_iter()
                .map(|prod_with_cat| StockStatusRow {
                    sku: prod_with_cat.product.sku,
                    name: prod_with_cat.product.name,
                    current_stock: prod_with_cat.product.current_stock,
                    min_level: prod_with_cat.product.min_stock_level,
                    max_level: prod_with_cat.product.max_stock_level.unwrap_or(0),
                    unit: prod_with_cat.product.unit,
                    status: if prod_with_cat.product.current_stock == 0 {
                        "OUT OF STOCK".to_string()
                    } else if prod_with_cat.product.current_stock <= prod_with_cat.product.min_stock_level {
                        "LOW STOCK".to_string()
                    } else {
                        "OK".to_string()
                    },
                })
                .collect();

            let table = Table::new(table_data);
            println!("{}", table);
        }
        Some(("in", sub_matches)) => {
            let sku = sub_matches.get_one::<String>("sku").unwrap();
            let quantity = *sub_matches.get_one::<i32>("quantity").unwrap();
            let unit_cost = sub_matches.get_one::<i32>("unit_cost").copied();
            let reference = sub_matches.get_one::<String>("reference");
            let notes = sub_matches.get_one::<String>("notes");

            let product = product_service.get_product_by_sku(sku)?
                .ok_or_else(|| crate::core::error::CLIERPError::ValidationError(
                    format!("Product with SKU '{}' not found", sku)
                ))?;

            let updated_product = product_service.update_stock(
                product.id,
                quantity,
                "in",
                unit_cost,
                reference.map(|s| s.as_str()),
                None,
                notes.map(|s| s.as_str()),
                None, // TODO: Add user context
            )?;

            println!("✅ Stock received:");
            println!("  Product: {} ({})", updated_product.name, updated_product.sku);
            println!("  Quantity: +{} {}", quantity, updated_product.unit);
            println!("  New stock: {} {}", updated_product.current_stock, updated_product.unit);
        }
        Some(("out", sub_matches)) => {
            let sku = sub_matches.get_one::<String>("sku").unwrap();
            let quantity = *sub_matches.get_one::<i32>("quantity").unwrap();
            let reference = sub_matches.get_one::<String>("reference");
            let notes = sub_matches.get_one::<String>("notes");

            let product = product_service.get_product_by_sku(sku)?
                .ok_or_else(|| crate::core::error::CLIERPError::ValidationError(
                    format!("Product with SKU '{}' not found", sku)
                ))?;

            let updated_product = product_service.update_stock(
                product.id,
                -quantity,
                "out",
                None,
                reference.map(|s| s.as_str()),
                None,
                notes.map(|s| s.as_str()),
                None, // TODO: Add user context
            )?;

            println!("✅ Stock issued:");
            println!("  Product: {} ({})", updated_product.name, updated_product.sku);
            println!("  Quantity: -{} {}", quantity, updated_product.unit);
            println!("  New stock: {} {}", updated_product.current_stock, updated_product.unit);
        }
        Some(("adjust", sub_matches)) => {
            let sku = sub_matches.get_one::<String>("sku").unwrap();
            let new_quantity = *sub_matches.get_one::<i32>("new_quantity").unwrap();
            let reason = sub_matches.get_one::<String>("reason").unwrap();
            let notes = sub_matches.get_one::<String>("notes");

            let product = product_service.get_product_by_sku(sku)?
                .ok_or_else(|| crate::core::error::CLIERPError::ValidationError(
                    format!("Product with SKU '{}' not found", sku)
                ))?;

            let adjustment = new_quantity - product.current_stock;
            let adjustment_notes = format!("Adjustment: {} - {}", reason, notes.map_or("", |s| s.as_str()));

            let updated_product = product_service.update_stock(
                product.id,
                adjustment,
                "adjustment",
                None,
                Some("manual_adjustment"),
                None,
                Some(&adjustment_notes),
                None, // TODO: Add user context
            )?;

            println!("✅ Stock adjusted:");
            println!("  Product: {} ({})", updated_product.name, updated_product.sku);
            println!("  Previous: {} {}", product.current_stock, updated_product.unit);
            println!("  New stock: {} {}", updated_product.current_stock, updated_product.unit);
            println!("  Adjustment: {:+} {}", adjustment, updated_product.unit);
            println!("  Reason: {}", reason);
        }
        Some(("check", sub_matches)) => {
            let low_stock_only = sub_matches.get_flag("low_stock");

            if low_stock_only {
                let low_stock_products = product_service.get_low_stock_products()?;

                if low_stock_products.is_empty() {
                    println!("No low stock products found.");
                    return Ok(());
                }

                let table_data: Vec<StockStatusRow> = low_stock_products
                    .into_iter()
                    .map(|prod_with_cat| StockStatusRow {
                        sku: prod_with_cat.product.sku,
                        name: prod_with_cat.product.name,
                        current_stock: prod_with_cat.product.current_stock,
                        min_level: prod_with_cat.product.min_stock_level,
                        max_level: prod_with_cat.product.max_stock_level.unwrap_or(0),
                        unit: prod_with_cat.product.unit,
                        status: "LOW STOCK".to_string(),
                    })
                    .collect();

                let count = table_data.len();
                let table = Table::new(table_data);
                println!("{}", table);
                println!("\n⚠️  {} products have low stock levels", count);
            }
        }
        Some(("history", sub_matches)) => {
            let sku = sub_matches.get_one::<String>("sku").unwrap();
            let page = *sub_matches.get_one::<usize>("page").unwrap();
            let per_page = *sub_matches.get_one::<i64>("per_page").unwrap();

            let product = product_service.get_product_by_sku(sku)?
                .ok_or_else(|| crate::core::error::CLIERPError::ValidationError(
                    format!("Product with SKU '{}' not found", sku)
                ))?;

            let pagination = PaginationParams::new(page, per_page);
            let result = product_service.get_stock_movements(product.id, &pagination)?;

            if result.data.is_empty() {
                println!("No stock movements found for product {}.", sku);
                return Ok(());
            }

            let pagination_info = (result.current_page(), result.total_pages(), result.total_items());

            let table_data: Vec<StockMovementRow> = result.data
                .into_iter()
                .map(|movement| StockMovementRow {
                    date: format_datetime(&movement.movement_date),
                    type_: movement.movement_type,
                    quantity: format!("{:+}", movement.quantity),
                    reference: movement.reference_type.unwrap_or_else(|| "-".to_string()),
                    notes: movement.notes.unwrap_or_else(|| "-".to_string()),
                })
                .collect();

            println!("Stock Movement History for {} ({})", product.name, product.sku);
            let table = Table::new(table_data);
            println!("{}", table);
            println!("\nPage {} of {} (Total: {} movements)",
                pagination_info.0, pagination_info.1, pagination_info.2);
        }
        _ => {
            eprintln!("Unknown stock subcommand");
        }
    }

    Ok(())
}

fn handle_audit_command(matches: &ArgMatches) -> CLIERPResult<()> {
    let audit_service = StockAuditService::new();

    match matches.subcommand() {
        Some(("create", sub_matches)) => {
            let name = sub_matches.get_one::<String>("name").unwrap();
            let date_str = sub_matches.get_one::<String>("date");
            let notes = sub_matches.get_one::<String>("notes");

            let audit_date = if let Some(date_str) = date_str {
                chrono::NaiveDate::parse_from_str(date_str, "%Y-%m-%d")
                    .map_err(|_| crate::core::error::CLIERPError::ValidationError(
                        "Invalid date format. Use YYYY-MM-DD".to_string()
                    ))?
            } else {
                chrono::Local::now().naive_local().date()
            };

            let audit = audit_service.create_audit(
                name,
                audit_date,
                None, // TODO: Add user context
                notes.map(|s| s.as_str()),
            )?;

            println!("✅ Stock audit created:");
            println!("  ID: {}", audit.id);
            println!("  Name: {}", audit.audit_name);
            println!("  Date: {}", audit.audit_date);
            println!("  Status: {}", audit.status);
        }
        Some(("list", sub_matches)) => {
            let status_filter = sub_matches.get_one::<String>("status");
            let page = *sub_matches.get_one::<usize>("page").unwrap();
            let per_page = *sub_matches.get_one::<i64>("per_page").unwrap();

            let pagination = PaginationParams::new(page, per_page);
            let result = audit_service.list_audits(&pagination, status_filter.map(|s| s.as_str()))?;

            if result.data.is_empty() {
                println!("No audits found.");
                return Ok(());
            }

            let pagination_info = (result.current_page(), result.total_pages(), result.total_items());

            let table_data: Vec<AuditTableRow> = result.data
                .into_iter()
                .map(|audit| AuditTableRow {
                    id: audit.id,
                    name: audit.audit_name,
                    date: audit.audit_date.to_string(),
                    status: audit.status,
                    conducted_by: audit.conducted_by.map_or_else(|| "-".to_string(), |id| id.to_string()),
                    created_at: format_datetime(&audit.created_at),
                })
                .collect();

            let table = Table::new(table_data);
            println!("{}", table);
            println!("\nPage {} of {} (Total: {} audits)",
                pagination_info.0, pagination_info.1, pagination_info.2);
        }
        Some(("start", sub_matches)) => {
            let id = *sub_matches.get_one::<i32>("id").unwrap();

            let audit_items = audit_service.start_audit(id)?;

            println!("✅ Audit started:");
            println!("  Created {} audit items", audit_items.len());
            println!("  Use 'clierp inv audit count' to record actual counts");
        }
        _ => {
            eprintln!("Unknown audit subcommand");
        }
    }

    Ok(())
}

#[derive(Tabled)]
struct StockStatusRow {
    #[tabled(rename = "SKU")]
    sku: String,
    #[tabled(rename = "Product")]
    name: String,
    #[tabled(rename = "Current")]
    current_stock: i32,
    #[tabled(rename = "Min")]
    min_level: i32,
    #[tabled(rename = "Max")]
    max_level: i32,
    #[tabled(rename = "Unit")]
    unit: String,
    #[tabled(rename = "Status")]
    status: String,
}

#[derive(Tabled)]
struct StockMovementRow {
    #[tabled(rename = "Date")]
    date: String,
    #[tabled(rename = "Type")]
    type_: String,
    #[tabled(rename = "Quantity")]
    quantity: String,
    #[tabled(rename = "Reference")]
    reference: String,
    #[tabled(rename = "Notes")]
    notes: String,
}

#[derive(Tabled)]
struct AuditTableRow {
    #[tabled(rename = "ID")]
    id: i32,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Date")]
    date: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Conducted By")]
    conducted_by: String,
    #[tabled(rename = "Created")]
    created_at: String,
}