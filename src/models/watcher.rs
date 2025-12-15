#[derive(Debug, serde::Deserialize, serde::Serialize, sqlx::FromRow)]
pub struct Watcher {
    pub id: i64, // Include all columns from the SELECT
    pub url: String,
}
