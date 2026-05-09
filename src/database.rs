use std::env;

use sqlx::sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions};

pub async fn setup_database() -> sqlx::SqlitePool {
    let db_uri = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let options = db_uri
        .parse::<SqliteConnectOptions>()
        .expect("Invalid DATABASE_URL")
        .create_if_missing(true)
        .journal_mode(SqliteJournalMode::Wal)
        .pragma("cache_size", "-5000")
        .pragma("busy_timeout", "5000");

    let pool = SqlitePoolOptions::new()
        .connect_with(options)
        .await
        .expect("Unable to open database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    pool
}
