use poem::{handler, http::StatusCode, web::Data, web::Json, Error, Result};
use sqlx::{Pool, Sqlite};

use super::bench_repository;

#[handler]
pub async fn bench_heavy(pool: Data<&Pool<Sqlite>>) -> Result<Json<serde_json::Value>> {
    let rows = bench_repository::query_heavy(pool.0)
        .await
        .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Json(serde_json::json!({ "rows": rows, "count": rows.len() })))
}

#[handler]
pub async fn bench_light(pool: Data<&Pool<Sqlite>>) -> Result<Json<serde_json::Value>> {
    let result = bench_repository::query_light(pool.0)
        .await
        .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Json(serde_json::json!(result)))
}

#[handler]
pub async fn bench_write(pool: Data<&Pool<Sqlite>>) -> Result<Json<serde_json::Value>> {
    let id = bench_repository::insert_ping(pool.0)
        .await
        .map_err(|e| Error::from_string(e.to_string(), StatusCode::INTERNAL_SERVER_ERROR))?;
    Ok(Json(serde_json::json!({ "inserted_id": id })))
}
