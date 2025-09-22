use diesel::prelude::*;
use diesel::result::Error as DieselError;
use crate::core::{error::CLIERPError, result::CLIERPResult};
use crate::database::connection::get_connection;

/// Transaction management trait for ensuring ACID compliance
pub trait Transactional {
    /// Execute a function within a database transaction
    /// Automatically rolls back on error, commits on success
    fn with_transaction<T, F>(&self, f: F) -> CLIERPResult<T>
    where
        F: FnOnce(&mut SqliteConnection) -> CLIERPResult<T>;
}

/// Default transaction implementation
pub struct TransactionManager;

impl Transactional for TransactionManager {
    fn with_transaction<T, F>(&self, f: F) -> CLIERPResult<T>
    where
        F: FnOnce(&mut SqliteConnection) -> CLIERPResult<T>,
    {
        let mut conn = get_connection()?;

        conn.transaction(|conn| {
            match f(conn) {
                Ok(result) => Ok(result),
                Err(e) => {
                    tracing::error!("Transaction failed: {}", e);
                    Err(e)
                }
            }
        }).map_err(|e| match e {
            DieselError::RollbackTransaction => {
                CLIERPError::Transaction("Transaction was rolled back".to_string())
            }
            other => CLIERPError::Database(other),
        })
    }
}

/// Unit of Work pattern for complex operations
pub struct UnitOfWork {
    operations: Vec<Box<dyn UnitOfWorkOperation>>,
}

impl UnitOfWork {
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    pub fn add_operation<T: UnitOfWorkOperation + 'static>(&mut self, operation: T) {
        self.operations.push(Box::new(operation));
    }

    pub fn execute(self, conn: &mut SqliteConnection) -> CLIERPResult<()> {
        for operation in self.operations {
            operation.execute(conn)?;
        }
        Ok(())
    }
}

pub trait UnitOfWorkOperation {
    fn execute(&self, conn: &mut SqliteConnection) -> CLIERPResult<()>;
}

/// Example: Stock update operation with audit trail
pub struct StockUpdateOperation {
    pub product_id: i32,
    pub quantity_change: i32,
    pub reference_type: String,
    pub reference_id: i32,
    pub user_id: Option<i32>,
    pub notes: Option<String>,
}

impl UnitOfWorkOperation for StockUpdateOperation {
    fn execute(&self, conn: &mut SqliteConnection) -> CLIERPResult<()> {
        use crate::database::schema::{products, stock_movements};
        use crate::database::models::NewStockMovement;
        use diesel::prelude::*;
        use chrono::Utc;

        // Update product stock
        diesel::update(products::table.find(self.product_id))
            .set(products::current_stock.eq(products::current_stock + self.quantity_change))
            .execute(conn)?;

        // Create stock movement record
        let movement = NewStockMovement {
            product_id: self.product_id,
            movement_type: if self.quantity_change > 0 { "in".to_string() } else { "out".to_string() },
            quantity: self.quantity_change.abs(),
            unit_cost: None,
            reference_type: Some(self.reference_type.clone()),
            reference_id: Some(self.reference_id),
            notes: self.notes.clone(),
            moved_by: self.user_id,
        };

        diesel::insert_into(stock_movements::table)
            .values(&movement)
            .execute(conn)?;

        Ok(())
    }
}

/// Optimistic locking support
pub trait OptimisticLocking {
    type Id;

    fn get_version(&self) -> i32;
    fn update_with_version_check(
        &self,
        conn: &mut SqliteConnection,
        id: Self::Id,
        expected_version: i32
    ) -> CLIERPResult<()>;
}

/// Macro for generating version-aware update operations
#[macro_export]
macro_rules! implement_optimistic_locking {
    ($model:ty, $table:ident, $id_field:ident, $version_field:ident) => {
        impl OptimisticLocking for $model {
            type Id = i32;

            fn get_version(&self) -> i32 {
                self.$version_field
            }

            fn update_with_version_check(
                &self,
                conn: &mut SqliteConnection,
                id: Self::Id,
                expected_version: i32,
            ) -> CLIERPResult<()> {
                use diesel::prelude::*;

                let updated_rows = diesel::update(
                    $table::table
                        .filter($table::$id_field.eq(id))
                        .filter($table::$version_field.eq(expected_version))
                )
                .set((
                    // Set your fields here
                    $table::$version_field.eq(expected_version + 1),
                ))
                .execute(conn)?;

                if updated_rows == 0 {
                    return Err(CLIERPError::ConcurrencyError(
                        "Record was modified by another user".to_string()
                    ));
                }

                Ok(())
            }
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction_rollback_on_error() {
        // Test that transactions properly roll back on errors
    }

    #[test]
    fn test_unit_of_work_execution() {
        // Test that all operations in a unit of work execute atomically
    }
}