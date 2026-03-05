/// `is_unique_err` checks if the error is a unique constraint violation
pub fn is_unique_err(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .and_then(sqlx::error::DatabaseError::code)
        .as_deref()
        == Some("2067")
}

#[macro_export]
macro_rules! ok_json {
    ($($json:tt)+) => {
        Ok(serde_json::json!($($json)+).to_string())
    };
}
