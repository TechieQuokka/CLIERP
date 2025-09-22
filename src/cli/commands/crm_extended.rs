use clap::{Args, Subcommand};
use chrono::{NaiveDate, NaiveDateTime};
use crate::core::result::CLIERPResult;
use crate::database::{
    DatabaseConnection, CustomerType, CustomerStatus, LeadStatus, LeadPriority,
    DealStage, CampaignType, CampaignStatus, ActivityType
};
use crate::modules::crm::{
    CustomerService, LeadService, DealService, CampaignService, ActivityService
};
use crate::utils::pagination::PaginationParams;
use crate::utils::filters::FilterOptions;

#[derive(Debug, Args)]
pub struct CrmExtendedCommands {
    #[command(subcommand)]
    pub action: CrmExtendedAction,
}

#[derive(Debug, Subcommand)]
pub enum CrmExtendedAction {
    Customer {
        #[command(subcommand)]
        action: CustomerAction,
    },
    Lead {
        #[command(subcommand)]
        action: LeadAction,
    },
    Deal {
        #[command(subcommand)]
        action: DealAction,
    },
    Campaign {
        #[command(subcommand)]
        action: CampaignAction,
    },
    Activity {
        #[command(subcommand)]
        action: ActivityAction,
    },
    Dashboard,
    Pipeline,
    Performance,
}

#[derive(Debug, Subcommand)]
pub enum CustomerAction {
    Create {
        #[arg(long)]
        name: String,
        #[arg(long, value_enum)]
        customer_type: CustomerType,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        phone: Option<String>,
        #[arg(long)]
        address: Option<String>,
        #[arg(long)]
        company_name: Option<String>,
        #[arg(long)]
        tax_id: Option<String>,
        #[arg(long)]
        credit_limit: Option<i32>,
        #[arg(long)]
        notes: Option<String>,
    },
    List {
        #[arg(long, default_value = "1")]
        page: i64,
        #[arg(long, default_value = "20")]
        per_page: i64,
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        customer_type: Option<String>,
        #[arg(long)]
        sort_by: Option<String>,
        #[arg(long)]
        sort_desc: bool,
    },
    Show {
        #[arg(long)]
        id: Option<i32>,
        #[arg(long)]
        code: Option<String>,
    },
    Update {
        id: i32,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        email: Option<String>,
        #[arg(long)]
        phone: Option<String>,
        #[arg(long)]
        address: Option<String>,
        #[arg(long)]
        company_name: Option<String>,
        #[arg(long)]
        tax_id: Option<String>,
        #[arg(long)]
        credit_limit: Option<i32>,
        #[arg(long)]
        status: Option<CustomerStatus>,
        #[arg(long)]
        notes: Option<String>,
    },
    Delete {
        id: i32,
    },
    Search {
        query: String,
    },
    Stats,
}

#[derive(Debug, Subcommand)]
pub enum LeadAction {
    Create {
        #[arg(long)]
        title: String,
        #[arg(long)]
        customer_id: Option<i32>,
        #[arg(long)]
        lead_source: String,
        #[arg(long)]
        estimated_value: i32,
        #[arg(long)]
        expected_close_date: Option<NaiveDate>,
        #[arg(long, value_enum)]
        priority: LeadPriority,
        #[arg(long)]
        assigned_to: Option<i32>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        notes: Option<String>,
    },
    List {
        #[arg(long, default_value = "1")]
        page: i64,
        #[arg(long, default_value = "20")]
        per_page: i64,
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        assigned_to: Option<i32>,
        #[arg(long)]
        date_from: Option<NaiveDate>,
        #[arg(long)]
        date_to: Option<NaiveDate>,
        #[arg(long)]
        sort_by: Option<String>,
        #[arg(long)]
        sort_desc: bool,
    },
    Show {
        id: i32,
    },
    UpdateStatus {
        id: i32,
        #[arg(value_enum)]
        status: LeadStatus,
        #[arg(long)]
        notes: Option<String>,
    },
    Update {
        id: i32,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        customer_id: Option<i32>,
        #[arg(long)]
        lead_source: Option<String>,
        #[arg(long)]
        estimated_value: Option<i32>,
        #[arg(long)]
        expected_close_date: Option<NaiveDate>,
        #[arg(long)]
        priority: Option<LeadPriority>,
        #[arg(long)]
        assigned_to: Option<i32>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        notes: Option<String>,
    },
    Assign {
        id: i32,
        assigned_to: i32,
    },
    Delete {
        id: i32,
    },
    ByStatus {
        #[arg(value_enum)]
        status: LeadStatus,
    },
    Stats,
}

