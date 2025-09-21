use thiserror::Error;

#[derive(Error, Debug)]
pub enum CLIERPError {
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("Database connection error: {0}")]
    DatabaseConnection(#[from] diesel::ConnectionError),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Authentication error: {0}")]
    Authentication(String),

    #[error("Authorization error: {0}")]
    Authorization(String),

    #[error("Configuration error: {0}")]
    Configuration(#[from] config::ConfigError),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("JWT error: {0}")]
    Jwt(#[from] jsonwebtoken::errors::Error),

    #[error("BCrypt error: {0}")]
    BCrypt(#[from] bcrypt::BcryptError),

    #[error("UUID error: {0}")]
    Uuid(#[from] uuid::Error),

    #[error("CLI parsing error: {0}")]
    CLI(#[from] clap::Error),

    #[error("Validation error: {0}")]
    Validation(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Already exists: {0}")]
    AlreadyExists(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Internal error: {0}")]
    Internal(String),
}