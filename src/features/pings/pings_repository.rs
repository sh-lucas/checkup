use chrono::{self, NaiveDateTime};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

#[derive(Debug, Serialize, Deserialize)]
pub struct Ping {
    pub id: i64,
    pub watcher_id: i64,
    pub timestamp: NaiveDateTime,
    /// 200 | 404 | 500 | etc
    pub status_code: i64,
    /// "online" | "offline"
    pub status: String,
}

#[derive(Debug, thiserror::Error)]
pub enum PingsErrors {
    #[error("internal error")]
    InternalError,
}

/// logs a status change if it's different from the latest log.
/// `status_code` is the HTTP status (e.g. 200, 404, 503),
/// `status` is "online" or "offline".
pub async fn log_status_change(
    pool: &Pool<Sqlite>,
    watcher_id: i64,
    status_code: u16,
    status: &str,
) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now();

    let latest_log = sqlx::query!(
        "SELECT status FROM pings WHERE watcher_id = ? ORDER BY timestamp DESC LIMIT 1",
        watcher_id
    )
    .fetch_optional(pool)
    .await?;

    // no previous entry or status differs → insert new one
    if latest_log.is_none()
        || latest_log
            .expect("No previous entry or status differs")
            .status
            != status
    {
        let code = i64::from(status_code);
        sqlx::query!(
            "INSERT INTO pings (watcher_id, status_code, status, timestamp) VALUES (?, ?, ?, ?)",
            watcher_id,
            code,
            status,
            now,
        )
        .execute(pool)
        .await?;
    }

    Ok(())
}

pub async fn get_status_changes(
    pool: &Pool<Sqlite>,
    watcher_id: i64,
) -> Result<Vec<Ping>, PingsErrors> {
    let result = sqlx::query_as!(
        Ping,
        "SELECT id, watcher_id, timestamp, status_code, status FROM pings WHERE watcher_id = ?",
        watcher_id
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(pings) => Ok(pings),
        Err(_e) => Err(PingsErrors::InternalError),
    }
}
