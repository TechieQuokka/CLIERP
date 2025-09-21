pub mod auth;
pub mod hr;
pub mod inventory;
pub mod system;

// Re-export command implementations
pub use auth::*;
pub use hr::*;
pub use inventory::*;
pub use system::*;
