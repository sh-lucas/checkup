use poem::{
    handler, http,
    web::{
        Data, Json, TypedHeader,
        headers::{Authorization, authorization::Bearer},
    },
};
use sqlx::{Pool, Sqlite};

use crate::{
    features::{users::jwt::parse_auth_token, watchers::watcher_repository},
    ok_json,
};

// model
#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct Watcher {
    pub id: i64,
    pub url: String,
    pub created_by: i64,
}

// handler
#[handler]
pub async fn post_watcher(watcher: Json<Watcher>, pool: Data<&Pool<Sqlite>>) -> String {
    let pool = pool.0;

    let result = watcher_repository::create_watcher(&watcher.url, pool).await;

    match result {
        Some(id) => format!("Watcher added with id: {id}"),
        None => "Could not save new watcher".to_string(),
    }
}

#[handler]
pub async fn get_my_watchers(
    pool: Data<&Pool<Sqlite>>,
    TypedHeader(auth_token): TypedHeader<Authorization<Bearer>>,
) -> Result<String, poem::Error> {
    let pool = pool.0;

    let claims = parse_auth_token(auth_token)?;

    let result = watcher_repository::list_watchers_by_user(pool, claims.sub).await;

    match result {
        Ok(watchers) => ok_json!({ "watchers": watchers }),
        Err(e) => Err(poem::Error::from_string(
            e.to_string(),
            http::StatusCode::INTERNAL_SERVER_ERROR,
        )),
    }
}
