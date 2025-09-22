use clap::{Arg, ArgMatches, Command};
use tabled::{Table, Tabled};

use crate::core::result::CLIERPResult;
use crate::modules::inventory::{SupplierService, PurchaseOrderService, PurchaseOrderItem, ReceiveItemData};
use crate::utils::formatting::{format_currency, format_datetime};
use crate::utils::pagination::PaginationParams;

pub fn purchase_command() -> Command {
    Command::new("purchase")
        .alias("po")
        .about("Purchase management commands")
        .subcommand_required(true)
        .subcommands([
            supplier_commands(),
            purchase_order_commands(),
        ])
}

fn supplier_commands() -> Command {
    Command::new("supplier")
        .alias("sup")
        .about("Supplier management")
        .subcommand_required(true)
        .subcommands([
            Command::new("add")
                .about("Add a new supplier")
                .args([
                    Arg::new("code")
                        .long("code")
                        .short('c')
                        .required(true)
                        .help("Supplier code"),
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .required(true)
                        .help("Supplier name"),
                    Arg::new("contact")
                        .long("contact")
                        .help("Contact person"),
                    Arg::new("email")
                        .long("email")
                        .short('e')
                        .help("Email address"),
                    Arg::new("phone")
                        .long("phone")
                        .short('p')
                        .help("Phone number"),
                    Arg::new("address")
                        .long("address")
                        .short('a')
                        .help("Address"),
                    Arg::new("payment-terms")
                        .long("payment-terms")
                        .help("Payment terms"),
                ]),
            Command::new("list")
                .about("List suppliers")
                .args([
                    Arg::new("search")
                        .long("search")
                        .short('s')
                        .help("Search by name or code"),
                    Arg::new("status")
                        .long("status")
                        .value_parser(["active", "inactive", "blacklisted"])
                        .help("Filter by status"),
                    Arg::new("page")
                        .long("page")
                        .value_parser(clap::value_parser!(u32))
                        .default_value("1")
                        .help("Page number"),
                    Arg::new("per-page")
                        .long("per-page")
                        .value_parser(clap::value_parser!(u32))
                        .default_value("20")
                        .help("Items per page"),
                ]),
            Command::new("show")
                .about("Show supplier details")
                .arg(
                    Arg::new("supplier_id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Supplier ID")
                ),
            Command::new("update")
                .about("Update supplier")
                .args([
                    Arg::new("supplier_id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Supplier ID"),
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .help("Supplier name"),
                    Arg::new("contact")
                        .long("contact")
                        .help("Contact person"),
                    Arg::new("email")
                        .long("email")
                        .short('e')
                        .help("Email address"),
                    Arg::new("phone")
                        .long("phone")
                        .short('p')
                        .help("Phone number"),
                    Arg::new("address")
                        .long("address")
                        .short('a')
                        .help("Address"),
                    Arg::new("payment-terms")
                        .long("payment-terms")
                        .help("Payment terms"),
                    Arg::new("status")
                        .long("status")
                        .value_parser(["active", "inactive", "blacklisted"])
                        .help("Supplier status"),
                ]),
        ])
}

fn purchase_order_commands() -> Command {
    Command::new("order")
        .alias("ord")
        .about("Purchase order management")
        .subcommand_required(true)
        .subcommands([
            Command::new("create")
                .about("Create a new purchase order")
                .args([
                    Arg::new("supplier_id")
                        .long("supplier-id")
                        .short('s')
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Supplier ID"),
                    Arg::new("expected-date")
                        .long("expected-date")
                        .help("Expected delivery date (YYYY-MM-DD)"),
                    Arg::new("notes")
                        .long("notes")
                        .short('n')
                        .help("Order notes"),
                    Arg::new("items")
                        .long("items")
                        .required(true)
                        .help("Items in format: product_id:quantity:unit_cost,product_id:quantity:unit_cost,..."),
                ]),
            Command::new("list")
                .about("List purchase orders")
                .args([
                    Arg::new("search")
                        .long("search")
                        .short('s')
                        .help("Search by PO number or supplier"),
                    Arg::new("status")
                        .long("status")
                        .value_parser(["pending", "approved", "sent", "received", "cancelled"])
                        .help("Filter by status"),
                    Arg::new("date-from")
                        .long("date-from")
                        .help("Filter from date (YYYY-MM-DD)"),
                    Arg::new("date-to")
                        .long("date-to")
                        .help("Filter to date (YYYY-MM-DD)"),
                    Arg::new("page")
                        .long("page")
                        .value_parser(clap::value_parser!(u32))
                        .default_value("1")
                        .help("Page number"),
                    Arg::new("per-page")
                        .long("per-page")
                        .value_parser(clap::value_parser!(u32))
                        .default_value("20")
                        .help("Items per page"),
                ]),
            Command::new("show")
                .about("Show purchase order details")
                .arg(
                    Arg::new("po_id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Purchase order ID")
                ),
            Command::new("approve")
                .about("Approve purchase order")
                .arg(
                    Arg::new("po_id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Purchase order ID")
                ),
            Command::new("receive")
                .about("Receive purchase order items")
                .args([
                    Arg::new("po_id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Purchase order ID"),
                    Arg::new("items")
                        .long("items")
                        .required(true)
                        .help("Received items in format: item_id:quantity,item_id:quantity,..."),
                ]),
        ])
}

