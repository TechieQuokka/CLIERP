mod common;

use common::setup_test_db;
use clierp::database::connection::get_connection;
use clierp::modules::hr::{EmployeeService, DepartmentService, PayrollService};
use clierp::modules::finance::{AccountService, TransactionService};
use clierp::modules::inventory::{ProductService, CategoryService, SupplierService, PurchaseOrderService};
use clierp::modules::crm::{CustomerService, LeadService, DealService, ActivityService};
use clierp::utils::pagination::PaginationParams;
use chrono::{NaiveDate, Utc};

#[test]
fn test_employee_purchase_workflow() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");

    // Services
    let dept_service = DepartmentService::new();
    let employee_service = EmployeeService::new();
    let category_service = CategoryService::new();
    let supplier_service = SupplierService::new();
    let product_service = ProductService::new();
    let po_service = PurchaseOrderService::new();

    // 1. Create department and employee
    let department = dept_service.create_department(
        &mut conn,
        "Purchasing Department",
        Some("Handles all procurement activities"),
        None,
    ).unwrap();

    let employee = employee_service.create_employee(
        &mut conn,
        "PURCH001",
        "John Purchaser",
        Some("purchaser@company.com"),
        Some("555-1234"),
        department.id,
        "Purchasing Manager",
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        60000, // $600.00 salary
    ).unwrap();

    // 2. Create product and supplier
    let category = category_service.create_category(
        "Office Supplies",
        Some("Office equipment and supplies"),
        None,
    ).unwrap();

    let supplier = supplier_service.create_supplier(
        "OFFICE001",
        "Office Supply Co.",
        Some("Sales Rep"),
        Some("sales@officesupply.com"),
        Some("555-5678"),
        Some("123 Supply St"),
        Some("Net 30"),
    ).unwrap();

    let product = product_service.create_product(
        "DESK001",
        "Office Desk",
        Some("Ergonomic office desk"),
        category.id,
        50000, // $500.00
        40000, // $400.00 cost
        0,     // No initial stock
        5,     // Min level
        Some(50),
        "EA",
        None,
    ).unwrap();

    // 3. Create purchase order (employee creating PO)
    let order_date = NaiveDate::from_ymd_opt(2024, 10, 1).unwrap();
    let expected_date = NaiveDate::from_ymd_opt(2024, 10, 15).unwrap();

    let po = po_service.create_purchase_order(
        supplier.id,
        order_date,
        Some(expected_date),
        Some("Requested by purchasing department"),
        vec![(product.id, 10, 40000)], // 10 desks at $400 each
    ).unwrap();

    // 4. Employee approves PO
    let approved_po = po_service.approve_purchase_order(po.id, Some(employee.id)).unwrap();
    assert_eq!(approved_po.status, "approved");

    // 5. Receive the goods (employee receives)
    let received_po = po_service.receive_purchase_order(
        po.id,
        vec![(product.id, 10)], // All 10 received
        Some("All items received in good condition"),
        Some(employee.id),
    ).unwrap();
    assert_eq!(received_po.status, "received");

    // 6. Verify stock was updated
    let updated_product = product_service.get_product_by_id(product.id).unwrap();
    assert_eq!(updated_product.current_stock, 10);

    // 7. Verify stock movement was recorded with employee reference
    let pagination = PaginationParams::new(1, 10);
    let movements = product_service.get_stock_movements(product.id, &pagination).unwrap();
    assert!(!movements.data.is_empty());

    // Should have movement with employee reference
    let receiving_movement = movements.data.iter().find(|m| m.movement_type == "in");
    assert!(receiving_movement.is_some());
    assert_eq!(receiving_movement.unwrap().moved_by, Some(employee.id));
}

