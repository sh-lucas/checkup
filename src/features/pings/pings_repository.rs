use sqlx::{Pool, Sqlite};

/// logs a status change if it's different from the latest log
pub async fn log_status_change(
    pool: &Pool<Sqlite>,
    watcher_id: i64,
    status_code: i64,
    status: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now();

    let latest_log = sqlx::query!(
        "SELECT status FROM pings WHERE watcher_id = ? ORDER BY timestamp DESC LIMIT 1",
        watcher_id
    )
    .fetch_optional(pool)
    .await?;

    if let Some(log) = latest_log
        && log.status == status
    {
        return Ok(());
    }

    sqlx::query!(
        "INSERT INTO pings (watcher_id, status_code, status, timestamp) VALUES (?, ?, ?, ?)",
        watcher_id,
        status_code,
        status,
        now,
    )
    .execute(pool)
    .await?;

    Ok(())
}

#[derive(Debug, Clone)]
pub struct PingRecord {
    pub watcher_id: i64,
    pub timestamp: chrono::NaiveDateTime,
    pub status: String,
}

/// Fetches all pings since a given timestamp, ordered by timestamp ascending
pub async fn get_pings_since(
    pool: &Pool<Sqlite>,
    since: chrono::NaiveDateTime,
) -> Result<Vec<PingRecord>, sqlx::Error> {
    sqlx::query_as!(
        PingRecord,
        r#"SELECT watcher_id, timestamp as "timestamp: chrono::NaiveDateTime", status FROM pings WHERE timestamp >= ? ORDER BY timestamp ASC"#,
        since
    )
    .fetch_all(pool)
    .await
}
