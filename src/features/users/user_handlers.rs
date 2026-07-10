use poem_openapi::payload::Json;
use sqlx::{Pool, Sqlite};

use super::{CreateUserRequest, CreateUserResponse, UserAuthTokens};
use crate::auth::{self, TokenType};

pub async fn create_user(pool: &Pool<Sqlite>, user: CreateUserRequest) -> CreateUserResponse {
    let passhash = match bcrypt::hash(&user.password, bcrypt::DEFAULT_COST) {
        Ok(hash) => hash,
        Err(e) => return CreateUserResponse::BadRequest(Json(e.to_string())),
    };

    let result = sqlx::query!(
        "INSERT INTO users (username, passhash) VALUES (?, ?) RETURNING id as \"id!: i64\"",
        user.username,
        passhash
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(record) => {
            let refresh_token = auth::gen_auth_token(record.id, TokenType::Refresh, 7 * 24);
            let access_token = auth::gen_auth_token(record.id, TokenType::Access, 8);

            CreateUserResponse::Ok(Json(UserAuthTokens {
                refresh_token,
                access_token,
            }))
        }
        Err(e) => {
            if crate::helpers::is_unique_err(&e) {
                CreateUserResponse::Conflict(Json("User already exists".to_string()))
            } else {
                CreateUserResponse::InternalError(Json(e.to_string()))
            }
        }
    }
}
