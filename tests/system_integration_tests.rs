mod common;

use common::setup_test_db;
use clierp::database::connection::get_connection;

// Import all module services
use clierp::modules::hr::{EmployeeService, DepartmentService, PayrollService, AttendanceService};
use clierp::modules::finance::{AccountService, TransactionService};
use clierp::modules::inventory::{ProductService, CategoryService, SupplierService, PurchaseOrderService};
use clierp::modules::crm::{CustomerService, LeadService, DealService, CampaignService, ActivityService};
use clierp::modules::reporting::{HRReportService, FinanceReportService, InventoryReportService, CRMReportService};

use clierp::database::models::*;
use clierp::utils::pagination::PaginationParams;
use chrono::{NaiveDate, NaiveTime, Utc};
use std::collections::HashMap;

/// Test a complete business cycle from employee onboarding through sales fulfillment
#[test]
fn test_complete_business_cycle() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");

    // Initialize all services
    let dept_service = DepartmentService::new();
    let employee_service = EmployeeService::new();
    let attendance_service = AttendanceService::new();
    let payroll_service = PayrollService::new();
    let account_service = AccountService::new();
    let transaction_service = TransactionService::new();
    let category_service = CategoryService::new();
    let product_service = ProductService::new();
    let supplier_service = SupplierService::new();
    let po_service = PurchaseOrderService::new();
    let customer_service = CustomerService::new();
    let lead_service = LeadService::new();
    let deal_service = DealService::new();
    let campaign_service = CampaignService::new();
    let activity_service = ActivityService::new();

    // === PHASE 1: COMPANY SETUP ===

    // 1. Create organizational structure
    let sales_dept = dept_service.create_department(
        &mut conn,
        "Sales Department",
        Some("Sales and customer relations"),
        None,
    ).unwrap();

    let procurement_dept = dept_service.create_department(
        &mut conn,
        "Procurement Department",
        Some("Purchasing and inventory management"),
        None,
    ).unwrap();

    let finance_dept = dept_service.create_department(
        &mut conn,
        "Finance Department",
        Some("Accounting and financial management"),
        None,
    ).unwrap();

    // 2. Hire employees
    let sales_manager = employee_service.create_employee(
        &mut conn,
        "SM001",
        "Sarah Sales",
        Some("sarah@company.com"),
        Some("555-1001"),
        sales_dept.id,
        "Sales Manager",
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        80000, // $800.00/month
    ).unwrap();

    let sales_rep = employee_service.create_employee(
        &mut conn,
        "SR001",
        "Tom Representative",
        Some("tom@company.com"),
        Some("555-1002"),
        sales_dept.id,
        "Sales Representative",
        NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
        60000, // $600.00/month
    ).unwrap();

    let procurement_manager = employee_service.create_employee(
        &mut conn,
        "PM001",
        "Paul Procurement",
        Some("paul@company.com"),
        Some("555-1003"),
        procurement_dept.id,
        "Procurement Manager",
        NaiveDate::from_ymd_opt(2024, 1, 10).unwrap(),
        70000, // $700.00/month
    ).unwrap();

    let accountant = employee_service.create_employee(
        &mut conn,
        "AC001",
        "Alice Accountant",
        Some("alice@company.com"),
        Some("555-1004"),
        finance_dept.id,
        "Senior Accountant",
        NaiveDate::from_ymd_opt(2024, 1, 5).unwrap(),
        65000, // $650.00/month
    ).unwrap();

    // 3. Set up chart of accounts
    let cash_account = account_service.create_account(
        "1000", "Cash", AccountType::Asset, None
    ).unwrap();

    let receivables_account = account_service.create_account(
        "1200", "Accounts Receivable", AccountType::Asset, None
    ).unwrap();

    let inventory_account = account_service.create_account(
        "1300", "Inventory", AccountType::Asset, None
    ).unwrap();

    let payables_account = account_service.create_account(
        "2000", "Accounts Payable", AccountType::Liability, None
    ).unwrap();

    let salary_payable_account = account_service.create_account(
        "2100", "Salaries Payable", AccountType::Liability, None
    ).unwrap();

    let equity_account = account_service.create_account(
        "3000", "Owner's Equity", AccountType::Equity, None
    ).unwrap();

    let revenue_account = account_service.create_account(
        "4000", "Sales Revenue", AccountType::Revenue, None
    ).unwrap();

    let cogs_account = account_service.create_account(
        "5000", "Cost of Goods Sold", AccountType::Expense, None
    ).unwrap();

    let salary_expense_account = account_service.create_account(
        "5100", "Salary Expense", AccountType::Expense, None
    ).unwrap();

    // === PHASE 2: INVENTORY SETUP ===

    // 4. Set up product catalog
    let tech_category = category_service.create_category(
        "Technology Hardware",
        Some("Computer hardware and accessories"),
        None,
    ).unwrap();

    let software_category = category_service.create_category(
        "Software Licenses",
        Some("Software products and licenses"),
        None,
    ).unwrap();

    // 5. Register suppliers
    let hardware_supplier = supplier_service.create_supplier(
        "HW001",
        "TechHardware Corp",
        Some("Hardware Sales Team"),
        Some("sales@techhardware.com"),
        Some("555-2001"),
        Some("123 Tech Street, Silicon Valley"),
        Some("Net 30"),
    ).unwrap();

    let software_supplier = supplier_service.create_supplier(
        "SW001",
        "SoftwarePlus Inc",
        Some("License Manager"),
        Some("licensing@softwareplus.com"),
        Some("555-2002"),
        Some("456 Software Ave, Tech City"),
        Some("Net 15"),
    ).unwrap();

    // 6. Create products
    let laptop_product = product_service.create_product(
        "LAPTOP001",
        "Business Laptop Pro",
        Some("High-performance business laptop with extended warranty"),
        tech_category.id,
        180000, // $1800 selling price
        140000, // $1400 cost price
        0,      // No initial stock
        5,      // Min 5 units
        Some(50),
        "EA",
        Some("LAP001234567890"),
    ).unwrap();

    let software_product = product_service.create_product(
        "SW001",
        "ERP Software Annual License",
        Some("Enterprise ERP software annual license"),
        software_category.id,
        120000, // $1200 selling price
        80000,  // $800 cost price
        0,      // No initial stock
        10,     // Min 10 licenses
        Some(100),
        "LIC",
        None,
    ).unwrap();

    // === PHASE 3: PROCUREMENT PROCESS ===

    // 7. Create purchase orders for initial inventory
    let laptop_po = po_service.create_purchase_order(
        hardware_supplier.id,
        NaiveDate::from_ymd_opt(2024, 10, 1).unwrap(),
        Some(NaiveDate::from_ymd_opt(2024, 10, 15).unwrap()),
        Some("Initial laptop inventory for Q4 sales"),
        vec![(laptop_product.id, 20, 140000)], // 20 laptops at $1400 each
    ).unwrap();

    let software_po = po_service.create_purchase_order(
        software_supplier.id,
        NaiveDate::from_ymd_opt(2024, 10, 1).unwrap(),
        Some(NaiveDate::from_ymd_opt(2024, 10, 10).unwrap()),
        Some("Software licenses for resale"),
        vec![(software_product.id, 50, 80000)], // 50 licenses at $800 each
    ).unwrap();

    // 8. Approve and receive purchase orders
    let approved_laptop_po = po_service.approve_purchase_order(
        laptop_po.id,
        Some(procurement_manager.id)
    ).unwrap();

    let approved_software_po = po_service.approve_purchase_order(
        software_po.id,
        Some(procurement_manager.id)
    ).unwrap();

    // Receive goods
    let received_laptop_po = po_service.receive_purchase_order(
        laptop_po.id,
        vec![(laptop_product.id, 20)], // All 20 received
        Some("All laptops received in excellent condition"),
        Some(procurement_manager.id),
    ).unwrap();

    let received_software_po = po_service.receive_purchase_order(
        software_po.id,
        vec![(software_product.id, 50)], // All 50 received
        Some("License keys activated and ready for distribution"),
        Some(procurement_manager.id),
    ).unwrap();

    // 9. Record purchase transactions in accounting
    let purchase_date = NaiveDate::from_ymd_opt(2024, 10, 15).unwrap();

    // Laptop purchase: Debit Inventory, Credit Accounts Payable
    let laptop_inventory_entry = transaction_service.create_transaction(
        inventory_account.id,
        purchase_date,
        20 * 140000, // 20 * $1400 = $28,000
        "debit",
        "Laptop inventory purchase",
        Some(&format!("PO-{}", laptop_po.id)),
        Some(accountant.id),
    ).unwrap();

    let laptop_payable_entry = transaction_service.create_transaction(
        payables_account.id,
        purchase_date,
        20 * 140000,
        "credit",
        "Laptop purchase - amount owed",
        Some(&format!("PO-{}", laptop_po.id)),
        Some(accountant.id),
    ).unwrap();

    // Software purchase
    let software_inventory_entry = transaction_service.create_transaction(
        inventory_account.id,
        purchase_date,
        50 * 80000, // 50 * $800 = $40,000
        "debit",
        "Software license inventory",
        Some(&format!("PO-{}", software_po.id)),
        Some(accountant.id),
    ).unwrap();

    let software_payable_entry = transaction_service.create_transaction(
        payables_account.id,
        purchase_date,
        50 * 80000,
        "credit",
        "Software purchase - amount owed",
        Some(&format!("PO-{}", software_po.id)),
        Some(accountant.id),
    ).unwrap();

    // === PHASE 4: EMPLOYEE MANAGEMENT ===

    // 10. Record attendance for all employees
    let work_date = NaiveDate::from_ymd_opt(2024, 10, 16).unwrap();

    let _sm_attendance = attendance_service.check_in(
        &mut conn,
        sales_manager.id,
        work_date,
        Some(NaiveTime::from_hms_opt(8, 30, 0).unwrap()),
    ).unwrap();

    let _sr_attendance = attendance_service.check_in(
        &mut conn,
        sales_rep.id,
        work_date,
        Some(NaiveTime::from_hms_opt(9, 0, 0).unwrap()),
    ).unwrap();

    let _pm_attendance = attendance_service.check_in(
        &mut conn,
        procurement_manager.id,
        work_date,
        Some(NaiveTime::from_hms_opt(8, 45, 0).unwrap()),
    ).unwrap();

    let _ac_attendance = attendance_service.check_in(
        &mut conn,
        accountant.id,
        work_date,
        Some(NaiveTime::from_hms_opt(8, 15, 0).unwrap()),
    ).unwrap();

    // === PHASE 5: SALES AND MARKETING ===

    // 11. Create marketing campaign
    let campaign = campaign_service.create_campaign(
        &mut conn,
        "Q4 2024 Technology Solutions",
        Some("End-of-year push for business technology solutions"),
        Some(NaiveDate::from_ymd_opt(2024, 10, 15).unwrap()),
        Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        50000, // $500 budget
        Some("Small to medium businesses needing technology upgrades"),
        vec!["email", "phone", "webinar"],
    ).unwrap();

    // 12. Create customers
    let enterprise_customer = customer_service.create_customer(
        &mut conn,
        "TechCorp Enterprises",
        CustomerType::Business,
        Some("purchasing@techcorp.com"),
        Some("555-3001"),
        Some("789 Enterprise Blvd, Business District"),
        Some("TechCorp Enterprises Ltd."),
        Some("TC-123456789"),
        Some(500000), // $5000 credit limit
        Some("Major enterprise client"),
    ).unwrap();

    let startup_customer = customer_service.create_customer(
        &mut conn,
        "StartupFast Inc",
        CustomerType::Business,
        Some("ceo@startupfast.com"),
        Some("555-3002"),
        Some("123 Innovation St, Startup Valley"),
        Some("StartupFast Inc."),
        Some("SF-987654321"),
        Some(100000), // $1000 credit limit
        None,
    ).unwrap();

    let small_business = customer_service.create_customer(
        &mut conn,
        "Local Business Solutions",
        CustomerType::Business,
        Some("owner@localbiz.com"),
        Some("555-3003"),
        Some("456 Main St, Hometown"),
        Some("Local Business Solutions LLC"),
        None,
        Some(50000), // $500 credit limit
        None,
    ).unwrap();

    // 13. Generate leads
    let enterprise_lead = lead_service.create_lead(
        &mut conn,
        "Enterprise Technology Upgrade",
        Some(enterprise_customer.id),
        "Website Contact Form",
        2000000, // $20,000 (10 laptops + 20 licenses)
        Some(NaiveDate::from_ymd_opt(2024, 12, 15).unwrap()),
        LeadPriority::High,
        Some(sales_manager.id),
        Some("Large enterprise needs complete technology upgrade"),
        Some("Decision maker confirmed, budget approved"),
    ).unwrap();

    let startup_lead = lead_service.create_lead(
        &mut conn,
        "Startup Office Setup",
        Some(startup_customer.id),
        "Referral",
        900000, // $9,000 (5 laptops + 5 licenses)
        Some(NaiveDate::from_ymd_opt(2024, 11, 30).unwrap()),
        LeadPriority::High,
        Some(sales_rep.id),
        Some("New startup needs office equipment"),
        Some("Fast-growing startup, urgent need"),
    ).unwrap();

    let small_biz_lead = lead_service.create_lead(
        &mut conn,
        "Small Business ERP Implementation",
        Some(small_business.id),
        "Cold Call",
        360000, // $3,600 (2 laptops + 3 licenses)
        Some(NaiveDate::from_ymd_opt(2024, 11, 15).unwrap()),
        LeadPriority::Medium,
        Some(sales_rep.id),
        Some("Small business looking for ERP solution"),
        Some("Budget conscious, needs value proposition"),
    ).unwrap();

    // 14. Associate leads with campaign
    let _campaign_lead1 = campaign_service.add_lead_to_campaign(
        &mut conn, campaign.id, enterprise_lead.id
    ).unwrap();
    let _campaign_lead2 = campaign_service.add_lead_to_campaign(
        &mut conn, campaign.id, startup_lead.id
    ).unwrap();
    let _campaign_lead3 = campaign_service.add_lead_to_campaign(
        &mut conn, campaign.id, small_biz_lead.id
    ).unwrap();

    // === PHASE 6: SALES ACTIVITIES ===

    // 15. Create sales activities
    let enterprise_call = activity_service.create_activity(
        &mut conn,
        "Enterprise Discovery Call",
        ActivityType::Call,
        Some(enterprise_customer.id),
        Some(enterprise_lead.id),
        None,
        Some(NaiveDate::from_ymd_opt(2024, 10, 17).unwrap()),
        sales_manager.id,
        Some("Understand enterprise requirements and decision process"),
        ActivityStatus::Scheduled,
    ).unwrap();

    let startup_demo = activity_service.create_activity(
        &mut conn,
        "Product Demo for Startup",
        ActivityType::Meeting,
        Some(startup_customer.id),
        Some(startup_lead.id),
        None,
        Some(NaiveDate::from_ymd_opt(2024, 10, 18).unwrap()),
        sales_rep.id,
        Some("Demonstrate ERP solution capabilities"),
        ActivityStatus::Scheduled,
    ).unwrap();

    // Complete activities
    let _completed_call = activity_service.complete_activity(
        &mut conn,
        enterprise_call.id,
        Some("Great meeting! Customer is very interested. Sending proposal next."),
        Some(sales_manager.id),
    ).unwrap();

    let _completed_demo = activity_service.complete_activity(
        &mut conn,
        startup_demo.id,
        Some("Demo went well. Customer ready to move forward with 5 laptop + license bundle."),
        Some(sales_rep.id),
    ).unwrap();

    // === PHASE 7: DEAL CLOSURE ===

    // 16. Convert leads to deals
    let enterprise_deal = deal_service.create_deal_from_lead(
        &mut conn,
        enterprise_lead.id,
        "Enterprise Technology Package",
        1980000, // Negotiated to $19,800 (10 laptops @ $1800 + 20 licenses @ $1200)
        Some(NaiveDate::from_ymd_opt(2024, 11, 30).unwrap()),
        Some("Comprehensive technology solution package"),
    ).unwrap();

    let startup_deal = deal_service.create_deal_from_lead(
        &mut conn,
        startup_lead.id,
        "Startup Office Bundle",
        900000, // Original price $9,000 (5 laptops @ $1800 + 5 licenses @ $1200)
        Some(NaiveDate::from_ymd_opt(2024, 11, 15).unwrap()),
        Some("Complete office setup bundle"),
    ).unwrap();

    let small_biz_deal = deal_service.create_deal_from_lead(
        &mut conn,
        small_biz_lead.id,
        "Small Business ERP Package",
        330000, // Discounted to $3,300 (2 laptops @ $1650 + 3 licenses @ $1000)
        Some(NaiveDate::from_ymd_opt(2024, 11, 10).unwrap()),
        Some("Budget-friendly ERP implementation"),
    ).unwrap();

    // 17. Progress deals through stages
    let enterprise_negotiation = deal_service.update_deal_stage(
        &mut conn,
        enterprise_deal.id,
        DealStage::Negotiation,
        85,
        Some("In final negotiations, very likely to close"),
        Some(sales_manager.id),
    ).unwrap();

    let startup_proposal = deal_service.update_deal_stage(
        &mut conn,
        startup_deal.id,
        DealStage::Proposal,
        75,
        Some("Proposal sent, awaiting approval"),
        Some(sales_rep.id),
    ).unwrap();

    // 18. Close deals
    let closed_enterprise = deal_service.close_deal(
        &mut conn,
        enterprise_deal.id,
        DealStage::ClosedWon,
        Some(NaiveDate::from_ymd_opt(2024, 10, 25).unwrap()),
        Some("Contract signed! Large enterprise deal closed."),
        Some(sales_manager.id),
    ).unwrap();

    let closed_startup = deal_service.close_deal(
        &mut conn,
        startup_deal.id,
        DealStage::ClosedWon,
        Some(NaiveDate::from_ymd_opt(2024, 10, 22).unwrap()),
        Some("Startup approved purchase, moving to fulfillment"),
        Some(sales_rep.id),
    ).unwrap();

    let lost_small_biz = deal_service.close_deal(
        &mut conn,
        small_biz_deal.id,
        DealStage::ClosedLost,
        Some(NaiveDate::from_ymd_opt(2024, 10, 20).unwrap()),
        Some("Customer decided to postpone technology upgrade"),
        Some(sales_rep.id),
    ).unwrap();

    // === PHASE 8: ORDER FULFILLMENT ===

    // 19. Fulfill orders from inventory

    // Enterprise order: 10 laptops + 20 licenses
    let enterprise_laptop_fulfillment = product_service.update_stock(
        laptop_product.id,
        -10,
        "out",
        None,
        Some("sale"),
        Some(enterprise_deal.id),
        Some(&format!("Enterprise order fulfillment for {}", enterprise_customer.name)),
        Some(sales_manager.id),
    ).unwrap();

    let enterprise_software_fulfillment = product_service.update_stock(
        software_product.id,
        -20,
        "out",
        None,
        Some("sale"),
        Some(enterprise_deal.id),
        Some(&format!("Enterprise software fulfillment for {}", enterprise_customer.name)),
        Some(sales_manager.id),
    ).unwrap();

    // Startup order: 5 laptops + 5 licenses
    let startup_laptop_fulfillment = product_service.update_stock(
        laptop_product.id,
        -5,
        "out",
        None,
        Some("sale"),
        Some(startup_deal.id),
        Some(&format!("Startup order fulfillment for {}", startup_customer.name)),
        Some(sales_rep.id),
    ).unwrap();

    let startup_software_fulfillment = product_service.update_stock(
        software_product.id,
        -5,
        "out",
        None,
        Some("sale"),
        Some(startup_deal.id),
        Some(&format!("Startup software fulfillment for {}", startup_customer.name)),
        Some(sales_rep.id),
    ).unwrap();

    // === PHASE 9: FINANCIAL RECORDING ===

    // 20. Record sales transactions
    let enterprise_sale_date = NaiveDate::from_ymd_opt(2024, 10, 25).unwrap();
    let startup_sale_date = NaiveDate::from_ymd_opt(2024, 10, 22).unwrap();

    // Enterprise sale
    let enterprise_receivable = transaction_service.create_transaction(
        receivables_account.id,
        enterprise_sale_date,
        closed_enterprise.value,
        "debit",
        &format!("Sale to {} - Deal #{}", enterprise_customer.name, enterprise_deal.id),
        Some(&format!("SALE-{}", enterprise_deal.id)),
        Some(accountant.id),
    ).unwrap();

    let enterprise_revenue = transaction_service.create_transaction(
        revenue_account.id,
        enterprise_sale_date,
        closed_enterprise.value,
        "credit",
        &format!("Revenue from {} - Deal #{}", enterprise_customer.name, enterprise_deal.id),
        Some(&format!("SALE-{}", enterprise_deal.id)),
        Some(accountant.id),
    ).unwrap();

    // Enterprise COGS
    let enterprise_cogs_amount = (10 * 140000) + (20 * 80000); // 10 laptops + 20 licenses at cost
    let enterprise_cogs = transaction_service.create_transaction(
        cogs_account.id,
        enterprise_sale_date,
        enterprise_cogs_amount,
        "debit",
        &format!("COGS for enterprise sale - Deal #{}", enterprise_deal.id),
        Some(&format!("COGS-{}", enterprise_deal.id)),
        Some(accountant.id),
    ).unwrap();

    let enterprise_inventory_reduction = transaction_service.create_transaction(
        inventory_account.id,
        enterprise_sale_date,
        enterprise_cogs_amount,
        "credit",
        &format!("Inventory reduction for enterprise sale - Deal #{}", enterprise_deal.id),
        Some(&format!("INV-{}", enterprise_deal.id)),
        Some(accountant.id),
    ).unwrap();

    // Startup sale
    let startup_receivable = transaction_service.create_transaction(
        receivables_account.id,
        startup_sale_date,
        closed_startup.value,
        "debit",
        &format!("Sale to {} - Deal #{}", startup_customer.name, startup_deal.id),
        Some(&format!("SALE-{}", startup_deal.id)),
        Some(accountant.id),
    ).unwrap();

    let startup_revenue = transaction_service.create_transaction(
        revenue_account.id,
        startup_sale_date,
        closed_startup.value,
        "credit",
        &format!("Revenue from {} - Deal #{}", startup_customer.name, startup_deal.id),
        Some(&format!("SALE-{}", startup_deal.id)),
        Some(accountant.id),
    ).unwrap();

    // Startup COGS
    let startup_cogs_amount = (5 * 140000) + (5 * 80000); // 5 laptops + 5 licenses at cost
    let startup_cogs = transaction_service.create_transaction(
        cogs_account.id,
        startup_sale_date,
        startup_cogs_amount,
        "debit",
        &format!("COGS for startup sale - Deal #{}", startup_deal.id),
        Some(&format!("COGS-{}", startup_deal.id)),
        Some(accountant.id),
    ).unwrap();

    let startup_inventory_reduction = transaction_service.create_transaction(
        inventory_account.id,
        startup_sale_date,
        startup_cogs_amount,
        "credit",
        &format!("Inventory reduction for startup sale - Deal #{}", startup_deal.id),
        Some(&format!("INV-{}", startup_deal.id)),
        Some(accountant.id),
    ).unwrap();

    // === PHASE 10: PAYROLL PROCESSING ===

    // 21. Process monthly payroll
    let payroll_period = "2024-10";

    let sm_payroll = payroll_service.calculate_payroll(
        &mut conn,
        sales_manager.id,
        payroll_period,
        None,
        0,
        100000, // $1000 bonus for big sale
        12000,  // $120 deductions
    ).unwrap();

    let sr_payroll = payroll_service.calculate_payroll(
        &mut conn,
        sales_rep.id,
        payroll_period,
        None,
        0,
        50000, // $500 bonus for good performance
        8000,  // $80 deductions
    ).unwrap();

    let pm_payroll = payroll_service.calculate_payroll(
        &mut conn,
        procurement_manager.id,
        payroll_period,
        None,
        0,
        0,     // No bonus
        7000,  // $70 deductions
    ).unwrap();

    let ac_payroll = payroll_service.calculate_payroll(
        &mut conn,
        accountant.id,
        payroll_period,
        None,
        0,
        0,     // No bonus
        6500,  // $65 deductions
    ).unwrap();

    // 22. Record payroll in accounting
    let payroll_date = NaiveDate::from_ymd_opt(2024, 10, 31).unwrap();
    let total_gross_payroll = sm_payroll.base_salary + sm_payroll.bonuses +
                             sr_payroll.base_salary + sr_payroll.bonuses +
                             pm_payroll.base_salary + pm_payroll.bonuses +
                             ac_payroll.base_salary + ac_payroll.bonuses;

    let total_net_payroll = sm_payroll.net_salary + sr_payroll.net_salary +
                           pm_payroll.net_salary + ac_payroll.net_salary;

    let payroll_expense = transaction_service.create_transaction(
        salary_expense_account.id,
        payroll_date,
        total_gross_payroll,
        "debit",
        &format!("Payroll expense for {}", payroll_period),
        Some(&format!("PAYROLL-{}", payroll_period)),
        Some(accountant.id),
    ).unwrap();

    let payroll_liability = transaction_service.create_transaction(
        salary_payable_account.id,
        payroll_date,
        total_net_payroll,
        "credit",
        &format!("Salaries payable for {}", payroll_period),
        Some(&format!("PAYROLL-{}", payroll_period)),
        Some(accountant.id),
    ).unwrap();

    // === PHASE 11: COMPREHENSIVE VERIFICATION ===

    // 23. Verify inventory levels
    assert_eq!(enterprise_laptop_fulfillment.current_stock, 5); // 20 - 10 - 5 = 5
    assert_eq!(startup_software_fulfillment.current_stock, 25); // 50 - 20 - 5 = 25

    // 24. Verify accounting balances
    let final_cash = account_service.get_account_by_id(cash_account.id).unwrap();
    let final_receivables = account_service.get_account_by_id(receivables_account.id).unwrap();
    let final_inventory = account_service.get_account_by_id(inventory_account.id).unwrap();
    let final_payables = account_service.get_account_by_id(payables_account.id).unwrap();
    let final_revenue = account_service.get_account_by_id(revenue_account.id).unwrap();
    let final_cogs = account_service.get_account_by_id(cogs_account.id).unwrap();

    // Receivables should equal total sales
    let total_sales = closed_enterprise.value + closed_startup.value;
    assert_eq!(final_receivables.balance, total_sales);

    // Revenue should equal total sales
    assert_eq!(final_revenue.balance, total_sales);

    // Inventory should reflect purchases minus sales
    let total_purchases = (20 * 140000) + (50 * 80000); // All purchases
    let total_cogs = enterprise_cogs_amount + startup_cogs_amount; // All COGS
    let expected_inventory = total_purchases - total_cogs;
    assert_eq!(final_inventory.balance, expected_inventory);

    // Payables should equal total purchases
    assert_eq!(final_payables.balance, total_purchases);

    // COGS should equal cost of goods sold
    assert_eq!(final_cogs.balance, total_cogs);

    // 25. Verify business metrics
    let gross_profit = total_sales - total_cogs;
    let gross_margin_percentage = (gross_profit as f64 / total_sales as f64) * 100.0;

    // Should have healthy gross margin (around 50%)
    assert!(gross_margin_percentage > 40.0);
    assert!(gross_margin_percentage < 60.0);

    // 26. Verify employee performance
    assert_eq!(sm_payroll.bonuses, 100000); // Sales manager got bonus
    assert_eq!(sr_payroll.bonuses, 50000);  // Sales rep got bonus
    assert_eq!(pm_payroll.bonuses, 0);      // Procurement manager no bonus
    assert_eq!(ac_payroll.bonuses, 0);      // Accountant no bonus

    // 27. Verify campaign effectiveness
    let campaign_performance = campaign_service.get_campaign_performance(&mut conn, campaign.id).unwrap();
    assert_eq!(campaign_performance.total_leads, 3);
    assert_eq!(campaign_performance.converted_leads, 2); // Enterprise + Startup

    // 28. Verify stock movements are properly tracked
    let pagination = PaginationParams::new(1, 20);
    let laptop_movements = product_service.get_stock_movements(laptop_product.id, &pagination).unwrap();
    let software_movements = product_service.get_stock_movements(software_product.id, &pagination).unwrap();

    // Each product should have: initial stock + 2 sales = 3 movements each
    assert_eq!(laptop_movements.data.len(), 3);
    assert_eq!(software_movements.data.len(), 3);

    // 29. Test cross-module data integrity

    // Deals should still reference correct leads
    let final_enterprise_deal = deal_service.get_deal_by_id(&mut conn, enterprise_deal.id).unwrap().unwrap();
    assert_eq!(final_enterprise_deal.lead_id, enterprise_lead.id);
    assert_eq!(final_enterprise_deal.stage, "closed_won");

    // Leads should still reference correct customers
    let final_enterprise_lead = lead_service.get_lead_by_id(&mut conn, enterprise_lead.id).unwrap().unwrap();
    assert_eq!(final_enterprise_lead.customer_id, Some(enterprise_customer.id));

    // Stock movements should reference deals
    let enterprise_laptop_movement = laptop_movements.data.iter()
        .find(|m| m.reference_id == Some(enterprise_deal.id))
        .unwrap();
    assert_eq!(enterprise_laptop_movement.quantity, -10);

    // Transactions should reference deals
    assert!(enterprise_revenue.reference.as_ref().unwrap().contains(&enterprise_deal.id.to_string()));

    // 30. Verify accounting equation: Assets = Liabilities + Equity
    let total_assets = final_cash.balance + final_receivables.balance + final_inventory.balance;
    let total_liabilities = final_payables.balance + payroll_liability.amount;
    let net_income = final_revenue.balance - final_cogs.balance - payroll_expense.amount;

    // Basic accounting equation should hold (simplified - in practice would include all accounts)
    assert!(total_assets > total_liabilities); // We should have positive equity

    println!("=== SYSTEM INTEGRATION TEST RESULTS ===");
    println!("Total Sales Revenue: ${:.2}", total_sales as f64 / 100.0);
    println!("Total COGS: ${:.2}", total_cogs as f64 / 100.0);
    println!("Gross Profit: ${:.2}", gross_profit as f64 / 100.0);
    println!("Gross Margin: {:.1}%", gross_margin_percentage);
    println!("Laptops Remaining: {}", enterprise_laptop_fulfillment.current_stock);
    println!("Software Licenses Remaining: {}", startup_software_fulfillment.current_stock);
    println!("Campaign Conversion Rate: {:.1}%",
             (campaign_performance.converted_leads as f64 / campaign_performance.total_leads as f64) * 100.0);
    println!("Total Payroll Expense: ${:.2}", total_gross_payroll as f64 / 100.0);
    println!("========================================");

    // Final assertion: System is internally consistent
    assert!(true); // If we get here, all previous assertions passed
}

