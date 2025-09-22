mod common;

use common::setup_test_db;
use clierp::database::connection::get_connection;
use clierp::modules::hr::{EmployeeService, DepartmentService, PayrollService, AttendanceService};
use clierp::modules::finance::{AccountService, TransactionService};
use clierp::modules::inventory::{ProductService, CategoryService, SupplierService, PurchaseOrderService};
use clierp::modules::crm::{CustomerService, LeadService, DealService, CampaignService};
use clierp::modules::reporting::{HRReportService, FinanceReportService, InventoryReportService, CRMReportService};
use chrono::{NaiveDate, Utc};
use std::collections::HashMap;

#[test]
fn test_hr_report_accuracy() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");

    // Services
    let dept_service = DepartmentService::new();
    let employee_service = EmployeeService::new();
    let payroll_service = PayrollService::new();
    let attendance_service = AttendanceService::new();
    let hr_report_service = HRReportService::new();

    // 1. Create test data
    let dept1 = dept_service.create_department(
        &mut conn,
        "Engineering",
        Some("Software development team"),
        None,
    ).unwrap();

    let dept2 = dept_service.create_department(
        &mut conn,
        "Marketing",
        Some("Marketing and sales support"),
        None,
    ).unwrap();

    // Create employees with different salaries
    let emp1 = employee_service.create_employee(
        &mut conn,
        "ENG001",
        "Alice Engineer",
        Some("alice@company.com"),
        None,
        dept1.id,
        "Senior Developer",
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        80000, // $800.00/month
    ).unwrap();

    let emp2 = employee_service.create_employee(
        &mut conn,
        "ENG002",
        "Bob Developer",
        Some("bob@company.com"),
        None,
        dept1.id,
        "Junior Developer",
        NaiveDate::from_ymd_opt(2024, 2, 1).unwrap(),
        60000, // $600.00/month
    ).unwrap();

    let emp3 = employee_service.create_employee(
        &mut conn,
        "MKT001",
        "Carol Marketer",
        Some("carol@company.com"),
        None,
        dept2.id,
        "Marketing Specialist",
        NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        50000, // $500.00/month
    ).unwrap();

    // 2. Create payroll records
    let payroll_period = "2024-10";

    let payroll1 = payroll_service.calculate_payroll(
        &mut conn,
        emp1.id,
        payroll_period,
        None, // Use base salary
        0,    // No overtime
        5000, // $50.00 bonus
        8000, // $80.00 deductions
    ).unwrap();

    let payroll2 = payroll_service.calculate_payroll(
        &mut conn,
        emp2.id,
        payroll_period,
        None,
        10000, // $100.00 overtime
        0,     // No bonus
        6000,  // $60.00 deductions
    ).unwrap();

    let payroll3 = payroll_service.calculate_payroll(
        &mut conn,
        emp3.id,
        payroll_period,
        None,
        0,    // No overtime
        2000, // $20.00 bonus
        5000, // $50.00 deductions
    ).unwrap();

    // 3. Create attendance records
    let attendance_date = NaiveDate::from_ymd_opt(2024, 10, 15).unwrap();

    let _att1 = attendance_service.check_in(
        &mut conn,
        emp1.id,
        attendance_date,
        Some(chrono::NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
    ).unwrap();

    let _att2 = attendance_service.check_in(
        &mut conn,
        emp2.id,
        attendance_date,
        Some(chrono::NaiveTime::from_hms_opt(8, 30, 0).unwrap()),
    ).unwrap();

    let _att3 = attendance_service.check_in(
        &mut conn,
        emp3.id,
        attendance_date,
        Some(chrono::NaiveTime::from_hms_opt(9, 15, 0).unwrap()), // Late
    ).unwrap();

    // 4. Generate HR reports and verify accuracy

    // Employee summary report
    let employee_summary = hr_report_service.generate_employee_summary_report(
        &mut conn,
        None, // All departments
    ).unwrap();

    assert_eq!(employee_summary.total_employees, 3);
    assert_eq!(employee_summary.departments.len(), 2);

    // Find engineering department stats
    let eng_stats = employee_summary.departments.iter()
        .find(|d| d.department_name == "Engineering")
        .unwrap();
    assert_eq!(eng_stats.employee_count, 2);
    assert_eq!(eng_stats.average_salary, 70000); // (80000 + 60000) / 2

    // Find marketing department stats
    let mkt_stats = employee_summary.departments.iter()
        .find(|d| d.department_name == "Marketing")
        .unwrap();
    assert_eq!(mkt_stats.employee_count, 1);
    assert_eq!(mkt_stats.average_salary, 50000);

    // Payroll summary report
    let payroll_summary = hr_report_service.generate_payroll_summary_report(
        &mut conn,
        payroll_period,
    ).unwrap();

    // Calculate expected totals manually
    let expected_gross = payroll1.base_salary + payroll1.bonuses + payroll1.overtime_pay +
                        payroll2.base_salary + payroll2.bonuses + payroll2.overtime_pay +
                        payroll3.base_salary + payroll3.bonuses + payroll3.overtime_pay;

    let expected_deductions = payroll1.deductions + payroll2.deductions + payroll3.deductions;
    let expected_net = payroll1.net_salary + payroll2.net_salary + payroll3.net_salary;

    assert_eq!(payroll_summary.total_gross_pay, expected_gross);
    assert_eq!(payroll_summary.total_deductions, expected_deductions);
    assert_eq!(payroll_summary.total_net_pay, expected_net);
    assert_eq!(payroll_summary.employee_count, 3);

    // Verify gross = net + deductions
    assert_eq!(expected_gross, expected_net + expected_deductions);

    // Attendance report
    let attendance_start = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
    let attendance_end = NaiveDate::from_ymd_opt(2024, 10, 31).unwrap();

    let attendance_report = hr_report_service.generate_attendance_report(
        &mut conn,
        Some(dept1.id), // Engineering department only
        attendance_start,
        attendance_end,
    ).unwrap();

    assert_eq!(attendance_report.total_employees, 2); // Alice and Bob
    assert!(attendance_report.employees.len() == 2);

    // Find Alice's attendance
    let alice_attendance = attendance_report.employees.iter()
        .find(|e| e.employee_name == "Alice Engineer")
        .unwrap();
    assert_eq!(alice_attendance.total_present_days, 1);
    assert_eq!(alice_attendance.total_late_days, 0);
}

