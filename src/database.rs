use std::env;

use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};

pub async fn setup_database() -> sqlx::SqlitePool {
    // database setup:
    let db_uri = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    // Turso (Limbo) doesn't support PRAGMA foreign_keys, so we avoid setting any
    // foreign_keys option — the default (None) won't send any PRAGMA on connect.
    let opts = db_uri
        .parse::<SqliteConnectOptions>()
        .expect("Invalid DATABASE_URL");

    let pool = SqlitePoolOptions::new()
        .connect_with(opts)
        .await
        .expect("Unable to open database");

    sqlx::migrate!()
        .run(&pool)
        .await
        .expect("Failed to run database migrations");

    pool
}
