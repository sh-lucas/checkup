use std::env;

pub async fn setup_database() -> sqlx::SqlitePool {
    // database setup:
    let db_uri = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let poll = sqlx::SqlitePool::connect(&db_uri)
        .await
        .expect("Unable to open database");

    sqlx::migrate!()
        .run(&poll)
        .await
        .expect("Failed to run database migrations");

    poll
}
