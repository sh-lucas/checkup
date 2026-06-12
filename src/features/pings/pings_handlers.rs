use sqlx::{Pool, Sqlite};
use poem_openapi::payload::Json;

use chrono::NaiveDateTime;
use super::{GetPingsResponse, Ping};

pub async fn get_down_pings(pool: &Pool<Sqlite>) -> GetPingsResponse {
    let result = sqlx::query_as!(
        Ping,
        "SELECT id, watcher_id, timestamp as \"timestamp: NaiveDateTime\", status_code, status FROM pings WHERE watcher_id = ?",
        1
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(pings) => GetPingsResponse::Ok(Json(pings)),
        Err(e) => GetPingsResponse::InternalError(Json(e.to_string())),
    }
}