#[test]
fn test_sales_to_finance_integration() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");

    // Services
    let customer_service = CustomerService::new();
    let lead_service = LeadService::new();
    let deal_service = DealService::new();
    let account_service = AccountService::new();
    let transaction_service = TransactionService::new();

    // 1. Create customer and lead
    let customer = customer_service.create_customer(
        &mut conn,
        "Big Corporation",
        crate::database::models::CustomerType::Business,
        Some("finance@bigcorp.com"),
        None,
        None,
        Some("Big Corporation Ltd."),
        Some("12-3456789"),
        Some(100000), // $1000 credit limit
        None,
    ).unwrap();

    let lead = lead_service.create_lead(
        &mut conn,
        "Software License Deal",
        Some(customer.id),
        "Direct Sales",
        75000, // $750.00
        Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        crate::database::models::LeadPriority::High,
        Some(1),
        Some("Annual software license"),
        None,
    ).unwrap();

    // 2. Convert to deal and close as won
    let deal = deal_service.create_deal_from_lead(
        &mut conn,
        lead.id,
        "Software License Agreement",
        80000, // Negotiated up to $800.00
        Some(NaiveDate::from_ymd_opt(2024, 11, 30).unwrap()),
        Some("Final proposal accepted"),
    ).unwrap();

    let closed_deal = deal_service.close_deal(
        &mut conn,
        deal.id,
        crate::database::models::DealStage::ClosedWon,
        Some(NaiveDate::from_ymd_opt(2024, 10, 25).unwrap()),
        Some("Contract signed, payment received"),
        Some(1),
    ).unwrap();

    // 3. Create revenue account if it doesn't exist
    let revenue_account = account_service.create_account(
        "4100",
        "Software Revenue",
        crate::database::models::AccountType::Revenue,
        None,
    ).unwrap();

    let receivables_account = account_service.create_account(
        "1200",
        "Accounts Receivable",
        crate::database::models::AccountType::Asset,
        None,
    ).unwrap();

    // 4. Record the sale in accounting
    let transaction_date = NaiveDate::from_ymd_opt(2024, 10, 25).unwrap();

    // Debit Accounts Receivable
    let receivable_transaction = transaction_service.create_transaction(
        receivables_account.id,
        transaction_date,
        80000, // $800.00
        "debit",
        &format!("Sale to {} - Deal #{}", customer.name, closed_deal.id),
        Some(&format!("DEAL-{}", closed_deal.id)),
        Some(1), // Created by employee
    ).unwrap();

    // Credit Revenue
    let revenue_transaction = transaction_service.create_transaction(
        revenue_account.id,
        transaction_date,
        80000, // $800.00
        "credit",
        &format!("Revenue from {} - Deal #{}", customer.name, closed_deal.id),
        Some(&format!("DEAL-{}", closed_deal.id)),
        Some(1),
    ).unwrap();

    // 5. Verify the accounting entries
    assert_eq!(receivable_transaction.amount, 80000);
    assert_eq!(receivable_transaction.debit_credit, "debit");
    assert_eq!(revenue_transaction.amount, 80000);
    assert_eq!(revenue_transaction.debit_credit, "credit");

    // 6. Verify account balances
    let updated_receivables = account_service.get_account_by_id(receivables_account.id).unwrap();
    let updated_revenue = account_service.get_account_by_id(revenue_account.id).unwrap();

    // Asset accounts increase with debits
    assert_eq!(updated_receivables.balance, 80000);
    // Revenue accounts increase with credits (stored as positive)
    assert_eq!(updated_revenue.balance, 80000);

    // 7. Verify we can trace back from transaction to deal
    assert!(receivable_transaction.reference.is_some());
    assert!(receivable_transaction.reference.as_ref().unwrap().contains(&closed_deal.id.to_string()));
}

