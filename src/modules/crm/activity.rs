use diesel::prelude::*;
use chrono::{Utc, NaiveDateTime};
use crate::core::result::CLIERPResult;

// Type alias for convenience
type Result<T> = CLIERPResult<T>;
use crate::database::{
    DatabaseConnection, Activity, NewActivity, ActivityType, Customer, Lead, Employee
};
use crate::database::schema::{activities, customers, leads, employees};
use crate::utils::validation::validate_required_string;
use crate::utils::pagination::{Paginate, PaginationParams, PaginatedResult, PaginateResult};
use crate::utils::filters::FilterOptions;

pub struct ActivityService;

impl ActivityService {
    pub fn create_activity(
        conn: &mut DatabaseConnection,
        activity_type: ActivityType,
        subject: &str,
        description: Option<&str>,
        customer_id: Option<i32>,
        lead_id: Option<i32>,
        deal_id: Option<i32>,
        assigned_to: Option<i32>,
        activity_date: NaiveDateTime,
        duration_minutes: Option<i32>,
    ) -> Result<Activity> {
        // Validate input
        validate_required_string(subject, "subject")?;
        if subject.len() < 2 {
            return Err(crate::core::error::CLIERPError::Validation(
                "Subject must be at least 2 characters long".to_string()
            ));
        }
        if subject.len() > 200 {
            return Err(crate::core::error::CLIERPError::Validation(
                "Subject cannot exceed 200 characters".to_string()
            ));
        }

        // Verify customer exists if provided
        if let Some(customer_id) = customer_id {
            customers::table
                .find(customer_id)
                .first::<Customer>(conn)?;
        }

        // Verify lead exists if provided
        if let Some(lead_id) = lead_id {
            leads::table
                .find(lead_id)
                .first::<Lead>(conn)?;
        }

        // Verify assigned employee exists if provided
        if let Some(assigned_to) = assigned_to {
            employees::table
                .find(assigned_to)
                .first::<Employee>(conn)?;
        }

        // Create new activity
        let new_activity = NewActivity {
            customer_id,
            lead_id,
            deal_id,
            activity_type: activity_type.to_string(),
            subject: subject.to_string(),
            description: description.map(|s| s.to_string()),
            activity_date,
            duration_minutes,
            outcome: None,
            assigned_to,
            completed: false,
        };

        diesel::insert_into(activities::table)
            .values(&new_activity)
            .execute(conn)?;

        // Get the inserted activity by searching for the most recent activity with matching criteria
        activities::table
            .filter(activities::dsl::subject.eq(&new_activity.subject))
            .filter(activities::dsl::activity_type.eq(&new_activity.activity_type))
            .filter(activities::dsl::assigned_to.eq(new_activity.assigned_to))
            .filter(activities::dsl::customer_id.eq(new_activity.customer_id))
            .filter(activities::dsl::lead_id.eq(new_activity.lead_id))
            .order(activities::dsl::created_at.desc())
            .first::<Activity>(conn)
            .map_err(Into::into)
    }

    pub fn get_activity_by_id(conn: &mut DatabaseConnection, activity_id: i32) -> Result<Option<Activity>> {
        activities::table
            .find(activity_id)
            .first::<Activity>(conn)
            .optional()
            .map_err(Into::into)
    }

    pub fn get_activity_with_details(conn: &mut DatabaseConnection, activity_id: i32) -> Result<Option<ActivityWithDetails>> {
        let activity = Self::get_activity_by_id(conn, activity_id)?;

        if let Some(activity) = activity {
            // Get customer info if available
            let customer = if let Some(customer_id) = activity.customer_id {
                customers::table
                    .find(customer_id)
                    .first::<Customer>(conn)
                    .optional()?
            } else {
                None
            };

            // Get lead info if available
            let lead = if let Some(lead_id) = activity.lead_id {
                leads::table
                    .find(lead_id)
                    .first::<Lead>(conn)
                    .optional()?
            } else {
                None
            };

            // Get assigned employee name
            let assigned_employee = if let Some(assigned_id) = activity.assigned_to {
                employees::table
                    .find(assigned_id)
                    .select(employees::name)
                    .first::<String>(conn)?
            } else {
                "Unassigned".to_string()
            };

            Ok(Some(ActivityWithDetails {
                activity,
                customer,
                lead,
                assigned_employee,
            }))
        } else {
            Ok(None)
        }
    }

