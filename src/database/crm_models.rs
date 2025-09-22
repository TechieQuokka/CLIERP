use chrono::{NaiveDate, NaiveDateTime};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use clap::ValueEnum;

use super::schema::{customers, leads, deals, campaigns, campaign_leads, activities};

// Customer models
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = customers)]
pub struct Customer {
    pub id: i32,
    pub customer_code: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub customer_type: String,
    pub company_name: Option<String>,
    pub tax_id: Option<String>,
    pub credit_limit: Option<i32>,
    pub status: String,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = customers)]
pub struct NewCustomer {
    pub customer_code: String,
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub customer_type: String,
    pub company_name: Option<String>,
    pub tax_id: Option<String>,
    pub credit_limit: Option<i32>,
    pub status: String,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum CustomerType {
    Individual,
    Business,
}

impl std::fmt::Display for CustomerType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomerType::Individual => write!(f, "individual"),
            CustomerType::Business => write!(f, "business"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum CustomerStatus {
    Active,
    Inactive,
    Suspended,
}

impl std::fmt::Display for CustomerStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CustomerStatus::Active => write!(f, "active"),
            CustomerStatus::Inactive => write!(f, "inactive"),
            CustomerStatus::Suspended => write!(f, "suspended"),
        }
    }
}

// Lead models
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = leads)]
pub struct Lead {
    pub id: i32,
    pub customer_id: Option<i32>,
    pub lead_source: String,
    pub status: String,
    pub priority: String,
    pub estimated_value: Option<i32>,
    pub probability: Option<i32>,
    pub expected_close_date: Option<NaiveDate>,
    pub assigned_to: Option<i32>,
    pub title: String,
    pub description: Option<String>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = leads)]
pub struct NewLead {
    pub customer_id: Option<i32>,
    pub lead_source: String,
    pub status: String,
    pub priority: String,
    pub estimated_value: Option<i32>,
    pub probability: Option<i32>,
    pub expected_close_date: Option<NaiveDate>,
    pub assigned_to: Option<i32>,
    pub title: String,
    pub description: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum LeadStatus {
    New,
    Contacted,
    Qualified,
    Proposal,
    Negotiation,
    ClosedWon,
    ClosedLost,
}

impl std::fmt::Display for LeadStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeadStatus::New => write!(f, "new"),
            LeadStatus::Contacted => write!(f, "contacted"),
            LeadStatus::Qualified => write!(f, "qualified"),
            LeadStatus::Proposal => write!(f, "proposal"),
            LeadStatus::Negotiation => write!(f, "negotiation"),
            LeadStatus::ClosedWon => write!(f, "closed_won"),
            LeadStatus::ClosedLost => write!(f, "closed_lost"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum LeadPriority {
    Low,
    Medium,
    High,
    Urgent,
}

impl std::fmt::Display for LeadPriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LeadPriority::Low => write!(f, "low"),
            LeadPriority::Medium => write!(f, "medium"),
            LeadPriority::High => write!(f, "high"),
            LeadPriority::Urgent => write!(f, "urgent"),
        }
    }
}

// Deal models
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = deals)]
pub struct Deal {
    pub id: i32,
    pub lead_id: Option<i32>,
    pub deal_name: String,
    pub stage: String,
    pub deal_value: i32,
    pub close_date: Option<NaiveDate>,
    pub probability: Option<i32>,
    pub assigned_to: Option<i32>,
    pub products: Option<String>, // JSON string
    pub discount_percent: Option<i32>,
    pub final_amount: Option<i32>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = deals)]
pub struct NewDeal {
    pub lead_id: Option<i32>,
    pub deal_name: String,
    pub stage: String,
    pub deal_value: i32,
    pub close_date: Option<NaiveDate>,
    pub probability: Option<i32>,
    pub assigned_to: Option<i32>,
    pub products: Option<String>,
    pub discount_percent: Option<i32>,
    pub final_amount: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum DealStage {
    Prospecting,
    Qualification,
    NeedsAnalysis,
    Proposal,
    Negotiation,
    Closing,
    ClosedWon,
    ClosedLost,
}

impl std::fmt::Display for DealStage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DealStage::Prospecting => write!(f, "prospecting"),
            DealStage::Qualification => write!(f, "qualification"),
            DealStage::NeedsAnalysis => write!(f, "needs_analysis"),
            DealStage::Proposal => write!(f, "proposal"),
            DealStage::Negotiation => write!(f, "negotiation"),
            DealStage::Closing => write!(f, "closing"),
            DealStage::ClosedWon => write!(f, "closed_won"),
            DealStage::ClosedLost => write!(f, "closed_lost"),
        }
    }
}

// Campaign models
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = campaigns)]
pub struct Campaign {
    pub id: i32,
    pub name: String,
    pub description: Option<String>,
    pub campaign_type: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub budget: Option<i32>,
    pub spent: Option<i32>,
    pub target_audience: Option<String>,
    pub status: String,
    pub created_by: Option<i32>,
    pub notes: Option<String>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = campaigns)]
