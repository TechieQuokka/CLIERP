use super::error::CLIERPError;

/// A specialized Result type for CLIERP operations.
pub type CLIERPResult<T> = Result<T, CLIERPError>;