#[test]
fn test_finance_report_accuracy() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");

    // Services
    let account_service = AccountService::new();
    let transaction_service = TransactionService::new();
    let finance_report_service = FinanceReportService::new();

    // 1. Create chart of accounts
    let cash_account = account_service.create_account(
        "1000",
        "Cash",
        crate::database::models::AccountType::Asset,
        None,
    ).unwrap();

    let revenue_account = account_service.create_account(
        "4000",
        "Sales Revenue",
        crate::database::models::AccountType::Revenue,
        None,
    ).unwrap();

    let expense_account = account_service.create_account(
        "5000",
        "Operating Expenses",
        crate::database::models::AccountType::Expense,
        None,
    ).unwrap();

    let liability_account = account_service.create_account(
        "2000",
        "Accounts Payable",
        crate::database::models::AccountType::Liability,
        None,
    ).unwrap();

    let equity_account = account_service.create_account(
        "3000",
        "Owner's Equity",
        crate::database::models::AccountType::Equity,
        None,
    ).unwrap();

    // 2. Create transactions for October 2024
    let transaction_date = NaiveDate::from_ymd_opt(2024, 10, 15).unwrap();

    // Sale transaction: Debit Cash, Credit Revenue
    let cash_from_sale = transaction_service.create_transaction(
        cash_account.id,
        transaction_date,
        100000, // $1000
        "debit",
        "Sale of products",
        Some("SALE-001"),
        Some(1),
    ).unwrap();

    let revenue_from_sale = transaction_service.create_transaction(
        revenue_account.id,
        transaction_date,
        100000, // $1000
        "credit",
        "Sale of products",
        Some("SALE-001"),
        Some(1),
    ).unwrap();

    // Expense transaction: Debit Expense, Credit Cash
    let expense_payment = transaction_service.create_transaction(
        expense_account.id,
        transaction_date,
        30000, // $300
        "debit",
        "Office rent payment",
        Some("RENT-001"),
        Some(1),
    ).unwrap();

    let cash_for_expense = transaction_service.create_transaction(
        cash_account.id,
        transaction_date,
        30000, // $300
        "credit",
        "Office rent payment",
        Some("RENT-001"),
        Some(1),
    ).unwrap();

    // Liability transaction: Credit Accounts Payable, Debit Expense
    let accounts_payable = transaction_service.create_transaction(
        liability_account.id,
        transaction_date,
        20000, // $200
        "credit",
        "Supplies purchased on credit",
        Some("SUPPLY-001"),
        Some(1),
    ).unwrap();

    let supplies_expense = transaction_service.create_transaction(
        expense_account.id,
        transaction_date,
        20000, // $200
        "debit",
        "Supplies purchased on credit",
        Some("SUPPLY-001"),
        Some(1),
    ).unwrap();

    // 3. Generate and verify financial reports

    // Income Statement (Profit & Loss)
    let period_start = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
    let period_end = NaiveDate::from_ymd_opt(2024, 10, 31).unwrap();

    let income_statement = finance_report_service.generate_income_statement(
        &mut conn,
        period_start,
        period_end,
    ).unwrap();

    // Verify revenue
    assert_eq!(income_statement.total_revenue, 100000); // $1000 from sale

    // Verify expenses
    let expected_expenses = 30000 + 20000; // Rent + Supplies = $500
    assert_eq!(income_statement.total_expenses, expected_expenses);

    // Verify net income
    let expected_net_income = 100000 - expected_expenses; // $1000 - $500 = $500
    assert_eq!(income_statement.net_income, expected_net_income);

    // Balance Sheet
    let balance_sheet_date = NaiveDate::from_ymd_opt(2024, 10, 31).unwrap();
    let balance_sheet = finance_report_service.generate_balance_sheet(
        &mut conn,
        balance_sheet_date,
    ).unwrap();

    // Verify assets
    // Cash: +$1000 (sale) -$300 (rent) = $700
    let expected_cash_balance = 100000 - 30000;
    assert_eq!(balance_sheet.total_assets, expected_cash_balance);

    // Verify liabilities
    assert_eq!(balance_sheet.total_liabilities, 20000); // $200 accounts payable

    // Verify equity (should include retained earnings from net income)
    let expected_equity = expected_net_income; // Net income becomes retained earnings
    assert_eq!(balance_sheet.total_equity, expected_equity);

    // Verify accounting equation: Assets = Liabilities + Equity
    assert_eq!(
        balance_sheet.total_assets,
        balance_sheet.total_liabilities + balance_sheet.total_equity
    );

    // Cash Flow Statement
    let cash_flow = finance_report_service.generate_cash_flow_statement(
        &mut conn,
        period_start,
        period_end,
    ).unwrap();

    // Operating cash flow should show cash from sales and cash for expenses
    let expected_operating_cash_flow = 100000 - 30000; // $1000 in - $300 out = $700
    assert_eq!(cash_flow.operating_cash_flow, expected_operating_cash_flow);

    // Verify total cash change
    assert_eq!(cash_flow.net_cash_change, expected_operating_cash_flow);

    // 4. Test account reconciliation
    let account_reconciliation = finance_report_service.generate_account_reconciliation(
        &mut conn,
        cash_account.id,
        period_start,
        period_end,
    ).unwrap();

    // Should show opening balance (0), transactions, and closing balance
    assert_eq!(account_reconciliation.opening_balance, 0);
    assert_eq!(account_reconciliation.closing_balance, expected_cash_balance);

    // Verify transaction count and totals
    assert_eq!(account_reconciliation.transaction_count, 2); // One debit, one credit
    let total_debits = account_reconciliation.transactions.iter()
        .filter(|t| t.debit_credit == "debit")
        .map(|t| t.amount)
        .sum::<i32>();
    let total_credits = account_reconciliation.transactions.iter()
        .filter(|t| t.debit_credit == "credit")
        .map(|t| t.amount)
        .sum::<i32>();

    assert_eq!(total_debits, 100000);  // Sale
    assert_eq!(total_credits, 30000); // Rent payment
}

