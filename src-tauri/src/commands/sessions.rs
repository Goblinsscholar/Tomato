use chrono::{NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;

use crate::db;
use crate::state::AppState;

// Re-export SessionRow for frontend use
pub use crate::db::sessions::SessionRow;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DailyStatsResponse {
    pub total_focus_seconds: i64,
    pub total_break_seconds: i64,
    pub session_count: i64,
    pub streak_days: i32,
}

#[tauri::command]
pub async fn get_today_sessions(
    state: State<'_, AppState>,
) -> Result<Vec<SessionRow>, String> {
    db::sessions::get_today_sessions(&state.db)
        .await
        .map_err(|e| format!("db_error: {}", e))
}

#[tauri::command]
pub async fn get_sessions_in_range(
    state: State<'_, AppState>,
    start_date: String,
    end_date: String,
) -> Result<Vec<SessionRow>, String> {
    db::sessions::get_sessions_in_range(&state.db, &start_date, &end_date)
        .await
        .map_err(|e| format!("db_error: {}", e))
}

#[tauri::command]
pub async fn get_daily_stats(
    state: State<'_, AppState>,
    date: Option<String>,
) -> Result<DailyStatsResponse, String> {
    let date_str = date.unwrap_or_else(|| Utc::now().format("%Y-%m-%d").to_string());

    // Query today's aggregate stats
    let row = sqlx::query(
        r#"SELECT
            COALESCE(SUM(CASE WHEN type='focus' THEN actual_duration ELSE 0 END), 0),
            COALESCE(SUM(CASE WHEN type='break' THEN actual_duration ELSE 0 END), 0),
            COUNT(*)
         FROM sessions
         WHERE date(start_time) = ?"#,
    )
    .bind(&date_str)
    .fetch_one(&state.db)
    .await
    .map_err(|e| format!("db_error: {}", e))?;

    let total_focus: i64 = row.get(0);
    let total_break: i64 = row.get(1);
    let session_count: i64 = row.get(2);

    // Streak calculation: single query for all distinct focus dates, then in-memory walk
    let today = NaiveDate::parse_from_str(&date_str, "%Y-%m-%d")
        .unwrap_or_else(|_| Utc::now().date_naive());

    let focus_date_strings: Vec<String> = sqlx::query_scalar::<_, String>(
        r#"SELECT DISTINCT date(start_time)
           FROM sessions
           WHERE type='focus' AND actual_duration IS NOT NULL"#,
    )
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("db_error: {}", e))?;

    let focus_dates: std::collections::HashSet<NaiveDate> = focus_date_strings
        .iter()
        .filter_map(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").ok())
        .collect();

    let mut streak = 0i32;
    for offset in 0..365 {
        let check_date = today - chrono::Duration::days(offset as i64);
        if focus_dates.contains(&check_date) {
            streak += 1;
        } else {
            break;
        }
    }

    Ok(DailyStatsResponse {
        total_focus_seconds: total_focus,
        total_break_seconds: total_break,
        session_count,
        streak_days: streak,
    })
}
