# Enterprise Migration Plan for CLIERP

## Phase 1: Database Architecture Overhaul

### 1.1 PostgreSQL Migration
**Current Issue**: SQLite is unsuitable for enterprise scale
**Solution**: Migrate to PostgreSQL with proper connection pooling

```rust
// New Cargo.toml dependencies
diesel = { version = "2.1", features = ["postgres", "chrono", "uuid", "r2d2"] }
diesel-async = "0.4"
bb8 = "0.8"
bb8-postgres = "0.8"
```

### 1.2 Enhanced Schema Design
**Add to schema.rs**:

```sql
-- Multi-tenancy support
CREATE TABLE organizations (
    id SERIAL PRIMARY KEY,
    code VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    settings JSONB DEFAULT '{}',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Master data management
CREATE TABLE system_configurations (
    id SERIAL PRIMARY KEY,
    organization_id INTEGER REFERENCES organizations(id),
    config_key VARCHAR(100) NOT NULL,
    config_value JSONB NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(organization_id, config_key)
);

-- Enhanced audit with context
CREATE TABLE enhanced_audit_logs (
    id BIGSERIAL PRIMARY KEY,
    organization_id INTEGER REFERENCES organizations(id),
    user_id INTEGER REFERENCES users(id),
    session_id UUID,
    table_name VARCHAR(100) NOT NULL,
    record_id BIGINT NOT NULL,
    action VARCHAR(20) NOT NULL,
    old_values JSONB,
    new_values JSONB,
    ip_address INET,
    user_agent TEXT,
    reason TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Workflow engine
CREATE TABLE workflow_definitions (
    id SERIAL PRIMARY KEY,
    organization_id INTEGER REFERENCES organizations(id),
    name VARCHAR(255) NOT NULL,
    entity_type VARCHAR(100) NOT NULL,
    definition JSONB NOT NULL,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMP DEFAULT NOW()
);

CREATE TABLE workflow_instances (
    id BIGSERIAL PRIMARY KEY,
    workflow_definition_id INTEGER REFERENCES workflow_definitions(id),
    entity_id BIGINT NOT NULL,
    current_step VARCHAR(100),
    status VARCHAR(50) DEFAULT 'pending',
    data JSONB DEFAULT '{}',
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);
```

## Phase 2: Transaction Management and Data Integrity

### 2.1 Service Layer with Transactions
```rust
// src/core/service.rs
use diesel::connection::TransactionManager;
use diesel::Connection;

pub trait TransactionalService {
    type Connection: Connection;

    fn with_transaction<T, F>(&self, f: F) -> Result<T, CLIERPError>
    where
        F: FnOnce(&mut Self::Connection) -> Result<T, CLIERPError>;
}

// Example usage in purchase orders
impl PurchaseOrderService {
    pub fn receive_items_transactional(
        &self,
        po_id: i32,
        items: Vec<ReceiveItemData>,
        user_id: i32,
    ) -> CLIERPResult<PurchaseOrder> {
        self.with_transaction(|conn| {
            // 1. Update purchase order status
            // 2. Update individual items
            // 3. Update inventory stock
            // 4. Create stock movements
            // 5. Create audit entries
            // All or nothing - proper ACID compliance
        })
    }
}
```

### 2.2 Domain Events System
```rust
// src/core/events.rs
pub trait DomainEvent: Send + Sync {
    fn event_type(&self) -> &'static str;
    fn entity_id(&self) -> i64;
    fn organization_id(&self) -> i32;
}

pub struct EventBus {
    handlers: HashMap<String, Vec<Box<dyn EventHandler>>>,
}

// Example: Stock level warnings
#[derive(Debug)]
pub struct StockLevelChanged {
    pub product_id: i32,
    pub old_level: i32,
    pub new_level: i32,
    pub min_level: i32,
}

impl DomainEvent for StockLevelChanged {
    fn event_type(&self) -> &'static str { "stock_level_changed" }
    fn entity_id(&self) -> i64 { self.product_id as i64 }
}
```

## Phase 3: Enhanced Security and Authorization

### 3.1 Role-Based Access Control (RBAC)
```sql
-- Enhanced RBAC schema
CREATE TABLE permissions (
    id SERIAL PRIMARY KEY,
    name VARCHAR(100) UNIQUE NOT NULL,
    resource VARCHAR(100) NOT NULL,
    action VARCHAR(50) NOT NULL,
    description TEXT
);

CREATE TABLE roles (
    id SERIAL PRIMARY KEY,
    organization_id INTEGER REFERENCES organizations(id),
    name VARCHAR(100) NOT NULL,
    description TEXT,
    is_system_role BOOLEAN DEFAULT false,
    UNIQUE(organization_id, name)
);

CREATE TABLE role_permissions (
    role_id INTEGER REFERENCES roles(id),
    permission_id INTEGER REFERENCES permissions(id),
    PRIMARY KEY(role_id, permission_id)
);

CREATE TABLE user_roles (
    user_id INTEGER REFERENCES users(id),
    role_id INTEGER REFERENCES roles(id),
    granted_by INTEGER REFERENCES users(id),
    granted_at TIMESTAMP DEFAULT NOW(),
    PRIMARY KEY(user_id, role_id)
);
```