#[test]
fn test_inventory_report_accuracy() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");

    // Services
    let category_service = CategoryService::new();
    let product_service = ProductService::new();
    let supplier_service = SupplierService::new();
    let po_service = PurchaseOrderService::new();
    let inventory_report_service = InventoryReportService::new();

    // 1. Create test data
    let category = category_service.create_category(
        "Electronics",
        Some("Electronic devices"),
        None,
    ).unwrap();

    let supplier = supplier_service.create_supplier(
        "ELEC001",
        "Electronics Supplier",
        None,
        None,
        None,
        None,
        None,
    ).unwrap();

    // Create products with different stock levels
    let product1 = product_service.create_product(
        "LAPTOP001",
        "Business Laptop",
        Some("High-performance laptop"),
        category.id,
        150000, // $1500 selling price
        120000, // $1200 cost
        25,     // 25 units in stock
        10,     // Min 10 units
        Some(50),
        "EA",
        None,
    ).unwrap();

    let product2 = product_service.create_product(
        "MOUSE001",
        "Wireless Mouse",
        Some("Ergonomic wireless mouse"),
        category.id,
        5000,  // $50 selling price
        3000,  // $30 cost
        5,     // 5 units in stock (below min)
        15,    // Min 15 units
        Some(100),
        "EA",
        None,
    ).unwrap();

    let product3 = product_service.create_product(
        "KEYBOARD001",
        "Mechanical Keyboard",
        Some("RGB mechanical keyboard"),
        category.id,
        12000, // $120 selling price
        8000,  // $80 cost
        50,    // 50 units in stock
        20,    // Min 20 units
        Some(200),
        "EA",
        None,
    ).unwrap();

    // 2. Create stock movements
    let movement_date = NaiveDate::from_ymd_opt(2024, 10, 15).unwrap();

    // Simulate sales
    let _laptop_sale = product_service.update_stock(
        product1.id,
        -3, // Sell 3 laptops
        "out",
        None,
        Some("sale"),
        None,
        Some("Corporate order"),
        Some(1),
    ).unwrap();

    let _mouse_sale = product_service.update_stock(
        product2.id,
        -2, // Sell 2 mice
        "out",
        None,
        Some("sale"),
        None,
        Some("Retail sale"),
        Some(1),
    ).unwrap();

    // Simulate restocking
    let _keyboard_restock = product_service.update_stock(
        product3.id,
        15, // Add 15 keyboards
        "in",
        Some(8000),
        Some("purchase"),
        None,
        Some("Restocking"),
        Some(1),
    ).unwrap();

    // 3. Generate and verify inventory reports

    // Stock levels report
    let stock_report = inventory_report_service.generate_stock_levels_report(
        &mut conn,
        Some(category.id),
    ).unwrap();

    assert_eq!(stock_report.total_products, 3);

    // Find specific products and verify their stock levels
    let laptop_stock = stock_report.products.iter()
        .find(|p| p.product_sku == "LAPTOP001")
        .unwrap();
    assert_eq!(laptop_stock.current_stock, 22); // 25 - 3 = 22

    let mouse_stock = stock_report.products.iter()
        .find(|p| p.product_sku == "MOUSE001")
        .unwrap();
    assert_eq!(mouse_stock.current_stock, 3); // 5 - 2 = 3

    let keyboard_stock = stock_report.products.iter()
        .find(|p| p.product_sku == "KEYBOARD001")
        .unwrap();
    assert_eq!(keyboard_stock.current_stock, 65); // 50 + 15 = 65

    // Low stock report
    let low_stock_report = inventory_report_service.generate_low_stock_report(
        &mut conn,
    ).unwrap();

    // Mouse should be in low stock (3 < 15)
    assert_eq!(low_stock_report.total_low_stock_items, 1);
    let low_stock_item = &low_stock_report.products[0];
    assert_eq!(low_stock_item.product_sku, "MOUSE001");
    assert_eq!(low_stock_item.current_stock, 3);
    assert_eq!(low_stock_item.min_stock_level, 15);

    // Stock movement report
    let movement_start = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
    let movement_end = NaiveDate::from_ymd_opt(2024, 10, 31).unwrap();

    let movement_report = inventory_report_service.generate_stock_movement_report(
        &mut conn,
        Some(product1.id), // Laptop only
        movement_start,
        movement_end,
    ).unwrap();

    // Should show initial stock + sale movement
    assert_eq!(movement_report.total_movements, 2); // Initial + sale

    let sale_movement = movement_report.movements.iter()
        .find(|m| m.movement_type == "out")
        .unwrap();
    assert_eq!(sale_movement.quantity, -3);

    // Inventory valuation report
    let valuation_report = inventory_report_service.generate_inventory_valuation_report(
        &mut conn,
        NaiveDate::from_ymd_opt(2024, 10, 31).unwrap(),
    ).unwrap();

    // Calculate expected total value
    let expected_value =
        22 * 120000 + // 22 laptops at $1200 cost each
        3 * 3000 +    // 3 mice at $30 cost each
        65 * 8000;    // 65 keyboards at $80 cost each

    assert_eq!(valuation_report.total_inventory_value, expected_value);
    assert_eq!(valuation_report.total_products, 3);

    // Verify individual product valuations
    let laptop_valuation = valuation_report.products.iter()
        .find(|p| p.product_sku == "LAPTOP001")
        .unwrap();
    assert_eq!(laptop_valuation.total_value, 22 * 120000);

    // Stock turnover report (if implemented)
    let turnover_report = inventory_report_service.generate_stock_turnover_report(
        &mut conn,
        movement_start,
        movement_end,
    ).unwrap();

    // For products that had sales, turnover should be calculated
    let laptop_turnover = turnover_report.products.iter()
        .find(|p| p.product_sku == "LAPTOP001")
        .unwrap();

    // Turnover = COGS / Average Inventory
    // COGS = 3 * 120000 = 360000
    // Average Inventory = ((25 * 120000) + (22 * 120000)) / 2 = (3000000 + 2640000) / 2
    let expected_cogs = 3 * 120000;
    assert_eq!(laptop_turnover.cost_of_goods_sold, expected_cogs);
}