pub struct NewCampaign {
    pub name: String,
    pub description: Option<String>,
    pub campaign_type: String,
    pub start_date: Option<NaiveDate>,
    pub end_date: Option<NaiveDate>,
    pub budget: Option<i32>,
    pub spent: Option<i32>,
    pub target_audience: Option<String>,
    pub status: String,
    pub created_by: Option<i32>,
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum CampaignType {
    Email,
    Phone,
    Social,
    Event,
    Advertising,
}

impl std::fmt::Display for CampaignType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CampaignType::Email => write!(f, "email"),
            CampaignType::Phone => write!(f, "phone"),
            CampaignType::Social => write!(f, "social"),
            CampaignType::Event => write!(f, "event"),
            CampaignType::Advertising => write!(f, "advertising"),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum CampaignStatus {
    Draft,
    Planned,
    Active,
    Paused,
    Completed,
    Cancelled,
}

impl std::fmt::Display for CampaignStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CampaignStatus::Draft => write!(f, "draft"),
            CampaignStatus::Planned => write!(f, "planned"),
            CampaignStatus::Active => write!(f, "active"),
            CampaignStatus::Paused => write!(f, "paused"),
            CampaignStatus::Completed => write!(f, "completed"),
            CampaignStatus::Cancelled => write!(f, "cancelled"),
        }
    }
}

// Activity models
#[derive(Debug, Clone, Serialize, Deserialize, Queryable, Selectable)]
#[diesel(table_name = activities)]
pub struct Activity {
    pub id: i32,
    pub customer_id: Option<i32>,
    pub lead_id: Option<i32>,
    pub deal_id: Option<i32>,
    pub activity_type: String,
    pub subject: String,
    pub description: Option<String>,
    pub activity_date: NaiveDateTime,
    pub duration_minutes: Option<i32>,
    pub outcome: Option<String>,
    pub assigned_to: Option<i32>,
    pub completed: bool,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[derive(Debug, Serialize, Deserialize, Insertable)]
#[diesel(table_name = activities)]
pub struct NewActivity {
    pub customer_id: Option<i32>,
    pub lead_id: Option<i32>,
    pub deal_id: Option<i32>,
    pub activity_type: String,
    pub subject: String,
    pub description: Option<String>,
    pub activity_date: NaiveDateTime,
    pub duration_minutes: Option<i32>,
    pub outcome: Option<String>,
    pub assigned_to: Option<i32>,
    pub completed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, ValueEnum)]
pub enum ActivityType {
    Call,
    Email,
    Meeting,
    Task,
    Note,
}

impl std::fmt::Display for ActivityType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ActivityType::Call => write!(f, "call"),
            ActivityType::Email => write!(f, "email"),
            ActivityType::Meeting => write!(f, "meeting"),
            ActivityType::Task => write!(f, "task"),
            ActivityType::Note => write!(f, "note"),
        }
    }
}

// DTOs for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerWithStats {
    pub customer: Customer,
    pub total_leads: i64,
    pub active_deals: i64,
    pub total_deal_value: i32,
    pub last_activity: Option<NaiveDateTime>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeadWithCustomer {
    pub lead: Lead,
    pub customer: Option<Customer>,
    pub assigned_employee: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DealWithDetails {
    pub deal: Deal,
    pub lead: Option<Lead>,
    pub customer: Option<Customer>,
    pub assigned_employee: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SalesPipelineStage {
    pub stage: String,
    pub deal_count: i64,
    pub total_value: i32,
    pub average_probability: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerSummary {
    pub id: i32,
    pub customer_code: String,
    pub name: String,
    pub customer_type: String,
    pub total_deals: i64,
    pub total_value: i32,
    pub status: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CampaignWithStats {
    pub campaign: Campaign,
    pub total_leads: i64,
    pub converted_leads: i64,
    pub conversion_rate: f64,
    pub total_revenue: i32,
    pub roi: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityWithDetails {
    pub activity: Activity,
    pub customer: Option<Customer>,
    pub lead: Option<Lead>,
    pub assigned_employee: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct PipelineStage {
    pub stage: String,
    pub deal_count: i64,
    pub total_value: i32,
    pub average_value: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CampaignPerformance {
    pub campaign_name: String,
    pub campaign_code: String,
    pub leads_generated: i64,
    pub conversion_rate: f64,
    pub roi: f64,
}

// Statistics structures
#[derive(Debug, Serialize, Deserialize)]
pub struct CustomerStatistics {
    pub total_customers: i64,
    pub active_customers: i64,
    pub business_customers: i64,
    pub individual_customers: i64,
    pub total_credit_limit: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LeadStatistics {
    pub total_leads: i64,
    pub new_leads: i64,
    pub qualified_leads: i64,
    pub converted_leads: i64,
    pub conversion_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DealStatistics {
    pub total_deals: i64,
    pub active_deals: i64,
    pub won_deals: i64,
    pub lost_deals: i64,
    pub total_value: i32,
    pub total_won_value: i32,
    pub average_deal_size: f64,
    pub win_rate: f64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CampaignStatistics {
    pub total_campaigns: i64,
    pub active_campaigns: i64,
    pub draft_campaigns: i64,
    pub completed_campaigns: i64,
    pub total_budget: i32,
    pub total_actual_cost: i32,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ActivityStatistics {
    pub total_activities: i64,
    pub pending_activities: i64,
    pub completed_activities: i64,
    pub overdue_activities: i64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DealProduct {
    pub product_id: i32,
    pub quantity: i32,
    pub unit_price: i32,
}

impl DealProduct {
    pub fn total_price(&self) -> i32 {
        self.quantity * self.unit_price
    }
}