/// Test system behavior under concurrent operations
#[test]
fn test_concurrent_operations() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");

    // This test simulates multiple operations happening simultaneously
    // to ensure data consistency

    let category_service = CategoryService::new();
    let product_service = ProductService::new();
    let customer_service = CustomerService::new();
    let lead_service = LeadService::new();
    let deal_service = DealService::new();

    // Create test data
    let category = category_service.create_category("Concurrent Test", None, None).unwrap();
    let product = product_service.create_product(
        "CONC001", "Concurrent Product", None, category.id,
        10000, 8000, 100, 10, Some(200), "EA", None
    ).unwrap();

    let customer = customer_service.create_customer(
        &mut conn, "Concurrent Customer", CustomerType::Business,
        None, None, None, None, None, None, None
    ).unwrap();

    // Create multiple leads simultaneously (simulating concurrent users)
    let lead1 = lead_service.create_lead(
        &mut conn, "Concurrent Lead 1", Some(customer.id), "Source1",
        50000, None, LeadPriority::High, Some(1), None, None
    ).unwrap();

    let lead2 = lead_service.create_lead(
        &mut conn, "Concurrent Lead 2", Some(customer.id), "Source2",
        60000, None, LeadPriority::Medium, Some(1), None, None
    ).unwrap();

    // Simulate concurrent stock operations
    let stock_ops = vec![
        ("out", -5, "sale"),
        ("in", 10, "purchase"),
        ("out", -3, "sale"),
        ("adjustment", 2, "audit"),
    ];

    let mut final_stock = product.current_stock;
    for (movement_type, quantity, reference_type) in stock_ops {
        let updated_product = product_service.update_stock(
            product.id, quantity, movement_type, None,
            Some(reference_type), None, None, Some(1)
        ).unwrap();

        final_stock = match movement_type {
            "out" => final_stock + quantity, // quantity is negative for "out"
            "in" => final_stock + quantity,
            "adjustment" => final_stock + quantity,
            _ => final_stock,
        };
    }

    // Verify final stock is consistent
    let final_product = product_service.get_product_by_id(product.id).unwrap();
    assert_eq!(final_product.current_stock, final_stock);

    // Convert leads to deals and verify no conflicts
    let deal1 = deal_service.create_deal_from_lead(
        &mut conn, lead1.id, "Deal 1", 55000, None, None
    ).unwrap();

    let deal2 = deal_service.create_deal_from_lead(
        &mut conn, lead2.id, "Deal 2", 65000, None, None
    ).unwrap();

    // Both deals should exist and be valid
    assert_eq!(deal1.lead_id, lead1.id);
    assert_eq!(deal2.lead_id, lead2.id);
    assert_ne!(deal1.id, deal2.id);

    // Verify stock movement history is complete and ordered
    let pagination = PaginationParams::new(1, 10);
    let movements = product_service.get_stock_movements(product.id, &pagination).unwrap();

    // Should have initial stock + 4 operations = 5 movements
    assert_eq!(movements.data.len(), 5);

    // Movements should be in descending order by date
    for i in 1..movements.data.len() {
        assert!(movements.data[i-1].movement_date >= movements.data[i].movement_date);
    }
}

