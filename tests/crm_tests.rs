mod common;

use common::setup_test_db;
use clierp::modules::crm::{CustomerService, LeadService, DealService, CampaignService, ActivityService};
use clierp::database::connection::get_connection;
use clierp::database::models::{Customer, Lead, Deal, Campaign, CustomerType, LeadStatus, DealStage, LeadPriority, ActivityType, ActivityStatus};
use clierp::utils::pagination::PaginationParams;
use chrono::{NaiveDate, Utc};

#[test]
fn test_customer_creation_and_retrieval() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");
    let customer_service = CustomerService::new();

    // Test creating an individual customer
    let result = customer_service.create_customer(
        &mut conn,
        "John Doe",
        CustomerType::Individual,
        Some("john.doe@email.com"),
        Some("555-1234"),
        Some("123 Main St, City, State"),
        None,
        None,
        Some(10000), // $100.00 credit limit
        Some("VIP customer"),
    );

    assert!(result.is_ok());
    let customer = result.unwrap();
    assert_eq!(customer.name, "John Doe");
    assert_eq!(customer.email, Some("john.doe@email.com".to_string()));
    assert_eq!(customer.customer_type, "individual");
    assert_eq!(customer.credit_limit, 10000);

    // Test creating a business customer
    let business_result = customer_service.create_customer(
        &mut conn,
        "Acme Corporation",
        CustomerType::Business,
        Some("contact@acme.com"),
        Some("555-5678"),
        Some("456 Business Ave, City, State"),
        Some("Acme Corporation"),
        Some("12-3456789"),
        Some(50000), // $500.00 credit limit
        None,
    );

    assert!(business_result.is_ok());
    let business_customer = business_result.unwrap();
    assert_eq!(business_customer.name, "Acme Corporation");
    assert_eq!(business_customer.customer_type, "business");
    assert_eq!(business_customer.company_name, Some("Acme Corporation".to_string()));
    assert_eq!(business_customer.tax_id, Some("12-3456789".to_string()));

    // Test retrieving customer by ID
    let retrieved = customer_service.get_customer_by_id(&mut conn, customer.id);
    assert!(retrieved.is_ok());
    let retrieved_customer = retrieved.unwrap();
    assert!(retrieved_customer.is_some());
    assert_eq!(retrieved_customer.unwrap().name, "John Doe");

    // Test retrieving customer by code
    let by_code = customer_service.get_customer_by_code(&mut conn, &customer.customer_code);
    assert!(by_code.is_ok());
    let code_customer = by_code.unwrap();
    assert!(code_customer.is_some());
    assert_eq!(code_customer.unwrap().id, customer.id);
}

#[test]
fn test_customer_validation() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");
    let customer_service = CustomerService::new();

    // Test empty name validation
    let empty_name_result = customer_service.create_customer(
        &mut conn,
        "",
        CustomerType::Individual,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert!(empty_name_result.is_err());

    // Test invalid email validation
    let invalid_email_result = customer_service.create_customer(
        &mut conn,
        "Valid Customer",
        CustomerType::Individual,
        Some("invalid-email"),
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert!(invalid_email_result.is_err());

    // Test name too long
    let long_name = "A".repeat(201);
    let long_name_result = customer_service.create_customer(
        &mut conn,
        &long_name,
        CustomerType::Individual,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    );
    assert!(long_name_result.is_err());
}

#[test]
fn test_lead_creation_and_management() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");
    let customer_service = CustomerService::new();
    let lead_service = LeadService::new();

    // Create a customer first
    let customer = customer_service.create_customer(
        &mut conn,
        "Lead Test Customer",
        CustomerType::Business,
        Some("contact@leadtest.com"),
        None,
        None,
        Some("Lead Test Corp"),
        None,
        Some(25000),
        None,
    ).unwrap();

    // Create a lead
    let expected_close = NaiveDate::from_ymd_opt(2024, 12, 31).unwrap();
    let lead_result = lead_service.create_lead(
        &mut conn,
        "New Software Implementation",
        Some(customer.id),
        "Website Contact Form",
        150000, // $1,500.00
        Some(expected_close),
        LeadPriority::High,
        Some(1), // Assigned to test employee
        Some("Customer interested in ERP solution"),
        Some("Follow up in 2 days"),
    );

    assert!(lead_result.is_ok());
    let lead = lead_result.unwrap();
    assert_eq!(lead.title, "New Software Implementation");
    assert_eq!(lead.customer_id, Some(customer.id));
    assert_eq!(lead.lead_source, "Website Contact Form");
    assert_eq!(lead.estimated_value, 150000);
    assert_eq!(lead.status, "new");
    assert_eq!(lead.priority, "high");

    // Test lead status update
    let status_update_result = lead_service.update_lead_status(
        &mut conn,
        lead.id,
        LeadStatus::Contacted,
        Some("Initial contact made"),
        Some(1),
    );

    assert!(status_update_result.is_ok());
    let updated_lead = status_update_result.unwrap();
    assert_eq!(updated_lead.status, "contacted");

    // Test lead qualification
    let qualify_result = lead_service.qualify_lead(
        &mut conn,
        lead.id,
        75, // 75% probability
        Some("Customer has budget and authority"),
        Some(1),
    );

    assert!(qualify_result.is_ok());
    let qualified_lead = qualify_result.unwrap();
    assert_eq!(qualified_lead.status, "qualified");
    assert_eq!(qualified_lead.probability, 75);
}

