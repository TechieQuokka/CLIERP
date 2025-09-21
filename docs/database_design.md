# CLIERP 데이터베이스 상세 설계

## 1. 개요

CLIERP 시스템의 데이터베이스는 ERP 시스템의 핵심 데이터를 안전하고 효율적으로 관리하기 위해 설계되었습니다.

### 1.1 설계 원칙
- **정규화**: 데이터 중복 최소화 및 일관성 보장
- **성능**: 빠른 조회와 효율적인 인덱싱
- **확장성**: 향후 기능 추가에 대비한 유연한 구조
- **보안**: 민감한 데이터의 암호화 및 접근 제어
- **무결성**: 강력한 제약조건으로 데이터 품질 보장

### 1.2 기술 스택
- **개발 환경**: SQLite 3.x
- **운영 환경**: PostgreSQL 15+
- **ORM**: Diesel (Rust)
- **마이그레이션**: Diesel Migration System

## 2. 엔티티 관계도 (ERD)

### 2.1 전체 ERD

```
                            ┌─────────────────┐
                            │   departments   │
                            │─────────────────│
                            │ id (PK)         │
                            │ name            │
                            │ description     │
                            │ manager_id (FK) │
                            │ created_at      │
                            │ updated_at      │
                            └─────────────────┘
                                     │
                                     │ 1:N
                                     ▼
                            ┌─────────────────┐
                    ┌───────│   employees     │
                    │       │─────────────────│
                    │       │ id (PK)         │
                    │       │ employee_code   │
                    │       │ name            │
                    │       │ email           │
                    │       │ phone           │
                    │       │ department_id   │
                    │       │ position        │
                    │       │ hire_date       │
                    │       │ salary          │
                    │       │ status          │
                    │       │ created_at      │
                    │       │ updated_at      │
                    │       └─────────────────┘
                    │                │
                    │ 1:N            │ 1:N
                    ▼                ▼
            ┌─────────────────┐ ┌─────────────────┐
            │   attendances   │ │    payrolls     │
            │─────────────────│ │─────────────────│
            │ id (PK)         │ │ id (PK)         │
            │ employee_id(FK) │ │ employee_id(FK) │
            │ date            │ │ period          │
            │ check_in        │ │ base_salary     │
            │ check_out       │ │ overtime_pay    │
            │ break_time      │ │ bonuses         │
            │ overtime_hours  │ │ deductions      │
            │ status          │ │ net_salary      │
            │ notes           │ │ payment_date    │
            │ created_at      │ │ status          │
            └─────────────────┘ │ created_at      │
                                │ updated_at      │
                                └─────────────────┘

┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   customers     │    │   suppliers     │    │   categories    │
│─────────────────│    │─────────────────│    │─────────────────│
│ id (PK)         │    │ id (PK)         │    │ id (PK)         │
│ customer_code   │    │ supplier_code   │    │ name            │
│ name            │    │ name            │    │ description     │
│ email           │    │ contact_person  │    │ parent_id (FK)  │
│ phone           │    │ email           │    │ created_at      │
│ address         │    │ phone           │    │ updated_at      │
│ customer_type   │    │ address         │    └─────────────────┘
│ credit_limit    │    │ payment_terms   │             │
│ created_at      │    │ status          │             │ 1:N
│ updated_at      │    │ created_at      │             ▼
└─────────────────┘    │ updated_at      │    ┌─────────────────┐
         │              └─────────────────┘    │    products     │
         │ 1:N                   │ 1:N         │─────────────────│
         ▼                       ▼             │ id (PK)         │
┌─────────────────┐    ┌─────────────────┐    │ product_code    │
│     orders      │    │ purchase_orders │    │ name            │
│─────────────────│    │─────────────────│    │ description     │
│ id (PK)         │    │ id (PK)         │    │ category_id(FK) │
│ order_number    │    │ po_number       │    │ unit_price      │
│ customer_id(FK) │    │ supplier_id(FK) │    │ cost_price      │
│ order_date      │    │ order_date      │    │ unit_of_measure │
│ delivery_date   │    │ expected_date   │    │ weight          │
│ status          │    │ status          │    │ dimensions      │
│ total_amount    │    │ total_amount    │    │ status          │
│ notes           │    │ notes           │    │ created_at      │
│ created_at      │    │ created_at      │    │ updated_at      │
│ updated_at      │    │ updated_at      │    └─────────────────┘
└─────────────────┘    └─────────────────┘             │
         │                       │                      │ 1:N
         │ 1:N                   │ 1:N                  ▼
         ▼                       ▼             ┌─────────────────┐
┌─────────────────┐    ┌─────────────────┐    │   inventories   │
│  order_items    │    │ purchase_items  │    │─────────────────│
│─────────────────│    │─────────────────│    │ id (PK)         │
│ id (PK)         │    │ id (PK)         │    │ product_id (FK) │
│ order_id (FK)   │    │ po_id (FK)      │    │ quantity        │
│ product_id (FK) │    │ product_id (FK) │    │ reserved_qty    │
│ quantity        │    │ quantity        │    │ min_stock_level │
│ unit_price      │    │ unit_cost       │    │ max_stock_level │
│ total_price     │    │ total_cost      │    │ location        │
│ created_at      │    │ created_at      │    │ last_updated    │
└─────────────────┘    └─────────────────┘    │ created_at      │
                                              │ updated_at      │
                                              └─────────────────┘

┌─────────────────┐    ┌─────────────────┐    ┌─────────────────┐
│   accounts      │    │   campaigns     │    │     leads       │
│─────────────────│    │─────────────────│    │─────────────────│
│ id (PK)         │    │ id (PK)         │    │ id (PK)         │
│ account_code    │    │ name            │    │ customer_id(FK) │
│ account_name    │    │ description     │    │ source          │
│ account_type    │    │ start_date      │    │ status          │
│ parent_id (FK)  │    │ end_date        │    │ value           │
│ balance         │    │ budget          │    │ probability     │
│ is_active       │    │ spent           │    │ expected_close  │
│ created_at      │    │ target_audience │    │ assigned_to     │
│ updated_at      │    │ status          │    │ notes           │
└─────────────────┘    │ created_at      │    │ created_at      │
         │              │ updated_at      │    │ updated_at      │
         │ 1:N          └─────────────────┘    └─────────────────┘
         ▼                       │                       │
┌─────────────────┐              │ 1:N                   │ 1:N
│  transactions   │              ▼                       ▼
│─────────────────│    ┌─────────────────┐    ┌─────────────────┐
│ id (PK)         │    │ campaign_leads  │    │     deals       │
│ account_id (FK) │    │─────────────────│    │─────────────────│
│ transaction_date│    │ id (PK)         │    │ id (PK)         │
│ amount          │    │ campaign_id(FK) │    │ lead_id (FK)    │
│ debit_credit    │    │ lead_id (FK)    │    │ deal_name       │
│ description     │    │ created_at      │    │ stage           │
│ reference       │    └─────────────────┘    │ value           │
│ created_by      │                           │ close_date      │
│ created_at      │                           │ probability     │
│ updated_at      │                           │ assigned_to     │
└─────────────────┘                           │ notes           │
                                              │ created_at      │
                                              │ updated_at      │
                                              └─────────────────┘
```

