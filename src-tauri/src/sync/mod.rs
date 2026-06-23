// Sync layer: durable transactional outbox + background drain + nightly safety
// net (reconciliation + snapshot). The queue is the source of truth for what is
// unsent — there is NO timestamp-diff anywhere.

pub mod client;
pub mod identity;
pub mod nightly;
pub mod outbox;
pub mod worker;

pub use identity::Identity;
