use clap::{Arg, ArgMatches, Command};
use tabled::{Table, Tabled};

use crate::core::result::CLIERPResult;
use crate::modules::crm::{CustomerService, LeadService};
use crate::database::{CustomerType, CustomerStatus, LeadPriority, LeadStatus};
use crate::utils::formatting::{format_currency, format_datetime, format_date};
use crate::utils::pagination::PaginationParams;

pub fn crm_command() -> Command {
    Command::new("crm")
        .about("Customer Relationship Management commands")
        .subcommand_required(true)
        .subcommands([
            customer_commands(),
            lead_commands(),
        ])
}

fn customer_commands() -> Command {
    Command::new("customer")
        .alias("cust")
        .about("Customer management")
        .subcommand_required(true)
        .subcommands([
            Command::new("add")
                .about("Add a new customer")
                .args([
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .required(true)
                        .help("Customer name"),
                    Arg::new("type")
                        .long("type")
                        .short('t')
                        .value_parser(["individual", "business"])
                        .default_value("individual")
                        .help("Customer type"),
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
                    Arg::new("company")
                        .long("company")
                        .help("Company name (for business customers)"),
                    Arg::new("tax-id")
                        .long("tax-id")
                        .help("Tax ID number"),
                    Arg::new("credit-limit")
                        .long("credit-limit")
                        .value_parser(clap::value_parser!(i32))
                        .help("Credit limit"),
                    Arg::new("notes")
                        .long("notes")
                        .help("Additional notes"),
                ]),
            Command::new("list")
                .about("List customers")
                .args([
                    Arg::new("search")
                        .long("search")
                        .short('s')
                        .help("Search by name, code, or email"),
                    Arg::new("type")
                        .long("type")
                        .value_parser(["individual", "business"])
                        .help("Filter by customer type"),
                    Arg::new("status")
                        .long("status")
                        .value_parser(["active", "inactive", "suspended"])
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
                .about("Show customer details")
                .arg(
                    Arg::new("customer_id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Customer ID")
                ),
            Command::new("update")
                .about("Update customer")
                .args([
                    Arg::new("customer_id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Customer ID"),
                    Arg::new("name")
                        .long("name")
                        .short('n')
                        .help("Customer name"),
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
                    Arg::new("company")
                        .long("company")
                        .help("Company name"),
                    Arg::new("tax-id")
                        .long("tax-id")
                        .help("Tax ID number"),
                    Arg::new("credit-limit")
                        .long("credit-limit")
                        .value_parser(clap::value_parser!(i32))
                        .help("Credit limit"),
                    Arg::new("status")
                        .long("status")
                        .value_parser(["active", "inactive", "suspended"])
                        .help("Customer status"),
                    Arg::new("notes")
                        .long("notes")
                        .help("Additional notes"),
                ]),
            Command::new("stats")
                .about("Show customer statistics"),
        ])
}

fn lead_commands() -> Command {
    Command::new("lead")
        .about("Lead management")
        .subcommand_required(true)
        .subcommands([
            Command::new("add")
                .about("Add a new lead")
                .args([
                    Arg::new("title")
                        .long("title")
                        .short('t')
                        .required(true)
                        .help("Lead title"),
                    Arg::new("customer-id")
                        .long("customer-id")
                        .short('c')
                        .value_parser(clap::value_parser!(i32))
                        .help("Customer ID"),
                    Arg::new("source")
                        .long("source")
                        .short('s')
                        .required(true)
                        .help("Lead source"),
                    Arg::new("value")
                        .long("value")
                        .short('v')
                        .value_parser(clap::value_parser!(i32))
                        .default_value("0")
                        .help("Estimated value"),
                    Arg::new("close-date")
                        .long("close-date")
                        .help("Expected close date (YYYY-MM-DD)"),
                    Arg::new("priority")
                        .long("priority")
                        .value_parser(["low", "medium", "high", "urgent"])
                        .default_value("medium")
                        .help("Lead priority"),
                    Arg::new("assigned-to")
                        .long("assigned-to")
                        .value_parser(clap::value_parser!(i32))
                        .help("Assigned employee ID"),
                    Arg::new("description")
                        .long("description")
                        .short('d')
                        .help("Lead description"),
                    Arg::new("notes")
                        .long("notes")
                        .short('n')
                        .help("Additional notes"),
                ]),
            Command::new("list")
                .about("List leads")
                .args([
                    Arg::new("search")
                        .long("search")
                        .short('s')
                        .help("Search by title, customer, or source"),
                    Arg::new("status")
                        .long("status")
                        .value_parser(["new", "contacted", "qualified", "proposal", "negotiation", "closed_won", "closed_lost"])
                        .help("Filter by status"),
                    Arg::new("priority")
                        .long("priority")
                        .value_parser(["low", "medium", "high", "urgent"])
                        .help("Filter by priority"),
                    Arg::new("assigned-to")
                        .long("assigned-to")
                        .value_parser(clap::value_parser!(i32))
                        .help("Filter by assigned employee"),
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
                .about("Show lead details")
                .arg(
                    Arg::new("lead_id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Lead ID")
                ),
            Command::new("update-status")
                .about("Update lead status")
                .args([
                    Arg::new("lead_id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Lead ID"),
                    Arg::new("status")
                        .required(true)
                        .value_parser(["new", "contacted", "qualified", "proposal", "negotiation", "closed_won", "closed_lost"])
                        .help("New status"),
                    Arg::new("notes")
                        .long("notes")
                        .short('n')
                        .help("Status change notes"),
                ]),
            Command::new("assign")
                .about("Assign lead to employee")
                .args([
                    Arg::new("lead_id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Lead ID"),
                    Arg::new("employee_id")
                        .required(true)
                        .value_parser(clap::value_parser!(i32))
                        .help("Employee ID"),
                ]),
            Command::new("stats")
                .about("Show lead statistics"),
        ])
}

pub fn handle_crm_command(matches: &ArgMatches) -> CLIERPResult<()> {
    match matches.subcommand() {
        Some(("customer", sub_matches)) => handle_customer_command(sub_matches),
        Some(("lead", sub_matches)) => handle_lead_command(sub_matches),
        _ => {
            println!("No subcommand provided. Use --help for available commands.");
            Ok(())
        }
    }
}

fn handle_customer_command(matches: &ArgMatches) -> CLIERPResult<()> {
    match matches.subcommand() {
        Some(("add", sub_matches)) => handle_customer_add(sub_matches),
        Some(("list", sub_matches)) => handle_customer_list(sub_matches),
        Some(("show", sub_matches)) => handle_customer_show(sub_matches),
        Some(("update", sub_matches)) => handle_customer_update(sub_matches),
        Some(("stats", _)) => handle_customer_stats(),
        _ => {
            println!("No subcommand provided. Use --help for available commands.");
            Ok(())
        }
    }
}

fn handle_lead_command(matches: &ArgMatches) -> CLIERPResult<()> {
    match matches.subcommand() {
        Some(("add", sub_matches)) => handle_lead_add(sub_matches),
        Some(("list", sub_matches)) => handle_lead_list(sub_matches),
        Some(("show", sub_matches)) => handle_lead_show(sub_matches),
        Some(("update-status", sub_matches)) => handle_lead_update_status(sub_matches),
        Some(("assign", sub_matches)) => handle_lead_assign(sub_matches),
        Some(("stats", _)) => handle_lead_stats(),
        _ => {
            println!("No subcommand provided. Use --help for available commands.");
            Ok(())
        }
    }
}

fn handle_customer_add(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let name = matches.get_one::<String>("name").unwrap();
    let customer_type = match matches.get_one::<String>("type").unwrap().as_str() {
        "business" => CustomerType::Business,
        _ => CustomerType::Individual,
    };
    let email = matches.get_one::<String>("email").map(|s| s.as_str());
    let phone = matches.get_one::<String>("phone").map(|s| s.as_str());
    let address = matches.get_one::<String>("address").map(|s| s.as_str());
    let company_name = matches.get_one::<String>("company").map(|s| s.as_str());
    let tax_id = matches.get_one::<String>("tax-id").map(|s| s.as_str());
    let credit_limit = matches.get_one::<i32>("credit-limit").copied();
    let notes = matches.get_one::<String>("notes").map(|s| s.as_str());

    let customer = CustomerService::create_customer(
        &mut conn,
        name,
        customer_type,
        email,
        phone,
        address,
        company_name,
        tax_id,
        credit_limit,
        notes,
    )?;

    println!("✅ Customer created successfully!");
    println!("ID: {}", customer.id);
    println!("Code: {}", customer.customer_code);
    println!("Name: {}", customer.name);
    println!("Type: {}", customer.customer_type);

    Ok(())
}

fn handle_customer_list(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let search = matches.get_one::<String>("search").map(|s| s.as_str());
    let customer_type = matches.get_one::<String>("type").map(|s| s.as_str());
    let status = matches.get_one::<String>("status").map(|s| s.as_str());
    let page = *matches.get_one::<u32>("page").unwrap();
    let per_page = *matches.get_one::<u32>("per-page").unwrap();

    let filters = crate::utils::filters::FilterOptions {
        search: search.map(|s| s.to_string()),
        filter_type: customer_type.map(|s| s.to_string()),
        status: status.map(|s| s.to_string()),
        ..Default::default()
    };

    let pagination = PaginationParams::new(page, per_page);
    let result = CustomerService::list_customers(&mut conn, &filters, &pagination)?;

    if result.items.is_empty() {
        println!("No customers found.");
        return Ok(());
    }

    let rows: Vec<CustomerTableRow> = result.items
        .into_iter()
        .map(|customer| CustomerTableRow {
            id: customer.id,
            code: customer.customer_code,
            name: customer.name,
            type_: customer.customer_type,
            email: customer.email.unwrap_or_else(|| "-".to_string()),
            phone: customer.phone.unwrap_or_else(|| "-".to_string()),
            status: customer.status,
            credit_limit: format_currency(customer.credit_limit),
            created_at: format_datetime(&customer.created_at),
        })
        .collect();

    let table = Table::new(rows);
    println!("{}", table);
    println!("Page {} of {} ({} total)", result.page, result.total_pages, result.total_items);

    Ok(())
}

fn handle_customer_show(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let customer_id = *matches.get_one::<i32>("customer_id").unwrap();

    let customer_with_stats = CustomerService::get_customer_with_stats(&mut conn, customer_id)?
        .ok_or_else(|| crate::core::error::CLIERPError::NotFound(format!("Customer with ID {} not found", customer_id)))?;

    println!("Customer Details:");
    println!("ID: {}", customer_with_stats.customer.id);
    println!("Code: {}", customer_with_stats.customer.customer_code);
    println!("Name: {}", customer_with_stats.customer.name);
    println!("Type: {}", customer_with_stats.customer.customer_type);
    println!("Email: {}", customer_with_stats.customer.email.unwrap_or_else(|| "-".to_string()));
    println!("Phone: {}", customer_with_stats.customer.phone.unwrap_or_else(|| "-".to_string()));
    println!("Address: {}", customer_with_stats.customer.address.unwrap_or_else(|| "-".to_string()));
    if let Some(company) = &customer_with_stats.customer.company_name {
        println!("Company: {}", company);
    }
    if let Some(tax_id) = &customer_with_stats.customer.tax_id {
        println!("Tax ID: {}", tax_id);
    }
    println!("Credit Limit: {}", format_currency(customer_with_stats.customer.credit_limit));
    println!("Status: {}", customer_with_stats.customer.status);
    if let Some(notes) = &customer_with_stats.customer.notes {
        println!("Notes: {}", notes);
    }
    println!("Created: {}", format_datetime(&customer_with_stats.customer.created_at));
    println!("Updated: {}", format_datetime(&customer_with_stats.customer.updated_at));
    println!();
    println!("Statistics:");
    println!("Total Leads: {}", customer_with_stats.total_leads);
    println!("Active Deals: {}", customer_with_stats.active_deals);
    println!("Total Deal Value: {}", format_currency(customer_with_stats.total_deal_value));

    Ok(())
}

fn handle_customer_update(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let customer_id = *matches.get_one::<i32>("customer_id").unwrap();
    let name = matches.get_one::<String>("name").map(|s| s.as_str());
    let email = matches.get_one::<String>("email").map(|s| Some(s.as_str()));
    let phone = matches.get_one::<String>("phone").map(|s| Some(s.as_str()));
    let address = matches.get_one::<String>("address").map(|s| Some(s.as_str()));
    let company_name = matches.get_one::<String>("company").map(|s| Some(s.as_str()));
    let tax_id = matches.get_one::<String>("tax-id").map(|s| Some(s.as_str()));
    let credit_limit = matches.get_one::<i32>("credit-limit").copied();
    let status = matches.get_one::<String>("status").map(|s| match s.as_str() {
        "inactive" => CustomerStatus::Inactive,
        "suspended" => CustomerStatus::Suspended,
        _ => CustomerStatus::Active,
    });
    let notes = matches.get_one::<String>("notes").map(|s| Some(s.as_str()));

    let customer = CustomerService::update_customer(
        &mut conn,
        customer_id,
        name,
        email,
        phone,
        address,
        company_name,
        tax_id,
        credit_limit,
        status,
        notes,
    )?;

    println!("✅ Customer updated successfully!");
    println!("ID: {}", customer.id);
    println!("Name: {}", customer.name);
    println!("Status: {}", customer.status);

    Ok(())
}

fn handle_customer_stats() -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let stats = CustomerService::get_customer_statistics(&mut conn)?;

    println!("Customer Statistics:");
    println!("Total Customers: {}", stats.total_customers);
    println!("Active Customers: {}", stats.active_customers);
    println!("Business Customers: {}", stats.business_customers);
    println!("Individual Customers: {}", stats.individual_customers);
    println!("Total Credit Limit: {}", format_currency(stats.total_credit_limit));

    Ok(())
}

fn handle_lead_add(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let title = matches.get_one::<String>("title").unwrap();
    let customer_id = matches.get_one::<i32>("customer-id").copied();
    let source = matches.get_one::<String>("source").unwrap();
    let estimated_value = *matches.get_one::<i32>("value").unwrap();
    let expected_close_date = matches.get_one::<String>("close-date").map(|s| s.parse().unwrap());
    let priority = match matches.get_one::<String>("priority").unwrap().as_str() {
        "low" => LeadPriority::Low,
        "high" => LeadPriority::High,
        "urgent" => LeadPriority::Urgent,
        _ => LeadPriority::Medium,
    };
    let assigned_to = matches.get_one::<i32>("assigned-to").copied();
    let description = matches.get_one::<String>("description").map(|s| s.as_str());
    let notes = matches.get_one::<String>("notes").map(|s| s.as_str());

    let lead = LeadService::create_lead(
        &mut conn,
        title,
        customer_id,
        source,
        estimated_value,
        expected_close_date,
        priority,
        assigned_to,
        description,
        notes,
    )?;

    println!("✅ Lead created successfully!");
    println!("ID: {}", lead.id);
    println!("Title: {}", lead.title);
    println!("Source: {}", lead.lead_source);
    println!("Estimated Value: {}", format_currency(lead.estimated_value));
    println!("Priority: {}", lead.priority);

    Ok(())
}

fn handle_lead_list(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let search = matches.get_one::<String>("search").map(|s| s.as_str());
    let status = matches.get_one::<String>("status").map(|s| s.as_str());
    let priority = matches.get_one::<String>("priority").map(|s| s.as_str());
    let assigned_to = matches.get_one::<i32>("assigned-to").copied();
    let page = *matches.get_one::<u32>("page").unwrap();
    let per_page = *matches.get_one::<u32>("per-page").unwrap();

    let filters = crate::utils::filters::FilterOptions {
        search: search.map(|s| s.to_string()),
        status: status.map(|s| s.to_string()),
        priority: priority.map(|s| s.to_string()),
        assigned_to,
        ..Default::default()
    };

    let pagination = PaginationParams::new(page, per_page);
    let result = LeadService::list_leads(&mut conn, &filters, &pagination)?;

    if result.items.is_empty() {
        println!("No leads found.");
        return Ok(());
    }

    let rows: Vec<LeadTableRow> = result.items
        .into_iter()
        .map(|lead_with_customer| LeadTableRow {
            id: lead_with_customer.lead.id,
            title: lead_with_customer.lead.title,
            customer: lead_with_customer.customer.map(|c| c.name).unwrap_or_else(|| "-".to_string()),
            source: lead_with_customer.lead.lead_source,
            status: lead_with_customer.lead.status,
            priority: lead_with_customer.lead.priority,
            value: format_currency(lead_with_customer.lead.estimated_value),
            probability: format!("{}%", lead_with_customer.lead.probability),
            assigned_to: lead_with_customer.assigned_employee.unwrap_or_else(|| "-".to_string()),
            close_date: lead_with_customer.lead.expected_close_date.map(|d| format_date(&d)).unwrap_or_else(|| "-".to_string()),
        })
        .collect();

    let table = Table::new(rows);
    println!("{}", table);
    println!("Page {} of {} ({} total)", result.page, result.total_pages, result.total_items);

    Ok(())
}

fn handle_lead_show(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let lead_id = *matches.get_one::<i32>("lead_id").unwrap();

    let lead_with_customer = LeadService::get_lead_with_customer(&mut conn, lead_id)?
        .ok_or_else(|| crate::core::error::CLIERPError::NotFound(format!("Lead with ID {} not found", lead_id)))?;

    println!("Lead Details:");
    println!("ID: {}", lead_with_customer.lead.id);
    println!("Title: {}", lead_with_customer.lead.title);
    if let Some(customer) = &lead_with_customer.customer {
        println!("Customer: {} ({})", customer.name, customer.customer_code);
    } else {
        println!("Customer: Not assigned");
    }
    println!("Source: {}", lead_with_customer.lead.lead_source);
    println!("Status: {}", lead_with_customer.lead.status);
    println!("Priority: {}", lead_with_customer.lead.priority);
    println!("Estimated Value: {}", format_currency(lead_with_customer.lead.estimated_value));
    println!("Probability: {}%", lead_with_customer.lead.probability);
    println!("Expected Close: {}", lead_with_customer.lead.expected_close_date.map(|d| format_date(&d)).unwrap_or_else(|| "-".to_string()));
    println!("Assigned To: {}", lead_with_customer.assigned_employee.unwrap_or_else(|| "-".to_string()));
    if let Some(description) = &lead_with_customer.lead.description {
        println!("Description: {}", description);
    }
    if let Some(notes) = &lead_with_customer.lead.notes {
        println!("Notes: {}", notes);
    }
    println!("Created: {}", format_datetime(&lead_with_customer.lead.created_at));
    println!("Updated: {}", format_datetime(&lead_with_customer.lead.updated_at));

    Ok(())
}

fn handle_lead_update_status(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let lead_id = *matches.get_one::<i32>("lead_id").unwrap();
    let status = match matches.get_one::<String>("status").unwrap().as_str() {
        "contacted" => LeadStatus::Contacted,
        "qualified" => LeadStatus::Qualified,
        "proposal" => LeadStatus::Proposal,
        "negotiation" => LeadStatus::Negotiation,
        "closed_won" => LeadStatus::ClosedWon,
        "closed_lost" => LeadStatus::ClosedLost,
        _ => LeadStatus::New,
    };
    let notes = matches.get_one::<String>("notes").map(|s| s.as_str());

    let lead = LeadService::update_lead_status(&mut conn, lead_id, status, notes)?;

    println!("✅ Lead status updated successfully!");
    println!("ID: {}", lead.id);
    println!("Title: {}", lead.title);
    println!("Status: {}", lead.status);
    println!("Probability: {}%", lead.probability);

    Ok(())
}

fn handle_lead_assign(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let lead_id = *matches.get_one::<i32>("lead_id").unwrap();
    let employee_id = *matches.get_one::<i32>("employee_id").unwrap();

    let lead = LeadService::assign_lead(&mut conn, lead_id, employee_id)?;

    println!("✅ Lead assigned successfully!");
    println!("ID: {}", lead.id);
    println!("Title: {}", lead.title);
    println!("Assigned To: Employee ID {}", employee_id);

    Ok(())
}

fn handle_lead_stats() -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;

    let stats = LeadService::get_lead_statistics(&mut conn)?;

    println!("Lead Statistics:");
    println!("Total Leads: {}", stats.total_leads);
    println!("New Leads: {}", stats.new_leads);
    println!("Qualified Leads: {}", stats.qualified_leads);
    println!("Closed Won: {}", stats.closed_won);
    println!("Closed Lost: {}", stats.closed_lost);
    println!("Total Estimated Value: {}", format_currency(stats.total_estimated_value));
    println!("Average Deal Size: {}", format_currency(stats.average_deal_size as i32));
    println!("Conversion Rate: {:.2}%", stats.conversion_rate);

    Ok(())
}

#[derive(Tabled)]
struct CustomerTableRow {
    #[tabled(rename = "ID")]
    id: i32,
    #[tabled(rename = "Code")]
    code: String,
    #[tabled(rename = "Name")]
    name: String,
    #[tabled(rename = "Type")]
    type_: String,
    #[tabled(rename = "Email")]
    email: String,
    #[tabled(rename = "Phone")]
    phone: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Credit Limit")]
    credit_limit: String,
    #[tabled(rename = "Created")]
    created_at: String,
}

#[derive(Tabled)]
struct LeadTableRow {
    #[tabled(rename = "ID")]
    id: i32,
    #[tabled(rename = "Title")]
    title: String,
    #[tabled(rename = "Customer")]
    customer: String,
    #[tabled(rename = "Source")]
    source: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Priority")]
    priority: String,
    #[tabled(rename = "Value")]
    value: String,
    #[tabled(rename = "Probability")]
    probability: String,
    #[tabled(rename = "Assigned To")]
    assigned_to: String,
    #[tabled(rename = "Close Date")]
    close_date: String,
}