#[test]
fn test_crm_report_accuracy() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");

    // Services
    let customer_service = CustomerService::new();
    let lead_service = LeadService::new();
    let deal_service = DealService::new();
    let campaign_service = CampaignService::new();
    let crm_report_service = CRMReportService::new();

    // 1. Create test data
    let customer1 = customer_service.create_customer(
        &mut conn,
        "Big Corp Ltd",
        crate::database::models::CustomerType::Business,
        Some("contact@bigcorp.com"),
        None,
        None,
        Some("Big Corp Ltd"),
        None,
        Some(100000),
        None,
    ).unwrap();

    let customer2 = customer_service.create_customer(
        &mut conn,
        "Small Business Inc",
        crate::database::models::CustomerType::Business,
        Some("info@smallbiz.com"),
        None,
        None,
        Some("Small Business Inc"),
        None,
        Some(50000),
        None,
    ).unwrap();

    let customer3 = customer_service.create_customer(
        &mut conn,
        "John Individual",
        crate::database::models::CustomerType::Individual,
        Some("john@email.com"),
        None,
        None,
        None,
        None,
        Some(10000),
        None,
    ).unwrap();

    // 2. Create leads with different outcomes
    let lead1 = lead_service.create_lead(
        &mut conn,
        "Enterprise Software Deal",
        Some(customer1.id),
        "Direct Sales",
        200000, // $2000
        Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        crate::database::models::LeadPriority::High,
        Some(1),
        None,
        None,
    ).unwrap();

    let lead2 = lead_service.create_lead(
        &mut conn,
        "Small Business Package",
        Some(customer2.id),
        "Website",
        50000, // $500
        Some(NaiveDate::from_ymd_opt(2024, 11, 30).unwrap()),
        crate::database::models::LeadPriority::Medium,
        Some(1),
        None,
        None,
    ).unwrap();

    let lead3 = lead_service.create_lead(
        &mut conn,
        "Personal License",
        Some(customer3.id),
        "Referral",
        15000, // $150
        Some(NaiveDate::from_ymd_opt(2024, 10, 31).unwrap()),
        crate::database::models::LeadPriority::Low,
        Some(1),
        None,
        None,
    ).unwrap();

    let lead4 = lead_service.create_lead(
        &mut conn,
        "Lost Opportunity",
        Some(customer2.id),
        "Cold Call",
        30000, // $300
        Some(NaiveDate::from_ymd_opt(2024, 10, 15).unwrap()),
        crate::database::models::LeadPriority::Medium,
        Some(1),
        None,
        None,
    ).unwrap();

    // 3. Convert some leads to deals and close them
    let deal1 = deal_service.create_deal_from_lead(
        &mut conn,
        lead1.id,
        "Enterprise Deal",
        220000, // Negotiated up
        Some(NaiveDate::from_ymd_opt(2024, 11, 30).unwrap()),
        None,
    ).unwrap();

    let deal2 = deal_service.create_deal_from_lead(
        &mut conn,
        lead2.id,
        "Small Business Deal",
        45000, // Negotiated down
        Some(NaiveDate::from_ymd_opt(2024, 10, 31).unwrap()),
        None,
    ).unwrap();

    let deal3 = deal_service.create_deal_from_lead(
        &mut conn,
        lead3.id,
        "Personal Deal",
        15000, // Same value
        Some(NaiveDate::from_ymd_opt(2024, 10, 15).unwrap()),
        None,
    ).unwrap();

    // Close some deals
    let _closed_deal1 = deal_service.close_deal(
        &mut conn,
        deal1.id,
        crate::database::models::DealStage::ClosedWon,
        Some(NaiveDate::from_ymd_opt(2024, 10, 25).unwrap()),
        Some("Contract signed"),
        Some(1),
    ).unwrap();

    let _closed_deal2 = deal_service.close_deal(
        &mut conn,
        deal2.id,
        crate::database::models::DealStage::ClosedWon,
        Some(NaiveDate::from_ymd_opt(2024, 10, 20).unwrap()),
        Some("Purchase order received"),
        Some(1),
    ).unwrap();

    let _closed_deal3 = deal_service.close_deal(
        &mut conn,
        deal3.id,
        crate::database::models::DealStage::ClosedLost,
        Some(NaiveDate::from_ymd_opt(2024, 10, 18).unwrap()),
        Some("Budget constraints"),
        Some(1),
    ).unwrap();

    // Mark lost lead
    let _lost_lead = lead_service.update_lead_status(
        &mut conn,
        lead4.id,
        crate::database::models::LeadStatus::ClosedLost,
        Some("Not interested"),
        Some(1),
    ).unwrap();

    // 4. Create campaign and associate leads
    let campaign = campaign_service.create_campaign(
        &mut conn,
        "Q4 2024 Sales Push",
        Some("End of year sales campaign"),
        Some(NaiveDate::from_ymd_opt(2024, 10, 1).unwrap()),
        Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        25000, // $250 budget
        Some("All business customers"),
        vec!["email", "phone"],
    ).unwrap();

    let _campaign_lead1 = campaign_service.add_lead_to_campaign(&mut conn, campaign.id, lead1.id).unwrap();
    let _campaign_lead2 = campaign_service.add_lead_to_campaign(&mut conn, campaign.id, lead2.id).unwrap();

    // 5. Generate and verify CRM reports

    // Sales pipeline report
    let pipeline_report = crm_report_service.generate_sales_pipeline_report(
        &mut conn,
        Some(1), // For specific sales rep
    ).unwrap();

    // Should show deals in various stages
    assert!(pipeline_report.total_deals >= 3);
    let total_pipeline_value = pipeline_report.deals.iter().map(|d| d.value).sum::<i32>();
    assert!(total_pipeline_value > 0);

    // Won deals should be in closed_won
    let won_deals: Vec<_> = pipeline_report.deals.iter()
        .filter(|d| d.stage == "closed_won")
        .collect();
    assert_eq!(won_deals.len(), 2); // deal1 and deal2

    // Lead conversion report
    let period_start = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
    let period_end = NaiveDate::from_ymd_opt(2024, 10, 31).unwrap();

    let conversion_report = crm_report_service.generate_lead_conversion_report(
        &mut conn,
        period_start,
        period_end,
        Some(1), // For specific sales rep
    ).unwrap();

    // Should show conversion metrics
    assert_eq!(conversion_report.total_leads, 4); // All 4 leads created
    assert_eq!(conversion_report.converted_leads, 2); // 2 deals won
    assert_eq!(conversion_report.lost_leads, 2); // 1 deal lost + 1 lead lost

    // Conversion rate should be 50% (2 won out of 4 total)
    let expected_conversion_rate = 50.0; // 2/4 * 100 = 50%
    assert!((conversion_report.conversion_rate - expected_conversion_rate).abs() < 0.1);

    // Total won value should be sum of won deals
    let expected_won_value = 220000 + 45000; // deal1 + deal2
    assert_eq!(conversion_report.total_won_value, expected_won_value);

    // Customer analysis report
    let customer_report = crm_report_service.generate_customer_analysis_report(
        &mut conn,
        period_start,
        period_end,
    ).unwrap();

    assert_eq!(customer_report.total_customers, 3);

    // Business customers should have higher value
    let business_customers = customer_report.customers.iter()
        .filter(|c| c.customer_type == "business")
        .count();
    assert_eq!(business_customers, 2);

    let individual_customers = customer_report.customers.iter()
        .filter(|c| c.customer_type == "individual")
        .count();
    assert_eq!(individual_customers, 1);

    // Campaign performance report
    let campaign_performance = crm_report_service.generate_campaign_performance_report(
        &mut conn,
        campaign.id,
    ).unwrap();

    assert_eq!(campaign_performance.campaign_name, "Q4 2024 Sales Push");
    assert_eq!(campaign_performance.total_leads, 2); // lead1 and lead2
    assert_eq!(campaign_performance.converted_leads, 2); // Both converted to won deals

    // ROI calculation: (Revenue - Cost) / Cost * 100
    let campaign_revenue = 220000 + 45000; // Won deals from campaign leads
    let campaign_cost = 25000; // Campaign budget
    let expected_roi = ((campaign_revenue - campaign_cost) as f64 / campaign_cost as f64) * 100.0;

    assert!((campaign_performance.roi - expected_roi).abs() < 0.1);

    // Sales performance report
    let sales_performance = crm_report_service.generate_sales_performance_report(
        &mut conn,
        1, // Sales rep ID
        period_start,
        period_end,
    ).unwrap();

    assert_eq!(sales_performance.total_leads_assigned, 4);
    assert_eq!(sales_performance.deals_won, 2);
    assert_eq!(sales_performance.deals_lost, 1);
    assert_eq!(sales_performance.total_revenue, expected_won_value);

    // Average deal size
    let expected_avg_deal_size = expected_won_value / 2; // 2 won deals
    assert_eq!(sales_performance.average_deal_size, expected_avg_deal_size);
}

