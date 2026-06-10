use chrono::NaiveDateTime;
use poem::web::Data; // Removemos o Json e o http daqui
use poem_openapi::{
    ApiResponse,
    Object,
    OpenApi,
    payload::Json, // Importamos o Json EXCLUSIVO do OpenAPI
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use crate::features::pings::pings_repository;

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

#[derive(ApiResponse)]
pub enum GetPingsResponse {
    #[oai(status = 200)]
    Ok(Json<Vec<Ping>>),

    #[oai(status = 500)]
    InternalError(Json<String>),
}

// Criamos uma struct para agrupar as rotas dessa feature
pub struct PingsApi;

#[OpenApi]
impl PingsApi {
    /// Get pings for specified watcher and time range
    #[oai(path = "/pings/down", method = "get")]
    pub async fn get_down_pings(&self, pool: Data<&Pool<Sqlite>>) -> GetPingsResponse {
        let pool = pool.0;

        let result = pings_repository::get_status_changes(pool, 1).await;

        match result {
            Ok(pings) => GetPingsResponse::Ok(Json(pings)),
            Err(e) => {
                // Retornando a string de erro envelopada no Json para o status 500
                GetPingsResponse::InternalError(Json(e.to_string()))
            }
        }
    }
}