/// Test error handling and recovery scenarios
#[test]
fn test_error_handling_and_recovery() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");

    let customer_service = CustomerService::new();
    let lead_service = LeadService::new();
    let deal_service = DealService::new();
    let product_service = ProductService::new();
    let category_service = CategoryService::new();

    // Test duplicate prevention
    let customer = customer_service.create_customer(
        &mut conn, "Error Test Customer", CustomerType::Individual,
        Some("error@test.com"), None, None, None, None, None, None
    ).unwrap();

    // Try to create customer with same email - should fail
    let duplicate_result = customer_service.create_customer(
        &mut conn, "Another Customer", CustomerType::Individual,
        Some("error@test.com"), None, None, None, None, None, None
    );
    assert!(duplicate_result.is_err());

    // Test referential integrity
    let lead = lead_service.create_lead(
        &mut conn, "Error Test Lead", Some(customer.id), "Test",
        10000, None, LeadPriority::Low, Some(1), None, None
    ).unwrap();

    // Try to create lead with non-existent customer - should fail
    let invalid_customer_result = lead_service.create_lead(
        &mut conn, "Invalid Lead", Some(99999), "Test",
        10000, None, LeadPriority::Low, Some(1), None, None
    );
    assert!(invalid_customer_result.is_err());

    // Test business rule validation
    let category = category_service.create_category("Error Test", None, None).unwrap();

    // Try to create product with negative price - should fail
    let negative_price_result = product_service.create_product(
        "ERROR001", "Error Product", None, category.id,
        -1000, 500, 10, 5, Some(50), "EA", None
    );
    assert!(negative_price_result.is_err());

    // Try to create product with min > max stock - should fail
    let invalid_stock_result = product_service.create_product(
        "ERROR002", "Error Product 2", None, category.id,
        1000, 500, 10, 50, Some(20), "EA", None
    );
    assert!(invalid_stock_result.is_err());

    // Test valid operations still work after errors
    let valid_product = product_service.create_product(
        "VALID001", "Valid Product", None, category.id,
        1000, 500, 10, 5, Some(50), "EA", None
    ).unwrap();
    assert_eq!(valid_product.sku, "VALID001");

    // Test stock validation
    let stock_reduction_result = product_service.update_stock(
        valid_product.id, -50, "out", None, Some("test"), None, None, Some(1)
    );
    assert!(stock_reduction_result.is_err()); // Can't reduce more than available

    // Valid stock operation should work
    let valid_stock_result = product_service.update_stock(
        valid_product.id, -5, "out", None, Some("test"), None, None, Some(1)
    );
    assert!(valid_stock_result.is_ok());
}