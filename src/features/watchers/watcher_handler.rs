use poem::{
    handler,
    web::{Data, Json},
};
use sqlx::{Pool, Sqlite};

// model
#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct Watcher {
    pub id: i64,
    pub url: String,
}

// handler
#[handler]
pub async fn add_watcher(watcher: Json<Watcher>, pool: Data<&Pool<Sqlite>>) -> String {
    let pool = pool.0;

    let result = super::create_watcher(&watcher.url, pool).await;

    match result {
        Some(id) => format!("Watcher added with id: {}", id),
        None => "Could not save new watcher".to_string(),
    }
}
