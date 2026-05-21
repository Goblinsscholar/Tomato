use tauri::State;

use crate::db;
use crate::state::AppState;

pub use crate::db::tags::TagRow;

#[tauri::command]
pub async fn create_tag(
    state: State<'_, AppState>,
    name: String,
) -> Result<bool, String> {
    let trimmed = name.trim().to_string();
    if trimmed.is_empty() {
        return Err("tag_name_empty".to_string());
    }
    let color = db::tags::assign_tag_color(&state.db, &trimmed).await;
    db::tags::insert_tag_if_not_exists(&state.db, &trimmed, &color)
        .await
        .map_err(|e| format!("db_error: {}", e))
}

#[tauri::command]
pub async fn delete_tag(
    state: State<'_, AppState>,
    name: String,
) -> Result<bool, String> {
    db::tags::delete_tag_by_name(&state.db, &name)
        .await
        .map_err(|e| format!("db_error: {}", e))
}

#[tauri::command]
pub async fn update_tag(
    state: State<'_, AppState>,
    current_name: String,
    new_name: String,
    new_color: String,
) -> Result<bool, String> {
    if new_name.trim().is_empty() {
        return Err("tag_name_empty".to_string());
    }
    db::tags::update_tag(&state.db, &current_name, &new_name.trim(), &new_color)
        .await
        .map_err(|e| format!("db_error: {}", e))
}

#[tauri::command]
pub async fn get_all_tags(
    state: State<'_, AppState>,
) -> Result<Vec<TagRow>, String> {
    db::tags::get_all_tags(&state.db)
        .await
        .map_err(|e| format!("db_error: {}", e))
}
