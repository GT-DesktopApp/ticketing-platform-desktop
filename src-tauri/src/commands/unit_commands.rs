// Unit commands — frontend-facing surface for the Units admin module.

use crate::domain::{Page, PageRequest, Unit};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn unit_list(
    state: State<'_, AppState>,
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    active_only: Option<bool>,
) -> Result<Page<Unit>, String> {
    let req = PageRequest::new(page, per_page, search, active_only);
    state.units.list(req).await.map_err(Into::into)
}

#[tauri::command]
pub async fn unit_create(
    state: State<'_, AppState>,
    unit_name: String,
    unit_code: String,
) -> Result<Unit, String> {
    state
        .units
        .create(unit_name, unit_code)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn unit_update(
    state: State<'_, AppState>,
    id: i64,
    unit_name: Option<String>,
    unit_code: Option<String>,
) -> Result<Unit, String> {
    state
        .units
        .update(id, unit_name, unit_code)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn unit_set_active(
    state: State<'_, AppState>,
    id: i64,
    is_active: bool,
) -> Result<Unit, String> {
    state.units.set_active(id, is_active).await.map_err(Into::into)
}

#[tauri::command]
pub async fn unit_delete(state: State<'_, AppState>, id: i64) -> Result<Unit, String> {
    state.units.delete(id).await.map_err(Into::into)
}
