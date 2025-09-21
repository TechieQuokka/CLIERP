use crate::core::{error::CLIERPError, result::CLIERPResult};
use once_cell::sync::Lazy;
use regex::Regex;

// Email validation regex
static EMAIL_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap());

// Phone validation regex (Korean format)
static PHONE_REGEX: Lazy<Regex> =
    Lazy::new(|| Regex::new(r"^01[0-9]-?[0-9]{4}-?[0-9]{4}$").unwrap());

/// Validate email address
pub fn validate_email(email: &str) -> CLIERPResult<()> {
    if EMAIL_REGEX.is_match(email) {
        Ok(())
    } else {
        Err(CLIERPError::Validation(format!(
            "Invalid email address: {}",
            email
        )))
    }
}

/// Validate phone number (Korean format)
pub fn validate_phone(phone: &str) -> CLIERPResult<()> {
    if PHONE_REGEX.is_match(phone) {
        Ok(())
    } else {
        Err(CLIERPError::Validation(format!(
            "Invalid phone number: {}",
            phone
        )))
    }
}

/// Validate employee code format
pub fn validate_employee_code(code: &str) -> CLIERPResult<()> {
    if code.len() < 3 || code.len() > 20 {
        return Err(CLIERPError::Validation(
            "Employee code must be between 3 and 20 characters".to_string(),
        ));
    }

    if !code.chars().all(|c| c.is_alphanumeric()) {
        return Err(CLIERPError::Validation(
            "Employee code must contain only alphanumeric characters".to_string(),
        ));
    }

    Ok(())
}

/// Validate salary amount
pub fn validate_salary(salary: i32) -> CLIERPResult<()> {
    if salary < 0 {
        return Err(CLIERPError::Validation(
            "Salary cannot be negative".to_string(),
        ));
    }

    if salary > 1_000_000_000 {
        return Err(CLIERPError::Validation(
            "Salary exceeds maximum allowed amount".to_string(),
        ));
    }

    Ok(())
}

/// Validate password strength
pub fn validate_password(password: &str) -> CLIERPResult<()> {
    if password.len() < 8 {
        return Err(CLIERPError::Validation(
            "Password must be at least 8 characters long".to_string(),
        ));
    }

    let has_letter = password.chars().any(|c| c.is_alphabetic());
    let has_digit = password.chars().any(|c| c.is_numeric());

    if !has_letter || !has_digit {
        return Err(CLIERPError::Validation(
            "Password must contain at least one letter and one digit".to_string(),
        ));
    }

    Ok(())
}

/// Validate department name
pub fn validate_department_name(name: &str) -> CLIERPResult<()> {
    if name.trim().is_empty() {
        return Err(CLIERPError::Validation(
            "Department name cannot be empty".to_string(),
        ));
    }

    if name.len() > 100 {
        return Err(CLIERPError::Validation(
            "Department name cannot exceed 100 characters".to_string(),
        ));
    }

    Ok(())
}

/// Validate required string field
pub fn validate_required_string(value: &str, field_name: &str) -> CLIERPResult<()> {
    if value.trim().is_empty() {
        return Err(CLIERPError::Validation(
            format!("{} cannot be empty", field_name),
        ));
    }
    Ok(())
}

/// Validation result type
pub type ValidationResult<T> = CLIERPResult<T>;
