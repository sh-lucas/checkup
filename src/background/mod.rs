use crate::features::metrics::MetricsCache;
use sqlx::pool::Pool;
use sqlx::sqlite::Sqlite;
use std::sync::Arc;
use tokio::task::JoinHandle;

mod garbage_collector;
mod heartbeat;
mod metrics_updater;
mod ping;

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

    garbage_collector::spawn(pool.clone());
    metrics_updater::spawn(pool.clone(), metrics_cache);
    heartbeat::spawn(pool.clone());
    ping::spawn(pool, interval_secs, num_workers)
}