#[derive(Debug, Subcommand)]
pub enum DealAction {
    Create {
        #[arg(long)]
        lead_id: i32,
        #[arg(long)]
        title: String,
        #[arg(long)]
        deal_value: i32,
        #[arg(long)]
        expected_close_date: Option<NaiveDate>,
        #[arg(long)]
        assigned_to: Option<i32>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        notes: Option<String>,
    },
    List {
        #[arg(long, default_value = "1")]
        page: i64,
        #[arg(long, default_value = "20")]
        per_page: i64,
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        stage: Option<String>,
        #[arg(long)]
        assigned_to: Option<i32>,
        #[arg(long)]
        date_from: Option<NaiveDate>,
        #[arg(long)]
        date_to: Option<NaiveDate>,
        #[arg(long)]
        sort_by: Option<String>,
        #[arg(long)]
        sort_desc: bool,
    },
    Show {
        id: i32,
    },
    UpdateStage {
        id: i32,
        #[arg(value_enum)]
        stage: DealStage,
        #[arg(long)]
        notes: Option<String>,
    },
    Update {
        id: i32,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        deal_value: Option<i32>,
        #[arg(long)]
        expected_close_date: Option<NaiveDate>,
        #[arg(long)]
        assigned_to: Option<i32>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        notes: Option<String>,
    },
    Delete {
        id: i32,
    },
    ByStage {
        #[arg(value_enum)]
        stage: DealStage,
    },
    Stats,
}

#[derive(Debug, Subcommand)]
pub enum CampaignAction {
    Create {
        #[arg(long)]
        name: String,
        #[arg(long, value_enum)]
        campaign_type: CampaignType,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        start_date: NaiveDate,
        #[arg(long)]
        end_date: Option<NaiveDate>,
        #[arg(long)]
        budget: Option<i32>,
        #[arg(long)]
        target_audience: Option<String>,
        #[arg(long)]
        goals: Option<String>,
    },
    List {
        #[arg(long, default_value = "1")]
        page: i64,
        #[arg(long, default_value = "20")]
        per_page: i64,
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        campaign_type: Option<String>,
        #[arg(long)]
        date_from: Option<NaiveDate>,
        #[arg(long)]
        date_to: Option<NaiveDate>,
        #[arg(long)]
        sort_by: Option<String>,
        #[arg(long)]
        sort_desc: bool,
    },
    Show {
        #[arg(long)]
        id: Option<i32>,
        #[arg(long)]
        code: Option<String>,
    },
    Update {
        id: i32,
        #[arg(long)]
        name: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        start_date: Option<NaiveDate>,
        #[arg(long)]
        end_date: Option<NaiveDate>,
        #[arg(long)]
        budget: Option<i32>,
        #[arg(long)]
        actual_cost: Option<i32>,
        #[arg(long)]
        target_audience: Option<String>,
        #[arg(long)]
        goals: Option<String>,
    },
    UpdateStatus {
        id: i32,
        #[arg(value_enum)]
        status: CampaignStatus,
    },
    Delete {
        id: i32,
    },
    ByStatus {
        #[arg(value_enum)]
        status: CampaignStatus,
    },
    Active,
    Performance,
    Stats,
}

