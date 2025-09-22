use chrono::{NaiveDate, NaiveDateTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use crate::core::result::CLIERPResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportConfig {
    pub title: String,
    pub description: Option<String>,
    pub date_range: Option<DateRange>,
    pub filters: HashMap<String, String>,
    pub format: ReportFormat,
    pub include_charts: bool,
    pub include_summary: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportFormat {
    Json,
    Csv,
    Html,
    Text,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateRange {
    pub start_date: NaiveDate,
    pub end_date: NaiveDate,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportResult {
    pub config: ReportConfig,
    pub generated_at: NaiveDateTime,
    pub data: ReportData,
    pub summary: Option<ReportSummary>,
    pub metadata: ReportMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ReportData {
    Table(TableData),
    Chart(ChartData),
    Mixed(Vec<ReportSection>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub headers: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub totals: Option<Vec<String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChartData {
    pub chart_type: ChartType,
    pub labels: Vec<String>,
    pub datasets: Vec<Dataset>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ChartType {
    Bar,
    Line,
    Pie,
    Area,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Dataset {
    pub label: String,
    pub data: Vec<f64>,
    pub color: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSection {
    pub title: String,
    pub section_type: SectionType,
    pub data: ReportData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SectionType {
    Summary,
    Detail,
    Chart,
    Analysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportSummary {
    pub key_metrics: HashMap<String, MetricValue>,
    pub insights: Vec<String>,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricValue {
    Number(f64),
    Currency(i32),
    Percentage(f64),
    Count(i64),
    Text(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportMetadata {
    pub total_records: i64,
    pub processing_time_ms: u64,
    pub filters_applied: Vec<String>,
    pub data_sources: Vec<String>,
}

pub trait ReportGenerator {
    fn generate_report(&self, config: ReportConfig) -> Result<ReportResult>;
    fn get_available_filters(&self) -> Vec<FilterDefinition>;
    fn get_report_info(&self) -> ReportInfo;
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterDefinition {
    pub name: String,
    pub label: String,
    pub filter_type: FilterType,
    pub required: bool,
    pub default_value: Option<String>,
    pub options: Option<Vec<FilterOption>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterType {
    Text,
    Date,
    Number,
    Select,
    MultiSelect,
    Boolean,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilterOption {
    pub value: String,
    pub label: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReportInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub supported_formats: Vec<ReportFormat>,
}

pub struct ReportEngine {
    generators: HashMap<String, Box<dyn ReportGenerator>>,
}

impl ReportEngine {
    pub fn new() -> Self {
        Self {
            generators: HashMap::new(),
        }
    }

    pub fn register_generator<T: ReportGenerator + 'static>(&mut self, id: String, generator: T) {
        self.generators.insert(id, Box::new(generator));
    }

    pub fn generate_report(&self, report_id: &str, config: ReportConfig) -> Result<ReportResult> {
        let generator = self.generators.get(report_id)
            .ok_or_else(|| crate::core::error::AppError::NotFound(
                format!("Report generator '{}' not found", report_id)
            ))?;

        generator.generate_report(config)
    }

    pub fn list_available_reports(&self) -> Vec<ReportInfo> {
        self.generators.values()
            .map(|generator| generator.get_report_info())
            .collect()
    }

    pub fn get_available_filters(&self, report_id: &str) -> Option<Vec<FilterDefinition>> {
        self.generators.get(report_id)
            .map(|generator| generator.get_available_filters())
    }
}

impl Default for ReportEngine {
    fn default() -> Self {
        Self::new()
    }
}

// Helper functions for report formatting
pub fn format_table_data(
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    totals: Option<Vec<String>>,
) -> String {
    let mut output = String::new();

    // Headers
    output.push_str(&format!("| {} |\n", headers.join(" | ")));
    output.push_str(&format!("|{}|\n", headers.iter().map(|h| "-".repeat(h.len() + 2)).collect::<Vec<_>>().join("|")));

    // Rows
    for row in rows {
        output.push_str(&format!("| {} |\n", row.join(" | ")));
    }

    // Totals
    if let Some(totals) = totals {
        output.push_str(&format!("|{}|\n", headers.iter().map(|h| "-".repeat(h.len() + 2)).collect::<Vec<_>>().join("|")));
        output.push_str(&format!("| {} |\n", totals.join(" | ")));
    }

    output
}

pub fn format_currency(amount: i32) -> String {
    format!("₩{:,}", amount)
}

pub fn format_percentage(value: f64) -> String {
    format!("{:.1}%", value)
}

pub fn format_metric_value(metric: &MetricValue) -> String {
    match metric {
        MetricValue::Number(n) => format!("{:.2}", n),
        MetricValue::Currency(c) => format_currency(*c),
        MetricValue::Percentage(p) => format_percentage(*p),
        MetricValue::Count(c) => format!("{:,}", c),
        MetricValue::Text(t) => t.clone(),
    }
}

// Chart data helpers
pub fn create_bar_chart(labels: Vec<String>, data: Vec<f64>, label: &str) -> ChartData {
    ChartData {
        chart_type: ChartType::Bar,
        labels,
        datasets: vec![Dataset {
            label: label.to_string(),
            data,
            color: Some("#3B82F6".to_string()),
        }],
    }
}

pub fn create_pie_chart(labels: Vec<String>, data: Vec<f64>) -> ChartData {
    ChartData {
        chart_type: ChartType::Pie,
        labels,
        datasets: vec![Dataset {
            label: "Distribution".to_string(),
            data,
            color: None,
        }],
    }
}

pub fn create_line_chart(labels: Vec<String>, datasets: Vec<Dataset>) -> ChartData {
    ChartData {
        chart_type: ChartType::Line,
        labels,
        datasets,
    }
}