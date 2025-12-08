use sqlx::{Pool, Sqlite};

pub async fn create_watcher(url: &str, interval: i64, pool: &Pool<Sqlite>) -> Option<i64> {
    let result = sqlx::query!(
        "INSERT INTO watchers (url, interval) 
        VALUES (?, ?) RETURNING id",
        url,
        interval
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(record) => Some(record.id),
        Err(_) => None,
    }
}
