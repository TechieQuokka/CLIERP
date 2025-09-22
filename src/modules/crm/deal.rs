use diesel::prelude::*;
use chrono::{Utc, NaiveDate};
use crate::core::result::CLIERPResult;
use crate::database::{
    DatabaseConnection, Deal, NewDeal, DealStage, Lead, Customer, Employee
};
use crate::database::schema::{deals, leads, customers, employees};
use crate::utils::validation::validate_required_string;
use crate::utils::pagination::{Paginate, PaginationParams, PaginatedResult};
use crate::utils::filters::FilterOptions;

pub struct DealService;

impl DealService {
    pub fn create_deal(
        conn: &mut DatabaseConnection,
        lead_id: i32,
        title: &str,
        deal_value: i32,
        expected_close_date: Option<NaiveDate>,
        assigned_to: Option<i32>,
        description: Option<&str>,
        notes: Option<&str>,
    ) -> Result<Deal> {
        // Validate input
        let validator = Validator::new();
        validator
            .required("title", title)?
            .min_length("title", title, 2)?
            .max_length("title", title, 200)?
            .positive("deal_value", deal_value as f64)?;

        // Verify lead exists
        let lead = leads::table
            .find(lead_id)
            .first::<Lead>(conn)?;

        // Verify assigned employee exists if provided
        if let Some(assigned_to) = assigned_to {
            employees::table
                .find(assigned_to)
                .first::<Employee>(conn)?;
        }

        // Create new deal
        let new_deal = NewDeal {
            lead_id,
            title: title.to_string(),
            deal_value,
            stage: DealStage::Qualification.to_string(),
            probability: Self::calculate_probability_for_stage(&DealStage::Qualification),
            expected_close_date,
            assigned_to,
            description: description.map(|s| s.to_string()),
            notes: notes.map(|s| s.to_string()),
        };

        diesel::insert_into(deals::table)
            .values(&new_deal)
            .returning(Deal::as_returning())
            .get_result(conn)
            .map_err(Into::into)
    }

    pub fn get_deal_by_id(conn: &mut DatabaseConnection, deal_id: i32) -> Result<Option<Deal>> {
        deals::table
            .find(deal_id)
            .first::<Deal>(conn)
            .optional()
            .map_err(Into::into)
    }

