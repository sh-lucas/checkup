use poem::{
    http,
    web::{Data, Json},
};
use sqlx::{Pool, Sqlite};

use crate::{features::pings::pings_repository, ok_json};

/// Get pings for specified watcher and time range
#[poem::handler]
pub async fn get_down_pings(pool: Data<&Pool<Sqlite>>) -> Result<String, poem::Error> {
    let pool = pool.0;

    let result = pings_repository::get_status_changes(pool, 1).await;

    match result {
        Err(e) => Err(poem::Error::from_string(
            e.to_string(),
            http::StatusCode::INTERNAL_SERVER_ERROR,
        )),
        Ok(pings) => ok_json!({
            "pings": pings,
        }),
    }
}