    pub fn list_activities(
        conn: &mut DatabaseConnection,
        filters: &FilterOptions,
        pagination: &PaginationParams,
    ) -> Result<PaginatedResult<ActivityWithDetails>> {
        let mut query = activities::table
            .left_join(customers::table.on(customers::dsl::id.eq(activities::dsl::customer_id.nullable())))
            .left_join(leads::table.on(leads::dsl::id.eq(activities::dsl::lead_id.nullable())))
            .left_join(employees::table.on(employees::dsl::id.eq(activities::dsl::assigned_to.nullable())))
            .select((
                Activity::as_select(),
                customers::all_columns.nullable(),
                leads::all_columns.nullable(),
                employees::name.nullable(),
            ))
            .into_boxed();

        // Apply filters
        if let Some(search) = &filters.search {
            query = query.filter(
                activities::dsl::subject.like(format!("%{}%", search))
                    .or(activities::dsl::description.like(format!("%{}%", search)))
                    .or(customers::dsl::name.like(format!("%{}%", search)))
                    .or(leads::dsl::title.like(format!("%{}%", search)))
            );
        }

        if let Some(status_filter) = &filters.status {
            let completed = status_filter == "completed";
            query = query.filter(activities::dsl::completed.eq(completed));
        }

        if let Some(type_filter) = &filters.filter_type {
            query = query.filter(activities::dsl::activity_type.eq(type_filter));
        }

        // Priority field does not exist in activities schema - removed

        if let Some(assigned_to) = filters.assigned_to {
            query = query.filter(activities::dsl::assigned_to.eq(assigned_to));
        }

        if let Some(date_from) = filters.date_from {
            let datetime_from = date_from.and_hms_opt(0, 0, 0).unwrap();
            query = query.filter(activities::dsl::activity_date.ge(datetime_from));
        }

        if let Some(date_to) = filters.date_to {
            let datetime_to = date_to.and_hms_opt(23, 59, 59).unwrap();
            query = query.filter(activities::dsl::activity_date.le(datetime_to));
        }

        // Apply sorting
        query = match filters.sort_by.as_deref() {
            Some("title") => {
                if filters.sort_desc {
                    query.order(activities::dsl::subject.desc())
                } else {
                    query.order(activities::dsl::subject.asc())
                }
            }
            Some("type") => {
                if filters.sort_desc {
                    query.order(activities::dsl::activity_type.desc())
                } else {
                    query.order(activities::dsl::activity_type.asc())
                }
            }
            Some("activity_date") => {
                if filters.sort_desc {
                    query.order(activities::dsl::activity_date.desc())
                } else {
                    query.order(activities::dsl::activity_date.asc())
                }
            }
            Some("created_at") => {
                if filters.sort_desc {
                    query.order(activities::dsl::created_at.desc())
                } else {
                    query.order(activities::dsl::created_at.asc())
                }
            }
            _ => query.order(activities::dsl::created_at.desc()),
        };

        let results: Vec<(Activity, Option<Customer>, Option<Lead>, Option<String>)> = query
            .offset(pagination.offset())
            .limit(pagination.limit())
            .load(conn)?;

        let total_items = activities::table.count().get_result::<i64>(conn)?;

        let activities_with_details: Vec<ActivityWithDetails> = results
            .into_iter()
            .map(|(activity, customer, lead, assigned_employee)| ActivityWithDetails {
                activity,
                customer,
                lead,
                assigned_employee: assigned_employee.unwrap_or_else(|| "Unassigned".to_string()),
            })
            .collect();

        Ok(PaginatedResult::new(activities_with_details, pagination, total_items))
    }

