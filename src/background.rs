use crate::features::metrics::{self, MetricsCache, MetricsState};
use crate::features::{pings, watchers};
use futures::StreamExt;
use sqlx::pool::Pool;
use sqlx::sqlite::Sqlite;
use std::sync::Arc;
use tokio::task::JoinHandle;

/// Spawns the background ping loop. Pings every watcher once per `interval`
/// seconds using `num_workers` concurrent tasks. The returned handle lets
/// `main` abort the loop on shutdown.
pub fn start_watching(
    pool: &Pool<Sqlite>,
    interval_secs: u64,
    num_workers: usize,
    metrics_cache: Arc<MetricsCache>,
) -> JoinHandle<()> {
    let pool = pool.clone();

    // Spawn the garbage collector task
    let gc_pool = pool.clone();
    tokio::spawn(async move {
        // Run once every hour
        let mut gc_ticker = tokio::time::interval(std::time::Duration::from_hours(1));
        loop {
            gc_ticker.tick().await;

            let Some(cutoff) = chrono::Utc::now().checked_sub_signed(chrono::Duration::days(7))
            else {
                eprintln!("Overflow calculating cutoff datetime");
                continue;
            };

            // Delete old pings
            if let Err(e) = sqlx::query!("DELETE FROM pings WHERE timestamp < ?", cutoff)
                .execute(&gc_pool)
                .await
            {
                eprintln!("Error running garbage collector for pings: {e}");
            }

            // Delete old system uptime events
            if let Err(e) = sqlx::query!(
                "DELETE FROM system_uptime_events WHERE timestamp < ?",
                cutoff
            )
            .execute(&gc_pool)
            .await
            {
                eprintln!("Error running garbage collector for system events: {e}");
            }

            // Reclaim pages freed by the deletion since auto_vacuum is INCREMENTAL
            if let Err(e) = sqlx::query("PRAGMA incremental_vacuum")
                .execute(&gc_pool)
                .await
            {
                eprintln!("Error running incremental_vacuum: {e}");
            }
        }
    });

    // Spawn metrics updater task
    let metrics_pool = pool.clone();
    tokio::spawn(async move {
        // Run once every 1 second
        let mut metrics_ticker = tokio::time::interval(std::time::Duration::from_secs(1));
        let mut state = MetricsState::default();

        loop {
            metrics_ticker.tick().await;
            metrics::update_metrics(&metrics_pool, &metrics_cache, &mut state).await;
        }
    });

    // Spawn heartbeat updater task
    let heartbeat_pool = pool.clone();
    tokio::spawn(async move {
        // Run immediately on boot to catch downtime gap
        if let Err(e) = run_heartbeat(&heartbeat_pool, "system").await {
            eprintln!("Error running initial heartbeat: {e}");
        }

        // Run once every 5 minutes
        let mut heartbeat_ticker = tokio::time::interval(std::time::Duration::from_secs(300));
        // Skip first tick because we just ran it
        heartbeat_ticker.tick().await;

        loop {
            heartbeat_ticker.tick().await;
            if let Err(e) = run_heartbeat(&heartbeat_pool, "system").await {
                eprintln!("Error running heartbeat: {e}");
            }
        }
    });

    // spawn ping worker(s)
    tokio::spawn(async move {
        let duration = std::time::Duration::from_secs(interval_secs);
        let mut ticker = tokio::time::interval(duration);

        loop {
            ticker.tick().await;

            let rx = stream_watchers(&pool);
            let mut handles = Vec::with_capacity(num_workers);

            for _ in 0..num_workers {
                let rx = rx.clone();
                let pool = pool.clone();
                handles.push(tokio::spawn(async move {
                    ping_from(rx, &pool).await;
                }));
            }

            for handle in handles {
                if let Err(e) = handle.await {
                    eprintln!("Ping worker panicked: {e}");
                }
            }
        }
    })
}

fn stream_watchers(pool: &Pool<Sqlite>) -> async_channel::Receiver<watchers::Watcher> {
    let pool = pool.clone();
    let (tx, rx) = async_channel::bounded::<watchers::Watcher>(10);

    tokio::spawn(async move {
        let mut stream =
            sqlx::query_as::<_, watchers::Watcher>("SELECT * FROM watchers").fetch(&pool);

        while let Some(result) = stream.next().await {
            match result {
                Err(e) => {
                    eprintln!("Error fetching watcher: {e}");
                    break;
                }
                Ok(watcher) => {
                    if tx.send(watcher).await.is_err() {
                        eprintln!("Error sending watcher to channel");
                        break;
                    }
                }
            }
        }
    });

    rx
}

