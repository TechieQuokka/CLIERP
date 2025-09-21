use std::fs::File;
use std::io::Write;
use std::path::Path;
use crate::core::result::CLIERPResult;
use crate::core::error::CLIERPError;
use serde::Serialize;

#[derive(Default)]
pub struct ExportService;

impl ExportService {
    pub fn new() -> Self {
        Self
    }

    /// Export data to CSV format
    pub fn export_to_csv<T>(&self, data: &[T], headers: &[&str], file_path: &str) -> CLIERPResult<()>
    where
        T: CsvSerializable,
    {
        let mut file = File::create(file_path)
            .map_err(|e| CLIERPError::IoError(format!("Failed to create file {}: {}", file_path, e)))?;

        // Write headers
        writeln!(file, "{}", headers.join(","))
            .map_err(|e| CLIERPError::IoError(format!("Failed to write headers: {}", e)))?;

        // Write data rows
        for item in data {
            let row = item.to_csv_row();
            writeln!(file, "{}", row.join(","))
                .map_err(|e| CLIERPError::IoError(format!("Failed to write data row: {}", e)))?;
        }

        Ok(())
    }

    /// Export data to JSON format
    pub fn export_to_json<T>(&self, data: &[T], file_path: &str) -> CLIERPResult<()>
    where
        T: Serialize,
    {
        let json_string = serde_json::to_string_pretty(data)
            .map_err(|e| CLIERPError::SerializationError(format!("Failed to serialize to JSON: {}", e)))?;

        std::fs::write(file_path, json_string)
            .map_err(|e| CLIERPError::IoError(format!("Failed to write JSON file {}: {}", file_path, e)))?;

        Ok(())
    }

    /// Get file extension from format
    pub fn get_file_extension(format: &str) -> &str {
        match format.to_lowercase().as_str() {
            "csv" => "csv",
            "json" => "json",
            _ => "txt",
        }
    }

    /// Generate default filename with timestamp
    pub fn generate_filename(prefix: &str, format: &str) -> String {
        let now = chrono::Utc::now();
        let timestamp = now.format("%Y%m%d_%H%M%S");
        let extension = Self::get_file_extension(format);
        format!("{}_{}.{}", prefix, timestamp, extension)
    }

    /// Validate file path and create directory if needed
    pub fn prepare_file_path(file_path: &str) -> CLIERPResult<()> {
        if let Some(parent) = Path::new(file_path).parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)
                    .map_err(|e| CLIERPError::IoError(format!("Failed to create directory: {}", e)))?;
            }
        }
        Ok(())
    }
}

/// Trait for types that can be serialized to CSV
pub trait CsvSerializable {
    fn to_csv_row(&self) -> Vec<String>;
}

/// Helper function to escape CSV values
pub fn escape_csv_value(value: &str) -> String {
    if value.contains(',') || value.contains('"') || value.contains('\n') {
        format!("\"{}\"", value.replace('"', "\"\""))
    } else {
        value.to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_escape_csv_value() {
        assert_eq!(escape_csv_value("simple"), "simple");
        assert_eq!(escape_csv_value("with,comma"), "\"with,comma\"");
        assert_eq!(escape_csv_value("with\"quote"), "\"with\"\"quote\"");
    }

    #[test]
    fn test_get_file_extension() {
        assert_eq!(ExportService::get_file_extension("csv"), "csv");
        assert_eq!(ExportService::get_file_extension("CSV"), "csv");
        assert_eq!(ExportService::get_file_extension("json"), "json");
        assert_eq!(ExportService::get_file_extension("unknown"), "txt");
    }

    #[test]
    fn test_generate_filename() {
        let filename = ExportService::generate_filename("employees", "csv");
        assert!(filename.starts_with("employees_"));
        assert!(filename.ends_with(".csv"));
    }
}