#[test]
fn test_inventory_to_sales_integration() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");

    // Services
    let category_service = CategoryService::new();
    let product_service = ProductService::new();
    let customer_service = CustomerService::new();
    let lead_service = LeadService::new();
    let deal_service = DealService::new();

    // 1. Create product with stock
    let category = category_service.create_category(
        "Software Products",
        Some("Software licenses and products"),
        None,
    ).unwrap();

    let product = product_service.create_product(
        "SW001",
        "ERP Software License",
        Some("Annual ERP software license"),
        category.id,
        120000, // $1200.00 per license
        80000,  // $800.00 cost
        50,     // 50 licenses in stock
        10,     // Min 10 licenses
        Some(100),
        "LIC",
        None,
    ).unwrap();

    // 2. Create customer and sales process
    let customer = customer_service.create_customer(
        &mut conn,
        "Tech Startup Inc.",
        crate::database::models::CustomerType::Business,
        Some("admin@techstartup.com"),
        None,
        None,
        Some("Tech Startup Inc."),
        None,
        Some(50000),
        None,
    ).unwrap();

    let lead = lead_service.create_lead(
        &mut conn,
        "ERP Implementation Project",
        Some(customer.id),
        "Website Inquiry",
        600000, // 5 licenses @ $1200 each = $6000
        Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        crate::database::models::LeadPriority::High,
        Some(1),
        Some("Startup needs ERP solution"),
        None,
    ).unwrap();

    let deal = deal_service.create_deal_from_lead(
        &mut conn,
        lead.id,
        "ERP License Sale",
        600000,
        Some(NaiveDate::from_ymd_opt(2024, 11, 15).unwrap()),
        None,
    ).unwrap();

    // 3. Close deal and fulfill from inventory
    let _closed_deal = deal_service.close_deal(
        &mut conn,
        deal.id,
        crate::database::models::DealStage::ClosedWon,
        Some(NaiveDate::from_ymd_opt(2024, 10, 20).unwrap()),
        Some("Contract executed, fulfilling licenses"),
        Some(1),
    ).unwrap();

    // 4. Fulfill the order by reducing inventory
    let fulfilled_product = product_service.update_stock(
        product.id,
        -5, // Reduce by 5 licenses
        "out",
        None,
        Some("sale"),
        Some(deal.id),
        Some(&format!("Fulfillment for deal #{} - {}", deal.id, customer.name)),
        Some(1),
    ).unwrap();

    // 5. Verify inventory was reduced
    assert_eq!(fulfilled_product.current_stock, 45); // 50 - 5 = 45

    // 6. Verify stock movement was recorded with deal reference
    let pagination = PaginationParams::new(1, 10);
    let movements = product_service.get_stock_movements(product.id, &pagination).unwrap();

    let sale_movement = movements.data.iter().find(|m| {
        m.movement_type == "out" &&
        m.reference_type.as_ref().map_or(false, |rt| rt == "sale")
    });

    assert!(sale_movement.is_some());
    let movement = sale_movement.unwrap();
    assert_eq!(movement.quantity, -5);
    assert_eq!(movement.reference_id, Some(deal.id));

    // 7. Check if product is approaching low stock threshold
    let low_stock_products = product_service.get_low_stock_products().unwrap();

    // Product should not be in low stock yet (45 > 10)
    let is_low_stock = low_stock_products.iter().any(|p| p.product.id == product.id);
    assert!(!is_low_stock);

    // 8. Simulate more sales that trigger low stock
    let _low_stock_product = product_service.update_stock(
        product.id,
        -37, // Reduce by 37 more (45 - 37 = 8, which is below min of 10)
        "out",
        None,
        Some("sale"),
        None,
        Some("Additional sales - bulk order"),
        Some(1),
    ).unwrap();

    // Now check low stock again
    let low_stock_products_after = product_service.get_low_stock_products().unwrap();
    let is_now_low_stock = low_stock_products_after.iter().any(|p| p.product.id == product.id);
    assert!(is_now_low_stock);
}

