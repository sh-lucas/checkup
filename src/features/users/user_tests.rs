#![cfg(test)]

use sqlx::{Pool, Sqlite};

use crate::features::users::{User, user_handlers, user_repository};
use poem::{EndpointExt, test::TestClient};

#[sqlx::test]
async fn test_create_user(pool: Pool<Sqlite>) {
    let _ = dotenvy::from_filename(".env.test");

    let user = User {
        id: None,
        username: "john doe".to_string(),
        passhash: "password".to_string(),
    };

    let app = poem::Route::new()
        .at("/users", poem::post(user_handlers::post_user))
        .with(poem::middleware::AddData::new(pool.clone()));
    let cli = TestClient::new(app);

    let resp = cli.post("/users").body_json(&user).send().await;
    resp.assert_status_is_ok();

    let users = user_repository::get_all_users(&pool)
        .await
        .expect("Pool is broken after close on test_create_user");

    assert!(!users.is_empty(), "No users found");

    let user = users.first().expect("Could not get first user from pool");

    assert_eq!(user.username, user.username);
}
