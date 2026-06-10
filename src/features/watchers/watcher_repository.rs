use super::{Watcher, WatcherError};
use futures::Stream;
use sqlx::{Pool, Sqlite};
use std::pin::Pin;

pub async fn create_watcher(url: &str, pool: &Pool<Sqlite>) -> Result<i64, WatcherError> {
    let result = sqlx::query!(
        "INSERT INTO watchers (url) 
        VALUES (?) RETURNING id",
        url
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(record) => Ok(record.id),
        Err(e) => Err(WatcherError::Database(e)),
    }
}

pub async fn list_watchers_by_user(
    pool: &Pool<Sqlite>,
    user_id: i64,
) -> Result<Vec<Watcher>, WatcherError> {
    let result = sqlx::query_as!(
        Watcher,
        "SELECT * FROM watchers WHERE created_by = ?",
        user_id
    )
    .fetch_all(pool)
    .await;

    match result {
        Ok(watchers) => Ok(watchers),
        Err(e) => Err(WatcherError::Database(e)),
    }
}

pub fn stream_all_watchers<'a>(
    pool: &'a Pool<Sqlite>,
) -> Pin<Box<dyn Stream<Item = Result<Watcher, sqlx::Error>> + Send + 'a>> {
    Box::pin(sqlx::query_as::<_, Watcher>("SELECT * FROM watchers").fetch(pool))
}
