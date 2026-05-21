use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use sqlx::{Row, SqlitePool};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TagRow {
    pub id: i64,
    pub name: String,
    pub color: String,
}

/// 20 well-distinguishable colors (HSL-distributed for maximum contrast)
const ALL_COLORS: &[&str] = &[
    "#6366f1", // indigo
    "#22c55e", // green
    "#f59e0b", // amber
    "#ef4444", // red
    "#06b6d4", // cyan
    "#a855f7", // purple
    "#f97316", // orange
    "#ec4899", // pink
    "#14b8a6", // teal
    "#84cc16", // lime
    "#3b82f6", // blue
    "#d946ef", // fuchsia
    "#eab308", // yellow
    "#0ea5e9", // sky
    "#8b5cf6", // violet
    "#10b981", // emerald
    "#f43f5e", // rose
    "#2dd4bf", // light teal
    "#a3e635", // light lime
    "#38bdf8", // light blue
];

/// Deterministic color from tag name (fallback when DB is unavailable).
pub fn tag_color(name: &str) -> String {
    let hash: u64 = name.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    let idx = (hash as usize) % ALL_COLORS.len();
    ALL_COLORS[idx].to_string()
}

/// Assign a color to a new tag, picking the least-used color among existing tags
/// for maximum visual diversity. Falls back to deterministic hash on DB error.
pub async fn assign_tag_color(pool: &SqlitePool, name: &str) -> String {
    let existing = match get_all_tags(pool).await {
        Ok(tags) => tags,
        Err(_) => return tag_color(name),
    };

    // Count how many times each color is already used
    let mut usage: HashMap<&str, usize> = HashMap::new();
    for tag in &existing {
        *usage.entry(tag.color.as_str()).or_insert(0) += 1;
    }

    // Find the minimum usage count
    let min_usage = ALL_COLORS.iter()
        .map(|c| usage.get(c).copied().unwrap_or(0))
        .min()
        .unwrap_or(0);

    // Collect all colors tied at min_usage
    let candidates: Vec<&&str> = ALL_COLORS.iter()
        .filter(|c| usage.get(*c).copied().unwrap_or(0) == min_usage)
        .collect();

    // Deterministic tiebreaker from tag name
    let hash: u64 = name.bytes().fold(0u64, |acc, b| acc.wrapping_mul(31).wrapping_add(b as u64));
    let idx = (hash as usize) % candidates.len();
    candidates[idx].to_string()
}

/// Insert a tag if it doesn't already exist (name is UNIQUE).
/// Returns whether the tag was newly inserted.
pub async fn insert_tag_if_not_exists(pool: &SqlitePool, name: &str, color: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("INSERT OR IGNORE INTO tags (name, color) VALUES (?, ?)")
        .bind(name)
        .bind(color)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

/// Update an existing tag's name and/or color.
/// Returns true if a row was updated, false if no tag matched `current_name`.
pub async fn update_tag(
    pool: &SqlitePool,
    current_name: &str,
    new_name: &str,
    new_color: &str,
) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("UPDATE tags SET name = ?, color = ? WHERE name = ?")
        .bind(new_name)
        .bind(new_color)
        .bind(current_name)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn delete_tag_by_name(pool: &SqlitePool, name: &str) -> Result<bool, sqlx::Error> {
    let result = sqlx::query("DELETE FROM tags WHERE name = ?")
        .bind(name)
        .execute(pool)
        .await?;
    Ok(result.rows_affected() > 0)
}

pub async fn get_all_tags(pool: &SqlitePool) -> Result<Vec<TagRow>, sqlx::Error> {
    let rows = sqlx::query("SELECT id, name, color FROM tags ORDER BY name")
        .fetch_all(pool)
        .await?;
    Ok(rows
        .iter()
        .map(|r| TagRow {
            id: r.get(0),
            name: r.get(1),
            color: r.get(2),
        })
        .collect())
}
