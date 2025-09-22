use clap::{Arg, ArgMatches, Command};
use chrono::NaiveDate;
use std::collections::HashMap;

use crate::core::result::CLIERPResult;
use crate::modules::reporting::*;
use crate::utils::formatting::format_date;

pub fn reports_command() -> Command {
    Command::new("reports")
        .alias("report")
        .about("Generate various reports")
        .subcommand_required(true)
        .subcommands([
            hr_reports_commands(),
            finance_reports_commands(),
            inventory_reports_commands(),
            crm_reports_commands(),
        ])
}

fn hr_reports_commands() -> Command {
    Command::new("hr")
        .about("HR reports")
        .subcommand_required(true)
        .subcommands([
            Command::new("employee-summary")
                .about("Generate employee summary report")
                .args([
                    Arg::new("department")
                        .long("department")
                        .value_parser(clap::value_parser!(i32))
                        .help("Filter by department ID"),
                    Arg::new("start-date")
                        .long("start-date")
                        .help("Start date (YYYY-MM-DD)"),
                    Arg::new("end-date")
                        .long("end-date")
                        .help("End date (YYYY-MM-DD)"),
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "html", "text"])
                        .default_value("text")
                        .help("Output format"),
                ]),
            Command::new("attendance")
                .about("Generate attendance report")
                .args([
                    Arg::new("employee")
                        .long("employee")
                        .value_parser(clap::value_parser!(i32))
                        .help("Filter by employee ID"),
                    Arg::new("department")
                        .long("department")
                        .value_parser(clap::value_parser!(i32))
                        .help("Filter by department ID"),
                    Arg::new("start-date")
                        .long("start-date")
                        .help("Start date (YYYY-MM-DD)"),
                    Arg::new("end-date")
                        .long("end-date")
                        .help("End date (YYYY-MM-DD)"),
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "html", "text"])
                        .default_value("text")
                        .help("Output format"),
                ]),
            Command::new("payroll")
                .about("Generate payroll report")
                .args([
                    Arg::new("period")
                        .long("period")
                        .help("Payroll period (YYYY-MM)"),
                    Arg::new("department")
                        .long("department")
                        .value_parser(clap::value_parser!(i32))
                        .help("Filter by department ID"),
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "html", "text"])
                        .default_value("text")
                        .help("Output format"),
                ]),
        ])
}

fn finance_reports_commands() -> Command {
    Command::new("finance")
        .alias("fin")
        .about("Finance reports")
        .subcommand_required(true)
        .subcommands([
            Command::new("income-statement")
                .about("Generate income statement")
                .args([
                    Arg::new("start-date")
                        .long("start-date")
                        .required(true)
                        .help("Start date (YYYY-MM-DD)"),
                    Arg::new("end-date")
                        .long("end-date")
                        .required(true)
                        .help("End date (YYYY-MM-DD)"),
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "html", "text"])
                        .default_value("text")
                        .help("Output format"),
                ]),
            Command::new("balance-sheet")
                .about("Generate balance sheet")
                .args([
                    Arg::new("date")
                        .long("date")
                        .required(true)
                        .help("As of date (YYYY-MM-DD)"),
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "html", "text"])
                        .default_value("text")
                        .help("Output format"),
                ]),
            Command::new("cash-flow")
                .about("Generate cash flow statement")
                .args([
                    Arg::new("start-date")
                        .long("start-date")
                        .required(true)
                        .help("Start date (YYYY-MM-DD)"),
                    Arg::new("end-date")
                        .long("end-date")
                        .required(true)
                        .help("End date (YYYY-MM-DD)"),
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "html", "text"])
                        .default_value("text")
                        .help("Output format"),
                ]),
        ])
}

