use crate::{models, repositories};
use futures::StreamExt;
use sqlx::pool::Pool;
use sqlx::sqlite::Sqlite;

/// spawns a new thread with an infinite loop
/// pinging all watchers every interval seconds
/// doesn't block or wait for them to complete.
pub fn start_watching(pool: &Pool<Sqlite>, interval: u64) {
    // needs to clone the atomic counter to avoid borrowing issues
    let pool = pool.clone();

    // runs on a separate task, indefinitely pinging all watchers
    tokio::spawn(async move {
        let duration = std::time::Duration::from_secs(interval);
        let mut ticker = tokio::time::interval(duration);

        loop {
            ticker.tick().await;
            let chan = stream_watchers_from(&pool);
            ping_from_stream(chan, &pool).await;
        }
    });
}

/// starts another thread to lazy-fetch all watchers without blocking.
/// returns the reciever channel to the caller.
/// channels are basically iterators in Rust, so this is actually goated.
pub fn stream_watchers_from(pool: &Pool<Sqlite>) -> async_channel::Receiver<models::Watcher> {
    let pool = pool.clone();
    let (tx, rx) = async_channel::bounded::<models::Watcher>(10);

    tokio::spawn(async move {
        // Create the stream INSIDE the task using the owned `pool`
        let mut stream = repositories::stream_all_watchers(&pool);

        while let Some(result) = stream.next().await {
            match result {
                Err(e) => {
                    eprintln!("Error fetching watcher: {}", e);
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

/// consumes an rx channel and pings all watchers.
/// blocks until the last one.
pub async fn ping_from_stream(rx: async_channel::Receiver<models::Watcher>, pool: &Pool<Sqlite>) {
    while let Ok(watcher) = rx.recv().await {
        let response = reqwest::get(&watcher.url).await;

        let status_code = match response {
            Ok(resp) => resp.status().as_u16(),
            Err(e) => {
                eprintln!("Falha de conexão: {}", e);
                599
            }
        };

        if status_code >= 400 {
            let result = repositories::log_status_change(&pool, watcher.id, "offline").await;

            if let Err(e) = result {
                eprintln!("Error logging status change: {}", e);
            }
        }
    }
}
