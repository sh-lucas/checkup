mod pings_handlers;
mod pings_repository;

pub use pings_repository::log_status_change;

use chrono::NaiveDateTime;
use poem::web::Data;
use poem_openapi::{ApiResponse, Object, OpenApi, payload::Json};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

// 1. Error Enum for the feature
#[derive(Debug, thiserror::Error)]
pub enum PingsError {
    #[error("internal database error: {0}")]
    Database(#[from] sqlx::Error),
}

// 2. Domain Model
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

// 3. API Response Structs
#[derive(ApiResponse)]
pub enum GetPingsResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<Ping>>),

    #[oai(status = 500)]
    InternalError(Json<String>),
}

// 4. OpenAPI routing / delegation
pub struct PingsApi;

#[OpenApi]
impl PingsApi {
    /// Get pings for specified watcher and time range
    #[oai(path = "/pings/down", method = "get")]
    pub async fn get_down_pings(&self, pool: Data<&Pool<Sqlite>>) -> GetPingsResponse {
        pings_handlers::get_down_pings(pool.0).await
    }
}
