-- Drop CRM tables in reverse order
DROP INDEX IF EXISTS idx_activities_assigned_to;
DROP INDEX IF EXISTS idx_activities_type_date;
DROP INDEX IF EXISTS idx_activities_deal_id;
DROP INDEX IF EXISTS idx_activities_lead_id;
DROP INDEX IF EXISTS idx_activities_customer_id;

DROP INDEX IF EXISTS idx_campaigns_dates;
DROP INDEX IF EXISTS idx_campaigns_status;

DROP INDEX IF EXISTS idx_deals_close_date;
DROP INDEX IF EXISTS idx_deals_assigned_to;
DROP INDEX IF EXISTS idx_deals_stage;
DROP INDEX IF EXISTS idx_deals_lead_id;

DROP INDEX IF EXISTS idx_leads_expected_close;
DROP INDEX IF EXISTS idx_leads_assigned_to;
DROP INDEX IF EXISTS idx_leads_status;
DROP INDEX IF EXISTS idx_leads_customer_id;

DROP INDEX IF EXISTS idx_customers_type_status;
DROP INDEX IF EXISTS idx_customers_email;
DROP INDEX IF EXISTS idx_customers_name;
DROP INDEX IF EXISTS idx_customers_code;

DROP TABLE IF EXISTS activities;
DROP TABLE IF EXISTS campaign_leads;
DROP TABLE IF EXISTS campaigns;
DROP TABLE IF EXISTS deals;
DROP TABLE IF EXISTS leads;
DROP TABLE IF EXISTS customers;