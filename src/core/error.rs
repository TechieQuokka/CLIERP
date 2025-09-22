use thiserror::Error;

#[derive(Error, Debug)]
pub enum CLIERPError {
    #[error("Database error: {0}")]
    Database(#[from] diesel::result::Error),

    #[error("Database connection error: {0}")]
    DatabaseConnection(#[from] diesel::ConnectionError),

    #[error("Migration error: {0}")]
    Migration(String),

    #[error("Concurrency error: {0}")]
    ConcurrencyError(String),

    #[error("Business rule violation: {0}")]
    BusinessRuleViolation(String),

    #[error("Insufficient privileges: {0}")]
    InsufficientPrivileges(String),

    #[error("Data integrity error: {0}")]
    DataIntegrity(String),

    #[error("Transaction error: {0}")]
    Transaction(String),

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

    #[error("Authentication required")]
    AuthenticationRequired,

    #[error("Validation error: {0}")]
    ValidationError(String),

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

    #[error("Business logic error: {0}")]
    BusinessLogic(String),

    #[error("Database error: {0}")]
    DatabaseError(String),

    #[error("IO error: {0}")]
    IoError(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("Permission denied: {0}")]
    PermissionDenied(String),

    #[error("Internal error: {0}")]
    Internal(String),

    #[error("Invalid input: {0}")]
    InvalidInput(String),
}
