use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::env;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub timeout: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration: u64,
    pub password_rounds: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub format: String,
    pub file: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct CLIERPConfig {
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub logging: LoggingConfig,
    pub app_name: String,
    pub version: String,
}

impl Default for CLIERPConfig {
    fn default() -> Self {
        Self {
            database: DatabaseConfig {
                url: "sqlite:./clierp.db".to_string(),
                max_connections: 10,
                timeout: 30,
            },
            auth: AuthConfig {
                jwt_secret: "your-secret-key-change-this".to_string(),
                jwt_expiration: 3600, // 1 hour
                password_rounds: 12,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                format: "pretty".to_string(),
                file: None,
            },
            app_name: crate::APP_NAME.to_string(),
            version: crate::VERSION.to_string(),
        }
    }
}

impl CLIERPConfig {
    pub fn load() -> Result<Self, ConfigError> {
        let run_mode = env::var("RUN_MODE").unwrap_or_else(|_| "development".into());

        let builder = Config::builder()
            // Start with default configuration
            .add_source(Config::try_from(&CLIERPConfig::default())?)
            // Load configuration file based on run mode
            .add_source(File::with_name(&format!("config/{}", run_mode)).required(false))
            // Load local configuration file (gitignored)
            .add_source(File::with_name("config/local").required(false))
            // Override with environment variables with CLIERP_ prefix
            .add_source(Environment::with_prefix("CLIERP"));

        builder.build()?.try_deserialize()
    }

    pub fn validate(&self) -> Result<(), ConfigError> {
        // Validate JWT secret is not default
        if self.auth.jwt_secret == "your-secret-key-change-this" {
            eprintln!("Warning: Using default JWT secret. Please set CLIERP_AUTH__JWT_SECRET environment variable.");
        }

        // Validate database URL format
        if !self.database.url.starts_with("sqlite:")
            && !self.database.url.starts_with("postgres://")
        {
            return Err(ConfigError::Message(
                "Database URL must start with 'sqlite:' or 'postgres://'".to_string(),
            ));
        }

        Ok(())
    }
}
