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
    format!("₩{}", format_number_with_commas(amount))
}

/// Format number with thousands separators
pub fn format_number_with_commas(n: i32) -> String {
    let s = n.to_string();
    let mut result = String::new();
    let chars: Vec<char> = s.chars().collect();

    for (i, &c) in chars.iter().enumerate() {
        if i > 0 && (chars.len() - i) % 3 == 0 {
            result.push(',');
        }
        result.push(c);
    }

    result
}

/// Format percentage
pub fn format_percentage(value: f64) -> String {
    format!("{:.1}%", value)
}

/// Format datetime to readable string
pub fn format_datetime(datetime: &chrono::NaiveDateTime) -> String {
    datetime.format("%Y-%m-%d %H:%M:%S").to_string()
}

/// Format date to readable string
pub fn format_date(date: &chrono::NaiveDate) -> String {
    date.format("%Y-%m-%d").to_string()
}

/// Format table from headers and rows
pub fn format_table(headers: &[&str], rows: &[Vec<String>]) {
    if rows.is_empty() {
        println!("No data available");
        return;
    }

    // Calculate column widths
    let mut column_widths = headers.iter().map(|h| h.len()).collect::<Vec<_>>();

    for row in rows {
        for (i, cell) in row.iter().enumerate() {
            if i < column_widths.len() {
                column_widths[i] = column_widths[i].max(cell.len());
            }
        }
    }

    // Print header
    print!("│");
    for (i, header) in headers.iter().enumerate() {
        print!(" {:width$} │", header, width = column_widths[i]);
    }
    println!();

    // Print separator
    print!("├");
    for width in &column_widths {
        print!("─{:─<width$}─┼", "", width = width);
    }
    println!("┤");

    // Print rows
    for row in rows {
        print!("│");
        for (i, cell) in row.iter().enumerate() {
            if i < column_widths.len() {
                print!(" {:width$} │", cell, width = column_widths[i]);
            }
        }
        println!();
    }
}