### 3.2 Data-Level Security
```rust
// src/core/security.rs
pub struct SecurityContext {
    pub user_id: i32,
    pub organization_id: i32,
    pub roles: Vec<String>,
    pub permissions: Vec<Permission>,
}

pub trait SecureRepository<T> {
    fn find_authorized(&self, ctx: &SecurityContext, filters: &FilterOptions) -> Result<Vec<T>, CLIERPError>;
    fn can_access(&self, ctx: &SecurityContext, entity_id: i32) -> bool;
}
```

## Phase 4: CLI Experience Improvements

### 4.1 Interactive Command Mode
```rust
// src/cli/interactive.rs
pub struct InteractiveMode {
    session: SessionManager,
    context: CommandContext,
}

impl InteractiveMode {
    pub async fn start(&mut self) -> CLIERPResult<()> {
        println!("CLIERP Interactive Mode - Type 'help' for commands");

        loop {
            let input = self.read_input()?;
            match self.parse_and_execute(input).await {
                Ok(result) => self.display_result(result),
                Err(e) => self.display_error(e),
            }
        }
    }

    pub fn create_purchase_order_wizard(&self) -> CLIERPResult<()> {
        // Step-by-step wizard for complex operations
        let supplier = self.select_supplier()?;
        let items = self.add_items_interactive()?;
        let po = self.confirm_and_create(supplier, items)?;
        println!("✅ Purchase Order {} created successfully!", po.po_number);
        Ok(())
    }
}
```

### 4.2 Bulk Operations Support
```rust
// src/cli/bulk.rs
pub struct BulkOperations;

impl BulkOperations {
    pub async fn import_products_csv(&self, file_path: &str) -> CLIERPResult<BulkResult> {
        let mut reader = csv::Reader::from_path(file_path)?;
        let mut results = BulkResult::new();

        // Process in batches for memory efficiency
        for batch in reader.records().chunks(100) {
            let products: Vec<NewProduct> = batch
                .map(|record| self.parse_product_record(record))
                .collect::<Result<Vec<_>, _>>()?;

            let imported = ProductService::bulk_create(products)?;
            results.add_success(imported.len());
        }

        Ok(results)
    }
}
```

## Phase 5: Performance and Monitoring

### 5.1 Async Operations and Job Queue
```rust
// Cargo.toml additions
tokio = { version = "1.0", features = ["full"] }
sqlx = { version = "0.7", features = ["postgres", "runtime-tokio-rustls"] }
sidekiq = "0.10"

// src/jobs/mod.rs
#[async_trait]
pub trait BackgroundJob: Send + Sync {
    async fn perform(&self, args: JobArgs) -> Result<(), JobError>;
}

pub struct StockReorderJob;

#[async_trait]
impl BackgroundJob for StockReorderJob {
    async fn perform(&self, args: JobArgs) -> Result<(), JobError> {
        // Check low stock items
        // Generate purchase orders
        // Send notifications
        Ok(())
    }
}
```

### 5.2 Metrics and Monitoring
```rust
// src/monitoring/metrics.rs
use prometheus::{Counter, Histogram, Registry};

pub struct AppMetrics {
    pub command_executions: Counter,
    pub command_duration: Histogram,
    pub database_connections: Gauge,
}

impl AppMetrics {
    pub fn new() -> Self {
        Self {
            command_executions: Counter::new("cli_commands_total", "Total CLI commands executed")?,
            command_duration: Histogram::new("cli_command_duration_seconds", "CLI command execution time")?,
            database_connections: Gauge::new("database_connections_active", "Active database connections")?,
        }
    }
}
```

## Implementation Timeline

**Phase 1 (Weeks 1-2)**: Database migration and multi-tenancy
**Phase 2 (Weeks 3-4)**: Transaction management and events
**Phase 3 (Weeks 5-6)**: Enhanced security and RBAC
**Phase 4 (Weeks 7-8)**: CLI improvements and bulk operations
**Phase 5 (Weeks 9-10)**: Performance optimization and monitoring

## Risk Mitigation

1. **Data Migration**: Implement dual-write pattern during transition
2. **Backward Compatibility**: Maintain existing CLI commands during migration
3. **Testing Strategy**: Comprehensive integration tests for each phase
4. **Rollback Plan**: Database snapshots and feature flags for quick rollback