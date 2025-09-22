use crate::core::{config::CLIERPConfig, error::CLIERPError, result::CLIERPResult};
use crate::database::{
    connection::{DatabaseManager, get_connection},
    models::{NewUser, User, UserRole},
    schema::users,
};
use bcrypt::{hash, verify};
use chrono::{Duration, Utc};
use diesel::prelude::*;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,      // Subject (user id)
    pub username: String, // Username
    pub role: String,     // User role
    pub exp: usize,       // Expiration time
    pub iat: usize,       // Issued at
}

#[derive(Debug)]
pub struct AuthenticatedUser {
    pub id: i32,
    pub username: String,
    pub email: String,
    pub role: UserRole,
    pub employee_id: Option<i32>,
}

#[derive(Clone)]
pub struct AuthService {
    config: CLIERPConfig,
}

impl AuthService {
    pub fn new(config: CLIERPConfig) -> Self {
        Self { config }
    }

    /// Hash a password using bcrypt
    pub fn hash_password(&self, password: &str) -> CLIERPResult<String> {
        let cost = self.config.auth.password_rounds;
        hash(password, cost).map_err(CLIERPError::BCrypt)
    }

    /// Verify a password against its hash
    pub fn verify_password(&self, password: &str, hash: &str) -> CLIERPResult<bool> {
        verify(password, hash).map_err(CLIERPError::BCrypt)
    }

    /// Create a new user
    pub fn create_user(
        &self,
        username: String,
        email: String,
        password: String,
        role: UserRole,
        employee_id: Option<i32>,
    ) -> CLIERPResult<User> {
        let password_hash = self.hash_password(&password)?;

        let new_user = NewUser {
            username,
            email,
            password_hash,
            employee_id,
            role: role.to_string(),
            is_active: true,
        };

        let mut conn = get_connection()?;

        diesel::insert_into(users::table)
            .values(&new_user)
            .execute(&mut conn)
            .map_err(CLIERPError::Database)?;

        // Get the inserted user by username
        users::table
            .filter(users::username.eq(&new_user.username))
            .first(&mut conn)
            .map_err(CLIERPError::Database)
    }

    /// Authenticate user with username and password
    pub fn authenticate(&self, username: &str, password: &str) -> CLIERPResult<AuthenticatedUser> {
        let mut conn = get_connection()?;

        let user: User = users::table
            .filter(users::username.eq(username))
            .filter(users::is_active.eq(true))
            .first(&mut conn)
            .map_err(|_| CLIERPError::Authentication("Invalid username or password".to_string()))?;

        if !self.verify_password(password, &user.password_hash)? {
            return Err(CLIERPError::Authentication(
                "Invalid username or password".to_string(),
            ));
        }

        // Update last login
        diesel::update(users::table.filter(users::id.eq(user.id)))
            .set(users::last_login.eq(Utc::now().naive_utc()))
            .execute(&mut conn)
            .map_err(CLIERPError::Database)?;

        let role = match user.role.as_str() {
            "admin" => UserRole::Admin,
            "manager" => UserRole::Manager,
            "supervisor" => UserRole::Supervisor,
            "employee" => UserRole::Employee,
            "auditor" => UserRole::Auditor,
            _ => UserRole::Employee,
        };

        Ok(AuthenticatedUser {
            id: user.id,
            username: user.username,
            email: user.email,
            role,
            employee_id: user.employee_id,
        })
    }

    /// Generate a JWT token for authenticated user
    pub fn generate_token(&self, user: &AuthenticatedUser) -> CLIERPResult<String> {
        let now = Utc::now();
        let exp = now + Duration::seconds(self.config.auth.jwt_expiration as i64);

        let claims = Claims {
            sub: user.id.to_string(),
            username: user.username.clone(),
            role: user.role.to_string(),
            exp: exp.timestamp() as usize,
            iat: now.timestamp() as usize,
        };

        let header = Header::new(Algorithm::HS256);
        let encoding_key = EncodingKey::from_secret(self.config.auth.jwt_secret.as_ref());

        encode(&header, &claims, &encoding_key).map_err(CLIERPError::Jwt)
    }

    /// Validate and decode a JWT token
    pub fn validate_token(&self, token: &str) -> CLIERPResult<Claims> {
        let decoding_key = DecodingKey::from_secret(self.config.auth.jwt_secret.as_ref());
        let validation = Validation::new(Algorithm::HS256);

        let token_data =
            decode::<Claims>(token, &decoding_key, &validation).map_err(CLIERPError::Jwt)?;

        Ok(token_data.claims)
    }

    /// Get user by ID
    pub fn get_user_by_id(&self, user_id: i32) -> CLIERPResult<User> {
        let mut conn = get_connection()?;

        users::table
            .filter(users::id.eq(user_id))
            .first(&mut conn)
            .map_err(CLIERPError::Database)
    }

    /// Check if user has required role
    pub fn check_permission(&self, user_role: &UserRole, required_role: &UserRole) -> bool {
        use UserRole::*;

        let user_level = match user_role {
            Admin => 5,
            Manager => 4,
            Supervisor => 3,
            Employee => 2,
            Auditor => 1,
        };

        let required_level = match required_role {
            Admin => 5,
            Manager => 4,
            Supervisor => 3,
            Employee => 2,
            Auditor => 1,
        };

        user_level >= required_level
    }

    /// Create default admin user if none exists
    pub fn create_default_admin(&self) -> CLIERPResult<()> {
        let mut conn = get_connection()?;

        // Check if any admin user exists
        let admin_count: i64 = users::table
            .filter(users::role.eq("admin"))
            .count()
            .get_result(&mut conn)
            .map_err(CLIERPError::Database)?;

        if admin_count == 0 {
            tracing::info!("Creating default admin user...");

            let default_password =
                env::var("CLIERP_ADMIN_PASSWORD").unwrap_or_else(|_| "admin123".to_string());

            self.create_user(
                "admin".to_string(),
                "admin@clierp.local".to_string(),
                default_password,
                UserRole::Admin,
                None,
            )?;

            tracing::warn!(
                "Default admin user created with username 'admin'. Please change the password!"
            );
        }

        Ok(())
    }
}