#[test]
fn test_employee_payroll_to_finance_integration() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");

    // Services
    let dept_service = DepartmentService::new();
    let employee_service = EmployeeService::new();
    let payroll_service = PayrollService::new();
    let account_service = AccountService::new();
    let transaction_service = TransactionService::new();

    // 1. Create department and employees
    let department = dept_service.create_department(
        &mut conn,
        "Sales Department",
        Some("Sales and marketing team"),
        None,
    ).unwrap();

    let employee1 = employee_service.create_employee(
        &mut conn,
        "SALES001",
        "Alice Salesperson",
        Some("alice@company.com"),
        None,
        department.id,
        "Sales Representative",
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        50000, // $500.00/month
    ).unwrap();

    let employee2 = employee_service.create_employee(
        &mut conn,
        "SALES002",
        "Bob Manager",
        Some("bob@company.com"),
        None,
        department.id,
        "Sales Manager",
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        75000, // $750.00/month
    ).unwrap();

    // 2. Create expense accounts for payroll
    let salary_expense_account = account_service.create_account(
        "5100",
        "Salary Expense",
        crate::database::models::AccountType::Expense,
        None,
    ).unwrap();

    let payroll_liability_account = account_service.create_account(
        "2100",
        "Salaries Payable",
        crate::database::models::AccountType::Liability,
        None,
    ).unwrap();

    // 3. Process payroll for the month
    let payroll_period = "2024-10";

    let payroll1 = payroll_service.calculate_payroll(
        &mut conn,
        employee1.id,
        payroll_period,
        None, // Use base salary
        0,    // No overtime
        0,    // No bonuses
        5000, // $50.00 deductions (taxes, etc.)
    ).unwrap();

    let payroll2 = payroll_service.calculate_payroll(
        &mut conn,
        employee2.id,
        payroll_period,
        None,
        0,
        10000, // $100.00 bonus for manager
        7500,  // $75.00 deductions
    ).unwrap();

    // 4. Create accounting entries for payroll
    let total_gross_pay = payroll1.base_salary + payroll1.bonuses + payroll2.base_salary + payroll2.bonuses;
    let total_net_pay = payroll1.net_salary + payroll2.net_salary;
    let total_deductions = payroll1.deductions + payroll2.deductions;

    let payroll_date = NaiveDate::from_ymd_opt(2024, 10, 31).unwrap();

    // Debit Salary Expense for gross pay
    let expense_transaction = transaction_service.create_transaction(
        salary_expense_account.id,
        payroll_date,
        total_gross_pay,
        "debit",
        &format!("Payroll expense for {}", payroll_period),
        Some(&format!("PAYROLL-{}", payroll_period)),
        Some(1),
    ).unwrap();

    // Credit Salaries Payable for net pay (what we owe employees)
    let liability_transaction = transaction_service.create_transaction(
        payroll_liability_account.id,
        payroll_date,
        total_net_pay,
        "credit",
        &format!("Salaries payable for {}", payroll_period),
        Some(&format!("PAYROLL-{}", payroll_period)),
        Some(1),
    ).unwrap();

    // 5. Verify accounting entries
    assert_eq!(expense_transaction.amount, total_gross_pay);
    assert_eq!(liability_transaction.amount, total_net_pay);

    // 6. Verify account balances
    let updated_expense_account = account_service.get_account_by_id(salary_expense_account.id).unwrap();
    let updated_liability_account = account_service.get_account_by_id(payroll_liability_account.id).unwrap();

    // Expense accounts increase with debits
    assert_eq!(updated_expense_account.balance, total_gross_pay);
    // Liability accounts increase with credits
    assert_eq!(updated_liability_account.balance, total_net_pay);

    // 7. Verify payroll records exist and are linked
    assert_eq!(payroll1.period, payroll_period);
    assert_eq!(payroll2.period, payroll_period);
    assert_eq!(payroll1.employee_id, employee1.id);
    assert_eq!(payroll2.employee_id, employee2.id);

    // 8. Calculate total expected vs actual
    let expected_total_gross = 50000 + 75000 + 10000; // Alice + Bob + Bob's bonus
    let expected_total_deductions = 5000 + 7500;       // Alice + Bob deductions
    let expected_total_net = expected_total_gross - expected_total_deductions;

    assert_eq!(total_gross_pay, expected_total_gross);
    assert_eq!(total_deductions, expected_total_deductions);
    assert_eq!(total_net_pay, expected_total_net);
}