#[test]
fn test_cross_module_report_consistency() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");

    // This test verifies that reports from different modules are consistent
    // when they reference the same underlying data

    // Services
    let customer_service = CustomerService::new();
    let lead_service = LeadService::new();
    let deal_service = DealService::new();
    let account_service = AccountService::new();
    let transaction_service = TransactionService::new();
    let crm_report_service = CRMReportService::new();
    let finance_report_service = FinanceReportService::new();

    // 1. Create a complete sales-to-finance scenario
    let customer = customer_service.create_customer(
        &mut conn,
        "Consistency Corp",
        crate::database::models::CustomerType::Business,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ).unwrap();

    let lead = lead_service.create_lead(
        &mut conn,
        "Consistency Deal",
        Some(customer.id),
        "Direct",
        100000, // $1000
        None,
        crate::database::models::LeadPriority::High,
        Some(1),
        None,
        None,
    ).unwrap();

    let deal = deal_service.create_deal_from_lead(
        &mut conn,
        lead.id,
        "Consistency Sale",
        100000, // Same value
        Some(NaiveDate::from_ymd_opt(2024, 10, 31).unwrap()),
        None,
    ).unwrap();

    let closed_deal = deal_service.close_deal(
        &mut conn,
        deal.id,
        crate::database::models::DealStage::ClosedWon,
        Some(NaiveDate::from_ymd_opt(2024, 10, 25).unwrap()),
        Some("Consistency test"),
        Some(1),
    ).unwrap();

    // 2. Record the same sale in accounting
    let revenue_account = account_service.create_account(
        "4100",
        "Sales Revenue",
        crate::database::models::AccountType::Revenue,
        None,
    ).unwrap();

    let receivables_account = account_service.create_account(
        "1200",
        "Accounts Receivable",
        crate::database::models::AccountType::Asset,
        None,
    ).unwrap();

    let sale_date = NaiveDate::from_ymd_opt(2024, 10, 25).unwrap();

    let _receivable_entry = transaction_service.create_transaction(
        receivables_account.id,
        sale_date,
        100000, // Same amount as deal
        "debit",
        &format!("Sale - Deal #{}", deal.id),
        Some(&format!("DEAL-{}", deal.id)),
        Some(1),
    ).unwrap();

    let _revenue_entry = transaction_service.create_transaction(
        revenue_account.id,
        sale_date,
        100000, // Same amount as deal
        "credit",
        &format!("Revenue - Deal #{}", deal.id),
        Some(&format!("DEAL-{}", deal.id)),
        Some(1),
    ).unwrap();

    // 3. Generate reports from both modules
    let period_start = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
    let period_end = NaiveDate::from_ymd_opt(2024, 10, 31).unwrap();

    // CRM sales report
    let crm_conversion_report = crm_report_service.generate_lead_conversion_report(
        &mut conn,
        period_start,
        period_end,
        Some(1),
    ).unwrap();

    // Finance income statement
    let income_statement = finance_report_service.generate_income_statement(
        &mut conn,
        period_start,
        period_end,
    ).unwrap();

    // 4. Verify consistency between reports

    // Revenue in finance report should match won deals in CRM report
    assert_eq!(crm_conversion_report.total_won_value, 100000);
    assert_eq!(income_statement.total_revenue, 100000);

    // Both should show the same revenue amount
    assert_eq!(
        crm_conversion_report.total_won_value,
        income_statement.total_revenue
    );

    // 5. Test that we can trace the connection

    // Check that the transaction references the deal
    let receivables_balance = account_service.get_account_by_id(receivables_account.id).unwrap();
    assert_eq!(receivables_balance.balance, 100000);

    let revenue_balance = account_service.get_account_by_id(revenue_account.id).unwrap();
    assert_eq!(revenue_balance.balance, 100000);

    // Generate account reconciliation to see transaction details
    let reconciliation = finance_report_service.generate_account_reconciliation(
        &mut conn,
        receivables_account.id,
        period_start,
        period_end,
    ).unwrap();

    let deal_transaction = reconciliation.transactions.iter()
        .find(|t| t.reference.as_ref().map_or(false, |r| r.contains(&deal.id.to_string())))
        .unwrap();

    assert_eq!(deal_transaction.amount, closed_deal.value);

    // 6. Test date consistency
    assert_eq!(closed_deal.close_date.unwrap(), sale_date);
    assert_eq!(deal_transaction.transaction_date, sale_date);

    // 7. Test that both modules show consistent metrics
    assert_eq!(crm_conversion_report.converted_leads, 1);
    assert_eq!(crm_conversion_report.total_won_value, income_statement.total_revenue);

    // Conversion rate should be 100% (1 lead, 1 conversion)
    assert!((crm_conversion_report.conversion_rate - 100.0).abs() < 0.1);
}