use std::sync::Once;
use std::env;
use clierp::database::connection::establish_connection;
use diesel::prelude::*;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

static INIT: Once = Once::new();

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub fn setup_test_db() {
    INIT.call_once(|| {
        // Set test database URL
        env::set_var("DATABASE_URL", ":memory:");

        // Run migrations
        let mut connection = establish_connection()
            .expect("Failed to establish database connection for tests");

        connection
            .run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        // Insert test data
        insert_test_data(&mut connection);
    });
}

fn insert_test_data(connection: &mut SqliteConnection) {
    use clierp::database::schema::{categories, departments, employees, users};
    use clierp::database::models::{NewCategory, NewDepartment, NewEmployee, NewUser};
    use chrono::NaiveDate;

    // Insert test category
    let new_category = NewCategory {
        name: "Test Category".to_string(),
        description: Some("Test category for unit tests".to_string()),
        parent_id: None,
    };

    diesel::insert_into(categories::table)
        .values(&new_category)
        .execute(connection)
        .expect("Failed to insert test category");

    // Insert test department
    let new_department = NewDepartment {
        name: "Test Department".to_string(),
        description: Some("Test department for unit tests".to_string()),
        manager_id: None,
    };

    diesel::insert_into(departments::table)
        .values(&new_department)
        .execute(connection)
        .expect("Failed to insert test department");

    // Insert test employee
    let new_employee = NewEmployee {
        employee_code: "TEST001".to_string(),
        name: "Test Employee".to_string(),
        email: Some("test@example.com".to_string()),
        phone: Some("123-456-7890".to_string()),
        department_id: 1,
        position: "Test Position".to_string(),
        hire_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        salary: 50000,
        status: "active".to_string(),
    };

    diesel::insert_into(employees::table)
        .values(&new_employee)
        .execute(connection)
        .expect("Failed to insert test employee");

    // Insert test user
    let new_user = NewUser {
        username: "testuser".to_string(),
        email: "testuser@example.com".to_string(),
        password_hash: "$2b$12$test.hash.for.unit.tests".to_string(),
        employee_id: Some(1),
        role: "admin".to_string(),
        is_active: true,
    };

    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(connection)
        .expect("Failed to insert test user");
}

// Helper function to clean up test data
pub fn cleanup_test_data() {
    use clierp::database::schema::*;
    use clierp::database::connection::get_connection;

    let mut connection = get_connection().expect("Failed to get connection");

    // Delete in reverse order due to foreign key constraints
    let _ = diesel::delete(stock_movements::table).execute(&mut connection);
    let _ = diesel::delete(products::table).execute(&mut connection);
    let _ = diesel::delete(categories::table).execute(&mut connection);
    let _ = diesel::delete(employees::table).execute(&mut connection);
    let _ = diesel::delete(departments::table).execute(&mut connection);
    let _ = diesel::delete(users::table).execute(&mut connection);
}