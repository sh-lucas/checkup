use super::Watcher;
use futures::Stream;
use sqlx::Error;
use sqlx::{Pool, Sqlite};
use std::pin::Pin;

pub async fn create_watcher(url: &str, pool: &Pool<Sqlite>) -> Option<i64> {
    let result = sqlx::query!(
        "INSERT INTO watchers (url) 
        VALUES (?) RETURNING id",
        url
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(record) => Some(record.id),
        Err(_) => None,
    }
}

pub fn stream_all_watchers<'a>(
    pool: &'a Pool<Sqlite>,
) -> Pin<Box<dyn Stream<Item = Result<Watcher, Error>> + Send + 'a>> {
    Box::pin(sqlx::query_as::<_, Watcher>("SELECT * FROM watchers").fetch(pool))
}
