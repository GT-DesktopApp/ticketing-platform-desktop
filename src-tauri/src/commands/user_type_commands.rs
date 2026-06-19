// User type commands — frontend-facing surface for "Manage User Types".

use crate::domain::{Page, PageRequest, UserType};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn user_type_list(
    state: State<'_, AppState>,
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    active_only: Option<bool>,
) -> Result<Page<UserType>, String> {
    let req = PageRequest::new(page, per_page, search, active_only);
    state.user_types.list(req).await.map_err(Into::into)
}

#[tauri::command]
pub async fn user_type_create(
    state: State<'_, AppState>,
    name: String,
) -> Result<UserType, String> {
    state.user_types.create(name).await.map_err(Into::into)
}

#[tauri::command]
pub async fn user_type_update(
    state: State<'_, AppState>,
    id: i64,
    name: String,
) -> Result<UserType, String> {
    state.user_types.update(id, name).await.map_err(Into::into)
}

#[tauri::command]
pub async fn user_type_set_active(
    state: State<'_, AppState>,
    id: i64,
    is_active: bool,
) -> Result<UserType, String> {
    state
        .user_types
        .set_active(id, is_active)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn user_type_delete(
    state: State<'_, AppState>,
    id: i64,
) -> Result<UserType, String> {
    state.user_types.delete(id).await.map_err(Into::into)
}
