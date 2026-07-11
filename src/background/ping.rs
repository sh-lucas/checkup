use crate::features::{pings, watchers};
use futures::StreamExt;
use sqlx::pool::Pool;
use sqlx::sqlite::Sqlite;

pub fn spawn(
    pool: Pool<Sqlite>,
    interval_secs: u64,
    num_workers: usize,
) -> tokio::task::JoinHandle<()> {
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
