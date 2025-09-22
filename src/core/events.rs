use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::core::{error::CLIERPError, result::CLIERPResult};

/// Base trait for all domain events
pub trait DomainEvent: Send + Sync + std::fmt::Debug {
    fn event_type(&self) -> &'static str;
    fn entity_id(&self) -> String;
    fn organization_id(&self) -> i32;
    fn correlation_id(&self) -> Uuid;
    fn occurred_at(&self) -> DateTime<Utc>;
    fn event_data(&self) -> serde_json::Value;
}

/// Event handler trait
#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &dyn DomainEvent) -> CLIERPResult<()>;
    fn can_handle(&self, event_type: &str) -> bool;
}

/// Event bus for managing event publishing and handling
pub struct EventBus {
    handlers: Arc<Mutex<HashMap<String, Vec<Arc<dyn EventHandler>>>>>,
}

impl EventBus {
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// Register an event handler for specific event types
    pub fn register_handler<H: EventHandler + 'static>(
        &self,
        event_types: Vec<String>,
        handler: H,
    ) {
        let handler = Arc::new(handler);
        let mut handlers = self.handlers.lock().unwrap();

        for event_type in event_types {
            handlers
                .entry(event_type)
                .or_insert_with(Vec::new)
                .push(handler.clone());
        }
    }

    /// Publish an event to all registered handlers
    pub async fn publish(&self, event: &dyn DomainEvent) -> CLIERPResult<()> {
        let handlers = self.handlers.lock().unwrap();

        if let Some(event_handlers) = handlers.get(event.event_type()) {
            for handler in event_handlers {
                if handler.can_handle(event.event_type()) {
                    if let Err(e) = handler.handle(event).await {
                        tracing::error!(
                            "Event handler failed for event {}: {}",
                            event.event_type(),
                            e
                        );
                        // Continue processing other handlers even if one fails
                    }
                }
            }
        }

        Ok(())
    }
}

/// Base event implementation with common fields
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BaseEvent {
    pub event_type: String,
    pub entity_id: String,
    pub organization_id: i32,
    pub correlation_id: Uuid,
    pub occurred_at: DateTime<Utc>,
    pub event_data: serde_json::Value,
}

impl DomainEvent for BaseEvent {
    fn event_type(&self) -> &'static str {
        // This is a limitation - we'll need to use Box<str> or similar for dynamic types
        Box::leak(self.event_type.clone().into_boxed_str())
    }

    fn entity_id(&self) -> String {
        self.entity_id.clone()
    }

    fn organization_id(&self) -> i32 {
        self.organization_id
    }

    fn correlation_id(&self) -> Uuid {
        self.correlation_id
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        self.occurred_at
    }

    fn event_data(&self) -> serde_json::Value {
        self.event_data.clone()
    }
}

// Specific domain events

/// Inventory events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StockLevelChanged {
    pub product_id: i32,
    pub old_level: i32,
    pub new_level: i32,
    pub min_level: i32,
    pub organization_id: i32,
}