## 3. 테이블 상세 정의

### 3.1 HR 모듈 테이블

#### 3.1.1 departments (부서)
```sql
CREATE TABLE departments (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL UNIQUE,
    description TEXT,
    manager_id INTEGER REFERENCES employees(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

#### 3.1.2 employees (직원)
```sql
CREATE TABLE employees (
    id SERIAL PRIMARY KEY,
    employee_code VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(100) NOT NULL,
    email VARCHAR(255) UNIQUE,
    phone VARCHAR(20),
    department_id INTEGER NOT NULL REFERENCES departments(id),
    position VARCHAR(100) NOT NULL,
    hire_date DATE NOT NULL,
    salary INTEGER NOT NULL DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'inactive', 'terminated')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

#### 3.1.3 attendances (출퇴근)
```sql
CREATE TABLE attendances (
    id SERIAL PRIMARY KEY,
    employee_id INTEGER NOT NULL REFERENCES employees(id),
    date DATE NOT NULL,
    check_in TIME,
    check_out TIME,
    break_time INTEGER DEFAULT 0, -- 분 단위
    overtime_hours DECIMAL(4,2) DEFAULT 0,
    status VARCHAR(20) NOT NULL DEFAULT 'present'
        CHECK (status IN ('present', 'absent', 'late', 'early_leave', 'holiday')),
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(employee_id, date)
);
```

