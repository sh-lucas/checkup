use crate::repositories;
use poem::{
    handler,
    web::{Data, Json},
};
use sqlx::{Pool, Sqlite};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct Watcher {
    pub url: String,
    pub interval: u64,
}

#[handler]
pub async fn add_watcher(watcher: Json<Watcher>, pool: Data<&Pool<Sqlite>>) -> String {
    let pool = pool.0;

    let interval = watcher.interval as i64;
    let result = repositories::create_watcher(&watcher.url, interval, pool).await;

    match result {
        Some(id) => format!("Watcher added with id: {}", id).to_string(),
        None => format!("Could not save new watcher").to_string(),
    }
}
