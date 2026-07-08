use poem_openapi::payload::Json;
use sqlx::{Pool, Sqlite};

use super::{
    CreateWatcherRequest, CreateWatcherResponse, GetWatchersResponse, GetWatchersResult, Watcher,
};
use crate::auth::AuthClaims;

pub async fn post_watcher(
    pool: &Pool<Sqlite>,
    watcher: CreateWatcherRequest,
    claims: &AuthClaims,
) -> CreateWatcherResponse {
    let result = sqlx::query!(
        "INSERT INTO watchers (url, created_by) VALUES (?, ?) RETURNING id as \"id!: i64\"",
        watcher.url,
        claims.0.sub
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(record) => {
            CreateWatcherResponse::Created(Json(format!("Watcher added with id: {}", record.id)))
        }
        Err(e) => CreateWatcherResponse::InternalError(Json(e.to_string())),
    }
}

pub async fn get_my_watchers(pool: &Pool<Sqlite>, claims: &AuthClaims) -> GetWatchersResponse {
    let result = sqlx::query_as!(
        Watcher,
        "SELECT id as \"id!\", url as \"url!\", created_by as \"created_by!\" FROM watchers WHERE created_by = ?",
        claims.0.sub
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(watchers) => GetWatchersResponse::Ok(Json(GetWatchersResult { watchers })),
        Err(e) => GetWatchersResponse::InternalError(Json(e.to_string())),
    }
}