#[test]
fn test_deal_progression_workflow() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");
    let customer_service = CustomerService::new();
    let lead_service = LeadService::new();
    let deal_service = DealService::new();

    // Create customer and lead
    let customer = customer_service.create_customer(
        &mut conn,
        "Deal Test Customer",
        CustomerType::Business,
        Some("deals@testcorp.com"),
        None,
        None,
        Some("Test Corporation"),
        None,
        Some(100000),
        None,
    ).unwrap();

    let lead = lead_service.create_lead(
        &mut conn,
        "Enterprise Software Deal",
        Some(customer.id),
        "Sales Team",
        200000,
        Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        LeadPriority::High,
        Some(1),
        Some("Enterprise software implementation"),
        None,
    ).unwrap();

    // Convert lead to deal
    let deal_result = deal_service.create_deal_from_lead(
        &mut conn,
        lead.id,
        "Enterprise ERP Implementation",
        250000, // Increased value after qualification
        Some(NaiveDate::from_ymd_opt(2024, 11, 30).unwrap()),
        Some("Proposal being prepared"),
    );

    assert!(deal_result.is_ok());
    let deal = deal_result.unwrap();
    assert_eq!(deal.lead_id, lead.id);
    assert_eq!(deal.deal_name, "Enterprise ERP Implementation");
    assert_eq!(deal.value, 250000);
    assert_eq!(deal.stage, "proposal");

    // Test deal stage progression
    let stage_update_result = deal_service.update_deal_stage(
        &mut conn,
        deal.id,
        DealStage::Negotiation,
        85, // Increased probability
        Some("Customer reviewed proposal, entering negotiations"),
        Some(1),
    );

    assert!(stage_update_result.is_ok());
    let updated_deal = stage_update_result.unwrap();
    assert_eq!(updated_deal.stage, "negotiation");
    assert_eq!(updated_deal.probability, 85);

    // Test closing deal as won
    let close_result = deal_service.close_deal(
        &mut conn,
        deal.id,
        DealStage::ClosedWon,
        Some(NaiveDate::from_ymd_opt(2024, 10, 15).unwrap()),
        Some("Contract signed"),
        Some(1),
    );

    assert!(close_result.is_ok());
    let closed_deal = close_result.unwrap();
    assert_eq!(closed_deal.stage, "closed_won");
    assert_eq!(closed_deal.probability, 100);
    assert!(closed_deal.close_date.is_some());
}

