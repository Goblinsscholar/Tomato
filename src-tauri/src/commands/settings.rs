use serde::{Deserialize, Serialize};
use tauri::State;

use crate::db;
use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AppSettingsResponse {
    pub focus_duration: i32,
    pub break_duration: i32,
    pub long_break_duration: i32,
    pub sessions_before_long_break: i32,
    pub auto_start_break: bool,
    pub auto_start_focus: bool,
    pub theme: String,
}

fn default_settings() -> AppSettingsResponse {
    AppSettingsResponse {
        focus_duration: 25,
        break_duration: 5,
        long_break_duration: 15,
        sessions_before_long_break: 4,
        auto_start_break: true,
        auto_start_focus: true,
        theme: "dark".to_string(),
    }
}

fn map_settings(map: &std::collections::HashMap<String, String>) -> AppSettingsResponse {
    let def = default_settings();
    AppSettingsResponse {
        focus_duration: map
            .get("focus_duration")
            .and_then(|v| v.parse().ok())
            .unwrap_or(def.focus_duration),
        break_duration: map
            .get("break_duration")
            .and_then(|v| v.parse().ok())
            .unwrap_or(def.break_duration),
        long_break_duration: map
            .get("long_break_duration")
            .and_then(|v| v.parse().ok())
            .unwrap_or(def.long_break_duration),
        sessions_before_long_break: map
            .get("sessions_before_long_break")
            .and_then(|v| v.parse().ok())
            .unwrap_or(def.sessions_before_long_break),
        auto_start_break: map
            .get("auto_start_break")
            .map(|v| v == "true")
            .unwrap_or(def.auto_start_break),
        auto_start_focus: map
            .get("auto_start_focus")
            .map(|v| v == "true")
            .unwrap_or(def.auto_start_focus),
        theme: {
            let t = map.get("theme").cloned().unwrap_or(def.theme);
            if t == "system" { "dark".to_string() } else { t }
        },
    }
}

const VALID_KEYS: &[&str] = &[
    "focus_duration",
    "break_duration",
    "long_break_duration",
    "sessions_before_long_break",
    "auto_start_break",
    "auto_start_focus",
    "theme",
];

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppSettingsResponse, String> {
    let map = state
        .settings_cache
        .read()
        .map_err(|e| format!("lock_error: {}", e))?
        .clone();
    Ok(map_settings(&map))
}

#[tauri::command]
pub async fn update_setting(
    state: State<'_, AppState>,
    key: String,
    value: String,
) -> Result<(), String> {
    if !VALID_KEYS.contains(&key.as_str()) {
        return Err("unknown_setting_key".to_string());
    }

    // Validate value type
    match key.as_str() {
        "focus_duration"
        | "break_duration"
        | "long_break_duration"
        | "sessions_before_long_break" => {
            let num: i32 = value.parse().map_err(|_| "invalid_value".to_string())?;
            if num < 1 {
                return Err("invalid_value".to_string());
            }
        }
        "auto_start_break" | "auto_start_focus" => {
            if value != "true" && value != "false" {
                return Err("invalid_value".to_string());
            }
        }
        "theme" => {
            if !["light", "dark"].contains(&value.as_str()) {
                return Err("invalid_value".to_string());
            }
        }
        _ => return Err("unknown_setting_key".to_string()),
    }

    db::settings::set(&state.db, &key, &value)
        .await
        .map_err(|e| format!("db_error: {}", e))?;

    // Update in-memory cache
    if let Ok(mut cache) = state.settings_cache.write() {
        cache.insert(key, value);
    }

    Ok(())
}
