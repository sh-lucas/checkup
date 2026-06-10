#![cfg(test)]

use sqlx::{Pool, Sqlite};

use crate::features::users::{User, user_handlers};
use poem::{EndpointExt, test::TestClient};
use poem_openapi::OpenApiService;

#[sqlx::test]
async fn test_create_user(pool: Pool<Sqlite>) {
    let _ = dotenvy::from_filename(".env.test");

    let user = User {
        username: "john doe".to_string(),
        password: "password".to_string(),
    };

    let app = OpenApiService::new(user_handlers::UserApi, "User API", "1.0")
        .with(poem::middleware::AddData::new(pool));
    let cli = TestClient::new(app);

    let resp = cli.post("/users/create").body_json(&user).send().await;
    resp.assert_status(poem::http::StatusCode::CREATED);
}
