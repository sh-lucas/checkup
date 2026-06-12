mod watcher_handlers;
mod watcher_repository;
mod worker;

pub use worker::*;
pub use watcher_repository::stream_all_watchers;

use poem::web::Data;
use poem_openapi::{param::Header, payload::Json, Object, OpenApi};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

// 1. Domain Model
#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow, Object)]
pub struct Watcher {
    pub id: i64,
    pub url: String,
    pub created_by: i64,
}

// 2. API Request/Response Structs
#[derive(Debug, serde::Deserialize, Object)]
pub struct CreateWatcherRequest {
    pub url: String,
}

crate::api_response! {
    pub enum CreateWatcherResponse {
        #[oai(status = 201)]
        Created(Json<String>),

        #[oai(status = 500)]
        InternalError(Json<String>),
    }
}

#[derive(Serialize, Deserialize, Object)]
pub struct GetWatchersResult {
    pub watchers: Vec<Watcher>,
}

crate::api_response! {
    pub enum GetWatchersResponse {
        #[oai(status = 200)]
        Ok(Json<GetWatchersResult>),

        #[oai(status = 401)]
        Unauthorized(Json<String>),

        #[oai(status = 500)]
        InternalError(Json<String>),
    }
}

// 3. OpenAPI routing / delegation
pub struct WatchersApi;

#[OpenApi]
impl WatchersApi {
    #[oai(path = "/watchers/create", method = "post")]
    pub async fn post_watcher(
        &self,
        pool: Data<&Pool<Sqlite>>,
        watcher: Json<CreateWatcherRequest>,
    ) -> CreateWatcherResponse {
        watcher_handlers::post_watcher(pool.0, watcher.0).await
    }

    #[oai(path = "/watchers/list", method = "get")]
    pub async fn get_my_watchers(
        &self,
        pool: Data<&Pool<Sqlite>>,
        #[oai(name = "Authorization")] auth_header: Header<String>,
    ) -> GetWatchersResponse {
        watcher_handlers::get_my_watchers(pool.0, auth_header.0).await
    }
}
