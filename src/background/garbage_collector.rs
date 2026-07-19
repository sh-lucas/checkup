use sqlx::pool::Pool;
use sqlx::sqlite::Sqlite;

pub fn spawn(pool: Pool<Sqlite>) {
    tokio::spawn(async move {
        // Run once every hour
        let mut gc_ticker = tokio::time::interval(std::time::Duration::from_secs(3600));
        loop {
            gc_ticker.tick().await;

            let Some(cutoff) = chrono::Utc::now().checked_sub_signed(chrono::Duration::days(7))
            else {
                tracing::error!("overflow calculating garbage-collection cutoff");
                continue;
            };

            // Delete old pings
            if let Err(error) = sqlx::query!("DELETE FROM pings WHERE timestamp < ?", cutoff)
                .execute(&pool)
                .await
            {
                tracing::error!(%error, "failed to collect expired pings");
            }

            // Delete old system uptime events
            if let Err(error) = sqlx::query!(
                "DELETE FROM system_uptime_events WHERE timestamp < ?",
                cutoff
            )
            .execute(&pool)
            .await
            {
                tracing::error!(%error, "failed to collect expired system events");
            }

            // Reclaim pages freed by the deletion since auto_vacuum is INCREMENTAL
            if let Err(error) = sqlx::query("PRAGMA incremental_vacuum")
                .execute(&pool)
                .await
            {
                tracing::error!(%error, "failed to run incremental vacuum");
            }
        }
    });
}