#[derive(Debug, Subcommand)]
pub enum ActivityAction {
    Create {
        #[arg(long, value_enum)]
        activity_type: ActivityType,
        #[arg(long)]
        title: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        customer_id: Option<i32>,
        #[arg(long)]
        lead_id: Option<i32>,
        #[arg(long)]
        assigned_to: i32,
        #[arg(long)]
        due_date: Option<NaiveDateTime>,
        #[arg(long)]
        priority: Option<String>,
    },
    List {
        #[arg(long, default_value = "1")]
        page: i64,
        #[arg(long, default_value = "20")]
        per_page: i64,
        #[arg(long)]
        search: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        activity_type: Option<String>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        assigned_to: Option<i32>,
        #[arg(long)]
        date_from: Option<NaiveDate>,
        #[arg(long)]
        date_to: Option<NaiveDate>,
        #[arg(long)]
        sort_by: Option<String>,
        #[arg(long)]
        sort_desc: bool,
    },
    Show {
        id: i32,
    },
    Update {
        id: i32,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        due_date: Option<NaiveDateTime>,
        #[arg(long)]
        priority: Option<String>,
        #[arg(long)]
        assigned_to: Option<i32>,
    },
    Complete {
        id: i32,
        #[arg(long)]
        outcome: Option<String>,
    },
    Reopen {
        id: i32,
    },
    Delete {
        id: i32,
    },
    ByCustomer {
        customer_id: i32,
    },
    ByLead {
        lead_id: i32,
    },
    ByEmployee {
        employee_id: i32,
    },
    Overdue,
    Stats,
}

pub fn execute_crm_extended_command(
    conn: &mut DatabaseConnection,
    cmd: CrmExtendedCommands,
) -> CLIERPResult<()> {
    match cmd.action {
        CrmExtendedAction::Customer { action } => execute_customer_command(conn, action),
        CrmExtendedAction::Lead { action } => execute_lead_command(conn, action),
        CrmExtendedAction::Deal { action } => execute_deal_command(conn, action),
        CrmExtendedAction::Campaign { action } => execute_campaign_command(conn, action),
        CrmExtendedAction::Activity { action } => execute_activity_command(conn, action),
        CrmExtendedAction::Dashboard => execute_dashboard_command(conn),
        CrmExtendedAction::Pipeline => execute_pipeline_command(conn),
        CrmExtendedAction::Performance => execute_performance_command(conn),
    }
}

fn execute_customer_command(conn: &mut DatabaseConnection, action: CustomerAction) -> CLIERPResult<()> {
    match action {
        CustomerAction::Create {
            name,
            customer_type,
            email,
            phone,
            address,
            company_name,
            tax_id,
            credit_limit,
            notes,
        } => {
            let customer = CustomerService::create_customer(
                conn,
                &name,
                customer_type,
                email.as_deref(),
                phone.as_deref(),
                address.as_deref(),
                company_name.as_deref(),
                tax_id.as_deref(),
                credit_limit,
                notes.as_deref(),
            )?;
            println!("Customer created successfully:");
            println!("ID: {}, Code: {}, Name: {}", customer.id, customer.customer_code, customer.name);
        }
        CustomerAction::List {
            page,
            per_page,
            search,
            status,
            customer_type,
            sort_by,
            sort_desc,
        } => {
            let pagination = PaginationParams::new(page as usize, per_page);
            let filters = FilterOptions {
                search,
                status,
                filter_type: customer_type,
                sort_by,
                sort_desc,
                ..Default::default()
            };
            let result = CustomerService::list_customers(conn, &filters, &pagination)?;

            println!("Customers (Page {} of {}):", result.pagination.current_page, result.pagination.total_pages);
            println!("Total: {} customers", result.pagination.total_count);
            println!();

            for customer in result.data {
                println!("ID: {} | Code: {} | Name: {} | Type: {} | Status: {}",
                    customer.id,
                    customer.customer_code,
                    customer.name,
                    customer.customer_type,
                    customer.status
                );
            }
        }
        CustomerAction::Show { id, code } => {
            let customer = if let Some(id) = id {
                CustomerService::get_customer_with_stats(conn, id)?
            } else if let Some(code) = code {
                if let Some(customer) = CustomerService::get_customer_by_code(conn, &code)? {
                    CustomerService::get_customer_with_stats(conn, customer.id)?
                } else {
                    None
                }
            } else {
                return Err(crate::core::error::CLIERPError::Validation(
                    "Either --id or --code must be provided".to_string()
                ));
            };

            if let Some(customer_stats) = customer {
                println!("Customer Details:");
                println!("ID: {}", customer_stats.customer.id);
                println!("Code: {}", customer_stats.customer.customer_code);
                println!("Name: {}", customer_stats.customer.name);
                println!("Type: {}", customer_stats.customer.customer_type);
                println!("Status: {}", customer_stats.customer.status);
                if let Some(email) = &customer_stats.customer.email {
                    println!("Email: {}", email);
                }
                if let Some(phone) = &customer_stats.customer.phone {
                    println!("Phone: {}", phone);
                }
                if let Some(company) = &customer_stats.customer.company_name {
                    println!("Company: {}", company);
                }
                println!("Credit Limit: {}", customer_stats.customer.credit_limit.map_or("None".to_string(), |limit| limit.to_string()));
                println!();
                println!("Statistics:");
                println!("Total Leads: {}", customer_stats.total_leads);
                println!("Active Deals: {}", customer_stats.active_deals);
                println!("Total Deal Value: {}", customer_stats.total_deal_value);
            } else {
                println!("Customer not found");
            }
        }
        CustomerAction::Update {
            id,
            name,
            email,
            phone,
            address,
            company_name,
            tax_id,
            credit_limit,
            status,
            notes,
        } => {
            let customer = CustomerService::update_customer(
                conn,
                id,
                name.as_deref(),
                email.as_deref().map(Some),
                phone.as_deref().map(Some),
                address.as_deref().map(Some),
                company_name.as_deref().map(Some),
                tax_id.as_deref().map(Some),
                credit_limit,
                status,
                notes.as_deref().map(Some),
            )?;
            println!("Customer updated successfully:");
            println!("ID: {}, Name: {}", customer.id, customer.name);
        }
        CustomerAction::Delete { id } => {
            let deleted = CustomerService::delete_customer(conn, id)?;
            if deleted {
                println!("Customer deleted successfully");
            } else {
                println!("Customer not found");
            }
        }
        CustomerAction::Search { query } => {
            let customers = CustomerService::search_customers(conn, &query)?;
            println!("Search results for '{}':", query);
            for customer in customers {
                println!("ID: {} | Code: {} | Name: {} | Type: {}",
                    customer.id,
                    customer.customer_code,
                    customer.name,
                    customer.customer_type
                );
            }
        }
        CustomerAction::Stats => {
            let stats = CustomerService::get_customer_statistics(conn)?;
            println!("Customer Statistics:");
            println!("Total Customers: {}", stats.total_customers);
            println!("Active Customers: {}", stats.active_customers);
            println!("Business Customers: {}", stats.business_customers);
            println!("Individual Customers: {}", stats.individual_customers);
            println!("Total Credit Limit: {}", stats.total_credit_limit);
        }
    }
    Ok(())
}

