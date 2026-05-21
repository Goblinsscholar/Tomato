use std::collections::HashMap;

use sqlx::{Row, SqlitePool};

pub async fn get_all(pool: &SqlitePool) -> Result<HashMap<String, String>, sqlx::Error> {
    let rows = sqlx::query("SELECT key, value FROM settings")
        .fetch_all(pool)
        .await?;

    let map = rows
        .into_iter()
        .map(|row| {
            let key: String = row.get(0);
            let value: String = row.get(1);
            (key, value)
        })
        .collect();
    Ok(map)
}

pub async fn get(pool: &SqlitePool, key: &str) -> Result<Option<String>, sqlx::Error> {
    let row = sqlx::query("SELECT value FROM settings WHERE key = ?")
        .bind(key)
        .fetch_optional(pool)
        .await?;
    Ok(row.map(|r| r.get(0)))
}

pub async fn set(pool: &SqlitePool, key: &str, value: &str) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT OR REPLACE INTO settings (key, value) VALUES (?, ?)")
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
    Ok(())
}