#[test]
fn test_complete_business_workflow() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");

    // Initialize all services
    let dept_service = DepartmentService::new();
    let employee_service = EmployeeService::new();
    let customer_service = CustomerService::new();
    let lead_service = LeadService::new();
    let deal_service = DealService::new();
    let category_service = CategoryService::new();
    let product_service = ProductService::new();
    let supplier_service = SupplierService::new();
    let po_service = PurchaseOrderService::new();
    let account_service = AccountService::new();
    let transaction_service = TransactionService::new();

    // 1. Setup company structure
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

    let sales_rep = employee_service.create_employee(
        &mut conn,
        "REP001",
        "Sarah Sales",
        Some("sarah@company.com"),
        None,
        sales_dept.id,
        "Senior Sales Representative",
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        60000,
    ).unwrap();

    let procurement_manager = employee_service.create_employee(
        &mut conn,
        "PROC001",
        "Paul Procurement",
        Some("paul@company.com"),
        None,
        procurement_dept.id,
        "Procurement Manager",
        NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        70000,
    ).unwrap();

    // 2. Setup products and suppliers
    let tech_category = category_service.create_category(
        "Technology Hardware",
        Some("Computer hardware and tech equipment"),
        None,
    ).unwrap();

    let supplier = supplier_service.create_supplier(
        "TECH001",
        "TechSupplier Corp",
        Some("Tech Sales"),
        Some("sales@techsupplier.com"),
        None,
        None,
        Some("Net 30"),
    ).unwrap();

    let product = product_service.create_product(
        "LAPTOP001",
        "Business Laptop",
        Some("High-performance business laptop"),
        tech_category.id,
        150000, // $1500 selling price
        120000, // $1200 cost price
        0,      // No initial stock
        5,      // Min stock
        Some(50),
        "EA",
        None,
    ).unwrap();

    // 3. Procurement workflow - order inventory
    let po = po_service.create_purchase_order(
        supplier.id,
        NaiveDate::from_ymd_opt(2024, 10, 1).unwrap(),
        Some(NaiveDate::from_ymd_opt(2024, 10, 15).unwrap()),
        Some("Initial stock order"),
        vec![(product.id, 20, 120000)], // 20 laptops at $1200 each
    ).unwrap();

    let approved_po = po_service.approve_purchase_order(po.id, Some(procurement_manager.id)).unwrap();
    let received_po = po_service.receive_purchase_order(
        po.id,
        vec![(product.id, 20)],
        Some("All items received"),
        Some(procurement_manager.id),
    ).unwrap();

    // 4. Sales workflow - customer inquiry to closed deal
    let customer = customer_service.create_customer(
        &mut conn,
        "Enterprise Client Ltd.",
        crate::database::models::CustomerType::Business,
        Some("purchasing@enterprise.com"),
        None,
        None,
        Some("Enterprise Client Ltd."),
        None,
        Some(200000), // $2000 credit limit
        None,
    ).unwrap();

    let lead = lead_service.create_lead(
        &mut conn,
        "Laptop Procurement for Office",
        Some(customer.id),
        "Email Inquiry",
        750000, // 5 laptops @ $1500 = $7500
        Some(NaiveDate::from_ymd_opt(2024, 11, 30).unwrap()),
        crate::database::models::LeadPriority::High,
        Some(sales_rep.id),
        Some("Customer needs 5 laptops for new office"),
        None,
    ).unwrap();

    let deal = deal_service.create_deal_from_lead(
        &mut conn,
        lead.id,
        "Enterprise Laptop Order",
        750000,
        Some(NaiveDate::from_ymd_opt(2024, 11, 15).unwrap()),
        None,
    ).unwrap();

    let closed_deal = deal_service.close_deal(
        &mut conn,
        deal.id,
        crate::database::models::DealStage::ClosedWon,
        Some(NaiveDate::from_ymd_opt(2024, 10, 30).unwrap()),
        Some("Purchase order received"),
        Some(sales_rep.id),
    ).unwrap();

    // 5. Fulfill order from inventory
    let fulfilled_product = product_service.update_stock(
        product.id,
        -5, // Ship 5 laptops
        "out",
        None,
        Some("sale"),
        Some(deal.id),
        Some(&format!("Order fulfillment for {}", customer.name)),
        Some(sales_rep.id),
    ).unwrap();

    // 6. Create accounting entries
    let revenue_account = account_service.create_account(
        "4000",
        "Product Sales Revenue",
        crate::database::models::AccountType::Revenue,
        None,
    ).unwrap();

    let cogs_account = account_service.create_account(
        "5000",
        "Cost of Goods Sold",
        crate::database::models::AccountType::Expense,
        None,
    ).unwrap();

    let inventory_account = account_service.create_account(
        "1300",
        "Inventory Asset",
        crate::database::models::AccountType::Asset,
        None,
    ).unwrap();

    let receivables_account = account_service.create_account(
        "1200",
        "Accounts Receivable",
        crate::database::models::AccountType::Asset,
        None,
    ).unwrap();

    let sale_date = NaiveDate::from_ymd_opt(2024, 10, 30).unwrap();

    // Record the sale
    let _receivable_entry = transaction_service.create_transaction(
        receivables_account.id,
        sale_date,
        750000, // Sale amount
        "debit",
        &format!("Sale to {} - Deal #{}", customer.name, deal.id),
        Some(&format!("SALE-{}", deal.id)),
        Some(sales_rep.id),
    ).unwrap();

    let _revenue_entry = transaction_service.create_transaction(
        revenue_account.id,
        sale_date,
        750000,
        "credit",
        &format!("Revenue from {} - Deal #{}", customer.name, deal.id),
        Some(&format!("SALE-{}", deal.id)),
        Some(sales_rep.id),
    ).unwrap();

    // Record cost of goods sold
    let cogs_amount = 5 * 120000; // 5 laptops at $1200 cost each
    let _cogs_entry = transaction_service.create_transaction(
        cogs_account.id,
        sale_date,
        cogs_amount,
        "debit",
        &format!("COGS for sale to {} - Deal #{}", customer.name, deal.id),
        Some(&format!("COGS-{}", deal.id)),
        Some(sales_rep.id),
    ).unwrap();

    let _inventory_reduction = transaction_service.create_transaction(
        inventory_account.id,
        sale_date,
        cogs_amount,
        "credit",
        &format!("Inventory reduction for sale - Deal #{}", deal.id),
        Some(&format!("INV-{}", deal.id)),
        Some(sales_rep.id),
    ).unwrap();

    // 7. Verify all integrations worked correctly

    // Check inventory levels
    assert_eq!(fulfilled_product.current_stock, 15); // Started with 20, sold 5

    // Check deal was closed
    assert_eq!(closed_deal.stage, "closed_won");
    assert_eq!(closed_deal.value, 750000);

    // Check stock movement
    let pagination = PaginationParams::new(1, 10);
    let movements = product_service.get_stock_movements(product.id, &pagination).unwrap();
    let sale_movement = movements.data.iter().find(|m|
        m.movement_type == "out" &&
        m.reference_id == Some(deal.id)
    );
    assert!(sale_movement.is_some());

    // Check accounting balances
    let updated_revenue = account_service.get_account_by_id(revenue_account.id).unwrap();
    let updated_cogs = account_service.get_account_by_id(cogs_account.id).unwrap();
    let updated_receivables = account_service.get_account_by_id(receivables_account.id).unwrap();

    assert_eq!(updated_revenue.balance, 750000);
    assert_eq!(updated_cogs.balance, 600000); // 5 * 120000
    assert_eq!(updated_receivables.balance, 750000);

    // Check profit margin
    let gross_profit = 750000 - 600000; // Revenue - COGS
    assert_eq!(gross_profit, 150000); // $1500 profit on $7500 sale = 20% margin

    // 8. Verify employee actions are tracked
    assert_eq!(approved_po.approved_by, Some(procurement_manager.id));
    assert_eq!(closed_deal.assigned_to, Some(sales_rep.id));

    let receiving_movement = movements.data.iter().find(|m| m.movement_type == "in");
    assert_eq!(receiving_movement.unwrap().moved_by, Some(procurement_manager.id));

    let shipping_movement = movements.data.iter().find(|m| m.movement_type == "out");
    assert_eq!(shipping_movement.unwrap().moved_by, Some(sales_rep.id));
}

