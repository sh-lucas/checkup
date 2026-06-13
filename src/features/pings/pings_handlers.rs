use poem_openapi::payload::Json;
use sqlx::{Pool, Sqlite};

use super::{GetPingsResponse, Ping};
use chrono::NaiveDateTime;

/// Returns all pings where the watcher is offline. Optionally scoped to a
/// single watcher when `watcher_id` is provided.
pub async fn get_down_pings(pool: &Pool<Sqlite>, watcher_id: Option<i64>) -> GetPingsResponse {
    let result = match watcher_id {
        Some(id) => {
            sqlx::query_as!(
                Ping,
                r#"SELECT id, watcher_id, timestamp as "timestamp: NaiveDateTime",
                          status_code, status
                   FROM pings
                   WHERE watcher_id = ? AND status = 'offline'"#,
                id
            )
            .fetch_all(pool)
            .await
        }
        None => {
            sqlx::query_as!(
                Ping,
                r#"SELECT id, watcher_id, timestamp as "timestamp: NaiveDateTime",
                          status_code, status
                   FROM pings
                   WHERE status = 'offline'"#
            )
            .fetch_all(pool)
            .await
        }
    };

    match result {
        Ok(pings) => GetPingsResponse::Ok(Json(pings)),
        Err(e) => GetPingsResponse::InternalError(Json(e.to_string())),
    }
}
