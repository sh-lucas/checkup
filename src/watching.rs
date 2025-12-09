use sqlx::pool::Pool;
use sqlx::sqlite::Sqlite;

pub async fn watch(poll: &Pool<Sqlite>, watcher_id: i64) -> bool {
    let result = sqlx::query!("SELECT * FROM watchers WHERE id = ?", watcher_id)
        .fetch_one(poll)
        .await;

    let watcher = match result {
        Ok(watcher) => watcher,
        Err(e) => {
            eprintln!("Error fetching watcher; incorrect id? Error: {}", e);
            return false;
        }
    };

    let mut interval =
        tokio::time::interval(tokio::time::Duration::from_secs(watcher.interval as u64));

    // opens a new coroutine to handle the interval ticking and stuff
    tokio::spawn(async move {
        loop {
            println!("Checking service {}...", watcher.url);
            let response = reqwest::get(&watcher.url).await;

            if response.is_err() {
                println!("Service is down");
                continue;
            }

            interval.tick().await;
        }
    });

    true
}