    pub fn update_activity(
        conn: &mut DatabaseConnection,
        activity_id: i32,
        subject: Option<&str>,
        description: Option<Option<&str>>,
        activity_date: Option<NaiveDateTime>,
        duration_minutes: Option<Option<i32>>,
        assigned_to: Option<Option<i32>>,
        completed: Option<bool>,
    ) -> Result<Activity> {
        // Check if activity exists
        let _activity = Self::get_activity_by_id(conn, activity_id)?
            .ok_or_else(|| crate::core::error::CLIERPError::NotFound(
                format!("Activity with ID {} not found", activity_id)
            ))?;

        // Validate input
        if let Some(subject_val) = subject {
            validate_required_string(subject_val, "subject")?;
            if subject_val.len() < 2 {
                return Err(crate::core::error::CLIERPError::Validation(
                    "Subject must be at least 2 characters long".to_string()
                ));
            }
            if subject_val.len() > 200 {
                return Err(crate::core::error::CLIERPError::Validation(
                    "Subject cannot exceed 200 characters".to_string()
                ));
            }
        }

        // Verify assigned employee exists if provided
        if let Some(assigned_to_val) = assigned_to {
            if let Some(employee_id) = assigned_to_val {
                employees::table
                    .find(employee_id)
                    .first::<Employee>(conn)?;
            }
        }

        // Build update queries - apply updates individually for simplicity
        use crate::database::schema::activities::dsl;

        if let Some(subject_val) = subject {
            diesel::update(activities::table.find(activity_id))
                .set(dsl::subject.eq(subject_val))
                .execute(conn)?;
        }

        if let Some(desc_val) = description {
            diesel::update(activities::table.find(activity_id))
                .set(dsl::description.eq(desc_val.map(|s| s.to_string())))
                .execute(conn)?;
        }

        if let Some(date_val) = activity_date {
            diesel::update(activities::table.find(activity_id))
                .set(dsl::activity_date.eq(date_val))
                .execute(conn)?;
        }

        if let Some(duration_val) = duration_minutes {
            diesel::update(activities::table.find(activity_id))
                .set(dsl::duration_minutes.eq(duration_val))
                .execute(conn)?;
        }

        if let Some(assigned_val) = assigned_to {
            diesel::update(activities::table.find(activity_id))
                .set(dsl::assigned_to.eq(assigned_val))
                .execute(conn)?;
        }

        if let Some(completed_val) = completed {
            diesel::update(activities::table.find(activity_id))
                .set(dsl::completed.eq(completed_val))
                .execute(conn)?;
        }

        // Always update the updated_at timestamp
        diesel::update(activities::table.find(activity_id))
            .set(dsl::updated_at.eq(Utc::now().naive_utc()))
            .execute(conn)?;

        // Get the updated activity
        activities::table
            .find(activity_id)
            .first::<Activity>(conn)
            .map_err(Into::into)
    }

