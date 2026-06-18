// Repository layer: the ONLY place that contains SQL. Services depend on these
// abstractions, never on sqlx directly, so storage can evolve in isolation.

pub mod category_repository;
pub mod ticket_repository;
pub mod unit_repository;

pub use category_repository::CategoryRepository;
pub use ticket_repository::TicketRepository;
pub use unit_repository::UnitRepository;
