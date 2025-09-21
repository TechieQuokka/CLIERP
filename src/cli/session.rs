use crate::core::{
    auth::{AuthService, AuthenticatedUser},
    config::CLIERPConfig,
    error::CLIERPError,
    result::CLIERPResult,
};
use crate::database::models::UserRole;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionData {
    pub token: String,
    pub username: String,
    pub role: String,
    pub employee_id: Option<i32>,
    pub expires_at: i64,
}

pub struct SessionManager {
    config: CLIERPConfig,
    session_file: PathBuf,
}

impl SessionManager {
    pub fn new(config: CLIERPConfig) -> Self {
        let mut session_file = std::env::temp_dir();
        session_file.push("clierp_session.json");

        Self {
            config,
            session_file,
        }
    }

    /// Save a session token
    pub fn save_session(&self, token: &str) -> CLIERPResult<()> {
        // Decode token to get user info
        let auth_service = AuthService::new(self.config.clone());
        let claims = auth_service.validate_token(token)?;

        let session_data = SessionData {
            token: token.to_string(),
            username: claims.username,
            role: claims.role,
            employee_id: None, // We would need to look this up from the database
            expires_at: claims.exp as i64,
        };

        let json =
            serde_json::to_string_pretty(&session_data).map_err(CLIERPError::Serialization)?;

        fs::write(&self.session_file, json).map_err(CLIERPError::Io)?;

        tracing::debug!("Session saved to: {:?}", self.session_file);
        Ok(())
    }

    /// Load the current session
    pub fn load_session(&self) -> CLIERPResult<Option<SessionData>> {
        if !self.session_file.exists() {
            return Ok(None);
        }

        let content = fs::read_to_string(&self.session_file).map_err(CLIERPError::Io)?;

        let session_data: SessionData =
            serde_json::from_str(&content).map_err(CLIERPError::Serialization)?;

        // Check if token is expired
        let now = chrono::Utc::now().timestamp();
        if now > session_data.expires_at {
            self.clear_session()?;
            return Ok(None);
        }

        // Validate token
        let auth_service = AuthService::new(self.config.clone());
        match auth_service.validate_token(&session_data.token) {
            Ok(_) => Ok(Some(session_data)),
            Err(_) => {
                self.clear_session()?;
                Ok(None)
            }
        }
    }

    /// Get current authenticated user
    pub fn get_current_user(&self) -> CLIERPResult<Option<AuthenticatedUser>> {
        if let Some(session) = self.load_session()? {
            let role = match session.role.as_str() {
                "admin" => UserRole::Admin,
                "manager" => UserRole::Manager,
                "supervisor" => UserRole::Supervisor,
                "employee" => UserRole::Employee,
                "auditor" => UserRole::Auditor,
                _ => UserRole::Employee,
            };

            // We would need to get the user ID from the token claims
            // For now, we'll use a placeholder
            let auth_service = AuthService::new(self.config.clone());
            let claims = auth_service.validate_token(&session.token)?;
            let user_id: i32 = claims
                .sub
                .parse()
                .map_err(|_| CLIERPError::Internal("Invalid user ID in token".to_string()))?;

            // Get full user info from database
            let user_from_db = auth_service.get_user_by_id(user_id)?;

            Ok(Some(AuthenticatedUser {
                id: user_from_db.id,
                username: user_from_db.username,
                email: user_from_db.email,
                role,
                employee_id: user_from_db.employee_id,
            }))
        } else {
            Ok(None)
        }
    }

    /// Clear the current session
    pub fn clear_session(&self) -> CLIERPResult<()> {
        if self.session_file.exists() {
            fs::remove_file(&self.session_file).map_err(CLIERPError::Io)?;
        }
        tracing::debug!("Session cleared");
        Ok(())
    }

    /// Check if user is currently logged in
    pub fn is_authenticated(&self) -> bool {
        self.load_session().unwrap_or(None).is_some()
    }

    /// Get current session token
    pub fn get_token(&self) -> CLIERPResult<Option<String>> {
        Ok(self.load_session()?.map(|s| s.token))
    }
}
