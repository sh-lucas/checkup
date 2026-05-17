use chrono::NaiveDateTime;
use serde::Serialize;
use sqlx::{Pool, Sqlite};

#[derive(Debug, Serialize)]
pub struct HeavyRow {
    pub username: Option<String>,
    pub url: Option<String>,
    pub ping_count: i64,
    pub last_ping: Option<NaiveDateTime>,
    pub last_status: Option<String>,
}

pub async fn query_heavy(pool: &Pool<Sqlite>) -> Result<Vec<HeavyRow>, sqlx::Error> {
    sqlx::query_as!(
        HeavyRow,
        r#"
        SELECT
            u.username        AS username,
            w.url             AS url,
            COUNT(p.id)       AS ping_count,
            MAX(p.timestamp)  AS last_ping,
            MAX(p.status)     AS last_status
        FROM users u
        JOIN watchers w ON w.created_by = u.id
        LEFT JOIN pings p ON p.watcher_id = w.id
        GROUP BY u.id, w.id
        ORDER BY ping_count DESC
        LIMIT 50
        "#
    )
    .fetch_all(pool)
    .await
}

#[derive(Debug, Serialize)]
pub struct LightResult {
    pub user_count: i64,
    pub watcher_count: i64,
    pub ping_count: i64,
    pub online_count: i64,
    pub offline_count: i64,
}

pub async fn query_light(pool: &Pool<Sqlite>) -> Result<LightResult, sqlx::Error> {
    let user_count = sqlx::query_scalar!("SELECT COUNT(*) FROM users")
        .fetch_one(pool)
        .await?;
    let watcher_count = sqlx::query_scalar!("SELECT COUNT(*) FROM watchers")
        .fetch_one(pool)
        .await?;
    let ping_count = sqlx::query_scalar!("SELECT COUNT(*) FROM pings")
        .fetch_one(pool)
        .await?;
    let online_count = sqlx::query_scalar!("SELECT COUNT(*) FROM pings WHERE status = 'online'")
        .fetch_one(pool)
        .await?;
    let offline_count = sqlx::query_scalar!("SELECT COUNT(*) FROM pings WHERE status = 'offline'")
        .fetch_one(pool)
        .await?;

    Ok(LightResult {
        user_count,
        watcher_count,
        ping_count,
        online_count,
        offline_count,
    })
}

pub async fn insert_ping(pool: &Pool<Sqlite>) -> Result<i64, sqlx::Error> {
    let now = chrono::Utc::now().naive_utc();
    let rec = sqlx::query!(
        "INSERT INTO pings (watcher_id, timestamp, status_code, status) VALUES (1, ?, 200, 'online') RETURNING id",
        now
    )
    .fetch_one(pool)
    .await?;
    Ok(rec.id)
}