pub fn handle_purchase_command(matches: &ArgMatches) -> CLIERPResult<()> {
    match matches.subcommand() {
        Some(("supplier", sub_matches)) => handle_supplier_command(sub_matches),
        Some(("order", sub_matches)) => handle_purchase_order_command(sub_matches),
        _ => {
            println!("No subcommand provided. Use --help for available commands.");
            Ok(())
        }
    }
}

fn handle_supplier_command(matches: &ArgMatches) -> CLIERPResult<()> {
    match matches.subcommand() {
        Some(("add", sub_matches)) => handle_supplier_add(sub_matches),
        Some(("list", sub_matches)) => handle_supplier_list(sub_matches),
        Some(("show", sub_matches)) => handle_supplier_show(sub_matches),
        Some(("update", sub_matches)) => handle_supplier_update(sub_matches),
        _ => {
            println!("No subcommand provided. Use --help for available commands.");
            Ok(())
        }
    }
}

fn handle_purchase_order_command(matches: &ArgMatches) -> CLIERPResult<()> {
    match matches.subcommand() {
        Some(("create", sub_matches)) => handle_purchase_create(sub_matches),
        Some(("list", sub_matches)) => handle_purchase_list(sub_matches),
        Some(("show", sub_matches)) => handle_purchase_show(sub_matches),
        Some(("approve", sub_matches)) => handle_purchase_approve(sub_matches),
        Some(("receive", sub_matches)) => handle_purchase_receive(sub_matches),
        _ => {
            println!("No subcommand provided. Use --help for available commands.");
            Ok(())
        }
    }
}

fn handle_supplier_add(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let code = matches.get_one::<String>("code").unwrap();
    let name = matches.get_one::<String>("name").unwrap();
    let contact = matches.get_one::<String>("contact").map(|s| s.as_str());
    let email = matches.get_one::<String>("email").map(|s| s.as_str());
    let phone = matches.get_one::<String>("phone").map(|s| s.as_str());
    let address = matches.get_one::<String>("address").map(|s| s.as_str());
    let payment_terms = matches.get_one::<String>("payment-terms").map(|s| s.as_str());

    let supplier = SupplierService::create_supplier(
        &mut conn,
        code,
        name,
        contact,
        email,
        phone,
        address,
        payment_terms,
    )?;

    println!("✅ Supplier created successfully!");
    println!("ID: {}", supplier.id);
    println!("Code: {}", supplier.supplier_code);
    println!("Name: {}", supplier.name);

    Ok(())
}

fn handle_supplier_list(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let search = matches.get_one::<String>("search").map(|s| s.as_str());
    let status = matches.get_one::<String>("status").map(|s| s.as_str());
    let page = *matches.get_one::<u32>("page").unwrap();
    let per_page = *matches.get_one::<u32>("per-page").unwrap();

    let filters = crate::utils::filters::FilterOptions {
        search: search.map(|s| s.to_string()),
        status: status.map(|s| s.to_string()),
        ..Default::default()
    };

    let pagination = PaginationParams::new(page as usize, per_page as i64);
    let result = SupplierService::list_suppliers(&mut conn, &filters, &pagination)?;

    if result.data.is_empty() {
        println!("No suppliers found.");
        return Ok(());
    }

    let rows: Vec<SupplierTableRow> = result.data
        .into_iter()
        .map(|supplier| SupplierTableRow {
            id: supplier.id,
            code: supplier.supplier_code,
            name: supplier.name,
            contact: supplier.contact_person.unwrap_or_else(|| "-".to_string()),
            phone: supplier.phone.unwrap_or_else(|| "-".to_string()),
            status: supplier.status,
            created_at: format_datetime(&supplier.created_at),
        })
        .collect();

    let table = Table::new(rows);
    println!("{}", table);
    println!("Page {} of {} ({} total)", result.pagination.current_page, result.pagination.total_pages, result.pagination.total_count);

    Ok(())
}