impl DomainEvent for StockLevelChanged {
    fn event_type(&self) -> &'static str {
        "inventory.stock_level_changed"
    }

    fn entity_id(&self) -> String {
        self.product_id.to_string()
    }

    fn organization_id(&self) -> i32 {
        self.organization_id
    }

    fn correlation_id(&self) -> Uuid {
        Uuid::new_v4() // In practice, this should be passed in
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn event_data(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LowStockAlert {
    pub product_id: i32,
    pub current_level: i32,
    pub min_level: i32,
    pub product_name: String,
    pub organization_id: i32,
}

impl DomainEvent for LowStockAlert {
    fn event_type(&self) -> &'static str {
        "inventory.low_stock_alert"
    }

    fn entity_id(&self) -> String {
        self.product_id.to_string()
    }

    fn organization_id(&self) -> i32 {
        self.organization_id
    }

    fn correlation_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn event_data(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

/// Purchase events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PurchaseOrderApproved {
    pub po_id: i32,
    pub po_number: String,
    pub supplier_id: i32,
    pub total_amount: i32,
    pub approved_by: i32,
    pub organization_id: i32,
}

impl DomainEvent for PurchaseOrderApproved {
    fn event_type(&self) -> &'static str {
        "purchase.order_approved"
    }

    fn entity_id(&self) -> String {
        self.po_id.to_string()
    }

    fn organization_id(&self) -> i32 {
        self.organization_id
    }

    fn correlation_id(&self) -> Uuid {
        Uuid::new_v4()
    }

    fn occurred_at(&self) -> DateTime<Utc> {
        Utc::now()
    }

    fn event_data(&self) -> serde_json::Value {
        serde_json::to_value(self).unwrap()
    }
}

// Event handlers

/// Low stock alert handler
pub struct LowStockAlertHandler {
    // In practice, this would have dependencies for sending notifications
}

#[async_trait]
impl EventHandler for LowStockAlertHandler {
    async fn handle(&self, event: &dyn DomainEvent) -> CLIERPResult<()> {
        if let Ok(low_stock_event) = serde_json::from_value::<LowStockAlert>(event.event_data()) {
            tracing::warn!(
                "Low stock alert: Product {} ({}) has {} units remaining (minimum: {})",
                low_stock_event.product_name,
                low_stock_event.product_id,
                low_stock_event.current_level,
                low_stock_event.min_level
            );

            // Here you would:
            // 1. Send email notifications to purchasing team
            // 2. Create automatic reorder requests
            // 3. Update dashboard alerts
            // 4. Log to external monitoring systems
        }
        Ok(())
    }

    fn can_handle(&self, event_type: &str) -> bool {
        event_type == "inventory.low_stock_alert"
    }
}

/// Purchase order notification handler
pub struct PurchaseOrderNotificationHandler;

#[async_trait]
impl EventHandler for PurchaseOrderNotificationHandler {
    async fn handle(&self, event: &dyn DomainEvent) -> CLIERPResult<()> {
        if let Ok(po_event) = serde_json::from_value::<PurchaseOrderApproved>(event.event_data()) {
            tracing::info!(
                "Purchase order {} approved for ₩{}",
                po_event.po_number,
                po_event.total_amount
            );

            // Notify supplier, update inventory expectations, etc.
        }
        Ok(())
    }

    fn can_handle(&self, event_type: &str) -> bool {
        event_type == "purchase.order_approved"
    }
}

/// Global event bus instance
lazy_static::lazy_static! {
    pub static ref GLOBAL_EVENT_BUS: EventBus = {
        let bus = EventBus::new();

        // Register default handlers
        bus.register_handler(
            vec!["inventory.low_stock_alert".to_string()],
            LowStockAlertHandler {},
        );

        bus.register_handler(
            vec!["purchase.order_approved".to_string()],
            PurchaseOrderNotificationHandler,
        );

        bus
    };
}

/// Convenience macro for publishing events
#[macro_export]
macro_rules! publish_event {
    ($event:expr) => {
        if let Err(e) = crate::core::events::GLOBAL_EVENT_BUS.publish(&$event).await {
            tracing::error!("Failed to publish event: {}", e);
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_event_publishing() {
        let bus = EventBus::new();
        let event = LowStockAlert {
            product_id: 1,
            current_level: 5,
            min_level: 10,
            product_name: "Test Product".to_string(),
            organization_id: 1,
        };

        // Should not fail even with no handlers
        assert!(bus.publish(&event).await.is_ok());
    }

    #[tokio::test]
    async fn test_handler_registration() {
        let bus = EventBus::new();
        bus.register_handler(
            vec!["test.event".to_string()],
            LowStockAlertHandler {},
        );

        // Test that handler is registered and can be called
        let event = BaseEvent {
            event_type: "test.event".to_string(),
            entity_id: "1".to_string(),
            organization_id: 1,
            correlation_id: Uuid::new_v4(),
            occurred_at: Utc::now(),
            event_data: serde_json::Value::Null,
        };

        assert!(bus.publish(&event).await.is_ok());
    }
}