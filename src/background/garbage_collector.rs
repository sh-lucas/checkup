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
                eprintln!("Overflow calculating cutoff datetime");
                continue;
            };

            // Delete old pings
            if let Err(e) = sqlx::query!("DELETE FROM pings WHERE timestamp < ?", cutoff)
                .execute(&pool)
                .await
            {
                eprintln!("Error running garbage collector for pings: {e}");
            }

            // Delete old system uptime events
            if let Err(e) = sqlx::query!(
                "DELETE FROM system_uptime_events WHERE timestamp < ?",
                cutoff
            )
            .execute(&pool)
            .await
            {
                eprintln!("Error running garbage collector for system events: {e}");
            }

            // Reclaim pages freed by the deletion since auto_vacuum is INCREMENTAL
            if let Err(e) = sqlx::query("PRAGMA incremental_vacuum")
                .execute(&pool)
                .await
            {
                eprintln!("Error running incremental_vacuum: {e}");
            }
        }
    });
}
