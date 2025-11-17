use poem::{
    handler,
    web::{Data, Json},
};
use sqlx::{Pool, Sqlite};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
struct Watcher {
    url: String,
    interval: u64,
}

#[handler]
pub async fn add_watcher(watcher: Json<Watcher>, pool: Data<&Pool<Sqlite>>) -> String {
    let pool = pool.0;

    let interval = watcher.interval as i64;
    let result = sqlx::query!(
        "INSERT INTO watchers (url, interval) 
        VALUES (?, ?) RETURNING id",
        watcher.url,
        interval
    )
    .fetch_one(pool)
    .await;

    match result {
        Err(e) => format!("Could not save new watcher. Error: {}", e).to_string(),
        Ok(row) => format!("Watcher added with id: {}", row.id).to_string(),
    }
}
