use crate::core::{config::CLIERPConfig, result::CLIERPResult, error::CLIERPError};
use diesel::{
    connection::SimpleConnection,
    prelude::*,
    sqlite::SqliteConnection,
    r2d2::{ConnectionManager, Pool, PooledConnection},
};
use std::sync::Arc;
use once_cell::sync::OnceCell;

pub type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
pub type PooledSqliteConnection = PooledConnection<ConnectionManager<SqliteConnection>>;
pub type DatabaseConnection = PooledSqliteConnection;

static DATABASE_POOL: OnceCell<Arc<SqlitePool>> = OnceCell::new();

pub struct DatabaseManager;

impl DatabaseManager {
    pub fn initialize(config: &CLIERPConfig) -> CLIERPResult<()> {
        let database_url = &config.database.url.replace("sqlite:", "");

        let manager = ConnectionManager::<SqliteConnection>::new(database_url);
        let pool = Pool::builder()
            .max_size(config.database.max_connections)
            .connection_timeout(std::time::Duration::from_secs(config.database.timeout))
            .build(manager)
            .map_err(|e| CLIERPError::Internal(format!("Failed to create connection pool: {}", e)))?;

        // Test the connection
        let mut conn = pool.get()
            .map_err(|e| CLIERPError::DatabaseConnection(diesel::ConnectionError::BadConnection(e.to_string())))?;

        // Enable foreign key constraints for SQLite
        conn.batch_execute("PRAGMA foreign_keys = ON;")
            .map_err(CLIERPError::Database)?;

        DATABASE_POOL.set(Arc::new(pool))
            .map_err(|_| CLIERPError::Internal("Database pool already initialized".to_string()))?;

        tracing::info!("Database connection pool initialized");
        Ok(())
    }

    pub fn get_pool() -> CLIERPResult<Arc<SqlitePool>> {
        DATABASE_POOL.get()
            .cloned()
            .ok_or_else(|| CLIERPError::Internal("Database pool not initialized".to_string()))
    }

    pub fn get_connection(&self) -> CLIERPResult<DatabaseConnection> {
        let pool = Self::get_pool()?;
        pool.get()
            .map_err(|e| CLIERPError::DatabaseConnection(diesel::ConnectionError::BadConnection(e.to_string())))
    }

    pub fn new() -> CLIERPResult<Self> {
        Ok(DatabaseManager)
    }

    pub fn establish_connection(database_url: &str) -> CLIERPResult<SqliteConnection> {
        let database_url = database_url.replace("sqlite:", "");
        let mut conn = SqliteConnection::establish(&database_url)
            .map_err(CLIERPError::DatabaseConnection)?;

        // Enable foreign key constraints for SQLite
        conn.batch_execute("PRAGMA foreign_keys = ON;")
            .map_err(CLIERPError::Database)?;

        Ok(conn)
    }
}