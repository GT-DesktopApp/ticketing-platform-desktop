// Ticket entity + validation rules (pure domain logic).

use serde::{Deserialize, Serialize};

/// The lifecycle states a ticket can be in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum TicketStatus {
    Active,
    Used,
    Cancelled,
}

impl TicketStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TicketStatus::Active => "active",
            TicketStatus::Used => "used",
            TicketStatus::Cancelled => "cancelled",
        }
    }
}

/// A ticket as stored and exchanged. Serialised with snake_case fields so it
/// matches the TypeScript `Ticket` type exactly.
#[derive(Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct Ticket {
    pub id: i64,
    pub ticket_code: String,
    pub invoice_id: Option<i64>,
    pub valid_date: String,
    pub status: String,
    pub used_at: Option<String>,
    pub created_at: String,
}

/// Result of validating a ticket for entry — the pure decision, before any
/// state change is persisted.
pub enum ValidationOutcome {
    Granted,
    Denied(String),
}

impl Ticket {
    /// Decide whether this ticket may enter on `today` ("YYYY-MM-DD").
    /// Pure: takes the relevant facts, returns a decision; no DB writes.
    pub fn evaluate_entry(&self, today: &str) -> ValidationOutcome {
        match self.status.as_str() {
            "cancelled" => ValidationOutcome::Denied("Ticket has been cancelled".into()),
            "used" => ValidationOutcome::Denied(format!(
                "Already used at {}",
                self.used_at.clone().unwrap_or_default()
            )),
            _ if self.valid_date != today => {
                ValidationOutcome::Denied(format!("Not valid today (valid on {})", self.valid_date))
            }
            _ => ValidationOutcome::Granted,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn ticket(status: &str, valid_date: &str, used_at: Option<&str>) -> Ticket {
        Ticket {
            id: 1,
            ticket_code: "TKT-1".into(),
            invoice_id: None,
            valid_date: valid_date.into(),
            status: status.into(),
            used_at: used_at.map(|s| s.into()),
            created_at: "2026-01-01 00:00:00".into(),
        }
    }

    #[test]
    fn grants_active_ticket_valid_today() {
        assert!(matches!(
            ticket("active", "2026-06-15", None).evaluate_entry("2026-06-15"),
            ValidationOutcome::Granted
        ));
    }

    #[test]
    fn denies_cancelled() {
        assert!(matches!(
            ticket("cancelled", "2026-06-15", None).evaluate_entry("2026-06-15"),
            ValidationOutcome::Denied(_)
        ));
    }

    #[test]
    fn denies_already_used() {
        assert!(matches!(
            ticket("used", "2026-06-15", Some("2026-06-15 09:00")).evaluate_entry("2026-06-15"),
            ValidationOutcome::Denied(_)
        ));
    }

    #[test]
    fn denies_wrong_day() {
        assert!(matches!(
            ticket("active", "2026-06-20", None).evaluate_entry("2026-06-15"),
            ValidationOutcome::Denied(_)
        ));
    }
}
