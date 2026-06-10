use sqlx::{Pool, Sqlite};

use crate::features::pings::{Ping, PingsError};

/// logs a status change if it's different from the latest log
pub async fn log_status_change(
    pool: &Pool<Sqlite>,
    watcher_id: i64,
    status: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now();

    let latest_log = sqlx::query!(
        "SELECT status FROM pings WHERE watcher_id = ? ORDER BY timestamp DESC LIMIT 1",
        watcher_id
    )
    .fetch_one(pool)
    .await?;

    if latest_log.status == status {
        return Ok(());
    }

    sqlx::query!(
        "INSERT INTO pings (watcher_id, status_code, timestamp) VALUES (?, ?, ?)",
        watcher_id,
        status,
        now,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub async fn get_status_changes(
    pool: &Pool<Sqlite>,
    watcher_id: i64,
) -> Result<Vec<Ping>, PingsError> {
    let result = sqlx::query_as!(
        Ping,
        "SELECT id, watcher_id, timestamp, status_code, status FROM pings WHERE watcher_id = ?",
        watcher_id
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(pings) => Ok(pings),
        Err(e) => Err(PingsError::Database(e)),
    }
}