fn inventory_reports_commands() -> Command {
    Command::new("inventory")
        .alias("inv")
        .about("Inventory reports")
        .subcommand_required(true)
        .subcommands([
            Command::new("stock-levels")
                .about("Generate stock levels report")
                .args([
                    Arg::new("category")
                        .long("category")
                        .value_parser(clap::value_parser!(i32))
                        .help("Filter by category ID"),
                    Arg::new("low-stock-only")
                        .long("low-stock-only")
                        .action(clap::ArgAction::SetTrue)
                        .help("Show only low stock items"),
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "html", "text"])
                        .default_value("text")
                        .help("Output format"),
                ]),
            Command::new("movement")
                .about("Generate stock movement report")
                .args([
                    Arg::new("product")
                        .long("product")
                        .value_parser(clap::value_parser!(i32))
                        .help("Filter by product ID"),
                    Arg::new("start-date")
                        .long("start-date")
                        .help("Start date (YYYY-MM-DD)"),
                    Arg::new("end-date")
                        .long("end-date")
                        .help("End date (YYYY-MM-DD)"),
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "html", "text"])
                        .default_value("text")
                        .help("Output format"),
                ]),
            Command::new("valuation")
                .about("Generate inventory valuation report")
                .args([
                    Arg::new("date")
                        .long("date")
                        .help("As of date (YYYY-MM-DD, default: today)"),
                    Arg::new("method")
                        .long("method")
                        .value_parser(["fifo", "lifo", "average"])
                        .default_value("fifo")
                        .help("Valuation method"),
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "html", "text"])
                        .default_value("text")
                        .help("Output format"),
                ]),
        ])
}

fn crm_reports_commands() -> Command {
    Command::new("crm")
        .about("CRM reports")
        .subcommand_required(true)
        .subcommands([
            Command::new("sales-performance")
                .about("Generate sales performance report")
                .args([
                    Arg::new("employee")
                        .long("employee")
                        .value_parser(clap::value_parser!(i32))
                        .help("Filter by employee ID"),
                    Arg::new("start-date")
                        .long("start-date")
                        .help("Start date (YYYY-MM-DD)"),
                    Arg::new("end-date")
                        .long("end-date")
                        .help("End date (YYYY-MM-DD)"),
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "html", "text"])
                        .default_value("text")
                        .help("Output format"),
                ]),
            Command::new("pipeline")
                .about("Generate sales pipeline report")
                .args([
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "html", "text"])
                        .default_value("text")
                        .help("Output format"),
                ]),
            Command::new("customer-analysis")
                .about("Generate customer analysis report")
                .args([
                    Arg::new("customer-type")
                        .long("customer-type")
                        .value_parser(["individual", "business"])
                        .help("Filter by customer type"),
                    Arg::new("format")
                        .long("format")
                        .value_parser(["json", "csv", "html", "text"])
                        .default_value("text")
                        .help("Output format"),
                ]),
        ])
}

pub fn handle_reports_command(matches: &ArgMatches) -> CLIERPResult<()> {
    match matches.subcommand() {
        Some(("hr", sub_matches)) => handle_hr_reports(sub_matches),
        Some(("finance", sub_matches)) => handle_finance_reports(sub_matches),
        Some(("inventory", sub_matches)) => handle_inventory_reports(sub_matches),
        Some(("crm", sub_matches)) => handle_crm_reports(sub_matches),
        _ => {
            println!("Available report modules:");
            println!("  hr        - Human Resources reports");
            println!("  finance   - Financial reports");
            println!("  inventory - Inventory reports");
            println!("  crm       - Customer Relationship Management reports");
            println!();
            println!("Use 'clierp reports <module> --help' for more information");
            Ok(())
        }
    }
}

fn handle_hr_reports(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;
    let generator = HRReportsGenerator;

    match matches.subcommand() {
        Some(("employee-summary", sub_matches)) => {
            let mut config = create_report_config("employee_summary", sub_matches)?;
            let result = generator.generate_report(config)?;
            display_report_result(&result, sub_matches)?;
        }
        Some(("attendance", sub_matches)) => {
            let mut config = create_report_config("attendance_report", sub_matches)?;
            let result = generator.generate_report(config)?;
            display_report_result(&result, sub_matches)?;
        }
        Some(("payroll", sub_matches)) => {
            let mut config = create_report_config("payroll_report", sub_matches)?;
            let result = generator.generate_report(config)?;
            display_report_result(&result, sub_matches)?;
        }
        _ => {
            println!("Available HR reports:");
            println!("  employee-summary - Employee summary report");
            println!("  attendance       - Attendance report");
            println!("  payroll          - Payroll report");
        }
    }
    Ok(())
}

