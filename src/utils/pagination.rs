use serde::{Deserialize, Serialize};
use diesel::prelude::*;
use diesel::query_dsl::LoadQuery;
use diesel::sqlite::Sqlite;
use crate::core::result::CLIERPResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationParams {
    pub page: Option<i64>,
    pub per_page: Option<i64>,
}

impl Default for PaginationParams {
    fn default() -> Self {
        Self {
            page: Some(1),
            per_page: Some(50),
        }
    }
}

impl PaginationParams {
    pub fn new(page: usize, per_page: i64) -> Self {
        Self {
            page: Some(page as i64),
            per_page: Some(per_page)
        }
    }

    pub fn new_with_options(page: Option<i64>, per_page: Option<i64>) -> Self {
        Self { page, per_page }
    }

    pub fn page(&self) -> i64 {
        self.page.unwrap_or(1).max(1)
    }

    pub fn per_page(&self) -> i64 {
        self.per_page.unwrap_or(50).max(1).min(1000)
    }

    pub fn offset(&self) -> i64 {
        (self.page() - 1) * self.per_page()
    }

    pub fn limit(&self) -> i64 {
        self.per_page()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResult<T> {
    pub data: Vec<T>,
    pub pagination: PaginationInfo,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginationInfo {
    pub current_page: i64,
    pub per_page: i64,
    pub total_count: i64,
    pub total_pages: i64,
    pub has_next: bool,
    pub has_prev: bool,
}

impl PaginationInfo {
    pub fn new(current_page: i64, per_page: i64, total_count: i64) -> Self {
        let total_pages = if total_count > 0 {
            (total_count + per_page - 1) / per_page
        } else {
            0
        };

        Self {
            current_page,
            per_page,
            total_count,
            total_pages,
            has_next: current_page < total_pages,
            has_prev: current_page > 1,
        }
    }
}

impl<T> PaginatedResult<T> {
    pub fn new(data: Vec<T>, params: &PaginationParams, total_count: i64) -> Self {
        let pagination = PaginationInfo::new(params.page(), params.per_page(), total_count);

        Self { data, pagination }
    }
}

// Alias for backward compatibility
pub type PaginationResult<T> = PaginatedResult<T>;

impl<T> PaginationResult<T> {
    pub fn new_simple(data: Vec<T>, total_count: usize, params: &PaginationParams) -> Self {
        let pagination = PaginationInfo::new(params.page(), params.per_page(), total_count as i64);
        Self { data, pagination }
    }

    pub fn current_page(&self) -> i64 {
        self.pagination.current_page
    }

    pub fn total_pages(&self) -> i64 {
        self.pagination.total_pages
    }

    pub fn total_items(&self) -> i64 {
        self.pagination.total_count
    }
}

pub trait Paginate<T> {
    fn paginate(self, params: &PaginationParams) -> PaginatedResult<T>;
}

impl<T> Paginate<T> for Vec<T> {
    fn paginate(self, params: &PaginationParams) -> PaginatedResult<T> {
        let total_count = self.len() as i64;
        let offset = params.offset() as usize;
        let limit = params.limit() as usize;

        let data = if offset >= self.len() {
            Vec::new()
        } else {
            let end = (offset + limit).min(self.len());
            self.into_iter().skip(offset).take(end - offset).collect()
        };

        PaginatedResult::new(data, params, total_count)
    }
}

// Extension trait for paginating query results
pub trait PaginateResult<T> {
    fn paginate_result(
        self,
        params: &PaginationParams,
        conn: &mut SqliteConnection,
    ) -> CLIERPResult<PaginatedResult<T>>;
}

// Helper function to paginate any query that can be loaded
pub fn paginate_query<T, Q>(
    query: Q,
    params: &PaginationParams,
    conn: &mut SqliteConnection,
) -> CLIERPResult<PaginatedResult<T>>
where
    Q: LoadQuery<'static, SqliteConnection, T>,
{
    // Load all data to get total count (simplified approach)
    let all_data: Vec<T> = query.load(conn)?;
    let total_count = all_data.len() as i64;

    // Apply pagination in memory
    let offset = params.offset() as usize;
    let limit = params.limit() as usize;

    let data = if offset >= all_data.len() {
        Vec::new()
    } else {
        let end = (offset + limit).min(all_data.len());
        all_data.into_iter().skip(offset).take(end - offset).collect()
    };

    Ok(PaginatedResult::new(data, params, total_count))
}
