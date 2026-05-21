use chrono::{NaiveDate, Utc, Datelike};
use serde::{Deserialize, Serialize};
use sqlx::Row;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct WeeklySummaryRow {
    pub date: String,
    pub total_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TagStatRow {
    pub tag: String,
    pub total_seconds: i64,
    pub session_count: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MonthlyDayRow {
    pub date: String,
    pub session_count: i64,
    pub total_seconds: i64,
}

#[tauri::command]
pub async fn get_weekly_stats(
    state: State<'_, AppState>,
    start_date: Option<String>,
) -> Result<Vec<WeeklySummaryRow>, String> {
    // Default to this Monday
    let monday = match start_date {
        Some(ref d) => NaiveDate::parse_from_str(d, "%Y-%m-%d")
            .unwrap_or_else(|_| get_monday()),
        None => get_monday(),
    };

    let rows = sqlx::query(
        r#"SELECT date(start_time) as date, COALESCE(SUM(actual_duration), 0) as total_seconds
         FROM sessions
         WHERE type = 'focus' AND actual_duration IS NOT NULL
           AND date(start_time) >= ? AND date(start_time) < ?
         GROUP BY date(start_time)
         ORDER BY date"#,
    )
    .bind(monday.format("%Y-%m-%d").to_string())
    .bind((monday + chrono::Duration::days(7)).format("%Y-%m-%d").to_string())
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("db_error: {}", e))?;

    // Build a lookup map from query results
    let mut data: std::collections::HashMap<String, i64> = rows
        .iter()
        .map(|r| {
            let date: String = r.get(0);
            let seconds: i64 = r.get(1);
            (date, seconds)
        })
        .collect();

    // Fill all 7 days, defaulting missing days to 0
    let mut result = Vec::with_capacity(7);
    for i in 0..7 {
        let day = monday + chrono::Duration::days(i);
        let date_str = day.format("%Y-%m-%d").to_string();
        let total_seconds = data.remove(&date_str).unwrap_or(0);
        result.push(WeeklySummaryRow {
            date: date_str,
            total_seconds,
        });
    }

    Ok(result)
}

#[tauri::command]
pub async fn get_tag_distribution(
    state: State<'_, AppState>,
    start_date: Option<String>,
    end_date: Option<String>,
) -> Result<Vec<TagStatRow>, String> {
    let has_start = start_date.is_some();
    let has_end = end_date.is_some();

    let rows = if has_start && has_end {
        let start = start_date.unwrap();
        let end = end_date.unwrap();
        sqlx::query(
            r#"SELECT tag, COALESCE(SUM(actual_duration), 0) as total_seconds, COUNT(*) as session_count
             FROM sessions
             WHERE type = 'focus' AND actual_duration IS NOT NULL
               AND date(start_time) >= ? AND date(start_time) <= ?
             GROUP BY tag
             ORDER BY total_seconds DESC"#,
        )
        .bind(&start)
        .bind(&end)
        .fetch_all(&state.db)
        .await
        .map_err(|e| format!("db_error: {}", e))?
    } else {
        sqlx::query(
            r#"SELECT tag, COALESCE(SUM(actual_duration), 0) as total_seconds, COUNT(*) as session_count
             FROM sessions
             WHERE type = 'focus' AND actual_duration IS NOT NULL
             GROUP BY tag
             ORDER BY total_seconds DESC"#,
        )
        .fetch_all(&state.db)
        .await
        .map_err(|e| format!("db_error: {}", e))?
    };

    Ok(rows
        .iter()
        .map(|r| TagStatRow {
            tag: r.get(0),
            total_seconds: r.get(1),
            session_count: r.get(2),
        })
        .collect())
}

#[tauri::command]
pub async fn get_monthly_stats(
    state: State<'_, AppState>,
    year: Option<i32>,
    month: Option<u32>,
) -> Result<Vec<MonthlyDayRow>, String> {
    let now = Utc::now();
    let y = year.unwrap_or_else(|| now.format("%Y").to_string().parse().unwrap_or(2026));
    let m = month.unwrap_or_else(|| now.month());
    let ym = format!("{:04}-{:02}", y, m);

    let rows = sqlx::query(
        r#"SELECT date(start_time) as date, COUNT(*) as session_count, COALESCE(SUM(actual_duration), 0) as total_seconds
         FROM sessions
         WHERE type = 'focus' AND actual_duration IS NOT NULL AND strftime('%Y-%m', start_time) = ?
         GROUP BY date(start_time)
         ORDER BY date"#,
    )
    .bind(&ym)
    .fetch_all(&state.db)
    .await
    .map_err(|e| format!("db_error: {}", e))?;

    Ok(rows
        .iter()
        .map(|r| MonthlyDayRow {
            date: r.get(0),
            session_count: r.get(1),
            total_seconds: r.get(2),
        })
        .collect())
}

fn get_monday() -> NaiveDate {
    let today = Utc::now().date_naive();
    let weekday = today.format("%u").to_string().parse::<u32>().unwrap_or(1);
    today - chrono::Duration::days((weekday - 1) as i64)
}
