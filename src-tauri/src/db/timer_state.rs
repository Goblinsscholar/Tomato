use chrono::{DateTime, Utc};
use sqlx::{Row, SqlitePool};

use crate::timer::TimerState;

pub async fn save_state(pool: &SqlitePool, state: &TimerState) -> Result<(), sqlx::Error> {
    let now = Utc::now().to_rfc3339();
    match state {
        TimerState::Idle => {
            sqlx::query(
                "INSERT OR REPLACE INTO timer_state (id, state, updated_at) VALUES (1, 'idle', ?)",
            )
            .bind(&now)
            .execute(pool)
            .await?;
        }
        TimerState::Focusing {
            target_end,
            started_at,
            tag,
            session_id,
        } => {
            sqlx::query(
                r#"INSERT OR REPLACE INTO timer_state
                   (id, state, target_end_time, started_at, tag, session_id, updated_at)
                   VALUES (1, 'focusing', ?, ?, ?, ?, ?)"#,
            )
            .bind(target_end.to_rfc3339())
            .bind(started_at.to_rfc3339())
            .bind(tag)
            .bind(session_id)
            .bind(&now)
            .execute(pool)
            .await?;
        }
        TimerState::Breaking {
            target_end,
            started_at,
            session_id,
            tag,
        } => {
            sqlx::query(
                r#"INSERT OR REPLACE INTO timer_state
                   (id, state, target_end_time, started_at, session_id, tag, updated_at)
                   VALUES (1, 'breaking', ?, ?, ?, ?, ?)"#,
            )
            .bind(target_end.to_rfc3339())
            .bind(started_at.to_rfc3339())
            .bind(session_id)
            .bind(tag.as_deref())
            .bind(&now)
            .execute(pool)
            .await?;
        }
        TimerState::Paused {
            paused_phase,
            remaining_seconds,
            elapsed_seconds,
            tag,
            session_id,
        } => {
            // Store phase info in state column as "paused:focusing" or "paused:breaking"
            let state_val = match paused_phase {
                crate::timer::PausedPhase::Focusing => "paused:focusing",
                crate::timer::PausedPhase::Breaking => "paused:breaking",
            };
            sqlx::query(
                r#"INSERT OR REPLACE INTO timer_state
                   (id, state, paused_remaining_seconds, paused_elapsed_seconds, tag, session_id, updated_at)
                   VALUES (1, ?, ?, ?, ?, ?, ?)"#,
            )
            .bind(state_val)
            .bind(remaining_seconds)
            .bind(elapsed_seconds)
            .bind(tag)
            .bind(session_id)
            .bind(&now)
            .execute(pool)
            .await?;
        }
    }
    Ok(())
}

pub async fn load_state(pool: &SqlitePool) -> Result<Option<TimerState>, sqlx::Error> {
    let row = sqlx::query(
        r#"SELECT state, target_end_time, started_at, paused_remaining_seconds,
                  paused_elapsed_seconds, tag, session_id
         FROM timer_state WHERE id = 1"#,
    )
    .fetch_optional(pool)
    .await?;

    match row {
        None => Ok(None),
        Some(row) => {
            let state_str: String = row.get(0);
            match state_str.as_str() {
                "idle" => Ok(Some(TimerState::Idle)),
                "focusing" => {
                    let target_end: String = row.get(1);
                    let started_at: String = row.get(2);
                    let tag: String = row.get(5);
                    let session_id: i64 = row.get(6);
                    Ok(Some(TimerState::Focusing {
                        target_end: target_end.parse::<DateTime<Utc>>().unwrap_or_default(),
                        started_at: started_at.parse::<DateTime<Utc>>().unwrap_or_default(),
                        tag,
                        session_id,
                    }))
                }
                "breaking" => {
                    let target_end: String = row.get(1);
                    let started_at: String = row.get(2);
                    let tag: Option<String> = row.get(5);
                    let session_id: i64 = row.get(6);
                    Ok(Some(TimerState::Breaking {
                        target_end: target_end.parse::<DateTime<Utc>>().unwrap_or_default(),
                        started_at: started_at.parse::<DateTime<Utc>>().unwrap_or_default(),
                        session_id,
                        tag,
                    }))
                }
                s if s.starts_with("paused:") => {
                    let paused_phase = match s.trim_start_matches("paused:") {
                        "focusing" => crate::timer::PausedPhase::Focusing,
                        "breaking" => crate::timer::PausedPhase::Breaking,
                        _ => crate::timer::PausedPhase::Focusing,
                    };
                    let remaining_seconds: i64 = row.get(3);
                    let elapsed_seconds: i64 = row.get(4);
                    let tag: String = row.get(5);
                    let session_id: i64 = row.get(6);
                    Ok(Some(TimerState::Paused {
                        paused_phase,
                        remaining_seconds,
                        elapsed_seconds,
                        tag,
                        session_id,
                    }))
                }
                _ => Ok(Some(TimerState::Idle)),
            }
        }
    }
}

pub async fn clear_state(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("DELETE FROM timer_state WHERE id = 1")
        .execute(pool)
        .await?;
    Ok(())
}
