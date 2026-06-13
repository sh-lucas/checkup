mod watcher_handlers;

use poem::web::Data;
use poem_openapi::{Object, OpenApi, payload::Json};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use crate::auth::AuthClaims;

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

        #[oai(status = 500)]
        InternalError(Json<String>),
    }
}

// 3. OpenAPI routing / delegation
pub struct WatchersApi;

#[OpenApi]
impl WatchersApi {
    /// Register a new watcher owned by the authenticated user.
    #[oai(path = "/watchers/create", method = "post")]
    pub async fn post_watcher(
        &self,
        pool: Data<&Pool<Sqlite>>,
        watcher: Json<CreateWatcherRequest>,
        claims: AuthClaims,
    ) -> CreateWatcherResponse {
        watcher_handlers::post_watcher(pool.0, watcher.0, &claims).await
    }

    /// List watchers owned by the authenticated user.
    #[oai(path = "/watchers/list", method = "get")]
    pub async fn get_my_watchers(
        &self,
        pool: Data<&Pool<Sqlite>>,
        claims: AuthClaims,
    ) -> GetWatchersResponse {
        watcher_handlers::get_my_watchers(pool.0, &claims).await
    }
}
