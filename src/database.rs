use std::env;

pub async fn setup_database() -> sqlx::SqlitePool {
    // database setup:
    let db_uri = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let pool = sqlx::SqlitePool::connect(&db_uri)
        .await
        .expect("Unable to open database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    pool
}
