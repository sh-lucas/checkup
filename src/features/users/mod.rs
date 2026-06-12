pub mod jwt;
mod user_handlers;
mod user_tests;

use poem::web::Data;
use poem_openapi::{Object, OpenApi, payload::Json};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

// 1. Domain Models & DTOs
#[derive(Serialize, Deserialize, Debug, Object)]
pub struct CreateUserRequest {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug, Object)]
pub struct User {
    pub id: i64,
    pub username: String,
}

// 2. API Structs
#[derive(Serialize, Deserialize, Object)]
pub struct UserAuthTokens {
    pub refresh_token: String,
    pub access_token: String,
}

crate::api_response! {
    pub enum CreateUserResponse {
        #[oai(status = 201)]
        Ok(Json<UserAuthTokens>),

        #[oai(status = 409)]
        Conflict(Json<String>),

        #[oai(status = 400)]
        BadRequest(Json<String>),

        #[oai(status = 500)]
        InternalError(Json<String>),
    }
}

// 3. OpenAPI routing / delegation
pub struct UserApi;

#[OpenApi]
impl UserApi {
    #[oai(path = "/users/create", method = "post")]
    pub async fn create_user(
        &self,
        pool: Data<&Pool<Sqlite>>,
        user: Json<CreateUserRequest>,
    ) -> CreateUserResponse {
        user_handlers::create_user(pool.0, user.0).await
    }
}
