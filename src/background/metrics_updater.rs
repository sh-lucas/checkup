use crate::features::metrics::{self, MetricsCache, MetricsState};
use sqlx::pool::Pool;
use sqlx::sqlite::Sqlite;
use std::sync::Arc;

pub fn spawn(pool: Pool<Sqlite>, metrics_cache: Arc<MetricsCache>) {
    tokio::spawn(async move {
        // Run once every 1 second
        let mut metrics_ticker = tokio::time::interval(std::time::Duration::from_secs(1));
        let mut state = MetricsState::default();

        loop {
            metrics_ticker.tick().await;
            metrics::update_metrics(&pool, &metrics_cache, &mut state).await;
        }
    });
}
