use sqlx::{Pool, Sqlite};

use crate::{features::users::{User, UserError}, helpers};

/// `create_user` adds a user to the database, returning the user id or a UserError
pub async fn create_user(pool: &Pool<Sqlite>, user: &User) -> Result<i64, UserError> {
    let result = sqlx::query!(
        "INSERT INTO users (username, passhash) VALUES (?, ?) RETURNING id",
        user.username,
        user.password
    )
    .fetch_one(pool)
    .await;

    match result {
        Ok(record) => Ok(record.id),
        Err(e) => {
            if helpers::is_unique_err(&e) {
                return Err(UserError::Conflict);
            }
            Err(UserError::Database(e))
        }
    }
}
