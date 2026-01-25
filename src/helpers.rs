/// is_unique_err checks if the error is a unique constraint violation
pub fn is_unique_err(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .and_then(|db_err| db_err.code())
        .as_deref()
        == Some("2067")
}