async fn ping_from(rx: async_channel::Receiver<watchers::Watcher>, pool: &Pool<Sqlite>) {
    while let Ok(watcher) = rx.recv().await {
        let response = reqwest::get(&watcher.url).await;

        let status_code = match response {
            Ok(resp) => resp.status().as_u16(),
            Err(e) => {
                eprintln!("Connection failure for {}: {e}", watcher.url);
                599
            }
        };

        let status = if status_code < 400 {
            "online"
        } else {
            "offline"
        };

        if let Err(e) = pings::log_ping(pool, watcher.id, i64::from(status_code), status).await {
            eprintln!("Error logging ping: {e}");
        }
    }
}

pub async fn run_heartbeat(pool: &Pool<Sqlite>, reference: &str) -> Result<(), sqlx::Error> {
    let now = chrono::Utc::now();
    let last_event = sqlx::query!(
        r#"SELECT id, reference, event, timestamp as "timestamp: chrono::DateTime<chrono::Utc>"
           FROM system_uptime_events
           WHERE reference = ?
           ORDER BY timestamp DESC, id DESC
           LIMIT 1"#,
        reference
    )
    .fetch_optional(pool)
    .await?;

    match last_event {
        None => {
            // First run: insert online_until (event = 0) with "now"
            sqlx::query!(
                "INSERT INTO system_uptime_events (reference, event, timestamp) VALUES (?, 0, ?)",
                reference,
                now
            )
            .execute(pool)
            .await?;
        }
        Some(last) => {
            let diff = now.signed_duration_since(last.timestamp);
            let diff_mins = diff.num_minutes();

            if diff_mins <= 9 {
                if last.event == 0 {
                    // Update timestamp of the last online_until to now
                    sqlx::query!(
                        "UPDATE system_uptime_events SET timestamp = ? WHERE id = ?",
                        now,
                        last.id
                    )
                    .execute(pool)
                    .await?;
                } else {
                    // If the last one was offline_until, we start a new online_until
                    sqlx::query!(
                        "INSERT INTO system_uptime_events (reference, event, timestamp) VALUES (?, 0, ?)",
                        reference,
                        now
                    )
                    .execute(pool)
                    .await?;
                }
            } else {
                // Gap > 9 minutes -> outage detected!
                // 1. Insert offline_until (event = 1) at now
                sqlx::query!(
                    "INSERT INTO system_uptime_events (reference, event, timestamp) VALUES (?, 1, ?)",
                    reference,
                    now
                )
                .execute(pool)
                .await?;

                // 2. Open a new online_until (event = 0) at now
                sqlx::query!(
                    "INSERT INTO system_uptime_events (reference, event, timestamp) VALUES (?, 0, ?)",
                    reference,
                    now
                )
                .execute(pool)
                .await?;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
#[allow(clippy::arithmetic_side_effects)]
mod tests {
    use super::*;
    use chrono::{Duration, Utc};

    #[sqlx::test]
    async fn test_heartbeat_scenarios(pool: Pool<Sqlite>) {
        let reference = "system";

        // 1. First run: should insert a single online_until (event = 0)
        run_heartbeat(&pool, reference).await.unwrap();

        let events = sqlx::query!(
            r#"SELECT event as "event!", timestamp as "timestamp: chrono::DateTime<chrono::Utc>"
               FROM system_uptime_events
               WHERE reference = ?
               ORDER BY id ASC"#,
            reference
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event, 0);

        // 2. Second run within 9 minutes: should update the timestamp of the existing row (still 1 row)
        let five_mins_ago = Utc::now() - Duration::minutes(5);
        sqlx::query!(
            "UPDATE system_uptime_events SET timestamp = ? WHERE id = 1",
            five_mins_ago
        )
        .execute(&pool)
        .await
        .unwrap();

        run_heartbeat(&pool, reference).await.unwrap();

        let events = sqlx::query!(
            r#"SELECT event as "event!", timestamp as "timestamp: chrono::DateTime<chrono::Utc>"
               FROM system_uptime_events
               WHERE reference = ?
               ORDER BY id ASC"#,
            reference
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(events.len(), 1);
        assert_eq!(events[0].event, 0);
        assert!(events[0].timestamp > five_mins_ago);

        // 3. Third run after 10 minutes (outage): should insert offline_until (event = 1) and a new online_until (event = 0)
        let ten_mins_ago = Utc::now() - Duration::minutes(10);
        sqlx::query!(
            "UPDATE system_uptime_events SET timestamp = ? WHERE id = 1",
            ten_mins_ago
        )
        .execute(&pool)
        .await
        .unwrap();

        run_heartbeat(&pool, reference).await.unwrap();

        let events = sqlx::query!(
            r#"SELECT event as "event!", timestamp as "timestamp: chrono::DateTime<chrono::Utc>"
               FROM system_uptime_events
               WHERE reference = ?
               ORDER BY id ASC"#,
            reference
        )
        .fetch_all(&pool)
        .await
        .unwrap();

        assert_eq!(events.len(), 3);
        assert_eq!(events[0].event, 0);
        assert_eq!(events[1].event, 1);
        assert_eq!(events[2].event, 0);
    }
}
