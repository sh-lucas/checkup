use sqlx::{Pool, Sqlite};
use poem_openapi::payload::Json;

use crate::features::pings::pings_repository;
use super::{GetPingsResponse, PingsError};

pub async fn get_down_pings(pool: &Pool<Sqlite>) -> GetPingsResponse {
    let result = pings_repository::get_status_changes(pool, 1).await;

    match result {
        Ok(pings) => GetPingsResponse::Ok(Json(pings)),
        Err(PingsError::Database(e)) => {
            GetPingsResponse::InternalError(Json(e.to_string()))
        }
    }
}