fn handle_finance_reports(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;
    let generator = FinanceReportsGenerator;

    match matches.subcommand() {
        Some(("income-statement", sub_matches)) => {
            let mut config = create_report_config("income_statement", sub_matches)?;
            let result = generator.generate_report(config)?;
            display_report_result(&result, sub_matches)?;
        }
        Some(("balance-sheet", sub_matches)) => {
            let mut config = create_report_config("balance_sheet", sub_matches)?;
            let result = generator.generate_report(config)?;
            display_report_result(&result, sub_matches)?;
        }
        Some(("cash-flow", sub_matches)) => {
            let mut config = create_report_config("cash_flow", sub_matches)?;
            let result = generator.generate_report(config)?;
            display_report_result(&result, sub_matches)?;
        }
        _ => {
            println!("Available Finance reports:");
            println!("  income-statement - Income Statement");
            println!("  balance-sheet    - Balance Sheet");
            println!("  cash-flow        - Cash Flow Statement");
        }
    }
    Ok(())
}

fn handle_inventory_reports(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;
    let generator = InventoryReportsGenerator;

    match matches.subcommand() {
        Some(("stock-levels", sub_matches)) => {
            let mut config = create_report_config("stock_levels", sub_matches)?;
            let result = generator.generate_report(config)?;
            display_report_result(&result, sub_matches)?;
        }
        Some(("movement", sub_matches)) => {
            let mut config = create_report_config("stock_movement", sub_matches)?;
            let result = generator.generate_report(config)?;
            display_report_result(&result, sub_matches)?;
        }
        Some(("valuation", sub_matches)) => {
            let mut config = create_report_config("inventory_valuation", sub_matches)?;
            let result = generator.generate_report(config)?;
            display_report_result(&result, sub_matches)?;
        }
        _ => {
            println!("Available Inventory reports:");
            println!("  stock-levels - Current stock levels");
            println!("  movement     - Stock movement history");
            println!("  valuation    - Inventory valuation");
        }
    }
    Ok(())
}

fn handle_crm_reports(matches: &ArgMatches) -> CLIERPResult<()> {
    let mut conn = crate::database::get_connection()?;
    let generator = CrmReportsGenerator;

    match matches.subcommand() {
        Some(("sales-performance", sub_matches)) => {
            let mut config = create_report_config("sales_performance", sub_matches)?;
            let result = generator.generate_report(config)?;
            display_report_result(&result, sub_matches)?;
        }
        Some(("pipeline", sub_matches)) => {
            let mut config = create_report_config("sales_pipeline", sub_matches)?;
            let result = generator.generate_report(config)?;
            display_report_result(&result, sub_matches)?;
        }
        Some(("customer-analysis", sub_matches)) => {
            let mut config = create_report_config("customer_analysis", sub_matches)?;
            let result = generator.generate_report(config)?;
            display_report_result(&result, sub_matches)?;
        }
        _ => {
            println!("Available CRM reports:");
            println!("  sales-performance - Sales performance analysis");
            println!("  pipeline          - Sales pipeline report");
            println!("  customer-analysis - Customer analysis");
        }
    }
    Ok(())
}

