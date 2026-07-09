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
