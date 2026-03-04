use poem::{
    handler,
    http::{self},
    web::{Data, Json},
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use crate::features::users::user_repository;

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub username: String,

    #[serde(rename = "password")]
    pub passhash: String,
}

#[handler]
pub async fn post_user(user: Json<User>, pool: Data<&Pool<Sqlite>>) -> Result<String, poem::Error> {
    let pool = pool.0;

    let result = user_repository::create_user(pool, &user).await;

    match result {
        Ok(()) => Ok("User created".to_string()),
        Err(e) => Err(poem::Error::from_string(
            e.to_string(),
            http::StatusCode::BAD_REQUEST,
        )),
    }
}