#### 3.1.4 payrolls (급여)
```sql
CREATE TABLE payrolls (
    id SERIAL PRIMARY KEY,
    employee_id INTEGER NOT NULL REFERENCES employees(id),
    period VARCHAR(7) NOT NULL, -- YYYY-MM 형식
    base_salary INTEGER NOT NULL,
    overtime_pay INTEGER DEFAULT 0,
    bonuses INTEGER DEFAULT 0,
    deductions INTEGER DEFAULT 0,
    net_salary INTEGER NOT NULL,
    payment_date DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'processed', 'paid')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(employee_id, period)
);
```

### 3.2 Inventory 모듈 테이블

#### 3.2.1 categories (제품 카테고리)
```sql
CREATE TABLE categories (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) NOT NULL,
    description TEXT,
    parent_id INTEGER REFERENCES categories(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

#### 3.2.2 products (제품)
```sql
CREATE TABLE products (
    id SERIAL PRIMARY KEY,
    product_code VARCHAR(50) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    category_id INTEGER NOT NULL REFERENCES categories(id),
    unit_price INTEGER NOT NULL DEFAULT 0,
    cost_price INTEGER NOT NULL DEFAULT 0,
    unit_of_measure VARCHAR(20) NOT NULL DEFAULT 'EA',
    weight DECIMAL(10,3),
    dimensions VARCHAR(50),
    status VARCHAR(20) NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'inactive', 'discontinued')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

