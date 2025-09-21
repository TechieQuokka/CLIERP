//! CLIERP - CLI-based ERP System
//!
//! A comprehensive Enterprise Resource Planning system designed for CLI environments.
//! Built with Rust for performance, safety, and reliability.

pub mod cli;
pub mod config;
pub mod core;
pub mod database;
pub mod modules;
pub mod utils;

// Re-export main components for easier access
pub use cli::app::CLIApp;
pub use core::{error::CLIERPError, result::CLIERPResult};
pub use database::connection::{DatabaseConnection, DatabaseManager};

/// Application version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Application name
pub const APP_NAME: &str = "CLIERP";

/// Application description
pub const APP_DESCRIPTION: &str = "CLI-based ERP System";
