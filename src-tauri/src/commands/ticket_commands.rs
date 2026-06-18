// Ticket commands — the frontend-facing surface (the "Ticket API").
// Each maps to a ticketsApi.* call in the frontend.

use crate::domain::Ticket;
use crate::service::ValidationResult;
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn ticket_list(state: State<'_, AppState>) -> Result<Vec<Ticket>, String> {
    state.tickets.list().await.map_err(Into::into)
}

#[tauri::command]
pub async fn ticket_get(state: State<'_, AppState>, id: i64) -> Result<Ticket, String> {
    state.tickets.get(id).await.map_err(Into::into)
}

#[tauri::command]
pub async fn ticket_create(
    state: State<'_, AppState>,
    valid_date: String,
    invoice_id: Option<i64>,
) -> Result<Ticket, String> {
    state
        .tickets
        .create(valid_date, invoice_id)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn ticket_update(
    state: State<'_, AppState>,
    id: i64,
    status: Option<String>,
    valid_date: Option<String>,
) -> Result<Ticket, String> {
    state
        .tickets
        .update(id, status, valid_date)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn ticket_delete(state: State<'_, AppState>, id: i64) -> Result<(), String> {
    state.tickets.delete(id).await.map_err(Into::into)
}

#[tauri::command]
pub async fn ticket_validate(
    state: State<'_, AppState>,
    ticket_code: String,
) -> Result<ValidationResult, String> {
    state.tickets.validate(ticket_code).await.map_err(Into::into)
}