#### 3.2.3 inventories (재고)
```sql
CREATE TABLE inventories (
    id SERIAL PRIMARY KEY,
    product_id INTEGER NOT NULL REFERENCES products(id) UNIQUE,
    quantity INTEGER NOT NULL DEFAULT 0,
    reserved_qty INTEGER NOT NULL DEFAULT 0,
    min_stock_level INTEGER NOT NULL DEFAULT 0,
    max_stock_level INTEGER NOT NULL DEFAULT 1000,
    location VARCHAR(100),
    last_updated TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

#### 3.2.4 suppliers (공급업체)
```sql
CREATE TABLE suppliers (
    id SERIAL PRIMARY KEY,
    supplier_code VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    contact_person VARCHAR(100),
    email VARCHAR(255),
    phone VARCHAR(20),
    address TEXT,
    payment_terms VARCHAR(50),
    status VARCHAR(20) NOT NULL DEFAULT 'active'
        CHECK (status IN ('active', 'inactive', 'blacklisted')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

#### 3.2.5 purchase_orders (구매주문)
```sql
CREATE TABLE purchase_orders (
    id SERIAL PRIMARY KEY,
    po_number VARCHAR(50) NOT NULL UNIQUE,
    supplier_id INTEGER NOT NULL REFERENCES suppliers(id),
    order_date DATE NOT NULL,
    expected_date DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'approved', 'sent', 'received', 'cancelled')),
    total_amount INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

#### 3.2.6 purchase_items (구매주문 항목)
```sql
CREATE TABLE purchase_items (
    id SERIAL PRIMARY KEY,
    po_id INTEGER NOT NULL REFERENCES purchase_orders(id) ON DELETE CASCADE,
    product_id INTEGER NOT NULL REFERENCES products(id),
    quantity INTEGER NOT NULL,
    unit_cost INTEGER NOT NULL,
    total_cost INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

### 3.3 CRM 모듈 테이블

#### 3.3.1 customers (고객)
```sql
CREATE TABLE customers (
    id SERIAL PRIMARY KEY,
    customer_code VARCHAR(20) NOT NULL UNIQUE,
    name VARCHAR(200) NOT NULL,
    email VARCHAR(255),
    phone VARCHAR(20),
    address TEXT,
    customer_type VARCHAR(20) NOT NULL DEFAULT 'individual'
        CHECK (customer_type IN ('individual', 'business')),
    credit_limit INTEGER DEFAULT 0,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

#### 3.3.2 leads (리드)
```sql
CREATE TABLE leads (
    id SERIAL PRIMARY KEY,
    customer_id INTEGER NOT NULL REFERENCES customers(id),
    source VARCHAR(50) NOT NULL,
    status VARCHAR(20) NOT NULL DEFAULT 'new'
        CHECK (status IN ('new', 'contacted', 'qualified', 'proposal', 'negotiation', 'closed_won', 'closed_lost')),
    value INTEGER DEFAULT 0,
    probability INTEGER DEFAULT 0 CHECK (probability >= 0 AND probability <= 100),
    expected_close DATE,
    assigned_to INTEGER REFERENCES employees(id),
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

#### 3.3.3 deals (거래)
```sql
CREATE TABLE deals (
    id SERIAL PRIMARY KEY,
    lead_id INTEGER NOT NULL REFERENCES leads(id),
    deal_name VARCHAR(200) NOT NULL,
    stage VARCHAR(20) NOT NULL DEFAULT 'prospect'
        CHECK (stage IN ('prospect', 'qualified', 'proposal', 'negotiation', 'closed_won', 'closed_lost')),
    value INTEGER NOT NULL DEFAULT 0,
    close_date DATE,
    probability INTEGER DEFAULT 50 CHECK (probability >= 0 AND probability <= 100),
    assigned_to INTEGER REFERENCES employees(id),
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

#### 3.3.4 campaigns (캠페인)
```sql
CREATE TABLE campaigns (
    id SERIAL PRIMARY KEY,
    name VARCHAR(200) NOT NULL,
    description TEXT,
    start_date DATE,
    end_date DATE,
    budget INTEGER DEFAULT 0,
    spent INTEGER DEFAULT 0,
    target_audience TEXT,
    status VARCHAR(20) NOT NULL DEFAULT 'planned'
        CHECK (status IN ('planned', 'active', 'paused', 'completed', 'cancelled')),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

#### 3.3.5 campaign_leads (캠페인-리드 연결)
```sql
CREATE TABLE campaign_leads (
    id SERIAL PRIMARY KEY,
    campaign_id INTEGER NOT NULL REFERENCES campaigns(id) ON DELETE CASCADE,
    lead_id INTEGER NOT NULL REFERENCES leads(id) ON DELETE CASCADE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(campaign_id, lead_id)
);
```

### 3.4 Sales 모듈 테이블

#### 3.4.1 orders (주문)
```sql
CREATE TABLE orders (
    id SERIAL PRIMARY KEY,
    order_number VARCHAR(50) NOT NULL UNIQUE,
    customer_id INTEGER NOT NULL REFERENCES customers(id),
    order_date DATE NOT NULL,
    delivery_date DATE,
    status VARCHAR(20) NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'confirmed', 'shipped', 'delivered', 'cancelled')),
    total_amount INTEGER NOT NULL DEFAULT 0,
    notes TEXT,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

#### 3.4.2 order_items (주문 항목)
```sql
CREATE TABLE order_items (
    id SERIAL PRIMARY KEY,
    order_id INTEGER NOT NULL REFERENCES orders(id) ON DELETE CASCADE,
    product_id INTEGER NOT NULL REFERENCES products(id),
    quantity INTEGER NOT NULL,
    unit_price INTEGER NOT NULL,
    total_price INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

### 3.5 Finance 모듈 테이블

#### 3.5.1 accounts (계정)
```sql
CREATE TABLE accounts (
    id SERIAL PRIMARY KEY,
    account_code VARCHAR(20) NOT NULL UNIQUE,
    account_name VARCHAR(200) NOT NULL,
    account_type VARCHAR(20) NOT NULL
        CHECK (account_type IN ('asset', 'liability', 'equity', 'revenue', 'expense')),
    parent_id INTEGER REFERENCES accounts(id),
    balance INTEGER NOT NULL DEFAULT 0,
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

#### 3.5.2 transactions (거래)
```sql
CREATE TABLE transactions (
    id SERIAL PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id),
    transaction_date DATE NOT NULL,
    amount INTEGER NOT NULL,
    debit_credit VARCHAR(6) NOT NULL CHECK (debit_credit IN ('debit', 'credit')),
    description TEXT NOT NULL,
    reference VARCHAR(100),
    created_by INTEGER REFERENCES employees(id),
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

### 3.6 시스템 테이블

#### 3.6.1 users (사용자)
```sql
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    username VARCHAR(50) NOT NULL UNIQUE,
    email VARCHAR(255) NOT NULL UNIQUE,
    password_hash VARCHAR(255) NOT NULL,
    employee_id INTEGER REFERENCES employees(id),
    role VARCHAR(20) NOT NULL DEFAULT 'employee'
        CHECK (role IN ('admin', 'manager', 'supervisor', 'employee', 'auditor')),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    last_login TIMESTAMP WITH TIME ZONE,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

#### 3.6.2 audit_logs (감사 로그)
```sql
CREATE TABLE audit_logs (
    id SERIAL PRIMARY KEY,
    user_id INTEGER REFERENCES users(id),
    table_name VARCHAR(100) NOT NULL,
    record_id INTEGER NOT NULL,
    action VARCHAR(20) NOT NULL CHECK (action IN ('INSERT', 'UPDATE', 'DELETE')),
    old_values JSONB,
    new_values JSONB,
    changed_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);
```

## 4. 인덱스 설계

### 4.1 성능 최적화 인덱스

```sql
-- 직원 관련
CREATE INDEX idx_employees_department_id ON employees(department_id);
CREATE INDEX idx_employees_status ON employees(status);
CREATE INDEX idx_employees_hire_date ON employees(hire_date);

-- 출퇴근 관련
CREATE INDEX idx_attendances_employee_date ON attendances(employee_id, date);
CREATE INDEX idx_attendances_date ON attendances(date);

-- 급여 관련
CREATE INDEX idx_payrolls_employee_period ON payrolls(employee_id, period);
CREATE INDEX idx_payrolls_period ON payrolls(period);

-- 제품 관련
CREATE INDEX idx_products_category_id ON products(category_id);
CREATE INDEX idx_products_status ON products(status);
CREATE INDEX idx_products_code_name ON products(product_code, name);

-- 재고 관련
CREATE INDEX idx_inventories_product_id ON inventories(product_id);
CREATE INDEX idx_inventories_low_stock ON inventories(quantity) WHERE quantity <= min_stock_level;

-- 주문 관련
CREATE INDEX idx_orders_customer_id ON orders(customer_id);
CREATE INDEX idx_orders_date_status ON orders(order_date, status);
CREATE INDEX idx_order_items_order_id ON order_items(order_id);
CREATE INDEX idx_order_items_product_id ON order_items(product_id);

-- 구매주문 관련
CREATE INDEX idx_purchase_orders_supplier_id ON purchase_orders(supplier_id);
CREATE INDEX idx_purchase_orders_date_status ON purchase_orders(order_date, status);
CREATE INDEX idx_purchase_items_po_id ON purchase_items(po_id);

-- CRM 관련
CREATE INDEX idx_leads_customer_id ON leads(customer_id);
CREATE INDEX idx_leads_status_assigned ON leads(status, assigned_to);
CREATE INDEX idx_deals_lead_id ON deals(lead_id);
CREATE INDEX idx_deals_stage_close_date ON deals(stage, close_date);

-- 회계 관련
CREATE INDEX idx_transactions_account_id ON transactions(account_id);
CREATE INDEX idx_transactions_date ON transactions(transaction_date);
CREATE INDEX idx_accounts_type_active ON accounts(account_type, is_active);

-- 감사 로그
CREATE INDEX idx_audit_logs_table_record ON audit_logs(table_name, record_id);
CREATE INDEX idx_audit_logs_user_changed ON audit_logs(user_id, changed_at);
CREATE INDEX idx_audit_logs_changed_at ON audit_logs(changed_at);
```

### 4.2 전문 검색 인덱스 (PostgreSQL)

```sql
-- 제품명 검색
CREATE INDEX idx_products_name_search ON products USING gin(to_tsvector('korean', name));

-- 고객명 검색
CREATE INDEX idx_customers_name_search ON customers USING gin(to_tsvector('korean', name));

-- 직원명 검색
CREATE INDEX idx_employees_name_search ON employees USING gin(to_tsvector('korean', name));
```

## 5. 제약조건 및 트리거

### 5.1 체크 제약조건

```sql
-- 급여는 0보다 커야 함
ALTER TABLE employees ADD CONSTRAINT chk_employees_salary_positive
    CHECK (salary > 0);

-- 재고 수량은 0 이상이어야 함
ALTER TABLE inventories ADD CONSTRAINT chk_inventories_quantity_non_negative
    CHECK (quantity >= 0 AND reserved_qty >= 0);

-- 주문 수량과 가격은 양수여야 함
ALTER TABLE order_items ADD CONSTRAINT chk_order_items_positive
    CHECK (quantity > 0 AND unit_price > 0);

-- 거래 금액은 0이 아니어야 함
ALTER TABLE transactions ADD CONSTRAINT chk_transactions_amount_non_zero
    CHECK (amount != 0);
```

### 5.2 트리거 (PostgreSQL)

#### 5.2.1 업데이트 시간 자동 갱신
```sql
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = CURRENT_TIMESTAMP;
    RETURN NEW;
END;
$$ language 'plpgsql';

-- 모든 테이블에 적용
CREATE TRIGGER update_employees_updated_at BEFORE UPDATE ON employees
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

CREATE TRIGGER update_departments_updated_at BEFORE UPDATE ON departments
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- 기타 테이블들도 동일하게 적용...
```

#### 5.2.2 감사 로그 자동 생성
```sql
CREATE OR REPLACE FUNCTION audit_trigger_function()
RETURNS TRIGGER AS $$
BEGIN
    IF TG_OP = 'INSERT' THEN
        INSERT INTO audit_logs (table_name, record_id, action, new_values)
        VALUES (TG_TABLE_NAME, NEW.id, 'INSERT', row_to_json(NEW));
        RETURN NEW;
    ELSIF TG_OP = 'UPDATE' THEN
        INSERT INTO audit_logs (table_name, record_id, action, old_values, new_values)
        VALUES (TG_TABLE_NAME, NEW.id, 'UPDATE', row_to_json(OLD), row_to_json(NEW));
        RETURN NEW;
    ELSIF TG_OP = 'DELETE' THEN
        INSERT INTO audit_logs (table_name, record_id, action, old_values)
        VALUES (TG_TABLE_NAME, OLD.id, 'DELETE', row_to_json(OLD));
        RETURN OLD;
    END IF;
    RETURN NULL;
END;
$$ LANGUAGE plpgsql;

-- 중요 테이블에 감사 트리거 적용
CREATE TRIGGER audit_employees AFTER INSERT OR UPDATE OR DELETE ON employees
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();

CREATE TRIGGER audit_transactions AFTER INSERT OR UPDATE OR DELETE ON transactions
    FOR EACH ROW EXECUTE FUNCTION audit_trigger_function();
```

## 6. 마이그레이션 전략

### 6.1 Diesel 마이그레이션 구조

```
migrations/
├── 00000000000001_create_departments/
│   ├── up.sql
│   └── down.sql
├── 00000000000002_create_employees/
│   ├── up.sql
│   └── down.sql
├── 00000000000003_create_attendances/
│   ├── up.sql
│   └── down.sql
└── ...
```

### 6.2 마이그레이션 순서

1. **기초 테이블**: departments, categories, accounts
2. **마스터 데이터**: employees, products, customers, suppliers
3. **트랜잭션 데이터**: attendances, payrolls, orders, transactions
4. **관계 테이블**: order_items, purchase_items, campaign_leads
5. **시스템 테이블**: users, audit_logs
6. **인덱스 및 제약조건**
7. **트리거 및 함수**

### 6.3 롤백 전략

```sql
-- 각 마이그레이션의 down.sql 예시
-- down.sql for create_employees
DROP TRIGGER IF EXISTS audit_employees ON employees;
DROP TRIGGER IF EXISTS update_employees_updated_at ON employees;
DROP INDEX IF EXISTS idx_employees_department_id;
DROP INDEX IF EXISTS idx_employees_status;
DROP TABLE IF EXISTS employees;
```

## 7. 데이터 시딩

### 7.1 기본 데이터

```sql
-- 기본 부서 데이터
INSERT INTO departments (name, description) VALUES
    ('경영진', '회사 경영진'),
    ('개발팀', '소프트웨어 개발'),
    ('영업팀', '영업 및 마케팅'),
    ('인사팀', '인사 관리'),
    ('회계팀', '회계 및 재무');

-- 기본 제품 카테고리
INSERT INTO categories (name, description) VALUES
    ('전자제품', '전자 제품 카테고리'),
    ('사무용품', '사무용품 카테고리'),
    ('소프트웨어', '소프트웨어 제품');

-- 기본 계정 체계
INSERT INTO accounts (account_code, account_name, account_type) VALUES
    ('1000', '현금', 'asset'),
    ('1100', '매출채권', 'asset'),
    ('2000', '매입채무', 'liability'),
    ('3000', '자본금', 'equity'),
    ('4000', '매출', 'revenue'),
    ('5000', '매출원가', 'expense');
```

## 8. 백업 및 복구 전략

### 8.1 정기 백업

```sql
-- PostgreSQL 백업 스크립트
pg_dump -h localhost -U postgres -d clierp_db \
    --format=custom \
    --compress=9 \
    --file=backup_$(date +%Y%m%d_%H%M%S).dump

-- 데이터만 백업
pg_dump -h localhost -U postgres -d clierp_db \
    --data-only \
    --format=custom \
    --file=data_backup_$(date +%Y%m%d_%H%M%S).dump
```

### 8.2 복구 전략

```sql
-- 전체 복구
pg_restore -h localhost -U postgres -d clierp_db \
    --clean --if-exists \
    backup_20241201_120000.dump

-- 특정 테이블만 복구
pg_restore -h localhost -U postgres -d clierp_db \
    --table=employees \
    backup_20241201_120000.dump
```

## 9. 성능 모니터링

### 9.1 성능 뷰

```sql
-- 슬로우 쿼리 모니터링
CREATE VIEW slow_queries AS
SELECT
    query,
    calls,
    total_time,
    mean_time,
    rows
FROM pg_stat_statements
WHERE mean_time > 100
ORDER BY mean_time DESC;

-- 테이블 사용량 통계
CREATE VIEW table_stats AS
SELECT
    schemaname,
    tablename,
    n_tup_ins as inserts,
    n_tup_upd as updates,
    n_tup_del as deletes,
    n_live_tup as live_rows,
    n_dead_tup as dead_rows
FROM pg_stat_user_tables
ORDER BY live_rows DESC;
```

### 9.2 정기 유지보수

```sql
-- 자동 VACUUM 설정
ALTER TABLE transactions SET (
    autovacuum_vacuum_scale_factor = 0.1,
    autovacuum_analyze_scale_factor = 0.05
);

-- 정기 REINDEX (필요시)
REINDEX INDEX CONCURRENTLY idx_transactions_date;
```

## 10. 보안 고려사항

### 10.1 데이터 암호화

```sql
-- 민감한 데이터 암호화 (PostgreSQL)
CREATE EXTENSION IF NOT EXISTS pgcrypto;

-- 급여 정보 암호화 저장
UPDATE employees
SET salary = pgp_sym_encrypt(salary::text, 'encryption_key');
```

### 10.2 접근 제어

```sql
-- 역할별 권한 설정
CREATE ROLE hr_manager;
GRANT SELECT, INSERT, UPDATE ON employees TO hr_manager;
GRANT SELECT ON departments TO hr_manager;

CREATE ROLE accountant;
GRANT SELECT, INSERT, UPDATE ON transactions TO accountant;
GRANT SELECT ON accounts TO accountant;

-- 행 수준 보안 (RLS)
ALTER TABLE transactions ENABLE ROW LEVEL SECURITY;
CREATE POLICY transaction_access_policy ON transactions
    FOR ALL TO app_user
    USING (created_by = current_user_id());
```

이 데이터베이스 설계는 CLIERP 시스템의 확장성, 성능, 보안을 모두 고려하여 작성되었으며, 실제 구현 시 요구사항에 따라 세부 조정이 가능합니다.