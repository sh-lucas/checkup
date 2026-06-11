pub mod jwt;
mod user_handlers;
mod user_repository;
mod user_tests;

use poem::web::Data;
use poem_openapi::{ApiResponse, Object, OpenApi, payload::Json};
use serde::{Deserialize, Serialize};
use sqlx::{Pool, Sqlite};

// 1. Error Enum for the feature
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("user already exists")]
    Conflict,
    
    #[error("internal database error")]
    Database(#[from] sqlx::Error),

    #[error("password error: {0}")]
    Password(String),
}

// 2. Domain Model
#[derive(Serialize, Deserialize, Debug, sqlx::FromRow, Object)]
pub struct User {
    #[oai(read_only)]
    pub id: Option<i64>,

    pub username: String,

    #[sqlx(default)]
    #[oai(write_only)]
    #[serde(skip_serializing)]
    pub password: Option<String>,

    #[sqlx(default)]
    #[oai(skip)]
    #[serde(skip)]
    pub passhash: Option<String>,
}

// 3. API Structs
#[derive(Serialize, Deserialize, Object)]
pub struct UserAuthTokens {
    pub refresh_token: String,
    pub access_token: String,
}

#[derive(ApiResponse)]
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

// 4. OpenAPI routing / delegation
pub struct UserApi;

#[OpenApi]
impl UserApi {
    #[oai(path = "/users/create", method = "post")]
    pub async fn create_user(
        &self,
        pool: Data<&Pool<Sqlite>>,
        user: Json<User>,
    ) -> CreateUserResponse {
        user_handlers::create_user(pool.0, user.0).await
    }
}
