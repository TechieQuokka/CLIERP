use colored::*;
use tabled::{Table, Tabled};

/// Format success message with green color
pub fn success(message: &str) -> String {
    format!("✓ {}", message.green())
}

/// Format error message with red color
pub fn error(message: &str) -> String {
    format!("✗ {}", message.red())
}

/// Format warning message with yellow color
pub fn warning(message: &str) -> String {
    format!("⚠ {}", message.yellow())
}

/// Format info message with blue color
pub fn info(message: &str) -> String {
    format!("ℹ {}", message.blue())
}

/// Format header with bold text
pub fn header(message: &str) -> String {
    message.bold().to_string()
}

/// Create a table from data that implements Tabled trait
pub fn create_table<T: Tabled>(data: Vec<T>) -> String {
    if data.is_empty() {
        return "No data available".to_string();
    }

    Table::new(data).to_string()
}

/// Format currency amount
pub fn format_currency(amount: i32) -> String {
    format!("₩{}", amount)
}

/// Format percentage
pub fn format_percentage(value: f64) -> String {
    format!("{:.1}%", value)
}