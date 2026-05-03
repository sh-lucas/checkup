use poem::{
    handler,
    http::{self},
    web::{Data, Json},
};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use crate::{
    features::users::{jwt, user_repository},
    ok_json,
};

#[derive(Serialize, Deserialize, Debug)]
pub struct User {
    pub id: Option<i64>,
    pub username: String,

    #[serde(rename = "password")]
    pub passhash: String,
}

#[handler]
pub async fn post_user(user: Json<User>, pool: Data<&Pool<Sqlite>>) -> Result<String, poem::Error> {
    let pool = pool.0;

    let result = user_repository::create_user(pool, &user).await;

    let Ok(user_id) = result else {
        let err = result.expect_err("Impossible condition");
        return Err(poem::Error::from_string(
            err.to_string(),
            http::StatusCode::BAD_REQUEST,
        ));
    };

    let refresh_token = jwt::gen_auth_token(user_id, jwt::TokenType::Refresh, 7 * 24);
    let access_token = jwt::gen_auth_token(user_id, jwt::TokenType::Refresh, 8);

    ok_json!({
        "refresh_token": refresh_token,
        "access_token": access_token
    })
}
