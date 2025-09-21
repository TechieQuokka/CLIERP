use chrono::NaiveDate;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub from_date: Option<NaiveDate>,
    pub to_date: Option<NaiveDate>,
}

impl DateRange {
    pub fn new(from_date: Option<NaiveDate>, to_date: Option<NaiveDate>) -> Self {
        Self { from_date, to_date }
    }

    pub fn is_empty(&self) -> bool {
        self.from_date.is_none() && self.to_date.is_none()
    }

    pub fn contains(&self, date: &NaiveDate) -> bool {
        let after_from = self.from_date.map_or(true, |from| date >= &from);
        let before_to = self.to_date.map_or(true, |to| date <= &to);
        after_from && before_to
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQuery {
    pub query: Option<String>,
    pub fields: Vec<String>,
}

impl SearchQuery {
    pub fn new(query: Option<String>, fields: Vec<String>) -> Self {
        Self { query, fields }
    }

    pub fn is_empty(&self) -> bool {
        self.query.is_none() || self.query.as_ref().unwrap().trim().is_empty()
    }

    pub fn query(&self) -> Option<&str> {
        self.query
            .as_ref()
            .map(|q| q.trim())
            .filter(|q| !q.is_empty())
    }

    pub fn matches(&self, text: &str) -> bool {
        if let Some(query) = self.query() {
            text.to_lowercase().contains(&query.to_lowercase())
        } else {
            true
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SortOrder {
    pub field: String,
    pub direction: SortDirection,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SortDirection {
    Asc,
    Desc,
}

impl std::fmt::Display for SortDirection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SortDirection::Asc => write!(f, "asc"),
            SortDirection::Desc => write!(f, "desc"),
        }
    }
}

impl SortOrder {
    pub fn new(field: String, direction: SortDirection) -> Self {
        Self { field, direction }
    }

    pub fn asc(field: String) -> Self {
        Self::new(field, SortDirection::Asc)
    }

    pub fn desc(field: String) -> Self {
        Self::new(field, SortDirection::Desc)
    }
}

impl Default for SortOrder {
    fn default() -> Self {
        Self::asc("id".to_string())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterParams {
    pub search: SearchQuery,
    pub date_range: DateRange,
    pub sort: SortOrder,
    pub status_filter: Option<String>,
    pub type_filter: Option<String>,
}

impl FilterParams {
    pub fn new() -> Self {
        Self {
            search: SearchQuery::new(None, vec![]),
            date_range: DateRange::new(None, None),
            sort: SortOrder::default(),
            status_filter: None,
            type_filter: None,
        }
    }

    pub fn with_search(mut self, query: Option<String>, fields: Vec<String>) -> Self {
        self.search = SearchQuery::new(query, fields);
        self
    }

    pub fn with_date_range(
        mut self,
        from_date: Option<NaiveDate>,
        to_date: Option<NaiveDate>,
    ) -> Self {
        self.date_range = DateRange::new(from_date, to_date);
        self
    }

    pub fn with_sort(mut self, field: String, direction: SortDirection) -> Self {
        self.sort = SortOrder::new(field, direction);
        self
    }

    pub fn with_status_filter(mut self, status: Option<String>) -> Self {
        self.status_filter = status;
        self
    }

    pub fn with_type_filter(mut self, type_filter: Option<String>) -> Self {
        self.type_filter = type_filter;
        self
    }
}

impl Default for FilterParams {
    fn default() -> Self {
        Self::new()
    }
}

pub trait Filterable<T> {
    fn apply_filters(self, filters: &FilterParams) -> Vec<T>;
}

// Helper functions for common filtering operations
pub fn filter_by_status<T>(
    items: Vec<T>,
    status: &Option<String>,
    get_status: impl Fn(&T) -> &str,
) -> Vec<T> {
    if let Some(status_filter) = status {
        items
            .into_iter()
            .filter(|item| get_status(item) == status_filter)
            .collect()
    } else {
        items
    }
}

pub fn filter_by_search<T>(
    items: Vec<T>,
    search: &SearchQuery,
    get_searchable_text: impl Fn(&T) -> String,
) -> Vec<T> {
    if search.is_empty() {
        items
    } else {
        items
            .into_iter()
            .filter(|item| {
                let text = get_searchable_text(item);
                search.matches(&text)
            })
            .collect()
    }
}

pub fn filter_by_date_range<T>(
    items: Vec<T>,
    date_range: &DateRange,
    get_date: impl Fn(&T) -> NaiveDate,
) -> Vec<T> {
    if date_range.is_empty() {
        items
    } else {
        items
            .into_iter()
            .filter(|item| {
                let date = get_date(item);
                date_range.contains(&date)
            })
            .collect()
    }
}

pub fn sort_items<T>(
    mut items: Vec<T>,
    sort: &SortOrder,
    compare: impl Fn(&T, &T, &SortDirection) -> std::cmp::Ordering,
) -> Vec<T> {
    items.sort_by(|a, b| compare(a, b, &sort.direction));
    items
}

// Extended FilterOptions for CRM usage
#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct FilterOptions {
    pub search: Option<String>,
    pub status: Option<String>,
    pub priority: Option<String>,
    pub filter_type: Option<String>,
    pub assigned_to: Option<i32>,
    pub date_from: Option<NaiveDate>,
    pub date_to: Option<NaiveDate>,
    pub sort_by: Option<String>,
    pub sort_desc: bool,
}