fn handle_supplier_show(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let supplier_id = *matches.get_one::<i32>("supplier_id").unwrap();

    let supplier = SupplierService::get_supplier_by_id(&mut conn, supplier_id)?
        .ok_or_else(|| crate::core::error::CLIERPError::NotFound(format!("Supplier with ID {} not found", supplier_id)))?;

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
    println!("Created: {}", format_datetime(&supplier.created_at));
    println!("Updated: {}", format_datetime(&supplier.updated_at));
    println!();
    println!("Statistics:");
    println!("Total Orders: {}", stats.total_orders);
    println!("Pending Orders: {}", stats.pending_orders);
    println!("Total Amount: {}", format_currency(stats.total_amount));

    Ok(())
}

fn handle_supplier_update(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let supplier_id = *matches.get_one::<i32>("supplier_id").unwrap();
    let name = matches.get_one::<String>("name").map(|s| s.as_str());
    let contact = matches.get_one::<String>("contact").map(|s| Some(s.as_str()));
    let email = matches.get_one::<String>("email").map(|s| Some(s.as_str()));
    let phone = matches.get_one::<String>("phone").map(|s| Some(s.as_str()));
    let address = matches.get_one::<String>("address").map(|s| Some(s.as_str()));
    let payment_terms = matches.get_one::<String>("payment-terms").map(|s| Some(s.as_str()));
    let status = matches.get_one::<String>("status").map(|s| match s.as_str() {
        "active" => crate::database::SupplierStatus::Active,
        "inactive" => crate::database::SupplierStatus::Inactive,
        "blacklisted" => crate::database::SupplierStatus::Blacklisted,
        _ => crate::database::SupplierStatus::Active,
    });

    let supplier = SupplierService::update_supplier(
        &mut conn,
        supplier_id,
        name,
        contact,
        email,
        phone,
        address,
        payment_terms,
        status,
    )?;

    println!("✅ Supplier updated successfully!");
    println!("ID: {}", supplier.id);
    println!("Name: {}", supplier.name);
    println!("Status: {}", supplier.status);

    Ok(())
}

fn handle_purchase_create(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let supplier_id = *matches.get_one::<i32>("supplier_id").unwrap();
    let expected_date = matches.get_one::<String>("expected-date").map(|s| s.parse().unwrap());
    let notes = matches.get_one::<String>("notes").map(|s| s.as_str());
    let items_str = matches.get_one::<String>("items").unwrap();

    // Parse items string
    let items: Result<Vec<PurchaseOrderItem>, _> = items_str
        .split(',')
        .map(|item| {
            let parts: Vec<&str> = item.split(':').collect();
            if parts.len() != 3 {
                return Err(crate::core::error::CLIERPError::ValidationError(
                    "Items format should be: product_id:quantity:unit_cost".to_string()
                ));
            }
            Ok(PurchaseOrderItem {
                product_id: parts[0].parse().map_err(|_| crate::core::error::CLIERPError::ValidationError("Invalid product ID".to_string()))?,
                quantity: parts[1].parse().map_err(|_| crate::core::error::CLIERPError::ValidationError("Invalid quantity".to_string()))?,
                unit_cost: parts[2].parse().map_err(|_| crate::core::error::CLIERPError::ValidationError("Invalid unit cost".to_string()))?,
            })
        })
        .collect();

    let items = items?;
    let current_user_id = Some(1); // TODO: Get from session

    let po_with_details = PurchaseOrderService::create_purchase_order(
        &mut conn,
        supplier_id,
        expected_date,
        notes,
        items,
        current_user_id,
    )?;

    println!("✅ Purchase order created successfully!");
    println!("PO Number: {}", po_with_details.purchase_order.po_number);
    println!("Supplier: {}", po_with_details.supplier.name);
    println!("Total Amount: {}", format_currency(po_with_details.purchase_order.total_amount));
    println!("Items: {} products", po_with_details.items.len());

    Ok(())
}

fn handle_purchase_list(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let search = matches.get_one::<String>("search").map(|s| s.as_str());
    let status = matches.get_one::<String>("status").map(|s| s.as_str());
    let date_from = matches.get_one::<String>("date-from").map(|s| s.parse().unwrap());
    let date_to = matches.get_one::<String>("date-to").map(|s| s.parse().unwrap());
    let page = *matches.get_one::<u32>("page").unwrap();
    let per_page = *matches.get_one::<u32>("per-page").unwrap();

    let filters = crate::utils::filters::FilterOptions {
        search: search.map(|s| s.to_string()),
        status: status.map(|s| s.to_string()),
        date_from,
        date_to,
        ..Default::default()
    };

    let pagination = PaginationParams::new(page as usize, per_page as i64);
    let result = PurchaseOrderService::list_purchase_orders(&mut conn, &filters, &pagination)?;

    if result.data.is_empty() {
        println!("No purchase orders found.");
        return Ok(());
    }

    let rows: Vec<PurchaseOrderTableRow> = result.data
        .into_iter()
        .map(|po| PurchaseOrderTableRow {
            id: po.id,
            po_number: po.po_number,
            supplier: po.supplier_name,
            date: po.order_date.format("%Y-%m-%d").to_string(),
            status: po.status,
            items_count: po.items_count,
            total_amount: format_currency(po.total_amount),
        })
        .collect();

    let table = Table::new(rows);
    println!("{}", table);
    println!("Page {} of {} ({} total)", result.pagination.current_page, result.pagination.total_pages, result.pagination.total_count);

    Ok(())
}

