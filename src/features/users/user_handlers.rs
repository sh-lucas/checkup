use poem::web::Data;
use poem_openapi::{ApiResponse, Object, OpenApi, payload::Json};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

use crate::features::users::{jwt, user_repository};

#[derive(Serialize, Deserialize, Debug, Object)]
pub struct User {
    pub username: String,

    // #[serde(rename = "password")]
    // #[oai(rename = "password")]
    pub password: String,
}

#[derive(Serialize, Deserialize, Object)]
pub struct UserAuthTokens {
    pub refresh_token: String,
    pub access_token: String,
}

#[derive(ApiResponse)]
pub enum CreateUserResponse {
    #[oai(status = 201)]
    Ok(Json<UserAuthTokens>),

    #[oai(status = 500)]
    InternalError(Json<String>),
}

pub struct UserApi;

#[OpenApi]
impl UserApi {
    #[oai(path = "/users/create", method = "post")]
    pub async fn create_user(
        &self,
        pool: Data<&Pool<Sqlite>>,
        user: Json<User>,
    ) -> CreateUserResponse {
        let pool = pool.0;

        let result = user_repository::create_user(pool, &user.0).await;

        let Ok(user_id) = result else {
            let err = result.expect_err("Impossible condition");
            return CreateUserResponse::InternalError(Json(err.to_string()));
        };

        let refresh_token = jwt::gen_auth_token(user_id, jwt::TokenType::Refresh, 7 * 24);
        let access_token = jwt::gen_auth_token(user_id, jwt::TokenType::Refresh, 8);

        CreateUserResponse::Ok(Json(UserAuthTokens {
            refresh_token,
            access_token,
        }))
    }
}