fn execute_deal_command(conn: &mut DatabaseConnection, action: DealAction) -> CLIERPResult<()> {
    match action {
        DealAction::Create {
            lead_id,
            title,
            deal_value,
            expected_close_date,
            assigned_to,
            description,
            notes,
        } => {
            let deal = DealService::create_deal(
                conn,
                lead_id,
                &title,
                deal_value,
                expected_close_date,
                assigned_to,
                description.as_deref(),
                notes.as_deref(),
            )?;
            println!("Deal created successfully:");
            println!("ID: {}, Title: {}, Value: {}", deal.id, deal.deal_name, deal.deal_value);
        }
        DealAction::List {
            page,
            per_page,
            search,
            stage,
            assigned_to,
            date_from,
            date_to,
            sort_by,
            sort_desc,
        } => {
            let pagination = PaginationParams::new(page as usize, per_page);
            let filters = FilterOptions {
                search,
                status: stage,
                assigned_to,
                date_from,
                date_to,
                sort_by,
                sort_desc,
                ..Default::default()
            };
            let result = DealService::list_deals(conn, &filters, &pagination)?;

            println!("Deals (Page {} of {}):", result.pagination.current_page, result.pagination.total_pages);
            println!("Total: {} deals", result.pagination.total_count);
            println!();

            for deal_details in result.data {
                let customer_name = deal_details.customer
                    .as_ref()
                    .map(|c| c.name.as_str())
                    .unwrap_or("N/A");
                println!("ID: {} | Title: {} | Value: {} | Stage: {} | Customer: {}",
                    deal_details.deal.id,
                    deal_details.deal.deal_name,
                    deal_details.deal.deal_value,
                    deal_details.deal.stage,
                    customer_name
                );
            }
        }
        DealAction::Show { id } => {
            if let Some(deal_details) = DealService::get_deal_with_details(conn, id)? {
                println!("Deal Details:");
                println!("ID: {}", deal_details.deal.id);
                println!("Title: {}", deal_details.deal.deal_name);
                println!("Value: {}", deal_details.deal.deal_value);
                println!("Stage: {}", deal_details.deal.stage);
                println!("Probability: {}%", deal_details.deal.probability.map_or("N/A".to_string(), |p| p.to_string()));

                if let Some(customer) = &deal_details.customer {
                    println!("Customer: {} ({})", customer.name, customer.customer_code);
                }

                println!("Lead: {}", deal_details.lead.title);

                if let Some(assigned) = &deal_details.assigned_employee {
                    println!("Assigned to: {}", assigned);
                }

                if let Some(close_date) = deal_details.deal.close_date {
                    println!("Expected Close Date: {}", close_date);
                }

                if let Some(description) = &deal_details.deal.notes {
                    println!("Description: {}", description);
                }
            } else {
                println!("Deal not found");
            }
        }
        DealAction::UpdateStage { id, stage, notes } => {
            let deal = DealService::update_deal_stage(conn, id, stage, notes.as_deref())?;
            println!("Deal stage updated successfully:");
            println!("ID: {}, Stage: {}, Probability: {}%", deal.id, deal.stage, deal.probability.map_or("N/A".to_string(), |p| p.to_string()));
        }
        DealAction::Update {
            id,
            title,
            deal_value,
            expected_close_date,
            assigned_to,
            description,
            notes,
        } => {
            let deal = DealService::update_deal(
                conn,
                id,
                title.as_deref(),
                deal_value,
                expected_close_date.map(Some),
                assigned_to.map(Some),
                description.as_deref().map(Some),
                notes.as_deref().map(Some),
            )?;
            println!("Deal updated successfully:");
            println!("ID: {}, Title: {}", deal.id, deal.deal_name);
        }
        DealAction::Delete { id } => {
            let deleted = DealService::delete_deal(conn, id)?;
            if deleted {
                println!("Deal deleted successfully");
            } else {
                println!("Deal not found");
            }
        }
        DealAction::ByStage { stage } => {
            let deals = DealService::get_deals_by_stage(conn, stage)?;
            println!("Deals in {} stage:", stage.to_string());
            for deal_details in deals {
                let customer_name = deal_details.customer
                    .as_ref()
                    .map(|c| c.name.as_str())
                    .unwrap_or("N/A");
                println!("ID: {} | Title: {} | Value: {} | Customer: {}",
                    deal_details.deal.id,
                    deal_details.deal.deal_name,
                    deal_details.deal.deal_value,
                    customer_name
                );
            }
        }
        DealAction::Stats => {
            let stats = DealService::get_deal_statistics(conn)?;
            println!("Deal Statistics:");
            println!("Total Deals: {}", stats.total_deals);
            println!("Active Deals: {}", stats.active_deals);
            println!("Won Deals: {}", stats.won_deals);
            println!("Lost Deals: {}", stats.lost_deals);
            println!("Total Pipeline Value: {}", stats.total_pipeline_value);
            println!("Total Won Value: {}", stats.total_won_value);
            println!("Average Deal Size: {:.2}", stats.average_deal_size);
            println!("Win Rate: {:.1}%", stats.win_rate);
        }
    }
    Ok(())
}