    pub fn complete_activity(
        conn: &mut DatabaseConnection,
        activity_id: i32,
        outcome: Option<&str>,
    ) -> Result<Activity> {
        diesel::update(activities::table.find(activity_id))
            .set((
                activities::dsl::completed.eq(true),
                activities::dsl::outcome.eq(outcome.map(|s| s.to_string())),
                activities::dsl::updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(conn)?;

        // Get the updated activity
        activities::table
            .find(activity_id)
            .first::<Activity>(conn)
            .map_err(Into::into)
    }

    pub fn reopen_activity(
        conn: &mut DatabaseConnection,
        activity_id: i32,
    ) -> Result<Activity> {
        diesel::update(activities::table.find(activity_id))
            .set((
                activities::dsl::completed.eq(false),
                activities::dsl::outcome.eq(None::<Option<String>>),
                activities::dsl::updated_at.eq(Utc::now().naive_utc()),
            ))
            .execute(conn)?;

        // Get the updated activity
        activities::table
            .find(activity_id)
            .first::<Activity>(conn)
            .map_err(Into::into)
    }

    pub fn delete_activity(conn: &mut DatabaseConnection, activity_id: i32) -> Result<bool> {
        let deleted_rows = diesel::delete(activities::table.find(activity_id))
            .execute(conn)?;

        Ok(deleted_rows > 0)
    }

    pub fn get_activities_by_customer(
        conn: &mut DatabaseConnection,
        customer_id: i32,
    ) -> Result<Vec<ActivityWithDetails>> {
        let results: Vec<(Activity, Option<Customer>, Option<Lead>, String)> = activities::table
            .left_join(customers::table)
            .left_join(leads::table)
            .inner_join(employees::table.on(employees::dsl::id.eq(activities::dsl::assigned_to)))
            .filter(activities::dsl::customer_id.eq(customer_id))
            .select((
                Activity::as_select(),
                customers::all_columns.nullable(),
                leads::all_columns.nullable(),
                employees::name,
            ))
            .order(activities::dsl::created_at.desc())
            .load(conn)?;

        let activities_with_details: Vec<ActivityWithDetails> = results
            .into_iter()
            .map(|(activity, customer, lead, assigned_employee)| ActivityWithDetails {
                activity,
                customer,
                lead,
                assigned_employee,
            })
            .collect();

        Ok(activities_with_details)
    }

    pub fn get_activities_by_lead(
        conn: &mut DatabaseConnection,
        lead_id: i32,
    ) -> Result<Vec<ActivityWithDetails>> {
        let results: Vec<(Activity, Option<Customer>, Option<Lead>, String)> = activities::table
            .left_join(customers::table)
            .left_join(leads::table)
            .inner_join(employees::table.on(employees::dsl::id.eq(activities::dsl::assigned_to)))
            .filter(activities::dsl::lead_id.eq(lead_id))
            .select((
                Activity::as_select(),
                customers::all_columns.nullable(),
                leads::all_columns.nullable(),
                employees::name,
            ))
            .order(activities::dsl::created_at.desc())
            .load(conn)?;

        let activities_with_details: Vec<ActivityWithDetails> = results
            .into_iter()
            .map(|(activity, customer, lead, assigned_employee)| ActivityWithDetails {
                activity,
                customer,
                lead,
                assigned_employee,
            })
            .collect();

        Ok(activities_with_details)
    }

    pub fn get_activities_by_employee(
        conn: &mut DatabaseConnection,
        employee_id: i32,
    ) -> Result<Vec<ActivityWithDetails>> {
        let results: Vec<(Activity, Option<Customer>, Option<Lead>, String)> = activities::table
            .left_join(customers::table)
            .left_join(leads::table)
            .inner_join(employees::table.on(employees::dsl::id.eq(activities::dsl::assigned_to)))
            .filter(activities::dsl::assigned_to.eq(employee_id))
            .select((
                Activity::as_select(),
                customers::all_columns.nullable(),
                leads::all_columns.nullable(),
                employees::name,
            ))
            .order(activities::dsl::activity_date.asc().nulls_last())
            .load(conn)?;

        let activities_with_details: Vec<ActivityWithDetails> = results
            .into_iter()
            .map(|(activity, customer, lead, assigned_employee)| ActivityWithDetails {
                activity,
                customer,
                lead,
                assigned_employee,
            })
            .collect();

        Ok(activities_with_details)
    }

    pub fn get_overdue_activities(conn: &mut DatabaseConnection) -> Result<Vec<ActivityWithDetails>> {
        let now = Utc::now().naive_utc();

        let results: Vec<(Activity, Option<Customer>, Option<Lead>, String)> = activities::table
            .left_join(customers::table)
            .left_join(leads::table)
            .inner_join(employees::table.on(employees::dsl::id.eq(activities::dsl::assigned_to)))
            .filter(activities::dsl::completed.eq(false))
            .filter(activities::dsl::activity_date.lt(now))
            .select((
                Activity::as_select(),
                customers::all_columns.nullable(),
                leads::all_columns.nullable(),
                employees::name,
            ))
            .order(activities::dsl::activity_date.asc())
            .load(conn)?;

        let activities_with_details: Vec<ActivityWithDetails> = results
            .into_iter()
            .map(|(activity, customer, lead, assigned_employee)| ActivityWithDetails {
                activity,
                customer,
                lead,
                assigned_employee,
            })
            .collect();

        Ok(activities_with_details)
    }

    pub fn get_activity_statistics(conn: &mut DatabaseConnection) -> Result<ActivityStatistics> {
        // Total activities count
        let total_activities = activities::table
            .count()
            .get_result::<i64>(conn)?;

        // Pending activities count
        let pending_activities = activities::table
            .filter(activities::dsl::completed.eq(false))
            .count()
            .get_result::<i64>(conn)?;

        // Completed activities count
        let completed_activities = activities::table
            .filter(activities::dsl::completed.eq(true))
            .count()
            .get_result::<i64>(conn)?;

        // Overdue activities count
        let now = Utc::now().naive_utc();
        let overdue_activities = activities::table
            .filter(activities::dsl::completed.eq(false))
            .filter(activities::dsl::activity_date.lt(now))
            .count()
            .get_result::<i64>(conn)?;

        // Activities by type
        use crate::database::ActivityType as AT;
        let call_activities = activities::table
            .filter(activities::dsl::activity_type.eq(AT::Call.to_string()))
            .count()
            .get_result::<i64>(conn)?;

        let email_activities = activities::table
            .filter(activities::dsl::activity_type.eq(AT::Email.to_string()))
            .count()
            .get_result::<i64>(conn)?;

        let meeting_activities = activities::table
            .filter(activities::dsl::activity_type.eq(AT::Meeting.to_string()))
            .count()
            .get_result::<i64>(conn)?;

        let task_activities = activities::table
            .filter(activities::dsl::activity_type.eq(AT::Task.to_string()))
            .count()
            .get_result::<i64>(conn)?;

        Ok(ActivityStatistics {
            total_activities,
            pending_activities,
            completed_activities,
            overdue_activities,
            call_activities,
            email_activities,
            meeting_activities,
            task_activities,
        })
    }
}

#[derive(Debug, serde::Serialize)]
pub struct ActivityWithDetails {
    pub activity: Activity,
    pub customer: Option<Customer>,
    pub lead: Option<Lead>,
    pub assigned_employee: String,
}

#[derive(Debug, serde::Serialize)]
pub struct ActivityStatistics {
    pub total_activities: i64,
    pub pending_activities: i64,
    pub completed_activities: i64,
    pub overdue_activities: i64,
    pub call_activities: i64,
    pub email_activities: i64,
    pub meeting_activities: i64,
    pub task_activities: i64,
}