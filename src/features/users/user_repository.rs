use sqlx::{Pool, Sqlite};

use crate::{features::users::User, helpers};

#[derive(Debug, thiserror::Error)]
pub enum ServerErrors {
    #[error("user already exists")]
    ConflictError,
    #[error("internal error")]
    InternalError,
}

/// `create_user` adds a user to the database, returnin `poem::Error` or on failure
pub async fn create_user(pool: &Pool<Sqlite>, user: &User) -> Result<i64, ServerErrors> {
    let result = sqlx::query!(
        "INSERT INTO users (username, passhash) VALUES (?, ?) RETURNING id",
        user.username,
        user.passhash
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(record) => Ok(record.id),
        Err(e) => {
            if helpers::is_unique_err(&e) {
                return Err(ServerErrors::ConflictError);
            }
            Err(ServerErrors::InternalError)
        }
    }
}
