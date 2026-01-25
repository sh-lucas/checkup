use poem::http;
use sqlx::{Pool, Sqlite};

use crate::{features::users::User, helpers};

/// create_user adds a user to the database, returning poem::Error on failure
pub async fn create_user(pool: &Pool<Sqlite>, user: &User) -> Result<(), poem::Error> {
    let result = sqlx::query!(
        "INSERT INTO users (username, passhash) VALUES (?, ?)",
        user.username,
        user.passhash
    )
    .execute(pool)
    .await;

    match result {
        Ok(_) => Ok(()), // shouldn't be possible =)
        Err(e) => {
            if helpers::is_unique_err(&e) {
                return Err(poem::Error::from_string(
                    "Username already exists",
                    http::StatusCode::CONFLICT,
                ));
            }
            Err(poem::Error::from_string(
                e.to_string(),
                http::StatusCode::INTERNAL_SERVER_ERROR,
            ))
        }
    }
}
