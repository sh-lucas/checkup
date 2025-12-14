use crate::models;
use futures::StreamExt;
use sqlx::pool::Pool;
use sqlx::sqlite::Sqlite;

// pub async fn watch(poll: &Pool<Sqlite>, watcher_id: i64) -> bool {
//     let result = sqlx::query!("SELECT * FROM watchers WHERE id = ?", watcher_id)
//         .fetch_one(poll)
//         .await;

//     let watcher = match result {
//         Ok(watcher) => watcher,
//         Err(e) => {
//             eprintln!("Error fetching watcher; incorrect id? Error: {}", e);
//             return false;
//         }
//     };

//     let mut interval =
//         tokio::time::interval(tokio::time::Duration::from_secs(watcher.interval as u64));

//     // opens a new coroutine to handle the interval ticking and stuff
//     tokio::spawn(async move {
//         loop {
//             println!("Checking service {}...", watcher.url);
//             let response = reqwest::get(&watcher.url).await;

//             if response.is_err() {
//                 println!("Service is down");
//                 continue;
//             }

//             interval.tick().await;
//         }
//     });

//     true
// }

// starts another thread to lazy-fetch all watchers without blocking.
// returns the reciever channel to the caller.
pub fn stream_watchers_from(poll: &Pool<Sqlite>) -> async_channel::Receiver<models::Watcher> {
    let poll = poll.clone();
    let (tx, rx) = async_channel::bounded::<models::Watcher>(10);

    tokio::spawn(async move {
        // Create the stream INSIDE the task using the owned `poll`
        let mut stream =
            sqlx::query_as::<_, models::Watcher>("SELECT * FROM watchers").fetch(&poll);

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

// executes simple pings into the stream's watchers and inserts on poll.
pub async fn ping_from_stream(rx: async_channel::Receiver<models::Watcher>, poll: &Pool<Sqlite>) {
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
            let timestamp = chrono::Utc::now();
            let inserted = sqlx::query!(
                "INSERT INTO pings (watcher_id, status_code, timestamp) VALUES (?, ?, ?)",
                watcher.id,
                status_code,
                timestamp,
            )
            .execute(poll)
            .await;

            if let Err(e) = inserted {
                eprintln!("Error inserting ping: {}", e);
            }
        }
    }
}
