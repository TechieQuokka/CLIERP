use crate::core::{result::CLIERPResult, error::CLIERPError};
use uuid::Uuid;
use std::time::{SystemTime, UNIX_EPOCH};

/// Generate a unique identifier
pub fn generate_id() -> String {
    Uuid::new_v4().to_string()
}

/// Generate a short unique code (8 characters)
pub fn generate_short_id() -> String {
    Uuid::new_v4().to_string().replace("-", "")[..8].to_uppercase()
}

/// Generate a timestamp-based ID
pub fn generate_timestamp_id() -> CLIERPResult<String> {
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| CLIERPError::Internal(format!("Time error: {}", e)))?
        .as_secs();

    Ok(format!("{}{}", timestamp, generate_short_id()))
}

/// Generate employee code with prefix
pub fn generate_employee_code(prefix: &str) -> String {
    format!("{}{}", prefix, generate_short_id())
}

/// Mask sensitive data for logging
pub fn mask_sensitive(data: &str, show_chars: usize) -> String {
    if data.len() <= show_chars {
        "*".repeat(data.len())
    } else {
        let masked_len = data.len() - show_chars;
        format!("{}{}", "*".repeat(masked_len), &data[masked_len..])
    }
}

/// Generate secure random password
pub fn generate_password(length: usize) -> String {
    use rand::Rng;
    const CHARSET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ\
                            abcdefghijklmnopqrstuvwxyz\
                            0123456789\
                            !@#$%^&*";

    let mut rng = rand::thread_rng();
    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}