fn create_report_config(report_title: &str, matches: &ArgMatches) -> CLIERPResult<ReportConfig> {
    let mut filters = HashMap::new();

    // Add common filters
    if let Some(department) = matches.get_one::<i32>("department") {
        filters.insert("department_id".to_string(), department.to_string());
    }
    if let Some(employee) = matches.get_one::<i32>("employee") {
        filters.insert("employee_id".to_string(), employee.to_string());
    }
    if let Some(category) = matches.get_one::<i32>("category") {
        filters.insert("category_id".to_string(), category.to_string());
    }
    if let Some(product) = matches.get_one::<i32>("product") {
        filters.insert("product_id".to_string(), product.to_string());
    }
    if let Some(customer_type) = matches.get_one::<String>("customer-type") {
        filters.insert("customer_type".to_string(), customer_type.clone());
    }
    if let Some(period) = matches.get_one::<String>("period") {
        filters.insert("period".to_string(), period.clone());
    }
    if let Some(method) = matches.get_one::<String>("method") {
        filters.insert("valuation_method".to_string(), method.clone());
    }
    if matches.get_flag("low-stock-only") {
        filters.insert("low_stock_only".to_string(), "true".to_string());
    }

    // Handle date range
    let date_range = if let (Some(start_str), Some(end_str)) =
        (matches.get_one::<String>("start-date"), matches.get_one::<String>("end-date")) {
        let start_date = start_str.parse::<NaiveDate>()
            .map_err(|_| crate::core::error::CLIERPError::ValidationError(
                "Invalid start date format. Use YYYY-MM-DD".to_string()
            ))?;
        let end_date = end_str.parse::<NaiveDate>()
            .map_err(|_| crate::core::error::CLIERPError::ValidationError(
                "Invalid end date format. Use YYYY-MM-DD".to_string()
            ))?;
        Some(DateRange { start_date, end_date })
    } else if let Some(date_str) = matches.get_one::<String>("date") {
        let date = date_str.parse::<NaiveDate>()
            .map_err(|_| crate::core::error::CLIERPError::ValidationError(
                "Invalid date format. Use YYYY-MM-DD".to_string()
            ))?;
        Some(DateRange { start_date: date, end_date: date })
    } else {
        None
    };

    // Determine format
    let format = match matches.get_one::<String>("format").map(|s| s.as_str()) {
        Some("json") => ReportFormat::Json,
        Some("csv") => ReportFormat::Csv,
        Some("html") => ReportFormat::Html,
        _ => ReportFormat::Text,
    };

    Ok(ReportConfig {
        title: report_title.to_string(),
        description: Some(format!("Generated {} report", report_title.replace('_', " "))),
        date_range,
        filters,
        format,
        include_charts: false,
        include_summary: true,
    })
}

fn display_report_result(result: &ReportResult, matches: &ArgMatches) -> CLIERPResult<()> {
    match result.config.format {
        ReportFormat::Json => {
            let json = serde_json::to_string_pretty(result)?;
            println!("{}", json);
        }
        ReportFormat::Csv => {
            if let ReportData::Table(table_data) = &result.data {
                // Print CSV headers
                println!("{}", table_data.headers.join(","));
                // Print CSV rows
                for row in &table_data.rows {
                    println!("{}", row.join(","));
                }
            } else {
                println!("CSV format not supported for this report type");
            }
        }
        ReportFormat::Html => {
            println!("HTML format not yet implemented");
        }
        ReportFormat::Text => {
            println!("=== {} ===", result.config.title.replace('_', " ").to_uppercase());
            println!("Generated: {}", result.generated_at.format("%Y-%m-%d %H:%M:%S"));

            if let Some(date_range) = &result.config.date_range {
                println!("Period: {} to {}", date_range.start_date, date_range.end_date);
            }

            println!();

            match &result.data {
                ReportData::Table(table_data) => {
                    use tabled::{Table, Style};
                    let mut table = Table::new(&table_data.rows);
                    table.with(Style::modern());
                    println!("{}", table);
                }
                ReportData::Mixed(sections) => {
                    for section in sections {
                        println!("## {}", section.title);
                        if let Some(desc) = &section.description {
                            println!("{}", desc);
                        }
                        match &section.content {
                            ReportData::Table(table_data) => {
                                use tabled::{Table, Style};
                                let mut table = Table::new(&table_data.rows);
                                table.with(Style::modern());
                                println!("{}", table);
                            }
                            _ => println!("Content format not supported"),
                        }
                        println!();
                    }
                }
                _ => println!("Report format not supported"),
            }

            if let Some(summary) = &result.summary {
                println!("\n=== SUMMARY ===");
                for (key, value) in &summary.metrics {
                    println!("{}: {}", key, value);
                }
                if !summary.insights.is_empty() {
                    println!("\nKey Insights:");
                    for insight in &summary.insights {
                        println!("• {}", insight);
                    }
                }
            }
        }
    }
    Ok(())
}