fn handle_purchase_show(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let po_id = *matches.get_one::<i32>("po_id").unwrap();

    let po_details = PurchaseOrderService::get_purchase_order_with_details(&mut conn, po_id)?;

    println!("Purchase Order Details:");
    println!("PO Number: {}", po_details.purchase_order.po_number);
    println!("Supplier: {} ({})", po_details.supplier.name, po_details.supplier.supplier_code);
    println!("Order Date: {}", po_details.purchase_order.order_date);
    println!("Expected Date: {}", po_details.purchase_order.expected_date.map(|d| d.to_string()).unwrap_or_else(|| "-".to_string()));
    println!("Status: {}", po_details.purchase_order.status);
    println!("Total Amount: {}", format_currency(po_details.purchase_order.total_amount));
    if let Some(notes) = &po_details.purchase_order.notes {
        println!("Notes: {}", notes);
    }
    println!();

    println!("Items:");
    let item_rows: Vec<PurchaseItemTableRow> = po_details.items
        .into_iter()
        .map(|item| PurchaseItemTableRow {
            product: format!("{} ({})", item.product_name, item.product_sku),
            quantity: item.purchase_item.quantity,
            unit_cost: format_currency(item.purchase_item.unit_cost),
            total_cost: format_currency(item.purchase_item.total_cost),
            received: item.purchase_item.received_quantity,
            status: item.purchase_item.status,
        })
        .collect();

    let items_table = Table::new(item_rows);
    println!("{}", items_table);

    Ok(())
}

fn handle_purchase_approve(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let po_id = *matches.get_one::<i32>("po_id").unwrap();
    let current_user_id = 1; // TODO: Get from session

    let purchase_order = PurchaseOrderService::approve_purchase_order(&mut conn, po_id, current_user_id)?;

    println!("✅ Purchase order approved successfully!");
    println!("PO Number: {}", purchase_order.po_number);
    println!("Status: {}", purchase_order.status);

    Ok(())
}

fn handle_purchase_receive(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let po_id = *matches.get_one::<i32>("po_id").unwrap();
    let items_str = matches.get_one::<String>("items").unwrap();
    let current_user_id = Some(1); // TODO: Get from session

    // Parse received items string
    let received_items: Result<Vec<ReceiveItemData>, _> = items_str
        .split(',')
        .map(|item| {
            let parts: Vec<&str> = item.split(':').collect();
            if parts.len() != 2 {
                return Err(crate::core::error::CLIERPError::ValidationError(
                    "Items format should be: item_id:quantity".to_string()
                ));
            }
            Ok(ReceiveItemData {
                item_id: parts[0].parse().map_err(|_| crate::core::error::CLIERPError::ValidationError("Invalid item ID".to_string()))?,
                quantity: parts[1].parse().map_err(|_| crate::core::error::CLIERPError::ValidationError("Invalid quantity".to_string()))?,
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

    Ok(())
}

#[derive(Tabled)]
struct SupplierTableRow {
    #[tabled(rename = "ID")]
    id: i32,
    #[tabled(rename = "Code")]
    code: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Contact")]
    contact: String,
    #[tabled(rename = "Phone")]
    phone: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Created")]
    created_at: String,
}

#[derive(Tabled)]
struct PurchaseOrderTableRow {
    #[tabled(rename = "ID")]
    id: i32,
    #[tabled(rename = "PO Number")]
    po_number: String,
    #[tabled(rename = "Supplier")]
    supplier: String,
    #[tabled(rename = "Date")]
    date: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Items")]
    items_count: i64,
    #[tabled(rename = "Total")]
    total_amount: String,
}

#[derive(Tabled)]
struct PurchaseItemTableRow {
    #[tabled(rename = "Product")]
    product: String,
    #[tabled(rename = "Qty")]
    quantity: i32,
    #[tabled(rename = "Unit Cost")]
    unit_cost: String,
    #[tabled(rename = "Total")]
    total_cost: String,
    #[tabled(rename = "Received")]
    received: i32,
    #[tabled(rename = "Status")]
    status: String,
}