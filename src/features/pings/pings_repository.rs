use sqlx::{Pool, Sqlite};

/// logs a ping attempt
pub async fn log_ping(
    pool: &Pool<Sqlite>,
    watcher_id: i64,
    status_code: i64,
    status: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now();

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
        r#"SELECT watcher_id as "watcher_id!", timestamp as "timestamp: chrono::NaiveDateTime", status as "status!" FROM pings WHERE timestamp >= ? ORDER BY timestamp ASC"#,
        since
    )
    .fetch_all(pool)
    .await
}
