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

diesel::joinable!(employees -> departments (department_id));
diesel::joinable!(users -> employees (employee_id));
diesel::joinable!(audit_logs -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    departments,
    employees,
    users,
    audit_logs,
);