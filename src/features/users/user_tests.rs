#[cfg(test)]
mod tests {
    use sqlx::{Pool, Sqlite};

    use crate::features::users::{User, UserApi};
    use poem::{EndpointExt, test::TestClient};
    use poem_openapi::OpenApiService;

    #[sqlx::test]
    async fn test_create_user(pool: Pool<Sqlite>) {
        let _ = dotenvy::from_filename(".env.test");

        let user = User {
            username: "john doe".to_string(),
            password: "password".to_string(),
        };

        let app = OpenApiService::new(UserApi, "User API", "1.0")
            .with(poem::middleware::AddData::new(pool));
        let cli = TestClient::new(app);

        let resp = cli.post("/users/create").body_json(&user).send().await;
        resp.assert_status(poem::http::StatusCode::CREATED);
    }
}
