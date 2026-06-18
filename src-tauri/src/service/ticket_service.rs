// Ticket use-cases. This is where create/list/update/delete/validate are
// orchestrated using the repository (storage) and domain (rules).

use crate::domain::{Ticket, ValidationOutcome};
use crate::error::AppResult;
use crate::repository::TicketRepository;
use serde::Serialize;
use uuid::Uuid;

/// Result returned to the caller after validating a ticket for entry.
#[derive(Debug, Serialize)]
pub struct ValidationResult {
    pub valid: bool,
    pub reason: String,
    pub ticket_code: Option<String>,
    pub valid_date: Option<String>,
}

#[derive(Clone)]
pub struct TicketService {
    repo: TicketRepository,
}

impl TicketService {
    pub fn new(repo: TicketRepository) -> Self {
        Self { repo }
    }

    pub async fn list(&self) -> AppResult<Vec<Ticket>> {
        self.repo.list().await
    }

    pub async fn get(&self, id: i64) -> AppResult<Ticket> {
        self.repo
            .find_by_id(id)
            .await?
            .ok_or(crate::error::AppError::NotFound("ticket"))
    }

    /// Issue a new ticket with a unique, stable code.
    pub async fn create(&self, valid_date: String, invoice_id: Option<i64>) -> AppResult<Ticket> {
        if valid_date.trim().is_empty() {
            return Err(crate::error::AppError::Validation(
                "valid_date is required".into(),
            ));
        }
        let code = format!("TKT-{}", Uuid::new_v4().simple());
        self.repo.insert(&code, invoice_id, &valid_date).await
    }

    pub async fn update(
        &self,
        id: i64,
        status: Option<String>,
        valid_date: Option<String>,
    ) -> AppResult<Ticket> {
        self.repo
            .update(id, status.as_deref(), valid_date.as_deref())
            .await
    }

    pub async fn delete(&self, id: i64) -> AppResult<()> {
        self.repo.delete(id).await
    }

    /// Validate a ticket for entry. Reads the ticket, applies the pure domain
    /// rule, and on success marks it used so it can't enter twice.
    pub async fn validate(&self, ticket_code: String) -> AppResult<ValidationResult> {
        let ticket = match self.repo.find_by_code(&ticket_code).await? {
            Some(t) => t,
            None => {
                return Ok(ValidationResult {
                    valid: false,
                    reason: "Ticket not found".into(),
                    ticket_code: None,
                    valid_date: None,
                })
            }
        };

        let today = self.repo.today().await?;

        match ticket.evaluate_entry(&today) {
            ValidationOutcome::Denied(reason) => Ok(ValidationResult {
                valid: false,
                reason,
                ticket_code: Some(ticket.ticket_code),
                valid_date: Some(ticket.valid_date),
            }),
            ValidationOutcome::Granted => {
                let used = self.repo.mark_used(ticket.id).await?;
                Ok(ValidationResult {
                    valid: true,
                    reason: "Entry granted".into(),
                    ticket_code: Some(used.ticket_code),
                    valid_date: Some(used.valid_date),
                })
            }
        }
    }
}
