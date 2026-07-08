mod pings_handlers;
mod pings_repository;

pub use pings_repository::{log_ping, get_pings_since, PingRecord};

use chrono::NaiveDateTime;
use poem::web::Data;
use poem_openapi::{Object, OpenApi, param::Query, payload::Json};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

// 1. Domain Model
#[derive(Debug, Serialize, Deserialize, Object)]
pub struct Ping {
    pub id: i64,
    pub watcher_id: i64,
    pub timestamp: NaiveDateTime,
    /// 200 | 404 | 500 | etc
    pub status_code: i64,
    /// "online" | "offline"
    pub status: String,
}

// 2. API Response Structs
crate::api_response! {
    pub enum GetPingsResponse {
        #[oai(status = 200)]
        Ok(Json<Vec<Ping>>),

        #[oai(status = 500)]
        InternalError(Json<String>),
    }
}

// 4. OpenAPI routing / delegation
pub struct PingsApi;

#[OpenApi]
impl PingsApi {
    /// List ping records where the target watcher is offline.
    #[oai(path = "/pings/down", method = "get")]
    pub async fn get_down_pings(
        &self,
        pool: Data<&Pool<Sqlite>>,
        #[oai(name = "watcher_id")] watcher_id: Query<Option<i64>>,
    ) -> GetPingsResponse {
        pings_handlers::get_down_pings(pool.0, watcher_id.0).await
    }
}