    pub fn get_deal_with_details(conn: &mut DatabaseConnection, deal_id: i32) -> Result<Option<DealWithDetails>> {
        let deal = Self::get_deal_by_id(conn, deal_id)?;

        if let Some(deal) = deal {
            // Get lead and customer info
            let (lead, customer): (Lead, Option<Customer>) = leads::table
                .left_join(customers::table)
                .filter(leads::id.eq(deal.lead_id))
                .select((Lead::as_select(), customers::all_columns.nullable()))
                .first(conn)?;

            // Get assigned employee name if available
            let assigned_employee = if let Some(assigned_to) = deal.assigned_to {
                employees::table
                    .find(assigned_to)
                    .select(employees::name)
                    .first::<String>(conn)
                    .optional()?
            } else {
                None
            };

            Ok(Some(DealWithDetails {
                deal,
                lead,
                customer,
                assigned_employee,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_deals(
        conn: &mut DatabaseConnection,
        filters: &FilterOptions,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResult<DealWithDetails>> {
        let mut query = deals::table
            .inner_join(leads::table)
            .left_join(customers::table.on(customers::id.eq(leads::customer_id.nullable())))
            .left_join(employees::table.on(employees::id.eq(deals::assigned_to.nullable())))
            .select((
                Deal::as_select(),
                Lead::as_select(),
                customers::all_columns.nullable(),
                employees::name.nullable(),
            ))
            .into_boxed();

        // Apply filters
        if let Some(search) = &filters.search {
            query = query.filter(
                deals::title.like(format!("%{}%", search))
                    .or(customers::name.like(format!("%{}%", search)))
                    .or(leads::title.like(format!("%{}%", search)))
            );
        }

        if let Some(status_filter) = &filters.status {
            query = query.filter(deals::stage.eq(status_filter));
        }

        if let Some(assigned_to) = filters.assigned_to {
            query = query.filter(deals::assigned_to.eq(assigned_to));
        }

        if let Some(date_from) = filters.date_from {
            query = query.filter(deals::expected_close_date.ge(date_from));
        }

        if let Some(date_to) = filters.date_to {
            query = query.filter(deals::expected_close_date.le(date_to));
        }

        // Apply sorting
        query = match filters.sort_by.as_deref() {
            Some("title") => {
                if filters.sort_desc {
                    query.order(deals::title.desc())
                } else {
                    query.order(deals::title.asc())
                }
            }
            Some("stage") => {
                if filters.sort_desc {
                    query.order(deals::stage.desc())
                } else {
                    query.order(deals::stage.asc())
                }
            }
            Some("value") => {
                if filters.sort_desc {
                    query.order(deals::deal_value.desc())
                } else {
                    query.order(deals::deal_value.asc())
                }
            }
            Some("close_date") => {
                if filters.sort_desc {
                    query.order(deals::expected_close_date.desc())
                } else {
                    query.order(deals::expected_close_date.asc())
                }
            }
            Some("created_at") => {
                if filters.sort_desc {
                    query.order(deals::created_at.desc())
                } else {
                    query.order(deals::created_at.asc())
                }
            }
            _ => query.order(deals::created_at.desc()),
        };

        let results: Vec<(Deal, Lead, Option<Customer>, Option<String>)> = query
            .offset(pagination.offset())
            .limit(pagination.limit)
            .load(conn)?;

        let total_items = deals::table.count().get_result::<i64>(conn)?;

        let deals_with_details: Vec<DealWithDetails> = results
            .into_iter()
            .map(|(deal, lead, customer, assigned_employee)| DealWithDetails {
                deal,
                lead,
                customer,
                assigned_employee,
            })
            .collect();

        Ok(PaginatedResult {
            items: deals_with_details,
            total_items,
            page: pagination.page,
            per_page: pagination.per_page,
            total_pages: (total_items as f64 / pagination.per_page as f64).ceil() as i64,
        })
    }

    pub fn update_deal_stage(
        conn: &mut DatabaseConnection,
        deal_id: i32,
        new_stage: DealStage,
        notes: Option<&str>,
    ) -> Result<Deal> {
        let deal = Self::get_deal_by_id(conn, deal_id)?
            .ok_or_else(|| crate::core::error::CLIERPError::NotFound(
                format!("Deal with ID {} not found", deal_id)
            ))?;

        // Calculate new probability based on stage
        let new_probability = Self::calculate_probability_for_stage(&new_stage);

        let updated_notes = if let Some(new_notes) = notes {
            if let Some(existing_notes) = &deal.notes {
                Some(format!("{}\\n---\\n{}", existing_notes, new_notes))
            } else {
                Some(new_notes.to_string())
            }
        } else {
            deal.notes
        };

        // Set actual close date if deal is closed
        let actual_close_date = match new_stage {
            DealStage::ClosedWon | DealStage::ClosedLost => Some(Utc::now().naive_utc().date()),
            _ => deal.actual_close_date,
        };

        diesel::update(deals::table.find(deal_id))
            .set((
                deals::stage.eq(new_stage.to_string()),
                deals::probability.eq(new_probability),
                deals::notes.eq(updated_notes),
                deals::actual_close_date.eq(actual_close_date),
                deals::updated_at.eq(Utc::now().naive_utc()),
            ))
            .returning(Deal::as_returning())
            .get_result(conn)
            .map_err(Into::into)
    }

    pub fn update_deal(
        conn: &mut DatabaseConnection,
        deal_id: i32,
        title: Option<&str>,
        deal_value: Option<i32>,
        expected_close_date: Option<Option<NaiveDate>>,
        assigned_to: Option<Option<i32>>,
        description: Option<Option<&str>>,
        notes: Option<Option<&str>>,
    ) -> Result<Deal> {
        // Check if deal exists
        let _deal = Self::get_deal_by_id(conn, deal_id)?
            .ok_or_else(|| crate::core::error::CLIERPError::NotFound(
                format!("Deal with ID {} not found", deal_id)
            ))?;

        // Validate input
        let validator = Validator::new();
        if let Some(title) = title {
            validator
                .required("title", title)?
                .min_length("title", title, 2)?
                .max_length("title", title, 200)?;
        }

        if let Some(deal_value) = deal_value {
            validator.positive("deal_value", *deal_value as f64)?;
        }

        // Build update query
        use crate::database::schema::deals::dsl::*;
        let mut update_query = diesel::update(deals.find(deal_id));

        if let Some(title_val) = title {
            update_query = update_query.set(title.eq(title_val));
        }
        if let Some(value_val) = deal_value {
            update_query = update_query.set(deal_value.eq(*value_val));
        }
        if let Some(date_val) = expected_close_date {
            update_query = update_query.set(expected_close_date.eq(*date_val));
        }
        if let Some(assigned_val) = assigned_to {
            update_query = update_query.set(assigned_to.eq(*assigned_val));
        }
        if let Some(desc_val) = description {
            update_query = update_query.set(description.eq(desc_val.map(|s| s.to_string())));
        }
        if let Some(notes_val) = notes {
            update_query = update_query.set(notes.eq(notes_val.map(|s| s.to_string())));
        }

        // Always update the updated_at timestamp
        update_query = update_query.set(updated_at.eq(Utc::now().naive_utc()));

        update_query
            .returning(Deal::as_returning())
            .get_result(conn)
            .map_err(Into::into)
    }

    pub fn delete_deal(conn: &mut DatabaseConnection, deal_id: i32) -> Result<bool> {
        let deleted_rows = diesel::delete(deals::table.find(deal_id))
            .execute(conn)?;

        Ok(deleted_rows > 0)
    }

    pub fn get_deals_by_stage(
        conn: &mut DatabaseConnection,
        stage: DealStage,
    ) -> Result<Vec<DealWithDetails>> {
        let results: Vec<(Deal, Lead, Option<Customer>, Option<String>)> = deals::table
            .inner_join(leads::table)
            .left_join(customers::table.on(customers::id.eq(leads::customer_id.nullable())))
            .left_join(employees::table.on(employees::id.eq(deals::assigned_to.nullable())))
            .filter(deals::stage.eq(stage.to_string()))
            .select((
                Deal::as_select(),
                Lead::as_select(),
                customers::all_columns.nullable(),
                employees::name.nullable(),
            ))
            .order(deals::created_at.desc())
            .load(conn)?;

        let deals_with_details: Vec<DealWithDetails> = results
            .into_iter()
            .map(|(deal, lead, customer, assigned_employee)| DealWithDetails {
                deal,
                lead,
                customer,
                assigned_employee,
            })
            .collect();

        Ok(deals_with_details)
    }

    pub fn get_sales_pipeline(conn: &mut DatabaseConnection) -> Result<Vec<PipelineStage>> {
        use crate::database::DealStage as DS;

        let stages = vec![
            DS::Qualification,
            DS::NeedsAnalysis,
            DS::Proposal,
            DS::Negotiation,
            DS::ClosedWon,
            DS::ClosedLost,
        ];

        let mut pipeline = Vec::new();

        for stage in stages {
            let count = deals::table
                .filter(deals::stage.eq(stage.to_string()))
                .count()
                .get_result::<i64>(conn)?;

            let total_value: Option<i64> = deals::table
                .filter(deals::stage.eq(stage.to_string()))
                .select(diesel::dsl::sum(deals::deal_value))
                .first(conn)?;

            pipeline.push(PipelineStage {
                stage: stage.to_string(),
                count,
                total_value: total_value.unwrap_or(0) as i32,
                average_value: if count > 0 {
                    total_value.unwrap_or(0) as f64 / count as f64
                } else {
                    0.0
                },
            });
        }

        Ok(pipeline)
    }

    pub fn get_deal_statistics(conn: &mut DatabaseConnection) -> Result<DealStatistics> {
        // Total deals count
        let total_deals = deals::table
            .count()
            .get_result::<i64>(conn)?;

        // Active deals (not closed)
        let active_deals = deals::table
            .filter(
                deals::stage.ne(DealStage::ClosedWon.to_string())
                    .and(deals::stage.ne(DealStage::ClosedLost.to_string()))
            )
            .count()
            .get_result::<i64>(conn)?;

        // Won deals
        let won_deals = deals::table
            .filter(deals::stage.eq(DealStage::ClosedWon.to_string()))
            .count()
            .get_result::<i64>(conn)?;

        // Lost deals
        let lost_deals = deals::table
            .filter(deals::stage.eq(DealStage::ClosedLost.to_string()))
            .count()
            .get_result::<i64>(conn)?;

        // Total pipeline value (active deals)
        let total_pipeline_value: Option<i64> = deals::table
            .filter(
                deals::stage.ne(DealStage::ClosedWon.to_string())
                    .and(deals::stage.ne(DealStage::ClosedLost.to_string()))
            )
            .select(diesel::dsl::sum(deals::deal_value))
            .first(conn)?;

        // Total won value
        let total_won_value: Option<i64> = deals::table
            .filter(deals::stage.eq(DealStage::ClosedWon.to_string()))
            .select(diesel::dsl::sum(deals::deal_value))
            .first(conn)?;

        // Average deal size
        let average_deal_size = if total_deals > 0 {
            let total_value: Option<i64> = deals::table
                .select(diesel::dsl::sum(deals::deal_value))
                .first(conn)?;
            total_value.unwrap_or(0) as f64 / total_deals as f64
        } else {
            0.0
        };

        // Win rate
        let win_rate = if total_deals > 0 {
            (won_deals as f64 / total_deals as f64) * 100.0
        } else {
            0.0
        };

        Ok(DealStatistics {
            total_deals,
            active_deals,
            won_deals,
            lost_deals,
            total_pipeline_value: total_pipeline_value.unwrap_or(0) as i32,
            total_won_value: total_won_value.unwrap_or(0) as i32,
            average_deal_size,
            win_rate,
        })
    }

    fn calculate_probability_for_stage(stage: &DealStage) -> i32 {
        match stage {
            DealStage::Qualification => 20,
            DealStage::NeedsAnalysis => 40,
            DealStage::Proposal => 60,
            DealStage::Negotiation => 80,
            DealStage::ClosedWon => 100,
            DealStage::ClosedLost => 0,
        }
    }
}

#[derive(Debug, serde::Serialize)]
pub struct DealWithDetails {
    pub deal: Deal,
    pub lead: Lead,
    pub customer: Option<Customer>,
    pub assigned_employee: Option<String>,
}

#[derive(Debug, serde::Serialize)]
pub struct PipelineStage {
    pub stage: String,
    pub count: i64,
    pub total_value: i32,
    pub average_value: f64,
}

#[derive(Debug, serde::Serialize)]
pub struct DealStatistics {
    pub total_deals: i64,
    pub active_deals: i64,
    pub won_deals: i64,
    pub lost_deals: i64,
    pub total_pipeline_value: i32,
    pub total_won_value: i32,
    pub average_deal_size: f64,
    pub win_rate: f64,
}