#[test]
fn test_cross_module_data_consistency() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");

    // This test verifies that data remains consistent across module boundaries

    // Services
    let customer_service = CustomerService::new();
    let lead_service = LeadService::new();
    let deal_service = DealService::new();
    let product_service = ProductService::new();
    let category_service = CategoryService::new();

    // 1. Create interconnected data
    let customer = customer_service.create_customer(
        &mut conn,
        "Consistency Test Corp",
        crate::database::models::CustomerType::Business,
        Some("test@consistency.com"),
        None,
        None,
        None,
        None,
        None,
        None,
    ).unwrap();

    let category = category_service.create_category(
        "Test Products",
        None,
        None,
    ).unwrap();

    let product = product_service.create_product(
        "CONS001",
        "Consistency Product",
        None,
        category.id,
        10000,
        8000,
        100,
        10,
        Some(200),
        "EA",
        None,
    ).unwrap();

    let lead = lead_service.create_lead(
        &mut conn,
        "Consistency Test Lead",
        Some(customer.id),
        "Test",
        50000,
        None,
        crate::database::models::LeadPriority::Medium,
        Some(1),
        None,
        None,
    ).unwrap();

    let deal = deal_service.create_deal_from_lead(
        &mut conn,
        lead.id,
        "Consistency Test Deal",
        60000,
        None,
        None,
    ).unwrap();

    // 2. Test cascading updates

    // Update customer information
    let updated_customer = customer_service.update_customer(
        &mut conn,
        customer.id,
        Some("Updated Consistency Corp"),
        None, // Don't change type
        None, // Don't change email
        None, // Don't change phone
        None, // Don't change address
        None, // Don't change company name
        None, // Don't change tax ID
        Some(50000), // Update credit limit
        None, // Don't change notes
        None, // Don't change status
    ).unwrap();

    assert_eq!(updated_customer.name, "Updated Consistency Corp");
    assert_eq!(updated_customer.credit_limit, 50000);

    // Verify lead still references correct customer
    let retrieved_lead = lead_service.get_lead_by_id(&mut conn, lead.id).unwrap().unwrap();
    assert_eq!(retrieved_lead.customer_id, Some(customer.id));

    // Verify deal still references correct lead
    let retrieved_deal = deal_service.get_deal_by_id(&mut conn, deal.id).unwrap().unwrap();
    assert_eq!(retrieved_deal.lead_id, lead.id);

    // 3. Test stock consistency across multiple operations
    let initial_stock = product.current_stock;

    // Perform multiple stock operations
    let _updated1 = product_service.update_stock(
        product.id,
        10,  // Add 10
        "in",
        Some(8000),
        Some("purchase"),
        None,
        Some("Test restock"),
        Some(1),
    ).unwrap();

    let _updated2 = product_service.update_stock(
        product.id,
        -5,  // Remove 5
        "out",
        None,
        Some("sale"),
        Some(deal.id),
        Some("Test sale"),
        Some(1),
    ).unwrap();

    let final_product = product_service.get_product_by_id(product.id).unwrap();
    assert_eq!(final_product.current_stock, initial_stock + 10 - 5);

    // 4. Verify stock movement history is complete and consistent
    let pagination = PaginationParams::new(1, 20);
    let movements = product_service.get_stock_movements(product.id, &pagination).unwrap();

    // Should have initial stock movement + our 2 movements = 3 total
    assert_eq!(movements.data.len(), 3);

    // Calculate total from movements
    let total_movement: i32 = movements.data.iter().map(|m| m.quantity).sum();
    let expected_total = initial_stock + 10 - 5; // Initial + in - out
    assert_eq!(final_product.current_stock, expected_total);

    // 5. Test referential integrity

    // Movement should reference our deal
    let sale_movement = movements.data.iter().find(|m|
        m.movement_type == "out" &&
        m.reference_type.as_ref().map_or(false, |rt| rt == "sale")
    ).unwrap();
    assert_eq!(sale_movement.reference_id, Some(deal.id));

    // Deal should still exist and reference correct lead
    let final_deal = deal_service.get_deal_by_id(&mut conn, deal.id).unwrap().unwrap();
    assert_eq!(final_deal.lead_id, lead.id);

    // Lead should still reference correct customer
    let final_lead = lead_service.get_lead_by_id(&mut conn, lead.id).unwrap().unwrap();
    assert_eq!(final_lead.customer_id, Some(customer.id));
}