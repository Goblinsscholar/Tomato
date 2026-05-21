pub mod sessions;
pub mod settings;
pub mod tags;
pub mod timer_state;

use sqlx::SqlitePool;

pub async fn init_db(pool: &SqlitePool) -> Result<(), sqlx::Error> {
    sqlx::query("PRAGMA journal_mode=WAL;")
        .execute(pool)
        .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS sessions (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            tag TEXT NOT NULL DEFAULT 'default',
            start_time TEXT NOT NULL,
            end_time TEXT,
            planned_duration INTEGER NOT NULL,
            actual_duration INTEGER,
            type TEXT NOT NULL DEFAULT 'focus' CHECK(type IN ('focus', 'break')),
            created_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS timer_state (
            id INTEGER PRIMARY KEY CHECK(id = 1),
            state TEXT NOT NULL,
            target_end_time TEXT,
            started_at TEXT,
            paused_remaining_seconds INTEGER,
            paused_elapsed_seconds INTEGER,
            tag TEXT DEFAULT '',
            session_id INTEGER,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS tags (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL UNIQUE,
            color TEXT NOT NULL DEFAULT '#6366f1'
        );
        "#,
    )
    .execute(pool)
    .await?;

    // Default settings
    let defaults = [
        ("focus_duration", "25"),
        ("break_duration", "5"),
        ("long_break_duration", "15"),
        ("sessions_before_long_break", "4"),
        ("auto_start_break", "true"),
        ("auto_start_focus", "true"),
        ("theme", "dark"),
        ("notifications_enabled", "true"),
    ];

    for (key, value) in defaults {
        sqlx::query("INSERT OR IGNORE INTO settings (key, value) VALUES (?, ?)")
            .bind(key)
            .bind(value)
            .execute(pool)
            .await?;
    }

    Ok(())
}
