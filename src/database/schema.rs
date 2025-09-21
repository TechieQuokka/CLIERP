// @generated automatically by Diesel CLI.

diesel::table! {
    departments (id) {
        id -> Integer,
        name -> Text,
        description -> Nullable<Text>,
        manager_id -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    employees (id) {
        id -> Integer,
        employee_code -> Text,
        name -> Text,
        email -> Nullable<Text>,
        phone -> Nullable<Text>,
        department_id -> Integer,
        position -> Text,
        hire_date -> Date,
        salary -> Integer,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    users (id) {
        id -> Integer,
        username -> Text,
        email -> Text,
        password_hash -> Text,
        employee_id -> Nullable<Integer>,
        role -> Text,
        is_active -> Bool,
        last_login -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    audit_logs (id) {
        id -> Integer,
        user_id -> Nullable<Integer>,
        table_name -> Text,
        record_id -> Integer,
        action -> Text,
        old_values -> Nullable<Text>,
        new_values -> Nullable<Text>,
        changed_at -> Timestamp,
    }
}

diesel::table! {
    attendances (id) {
        id -> Integer,
        employee_id -> Integer,
        date -> Date,
        check_in -> Nullable<Time>,
        check_out -> Nullable<Time>,
        break_time -> Integer,
        overtime_hours -> Float,
        status -> Text,
        notes -> Nullable<Text>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    payrolls (id) {
        id -> Integer,
        employee_id -> Integer,
        period -> Text,
        base_salary -> Integer,
        overtime_pay -> Integer,
        bonuses -> Integer,
        deductions -> Integer,
        net_salary -> Integer,
        payment_date -> Nullable<Date>,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    accounts (id) {
        id -> Integer,
        account_code -> Text,
        account_name -> Text,
        account_type -> Text,
        parent_id -> Nullable<Integer>,
        balance -> Integer,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    transactions (id) {
        id -> Integer,
        account_id -> Integer,
        transaction_date -> Date,
        amount -> Integer,
        debit_credit -> Text,
        description -> Text,
        reference -> Nullable<Text>,
        created_by -> Nullable<Integer>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    categories (id) {
        id -> Integer,
        name -> Text,
        description -> Nullable<Text>,
        parent_id -> Nullable<Integer>,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    products (id) {
        id -> Integer,
        sku -> Text,
        name -> Text,
        description -> Nullable<Text>,
        category_id -> Integer,
        price -> Integer,
        cost_price -> Integer,
        current_stock -> Integer,
        min_stock_level -> Integer,
        max_stock_level -> Nullable<Integer>,
        unit -> Text,
        barcode -> Nullable<Text>,
        is_active -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    stock_movements (id) {
        id -> Integer,
        product_id -> Integer,
        movement_type -> Text,
        quantity -> Integer,
        unit_cost -> Nullable<Integer>,
        reference_type -> Nullable<Text>,
        reference_id -> Nullable<Integer>,
        notes -> Nullable<Text>,
        moved_by -> Nullable<Integer>,
        movement_date -> Timestamp,
    }
}

diesel::joinable!(employees -> departments (department_id));
diesel::joinable!(users -> employees (employee_id));
diesel::joinable!(audit_logs -> users (user_id));
diesel::joinable!(attendances -> employees (employee_id));
diesel::joinable!(payrolls -> employees (employee_id));
// Self-referential foreign key for parent accounts
// diesel::joinable!(accounts -> accounts (parent_id));
diesel::joinable!(transactions -> accounts (account_id));
diesel::joinable!(transactions -> users (created_by));
// Self-referential foreign key for parent categories
// diesel::joinable!(categories -> categories (parent_id));
diesel::joinable!(products -> categories (category_id));
diesel::joinable!(stock_movements -> products (product_id));
diesel::joinable!(stock_movements -> users (moved_by));

diesel::allow_tables_to_appear_in_same_query!(
    departments,
    employees,
    users,
    audit_logs,
    attendances,
    payrolls,
    accounts,
    transactions,
    categories,
    products,
    stock_movements,
);
