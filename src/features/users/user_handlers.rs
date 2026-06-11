use sqlx::{Pool, Sqlite};
use poem_openapi::payload::Json;

use crate::features::users::{jwt, user_repository};
use super::{CreateUserResponse, User, UserAuthTokens, UserError};

pub async fn create_user(
    pool: &Pool<Sqlite>,
    user: User,
) -> CreateUserResponse {
    let result = user_repository::create_user(pool, &user).await;

    match result {
        Ok(user_id) => {
            let refresh_token = jwt::gen_auth_token(user_id, jwt::TokenType::Refresh, 7 * 24);
            let access_token = jwt::gen_auth_token(user_id, jwt::TokenType::Access, 8);

            CreateUserResponse::Ok(Json(UserAuthTokens {
                refresh_token,
                access_token,
            }))
        }
        Err(UserError::Conflict) => {
            CreateUserResponse::Conflict(Json("User already exists".to_string()))
        }
        Err(UserError::Password(e)) => {
            CreateUserResponse::BadRequest(Json(e))
        }
        Err(e) => {
            CreateUserResponse::InternalError(Json(e.to_string()))
        }
    }
}
