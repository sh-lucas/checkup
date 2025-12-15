use chrono;
use sqlx::{Pool, Sqlite};

/// logs a status change if it's different from the latest log
pub async fn log_status_change(
    poll: &Pool<Sqlite>,
    watcher_id: i64,
    status: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now();

    let latest_log = sqlx::query!(
        "SELECT status FROM pings WHERE watcher_id = ? ORDER BY timestamp DESC LIMIT 1",
        watcher_id
    )
    .fetch_one(poll)
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
    .execute(poll)
    .await?;

    Ok(())
}