fn execute_lead_command(conn: &mut DatabaseConnection, action: LeadAction) -> CLIERPResult<()> {
    match action {
        LeadAction::Create {
            title,
            customer_id,
            lead_source,
            estimated_value,
            expected_close_date,
            priority,
            assigned_to,
            description,
            notes,
        } => {
            let lead = LeadService::create_lead(
                conn,
                &title,
                customer_id,
                &lead_source,
                estimated_value,
                expected_close_date,
                priority,
                assigned_to,
                description.as_deref(),
                notes.as_deref(),
            )?;
            println!("Lead created successfully:");
            println!("ID: {}, Title: {}, Value: {}", lead.id, lead.title, lead.estimated_value.map_or("N/A".to_string(), |v| v.to_string()));
        }
        _ => {
            // TODO: Implement other lead actions
            println!("Lead action not yet implemented");
        }
    }
    Ok(())
}

fn execute_campaign_command(conn: &mut DatabaseConnection, action: CampaignAction) -> CLIERPResult<()> {
    match action {
        CampaignAction::Create {
            name,
            campaign_type,
            description,
            start_date,
            end_date,
            budget,
            target_audience,
            goals,
        } => {
            let campaign = CampaignService::create_campaign(
                conn,
                &name,
                campaign_type,
                description.as_deref(),
                start_date,
                end_date,
                budget,
                target_audience.as_deref(),
                goals.as_deref(),
            )?;
            println!("Campaign created successfully:");
            println!("ID: {}, Type: {}, Name: {}", campaign.id, campaign.campaign_type, campaign.name);
        }
        CampaignAction::Stats => {
            let stats = CampaignService::get_campaign_statistics(conn)?;
            println!("Campaign Statistics:");
            println!("Total Campaigns: {}", stats.total_campaigns);
            println!("Active Campaigns: {}", stats.active_campaigns);
            println!("Draft Campaigns: {}", stats.draft_campaigns);
            println!("Completed Campaigns: {}", stats.completed_campaigns);
            println!("Total Budget: {}", stats.total_budget);
            println!("Total Actual Cost: {}", stats.total_actual_cost);
        }
        CampaignAction::Performance => {
            let performance = CampaignService::get_campaign_performance(conn)?;
            println!("Campaign Performance:");
            println!();
            for perf in performance {
                println!("Campaign: {} ({})", perf.campaign_name, perf.campaign_code);
                println!("  Type: {} | Status: {}", perf.campaign_type, perf.status);
                println!("  Leads: {} | Qualified: {} | Conversion: {:.1}%",
                    perf.total_leads, perf.qualified_leads, perf.conversion_rate);
                println!("  Budget: {} | Cost: {} | Cost/Lead: {:.2}",
                    perf.budget, perf.actual_cost, perf.cost_per_lead);
                println!();
            }
        }
        _ => {
            println!("Campaign action not yet implemented");
        }
    }
    Ok(())
}

