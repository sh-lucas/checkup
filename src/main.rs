#![warn(clippy::all, clippy::pedantic)]
#![deny(clippy::arithmetic_side_effects)]
#![deny(clippy::unwrap_used)]

use std::{env, time::Duration};

use dotenvy::dotenv;
use poem::{EndpointExt, Route, Server, listener::TcpListener, middleware::AddData};
use tokio::signal;

mod database;
mod features;
mod helpers;
mod middlewares;
mod routes;

// all initializion and setup validation should happen on main
#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();

    let pool = database::setup_database().await;

    // web api server:
    let app = routes::with_routes(Route::new())
        .with(AddData::new(pool.clone()))
        .with(middlewares::BasicLog);

    let port = env::var("PORT").expect("PORT not set in environment variables.");
    let host = format!("0.0.0.0:{port}");
    println!("Listening on http://{host}");

    features::watchers::start_watching(&pool, 10);

    Server::new(TcpListener::bind(host))
        .run_with_graceful_shutdown(app, shutdown_signal(), Some(Duration::from_secs(10)))
        .await?;

    println!("Server exiting.");
    Ok(())
}

/// returns a future that resolves when a sigterm or ctrl+c is received.
async fn shutdown_signal() {
    // signal specifically for ctrl+c:
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // signals for unix and non-unix (windows) systems:
    #[cfg(unix)]
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();

    // select the first signal to be received:
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }

    println!("\nSignal received, starting graceful shutdown...");
}
