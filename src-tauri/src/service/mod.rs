// Service layer: application use-cases. Orchestrates repositories + domain
// rules. Commands call into here; this layer never touches Tauri or SQL
// directly (SQL is behind the repository).

pub mod category_service;
pub mod ticket_service;
pub mod unit_service;

pub use category_service::CategoryService;
pub use ticket_service::{TicketService, ValidationResult};
pub use unit_service::UnitService;