fn execute_activity_command(conn: &mut DatabaseConnection, action: ActivityAction) -> CLIERPResult<()> {
    match action {
        ActivityAction::Create {
            activity_type,
            title,
            description,
            customer_id,
            lead_id,
            assigned_to,
            due_date,
            priority,
        } => {
            let activity = ActivityService::create_activity(
                conn,
                activity_type,
                &title,
                description.as_deref(),
                customer_id,
                lead_id,
                assigned_to,
                due_date,
                priority.as_deref(),
            )?;
            println!("Activity created successfully:");
            println!("ID: {}, Title: {}, Type: {}", activity.id, activity.subject, activity.activity_type);
        }
        ActivityAction::Stats => {
            let stats = ActivityService::get_activity_statistics(conn)?;
            println!("Activity Statistics:");
            println!("Total Activities: {}", stats.total_activities);
            println!("Pending Activities: {}", stats.pending_activities);
            println!("Completed Activities: {}", stats.completed_activities);
            println!("Overdue Activities: {}", stats.overdue_activities);
            println!();
            println!("By Type:");
            println!("  Calls: {}", stats.call_activities);
            println!("  Emails: {}", stats.email_activities);
            println!("  Meetings: {}", stats.meeting_activities);
            println!("  Tasks: {}", stats.task_activities);
        }
        ActivityAction::Overdue => {
            let activities = ActivityService::get_overdue_activities(conn)?;
            println!("Overdue Activities:");
            for activity_details in activities {
                let entity_name = if let Some(customer) = &activity_details.customer {
                    format!("Customer: {}", customer.name)
                } else if let Some(lead) = &activity_details.lead {
                    format!("Lead: {}", lead.title)
                } else {
                    "No entity".to_string()
                };

                println!("ID: {} | Title: {} | Type: {} | Due: {} | {}",
                    activity_details.activity.id,
                    activity_details.activity.subject,
                    activity_details.activity.activity_type,
                    activity_details.activity.activity_date.format("%Y-%m-%d %H:%M").to_string(),
                    entity_name
                );
            }
        }
        _ => {
            println!("Activity action not yet implemented");
        }
    }
    Ok(())
}