#[test]
fn test_campaign_management() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");
    let campaign_service = CampaignService::new();
    let lead_service = LeadService::new();

    // Create a campaign
    let start_date = NaiveDate::from_ymd_opt(2024, 1, 1).unwrap();
    let end_date = NaiveDate::from_ymd_opt(2024, 3, 31).unwrap();

    let campaign_result = campaign_service.create_campaign(
        &mut conn,
        "Q1 2024 Software Promotion",
        Some("Promote our ERP solution to small businesses"),
        Some(start_date),
        Some(end_date),
        50000, // $500.00 budget
        Some("Small and medium businesses"),
        vec!["email", "social_media", "webinar"], // channels
    );

    assert!(campaign_result.is_ok());
    let campaign = campaign_result.unwrap();
    assert_eq!(campaign.name, "Q1 2024 Software Promotion");
    assert_eq!(campaign.budget, 50000);
    assert_eq!(campaign.status, "active");

    // Create some leads for the campaign
    let lead1 = lead_service.create_lead(
        &mut conn,
        "Campaign Lead 1",
        None,
        "Campaign Email",
        25000,
        Some(NaiveDate::from_ymd_opt(2024, 2, 15).unwrap()),
        LeadPriority::Medium,
        Some(1),
        Some("Interested via email campaign"),
        None,
    ).unwrap();

    let lead2 = lead_service.create_lead(
        &mut conn,
        "Campaign Lead 2",
        None,
        "Campaign Webinar",
        35000,
        Some(NaiveDate::from_ymd_opt(2024, 2, 20).unwrap()),
        LeadPriority::High,
        Some(1),
        Some("Attended webinar"),
        None,
    ).unwrap();

    // Associate leads with campaign
    let associate_result1 = campaign_service.add_lead_to_campaign(
        &mut conn,
        campaign.id,
        lead1.id,
    );
    assert!(associate_result1.is_ok());

    let associate_result2 = campaign_service.add_lead_to_campaign(
        &mut conn,
        campaign.id,
        lead2.id,
    );
    assert!(associate_result2.is_ok());

    // Test campaign performance metrics
    let performance_result = campaign_service.get_campaign_performance(
        &mut conn,
        campaign.id,
    );

    assert!(performance_result.is_ok());
    let performance = performance_result.unwrap();
    assert_eq!(performance.total_leads, 2);
    assert_eq!(performance.total_estimated_value, 60000); // 25000 + 35000

    // Test updating campaign spend
    let spend_result = campaign_service.update_campaign_spend(
        &mut conn,
        campaign.id,
        15000, // $150.00 spent
        Some("Email campaign costs"),
    );

    assert!(spend_result.is_ok());
    let updated_campaign = spend_result.unwrap();
    assert_eq!(updated_campaign.spent, 15000);
}

#[test]
fn test_activity_tracking() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");
    let customer_service = CustomerService::new();
    let lead_service = LeadService::new();
    let activity_service = ActivityService::new();

    // Create customer and lead
    let customer = customer_service.create_customer(
        &mut conn,
        "Activity Test Customer",
        CustomerType::Individual,
        Some("activity@test.com"),
        Some("555-9999"),
        None,
        None,
        None,
        None,
        None,
    ).unwrap();

    let lead = lead_service.create_lead(
        &mut conn,
        "Activity Test Lead",
        Some(customer.id),
        "Phone Call",
        15000,
        Some(NaiveDate::from_ymd_opt(2024, 12, 15).unwrap()),
        LeadPriority::Medium,
        Some(1),
        None,
        None,
    ).unwrap();

    // Create a call activity
    let due_date = NaiveDate::from_ymd_opt(2024, 10, 20).unwrap();
    let call_activity_result = activity_service.create_activity(
        &mut conn,
        "Follow-up call",
        ActivityType::Call,
        Some(customer.id),
        Some(lead.id),
        None, // No deal yet
        Some(due_date),
        1, // Assigned to test employee
        Some("Discuss pricing options"),
        ActivityStatus::Scheduled,
    );

    assert!(call_activity_result.is_ok());
    let call_activity = call_activity_result.unwrap();
    assert_eq!(call_activity.title, "Follow-up call");
    assert_eq!(call_activity.activity_type, "call");
    assert_eq!(call_activity.customer_id, Some(customer.id));
    assert_eq!(call_activity.lead_id, Some(lead.id));
    assert_eq!(call_activity.status, "scheduled");

    // Complete the activity
    let complete_result = activity_service.complete_activity(
        &mut conn,
        call_activity.id,
        Some("Customer is interested, sending proposal"),
        Some(1),
    );

    assert!(complete_result.is_ok());
    let completed_activity = complete_result.unwrap();
    assert_eq!(completed_activity.status, "completed");
    assert!(completed_activity.completed_at.is_some());

    // Create an email activity
    let email_activity_result = activity_service.create_activity(
        &mut conn,
        "Send proposal email",
        ActivityType::Email,
        Some(customer.id),
        Some(lead.id),
        None,
        Some(NaiveDate::from_ymd_opt(2024, 10, 22).unwrap()),
        1,
        Some("Send detailed proposal with pricing"),
        ActivityStatus::InProgress,
    );

    assert!(email_activity_result.is_ok());

    // Test getting activities for customer
    let pagination = PaginationParams::new(1, 10);
    let customer_activities = activity_service.get_activities_for_customer(
        &mut conn,
        customer.id,
        &pagination,
    );

    assert!(customer_activities.is_ok());
    let activities = customer_activities.unwrap();
    assert_eq!(activities.data.len(), 2); // Call and email activities

    // Test getting overdue activities
    let overdue_activities = activity_service.get_overdue_activities(
        &mut conn,
        Some(1), // For specific employee
    );

    assert!(overdue_activities.is_ok());
    // Note: Whether activities are overdue depends on current date vs due date
}

