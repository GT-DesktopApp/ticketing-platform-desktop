// Category commands — frontend-facing surface for the Categories admin module.
// Thin handlers: build a sanitised PageRequest, delegate to the service,
// convert errors to String at the edge.

use crate::domain::{Category, Page, PageRequest};
use crate::AppState;
use tauri::State;

#[tauri::command]
pub async fn category_list(
    state: State<'_, AppState>,
    page: Option<u32>,
    per_page: Option<u32>,
    search: Option<String>,
    active_only: Option<bool>,
) -> Result<Page<Category>, String> {
    let req = PageRequest::new(page, per_page, search, active_only);
    state.categories.list(req).await.map_err(Into::into)
}

#[tauri::command]
pub async fn category_create(
    state: State<'_, AppState>,
    name: String,
    hsn_code: Option<String>,
) -> Result<Category, String> {
    state
        .categories
        .create(name, hsn_code)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn category_update(
    state: State<'_, AppState>,
    id: i64,
    name: Option<String>,
    // Present (even as null) => set HSN; absent => leave unchanged.
    hsn_code: Option<Option<String>>,
) -> Result<Category, String> {
    state
        .categories
        .update(id, name, hsn_code)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn category_set_active(
    state: State<'_, AppState>,
    id: i64,
    is_active: bool,
) -> Result<Category, String> {
    state
        .categories
        .set_active(id, is_active)
        .await
        .map_err(Into::into)
}

#[tauri::command]
pub async fn category_delete(
    state: State<'_, AppState>,
    id: i64,
) -> Result<Category, String> {
    // Soft delete -> returns the now-inactive row.
    state.categories.delete(id).await.map_err(Into::into)
}
