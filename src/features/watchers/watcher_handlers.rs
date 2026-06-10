use poem::web::Data;
use poem_openapi::{ApiResponse, Object, OpenApi, param::Header, payload::Json};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use crate::features::{users::jwt::parse_auth_token, watchers::watcher_repository};

// model
#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow, Object)]
pub struct Watcher {
    pub id: i64,
    pub url: String,
    pub created_by: i64,
}

#[derive(Debug, serde::Deserialize, Object)]
pub struct CreateWatcherRequest {
    pub url: String,
}

#[derive(ApiResponse)]
pub enum CreateWatcherResponse {
    #[oai(status = 201)]
    Created(Json<String>),

    #[oai(status = 500)]
    InternalError(Json<String>),
}

#[derive(Serialize, Deserialize, Object)]
pub struct GetWatchersResult {
    pub watchers: Vec<Watcher>,
}

#[derive(ApiResponse)]
pub enum GetWatchersResponse {
    #[oai(status = 200)]
    Ok(Json<GetWatchersResult>),

    #[oai(status = 401)]
    Unauthorized(Json<String>),

    #[oai(status = 500)]
    InternalError(Json<String>),
}

pub struct WatchersApi;

#[OpenApi]
impl WatchersApi {
    #[oai(path = "/watchers/create", method = "post")]
    pub async fn post_watcher(
        &self,
        pool: Data<&Pool<Sqlite>>,
        watcher: Json<CreateWatcherRequest>,
    ) -> CreateWatcherResponse {
        let pool = pool.0;

        let result = watcher_repository::create_watcher(&watcher.url, pool).await;

        match result {
            Some(id) => {
                CreateWatcherResponse::Created(Json(format!("Watcher added with id: {id}")))
            }
            None => {
                CreateWatcherResponse::InternalError(Json("Could not save new watcher".to_string()))
            }
        }
    }

    #[oai(path = "/watchers/list", method = "get")]
    pub async fn get_my_watchers(
        &self,
        pool: Data<&Pool<Sqlite>>,
        #[oai(name = "Authorization")] auth_header: Header<String>,
    ) -> GetWatchersResponse {
        let pool = pool.0;

        let token = auth_header.0.trim();
        let token = if token.starts_with("Bearer ") {
            &token[7..]
        } else {
            token
        };

        let claims = parse_auth_token(token);
        let Ok(claims) = claims else {
            return GetWatchersResponse::Unauthorized(Json("Invalid token".to_string()));
        };

        let result = watcher_repository::list_watchers_by_user(pool, claims.sub).await;

        match result {
            Ok(watchers) => GetWatchersResponse::Ok(Json(GetWatchersResult { watchers })),
            Err(e) => GetWatchersResponse::InternalError(Json(e.to_string())),
        }
    }
}
