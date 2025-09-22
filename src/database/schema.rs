// @generated automatically by generate_schema.py

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
    activities (id) {
        id -> Integer,
        customer_id -> Nullable<Integer>,
        lead_id -> Nullable<Integer>,
        deal_id -> Nullable<Integer>,
        activity_type -> Text,
        subject -> Text,
        description -> Nullable<Text>,
        activity_date -> Timestamp,
        duration_minutes -> Nullable<Integer>,
        outcome -> Nullable<Text>,
        assigned_to -> Nullable<Integer>,
        completed -> Bool,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    attendances (id) {
        id -> Integer,
        employee_id -> Integer,
        date -> Date,
        check_in -> Nullable<Time>,
        check_out -> Nullable<Time>,
        break_time -> Nullable<Integer>,
        overtime_hours -> Nullable<Float>,
        status -> Text,
        notes -> Nullable<Text>,
        created_at -> Timestamp,
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
    campaign_leads (id) {
        id -> Integer,
        campaign_id -> Integer,
        lead_id -> Integer,
        response -> Nullable<Text>,
        response_date -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    campaigns (id) {
        id -> Integer,
        name -> Text,
        description -> Nullable<Text>,
        campaign_type -> Text,
        start_date -> Nullable<Date>,
        end_date -> Nullable<Date>,
        budget -> Nullable<Integer>,
        spent -> Nullable<Integer>,
        target_audience -> Nullable<Text>,
        status -> Text,
        created_by -> Nullable<Integer>,
        notes -> Nullable<Text>,
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
    customers (id) {
        id -> Integer,
        customer_code -> Text,
        name -> Text,
        email -> Nullable<Text>,
        phone -> Nullable<Text>,
        address -> Nullable<Text>,
        customer_type -> Text,
        company_name -> Nullable<Text>,
        tax_id -> Nullable<Text>,
        credit_limit -> Nullable<Integer>,
        status -> Text,
        notes -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    deals (id) {
        id -> Integer,
        lead_id -> Nullable<Integer>,
        deal_name -> Text,
        stage -> Text,
        deal_value -> Integer,
        close_date -> Nullable<Date>,
        probability -> Nullable<Integer>,
        assigned_to -> Nullable<Integer>,
        products -> Nullable<Text>,
        discount_percent -> Nullable<Integer>,
        final_amount -> Nullable<Integer>,
        notes -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

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
    leads (id) {
        id -> Integer,
        customer_id -> Nullable<Integer>,
        lead_source -> Text,
        status -> Text,
        priority -> Text,
        estimated_value -> Nullable<Integer>,
        probability -> Nullable<Integer>,
        expected_close_date -> Nullable<Date>,
        assigned_to -> Nullable<Integer>,
        title -> Text,
        description -> Nullable<Text>,
        notes -> Nullable<Text>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    payrolls (id) {
        id -> Integer,
        employee_id -> Integer,
        period -> Text,
        base_salary -> Integer,
        overtime_pay -> Nullable<Integer>,
        bonuses -> Nullable<Integer>,
        deductions -> Nullable<Integer>,
        net_salary -> Integer,
        payment_date -> Nullable<Date>,
        status -> Text,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    product_attachments (id) {
        id -> Integer,
        product_id -> Integer,
        attachment_type -> Text,
        file_name -> Text,
        file_path -> Text,
        file_size -> Integer,
        mime_type -> Nullable<Text>,
        is_primary -> Bool,
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
    purchase_items (id) {
        id -> Integer,
        po_id -> Integer,
        product_id -> Integer,
        quantity -> Integer,
        unit_cost -> Integer,
        total_cost -> Integer,
        received_quantity -> Integer,
        status -> Text,
        created_at -> Timestamp,
    }
}

diesel::table! {
    purchase_orders (id) {
        id -> Integer,
        po_number -> Text,
        supplier_id -> Integer,
        order_date -> Date,
        expected_date -> Nullable<Date>,
        status -> Text,
        total_amount -> Integer,
        notes -> Nullable<Text>,
        created_by -> Nullable<Integer>,
        approved_by -> Nullable<Integer>,
        approved_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
        updated_at -> Timestamp,
    }
}

diesel::table! {
    stock_audit_items (id) {
        id -> Integer,
        audit_id -> Integer,
        product_id -> Integer,
        expected_quantity -> Integer,
        actual_quantity -> Nullable<Integer>,
        variance -> Nullable<Integer>,
        notes -> Nullable<Text>,
        audited_at -> Nullable<Timestamp>,
        created_at -> Timestamp,
    }
}

diesel::table! {
    stock_audits (id) {
        id -> Integer,
        audit_name -> Text,
        audit_date -> Date,
        status -> Text,
        conducted_by -> Nullable<Integer>,
        notes -> Nullable<Text>,
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

diesel::table! {
    suppliers (id) {
        id -> Integer,
        supplier_code -> Text,
        name -> Text,
        contact_person -> Nullable<Text>,
        email -> Nullable<Text>,
        phone -> Nullable<Text>,
        address -> Nullable<Text>,
        payment_terms -> Nullable<Text>,
        status -> Text,
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

diesel::joinable!(activities -> employees (assigned_to));
diesel::joinable!(activities -> deals (deal_id));
diesel::joinable!(activities -> leads (lead_id));
diesel::joinable!(activities -> customers (customer_id));
diesel::joinable!(attendances -> employees (employee_id));
diesel::joinable!(audit_logs -> users (user_id));
diesel::joinable!(campaign_leads -> leads (lead_id));
diesel::joinable!(campaign_leads -> campaigns (campaign_id));
diesel::joinable!(campaigns -> employees (created_by));
diesel::joinable!(deals -> employees (assigned_to));
diesel::joinable!(deals -> leads (lead_id));
diesel::joinable!(employees -> departments (department_id));
diesel::joinable!(leads -> employees (assigned_to));
diesel::joinable!(leads -> customers (customer_id));
diesel::joinable!(payrolls -> employees (employee_id));
diesel::joinable!(product_attachments -> products (product_id));
diesel::joinable!(products -> categories (category_id));
diesel::joinable!(purchase_items -> products (product_id));
diesel::joinable!(purchase_items -> purchase_orders (po_id));
// Note: purchase_orders has multiple FK to users (approved_by, created_by)
// Using one main relationship
diesel::joinable!(purchase_orders -> users (created_by));
diesel::joinable!(purchase_orders -> suppliers (supplier_id));
diesel::joinable!(stock_audit_items -> products (product_id));
diesel::joinable!(stock_audit_items -> stock_audits (audit_id));
diesel::joinable!(stock_audits -> users (conducted_by));
diesel::joinable!(stock_movements -> users (moved_by));
diesel::joinable!(stock_movements -> products (product_id));
diesel::joinable!(transactions -> users (created_by));
diesel::joinable!(transactions -> accounts (account_id));
diesel::joinable!(users -> employees (employee_id));

diesel::allow_tables_to_appear_in_same_query!(
    accounts,
    activities,
    attendances,
    audit_logs,
    campaign_leads,
    campaigns,
    categories,
    customers,
    deals,
    departments,
    employees,
    leads,
    payrolls,
    product_attachments,
    products,
    purchase_items,
    purchase_orders,
    stock_audit_items,
    stock_audits,
    stock_movements,
    suppliers,
    transactions,
    users,
);
