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
        )",
    )
    .execute(connection)?;

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
        )",
    )
    .execute(connection)?;

    // Create categories table for inventory
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS categories (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            description TEXT,
            parent_id INTEGER REFERENCES categories(id),
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(connection)?;

    // Create products table for inventory
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS products (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            sku TEXT NOT NULL UNIQUE,
            name TEXT NOT NULL,
            description TEXT,
            category_id INTEGER NOT NULL REFERENCES categories(id),
            price INTEGER NOT NULL DEFAULT 0,
            cost_price INTEGER NOT NULL DEFAULT 0,
            current_stock INTEGER NOT NULL DEFAULT 0,
            min_stock_level INTEGER NOT NULL DEFAULT 0,
            max_stock_level INTEGER,
            unit TEXT NOT NULL DEFAULT 'ea',
            barcode TEXT,
            is_active BOOLEAN NOT NULL DEFAULT TRUE,
            created_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP,
            updated_at DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(connection)?;

    // Create stock_movements table for inventory tracking
    diesel::sql_query(
        "CREATE TABLE IF NOT EXISTS stock_movements (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            product_id INTEGER NOT NULL REFERENCES products(id),
            movement_type TEXT NOT NULL CHECK (movement_type IN ('in', 'out', 'adjustment')),
            quantity INTEGER NOT NULL,
            unit_cost INTEGER,
            reference_type TEXT,
            reference_id INTEGER,
            notes TEXT,
            moved_by INTEGER REFERENCES users(id),
            movement_date DATETIME NOT NULL DEFAULT CURRENT_TIMESTAMP
        )",
    )
    .execute(connection)?;

    // Create indexes for better performance
    diesel::sql_query(
        "CREATE INDEX IF NOT EXISTS idx_employees_department_id ON employees(department_id)",
    )
    .execute(connection)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_employees_status ON employees(status)")
        .execute(connection)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_users_username ON users(username)")
        .execute(connection)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_users_email ON users(email)")
        .execute(connection)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_audit_logs_table_record ON audit_logs(table_name, record_id)").execute(connection)?;

    // Create indexes for inventory tables
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_categories_parent_id ON categories(parent_id)").execute(connection)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_products_category_id ON products(category_id)").execute(connection)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_products_sku ON products(sku)").execute(connection)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_products_stock_level ON products(current_stock, min_stock_level)").execute(connection)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_stock_movements_product_id ON stock_movements(product_id)").execute(connection)?;
    diesel::sql_query("CREATE INDEX IF NOT EXISTS idx_stock_movements_date ON stock_movements(movement_date)").execute(connection)?;

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
         ('회계팀', '회계 및 재무')",
    )
    .execute(connection)?;

    // Insert default categories
    diesel::sql_query(
        "INSERT OR IGNORE INTO categories (name, description) VALUES
         ('전자제품', '전자제품 및 디지털 기기'),
         ('사무용품', '사무실에서 사용하는 용품'),
         ('가구', '사무용 가구'),
         ('소프트웨어', '소프트웨어 제품'),
         ('기타', '기타 제품')",
    )
    .execute(connection)?;

    tracing::info!("Default data inserted successfully");
    Ok(())
}
