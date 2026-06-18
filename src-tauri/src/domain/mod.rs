// Domain layer: entities + pure business rules. No I/O, no SQL, no Tauri.

pub mod category;
pub mod pagination;
pub mod ticket;
pub mod unit;

pub use category::Category;
pub use pagination::{Page, PageRequest};
pub use ticket::{Ticket, TicketStatus, ValidationOutcome};
pub use unit::Unit;
