use sqlx::{Pool, Sqlite};
use poem_openapi::payload::Json;

use crate::features::users::jwt::parse_auth_token;
use super::{CreateWatcherRequest, CreateWatcherResponse, GetWatchersResponse, GetWatchersResult, Watcher};

pub async fn post_watcher(
    pool: &Pool<Sqlite>,
    watcher: CreateWatcherRequest,
) -> CreateWatcherResponse {
    let result = sqlx::query!(
        "INSERT INTO watchers (url) VALUES (?) RETURNING id",
        watcher.url
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(record) => {
            CreateWatcherResponse::Created(Json(format!("Watcher added with id: {}", record.id)))
        }
        Err(e) => {
            CreateWatcherResponse::InternalError(Json(e.to_string()))
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

    let result = sqlx::query_as!(
        Watcher,
        "SELECT id, url, created_by FROM watchers WHERE created_by = ?",
        claims.sub
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(watchers) => GetWatchersResponse::Ok(Json(GetWatchersResult { watchers })),
        Err(e) => GetWatchersResponse::InternalError(Json(e.to_string())),
    }
}
