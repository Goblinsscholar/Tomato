use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SessionRow {
    pub id: i64,
    pub tag: String,
    pub start_time: String,
    pub end_time: Option<String>,
    pub planned_duration: i32,
    pub actual_duration: Option<i32>,
    #[serde(rename = "type")]
    pub session_type: String,
    pub created_at: String,
}

pub async fn insert_session(
    pool: &SqlitePool,
    tag: &str,
    start_time: &str,
    planned_duration: i32,
    session_type: &str,
) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "INSERT INTO sessions (tag, start_time, planned_duration, type) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(tag)
    .bind(start_time)
    .bind(planned_duration)
    .bind(session_type)
    .fetch_one(pool)
    .await?;

    Ok(row.get::<i64, _>(0))
}

pub async fn complete_session(
    pool: &SqlitePool,
    id: i64,
    end_time: &str,
    actual_duration: i32,
) -> Result<(), sqlx::Error> {
    sqlx::query("UPDATE sessions SET end_time = ?, actual_duration = ? WHERE id = ?")
        .bind(end_time)
        .bind(actual_duration)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn cancel_session(pool: &SqlitePool, id: i64) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM sessions WHERE id = ? AND actual_duration IS NULL")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn get_today_sessions(pool: &SqlitePool) -> Result<Vec<SessionRow>, sqlx::Error> {
    let rows = sqlx::query(
        r#"SELECT id, tag, start_time, end_time, planned_duration, actual_duration, type, created_at
         FROM sessions
         WHERE date(start_time) = date('now')
         ORDER BY start_time DESC"#,
    )
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|r| SessionRow {
            id: r.get(0),
            tag: r.get(1),
            start_time: r.get(2),
            end_time: r.get(3),
            planned_duration: r.get(4),
            actual_duration: r.get(5),
            session_type: r.get(6),
            created_at: r.get(7),
        })
        .collect())
}

pub async fn get_sessions_in_range(
    pool: &SqlitePool,
    start_date: &str,
    end_date: &str,
) -> Result<Vec<SessionRow>, sqlx::Error> {
    let rows = sqlx::query(
        r#"SELECT id, tag, start_time, end_time, planned_duration, actual_duration, type, created_at
         FROM sessions
         WHERE date(start_time) >= ? AND date(start_time) <= ?
         ORDER BY start_time DESC"#,
    )
    .bind(start_date)
    .bind(end_date)
    .fetch_all(pool)
    .await?;

    Ok(rows
        .iter()
        .map(|r| SessionRow {
            id: r.get(0),
            tag: r.get(1),
            start_time: r.get(2),
            end_time: r.get(3),
            planned_duration: r.get(4),
            actual_duration: r.get(5),
            session_type: r.get(6),
            created_at: r.get(7),
        })
        .collect())
}

pub async fn get_focus_count_today(pool: &SqlitePool) -> Result<i64, sqlx::Error> {
    let row = sqlx::query(
        "SELECT COUNT(*) FROM sessions WHERE type = 'focus' AND date(start_time) = date('now')",
    )
    .fetch_one(pool)
    .await?;
    Ok(row.get::<i64, _>(0))
}