#[test]
fn test_customer_statistics() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");
    let customer_service = CustomerService::new();
    let lead_service = LeadService::new();
    let deal_service = DealService::new();

    // Create customer
    let customer = customer_service.create_customer(
        &mut conn,
        "Stats Test Customer",
        CustomerType::Business,
        Some("stats@test.com"),
        None,
        None,
        Some("Stats Corp"),
        None,
        Some(75000),
        None,
    ).unwrap();

    // Create multiple leads for the customer
    let lead1 = lead_service.create_lead(
        &mut conn,
        "Lead 1",
        Some(customer.id),
        "Website",
        20000,
        None,
        LeadPriority::Low,
        Some(1),
        None,
        None,
    ).unwrap();

    let lead2 = lead_service.create_lead(
        &mut conn,
        "Lead 2",
        Some(customer.id),
        "Referral",
        30000,
        None,
        LeadPriority::High,
        Some(1),
        None,
        None,
    ).unwrap();

    // Create a deal from one lead
    let _deal = deal_service.create_deal_from_lead(
        &mut conn,
        lead1.id,
        "First Deal",
        25000,
        Some(NaiveDate::from_ymd_opt(2024, 11, 30).unwrap()),
        None,
    ).unwrap();

    // Get customer statistics
    let stats_result = customer_service.get_customer_with_stats(&mut conn, customer.id);
    assert!(stats_result.is_ok());
    let customer_stats = stats_result.unwrap();
    assert!(customer_stats.is_some());

    let stats = customer_stats.unwrap();
    assert_eq!(stats.customer.id, customer.id);
    assert_eq!(stats.total_leads, 2);
    assert_eq!(stats.active_deals, 1);
    assert!(stats.total_deal_value >= 25000);
}

