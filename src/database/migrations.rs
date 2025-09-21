use crate::core::result::CLIERPResult;
use diesel::prelude::*;

pub fn run_migrations(connection: &mut SqliteConnection) -> CLIERPResult<()> {
    tracing::info!("Running database migrations...");

    // Create departments table
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS departments (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            manager_id INTEGER REFERENCES employees(id),
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(connection)?;

    // Create employees table
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS employees (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            employee_code TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            email TEXT UNIQUE,
            phone TEXT,
            department_id INTEGER NOT NULL REFERENCES departments(id),
            position TEXT NOT NULL,
            hire_date DATE NOT NULL,
            salary INTEGER NOT NULL DEFAULT 0,
            status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'inactive', 'terminated')),
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(connection)?;

    // Create users table
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS users (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            username TEXT NOT NULL UNIQUE,
            email TEXT NOT NULL UNIQUE,
            password_hash TEXT NOT NULL,
            employee_id INTEGER REFERENCES employees(id),
            role TEXT NOT NULL DEFAULT 'employee' CHECK (role IN ('admin', 'manager', 'supervisor', 'employee', 'auditor')),
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            last_login DATETIME,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(connection)?;

    // Create audit_logs table
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS audit_logs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            user_id INTEGER REFERENCES users(id),
            table_name TEXT NOT NULL,
            record_id INTEGER NOT NULL,
            action TEXT NOT NULL CHECK (action IN ('INSERT', 'UPDATE', 'DELETE')),
            old_values TEXT,
            new_values TEXT,
            changed_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )"
    ).execute(connection)?;

    // Create indexes for better performance
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_employees_department_id ON employees(department_id)").execute(connection)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_employees_status ON employees(status)").execute(connection)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)").execute(connection)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)").execute(connection)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_audit_logs_table_record ON audit_logs(table_name, record_id)").execute(connection)?;

    // Insert default data
    insert_default_data(connection)?;

    tracing::info!("Database migrations completed successfully");
    Ok(())
}

fn insert_default_data(connection: &mut SqliteConnection) -> CLIERPResult<()> {
    tracing::info!("Inserting default data...");

    // Insert default departments
    diesel::sql_query(
        "INSERT OR IGNORE INTO departments (name, description) VALUES
         ('경영진', '회사 경영진'),
         ('개발팀', '소프트웨어 개발'),
         ('영업팀', '영업 및 마케팅'),
         ('인사팀', '인사 관리'),
         ('회계팀', '회계 및 재무')"
    ).execute(connection)?;

    tracing::info!("Default data inserted successfully");
    Ok(())
}