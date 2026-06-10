use sqlx::{Pool, Sqlite};
use poem_openapi::payload::Json;

use crate::features::{users::jwt::parse_auth_token, watchers::watcher_repository};
use super::{CreateWatcherRequest, CreateWatcherResponse, GetWatchersResponse, GetWatchersResult};

pub async fn post_watcher(
    pool: &Pool<Sqlite>,
    watcher: CreateWatcherRequest,
) -> CreateWatcherResponse {
    let result = watcher_repository::create_watcher(&watcher.url, pool).await;

    match result {
        Ok(id) => {
            CreateWatcherResponse::Created(Json(format!("Watcher added with id: {id}")))
        }
        Err(_) => {
            CreateWatcherResponse::InternalError(Json("Could not save new watcher".to_string()))
        }
    }
}

pub async fn get_my_watchers(
    pool: &Pool<Sqlite>,
    auth_header: String,
) -> GetWatchersResponse {
    let token = auth_header.trim();
    let token = if let Some(stripped) = token.strip_prefix("Bearer ") {
        stripped
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
