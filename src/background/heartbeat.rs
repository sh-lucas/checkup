use sqlx::pool::Pool;
use sqlx::sqlite::Sqlite;

pub fn spawn(pool: Pool<Sqlite>) {
    tokio::spawn(async move {
        // Run immediately on boot to catch downtime gap
        if let Err(e) = run_heartbeat(&pool, "system").await {
            eprintln!("Error running initial heartbeat: {e}");
        }

        // Run once every 5 minutes
        let mut heartbeat_ticker = tokio::time::interval(std::time::Duration::from_secs(300));
        // Skip first tick because we just ran it
        heartbeat_ticker.tick().await;

        loop {
            heartbeat_ticker.tick().await;
            if let Err(e) = run_heartbeat(&pool, "system").await {
                eprintln!("Error running heartbeat: {e}");
            }
        }
    });
}

async fn run_heartbeat(pool: &Pool<Sqlite>, reference: &str) -> Result<(), sqlx::Error> {
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