fn execute_dashboard_command(conn: &mut DatabaseConnection) -> CLIERPResult<()> {
    println!("=== CRM Dashboard ===");
    println!();

    // Customer stats
    let customer_stats = CustomerService::get_customer_statistics(conn)?;
    println!("📊 Customer Overview:");
    println!("  Total: {} | Active: {} | Business: {} | Individual: {}",
        customer_stats.total_customers,
        customer_stats.active_customers,
        customer_stats.business_customers,
        customer_stats.individual_customers
    );
    println!();

    // Lead stats
    let lead_stats = LeadService::get_lead_statistics(conn)?;
    println!("🎯 Lead Overview:");
    println!("  Total: {} | New: {} | Qualified: {} | Won: {} | Lost: {}",
        lead_stats.total_leads,
        lead_stats.new_leads,
        lead_stats.qualified_leads,
        lead_stats.closed_won,
        lead_stats.closed_lost
    );
    println!("  Conversion Rate: {:.1}% | Avg Deal Size: {:.2}",
        lead_stats.conversion_rate,
        lead_stats.average_deal_size
    );
    println!();

    // Deal stats
    let deal_stats = DealService::get_deal_statistics(conn)?;
    println!("💰 Deal Overview:");
    println!("  Total: {} | Active: {} | Won: {} | Lost: {}",
        deal_stats.total_deals,
        deal_stats.active_deals,
        deal_stats.won_deals,
        deal_stats.lost_deals
    );
    println!("  Pipeline Value: {} | Won Value: {} | Win Rate: {:.1}%",
        deal_stats.total_pipeline_value,
        deal_stats.total_won_value,
        deal_stats.win_rate
    );
    println!();

    // Activity stats
    let activity_stats = ActivityService::get_activity_statistics(conn)?;
    println!("📋 Activity Overview:");
    println!("  Total: {} | Pending: {} | Completed: {} | Overdue: {}",
        activity_stats.total_activities,
        activity_stats.pending_activities,
        activity_stats.completed_activities,
        activity_stats.overdue_activities
    );

    Ok(())
}

fn execute_pipeline_command(conn: &mut DatabaseConnection) -> CLIERPResult<()> {
    println!("=== Sales Pipeline ===");
    println!();

    let pipeline = DealService::get_sales_pipeline(conn)?;

    for stage in pipeline {
        println!("📍 {}", stage.stage);
        println!("   Deals: {} | Total Value: {} | Avg Value: {:.2}",
            stage.count,
            stage.total_value,
            stage.average_value
        );
        println!();
    }

    Ok(())
}

fn execute_performance_command(conn: &mut DatabaseConnection) -> CLIERPResult<()> {
    println!("=== Performance Overview ===");
    println!();

    // Campaign performance
    let campaign_performance = CampaignService::get_campaign_performance(conn)?;
    if !campaign_performance.is_empty() {
        println!("🎯 Top Campaigns by Lead Generation:");
        let mut sorted_campaigns = campaign_performance;
        sorted_campaigns.sort_by(|a, b| b.total_leads.cmp(&a.total_leads));

        for (i, perf) in sorted_campaigns.iter().take(5).enumerate() {
            println!("{}. {} - {} leads ({:.1}% conversion)",
                i + 1,
                perf.campaign_name,
                perf.total_leads,
                perf.conversion_rate
            );
        }
        println!();
    }

    // Sales pipeline summary
    let pipeline = DealService::get_sales_pipeline(conn)?;
    let total_pipeline_value: i32 = pipeline.iter()
        .filter(|stage| stage.stage != "closed_won" && stage.stage != "closed_lost")
        .map(|stage| stage.total_value)
        .sum();

    println!("💰 Pipeline Health:");
    println!("   Active Pipeline Value: {}", total_pipeline_value);

    let qualification_deals = pipeline.iter()
        .find(|stage| stage.stage == "qualification")
        .map(|stage| stage.count)
        .unwrap_or(0);

    let negotiation_deals = pipeline.iter()
        .find(|stage| stage.stage == "negotiation")
        .map(|stage| stage.count)
        .unwrap_or(0);

    println!("   Deals in Qualification: {} | Deals in Negotiation: {}",
        qualification_deals, negotiation_deals);

    Ok(())
}