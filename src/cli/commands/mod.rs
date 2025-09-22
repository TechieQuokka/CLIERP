pub mod auth;
pub mod crm;
pub mod crm_extended;
pub mod hr;
pub mod inventory;
pub mod purchase;
pub mod reports;
pub mod system;

// Re-export command implementations
pub use auth::*;
pub use crm::*;
pub use crm_extended::*;
pub use hr::*;
pub use inventory::*;
pub use purchase::*;
pub use reports::*;
pub use system::*;
