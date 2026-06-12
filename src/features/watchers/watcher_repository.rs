use super::Watcher;
use futures::Stream;
use sqlx::{Pool, Sqlite};
use std::pin::Pin;

pub fn stream_all_watchers<'a>(
    pool: &'a Pool<Sqlite>,
) -> Pin<Box<dyn Stream<Item = Result<Watcher, sqlx::Error>> + Send + 'a>> {
    Box::pin(sqlx::query_as::<_, Watcher>("SELECT * FROM watchers").fetch(pool))
}