#[test]
fn test_lead_conversion_metrics() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");
    let customer_service = CustomerService::new();
    let lead_service = LeadService::new();
    let deal_service = DealService::new();

    // Create customers and leads
    let customer1 = customer_service.create_customer(
        &mut conn,
        "Conversion Customer 1",
        CustomerType::Individual,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ).unwrap();

    let customer2 = customer_service.create_customer(
        &mut conn,
        "Conversion Customer 2",
        CustomerType::Business,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    ).unwrap();

    // Create leads with different outcomes
    let lead1 = lead_service.create_lead(
        &mut conn,
        "Successful Lead",
        Some(customer1.id),
        "Website",
        40000,
        None,
        LeadPriority::High,
        Some(1),
        None,
        None,
    ).unwrap();

    let _lead2 = lead_service.create_lead(
        &mut conn,
        "Lost Lead",
        Some(customer2.id),
        "Cold Call",
        20000,
        None,
        LeadPriority::Low,
        Some(1),
        None,
        None,
    ).unwrap();

    // Convert one lead to a winning deal
    let deal = deal_service.create_deal_from_lead(
        &mut conn,
        lead1.id,
        "Successful Deal",
        45000,
        Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        None,
    ).unwrap();

    // Close the deal as won
    let _closed_deal = deal_service.close_deal(
        &mut conn,
        deal.id,
        DealStage::ClosedWon,
        Some(NaiveDate::from_ymd_opt(2024, 10, 30).unwrap()),
        Some("Contract signed"),
        Some(1),
    ).unwrap();

    // Mark the other lead as lost
    let _lost_lead = lead_service.update_lead_status(
        &mut conn,
        _lead2.id,
        LeadStatus::ClosedLost,
        Some("Budget constraints"),
        Some(1),
    ).unwrap();

    // Test conversion metrics
    let metrics_result = lead_service.get_conversion_metrics(
        &mut conn,
        Some(NaiveDate::from_ymd_opt(2024, 1, 1).unwrap()),
        Some(NaiveDate::from_ymd_opt(2024, 12, 31).unwrap()),
        Some(1), // For specific employee
    );

    assert!(metrics_result.is_ok());
    let metrics = metrics_result.unwrap();
    assert!(metrics.total_leads >= 2);
    assert!(metrics.converted_leads >= 1);
    assert!(metrics.conversion_rate > 0.0);
    assert!(metrics.total_won_value >= 45000);
}

#[test]
fn test_search_and_filtering() {
    setup_test_db();

    let mut conn = get_connection().expect("Failed to get connection");
    let customer_service = CustomerService::new();
    let lead_service = LeadService::new();

    // Create diverse test data
    let customer1 = customer_service.create_customer(
        &mut conn,
        "Alpha Corporation",
        CustomerType::Business,
        Some("alpha@corp.com"),
        None,
        None,
        Some("Alpha Corp"),
        None,
        None,
        None,
    ).unwrap();

    let customer2 = customer_service.create_customer(
        &mut conn,
        "Beta Individual",
        CustomerType::Individual,
        Some("beta@personal.com"),
        None,
        None,
        None,
        None,
        None,
        None,
    ).unwrap();

    // Create leads with different statuses
    let _lead1 = lead_service.create_lead(
        &mut conn,
        "Alpha Lead",
        Some(customer1.id),
        "Website",
        50000,
        None,
        LeadPriority::High,
        Some(1),
        None,
        None,
    ).unwrap();

    let lead2 = lead_service.create_lead(
        &mut conn,
        "Beta Lead",
        Some(customer2.id),
        "Email",
        25000,
        None,
        LeadPriority::Medium,
        Some(1),
        None,
        None,
    ).unwrap();

    // Update one lead status
    let _contacted_lead = lead_service.update_lead_status(
        &mut conn,
        lead2.id,
        LeadStatus::Contacted,
        Some("Made initial contact"),
        Some(1),
    ).unwrap();

    // Test customer search
    let pagination = PaginationParams::new(1, 10);
    let search_result = customer_service.search_customers(
        &mut conn,
        &pagination,
        Some("Alpha"), // Search term
        None, // No type filter
        None, // No status filter
    );

    assert!(search_result.is_ok());
    let search_results = search_result.unwrap();
    assert_eq!(search_results.data.len(), 1);
    assert_eq!(search_results.data[0].name, "Alpha Corporation");

    // Test customer filter by type
    let type_filter_result = customer_service.search_customers(
        &mut conn,
        &pagination,
        None,
        Some(CustomerType::Individual),
        None,
    );

    assert!(type_filter_result.is_ok());
    let type_results = type_filter_result.unwrap();
    assert_eq!(type_results.data.len(), 1);
    assert_eq!(type_results.data[0].name, "Beta Individual");

    // Test lead search by status
    let lead_filter_result = lead_service.search_leads(
        &mut conn,
        &pagination,
        None, // No search term
        Some(LeadStatus::Contacted),
        None, // No priority filter
        None, // No assigned filter
    );

    assert!(lead_filter_result.is_ok());
    let lead_results = lead_filter_result.unwrap();
    assert_eq!(lead_results.data.len(), 1);
    assert_eq!(lead_results.data[0].title, "Beta Lead");
}