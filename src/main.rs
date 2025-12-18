use std::env;

use dotenvy::dotenv;
use poem::{EndpointExt, Route, Server, listener::TcpListener, middleware::AddData};

mod database;
mod features;
mod middlewares;
mod routes;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();

    let pool = database::setup_database().await;

    // web api server:
    let app = routes::with_routes(Route::new())
        .with(AddData::new(pool.clone()))
        .with(middlewares::BasicLog);

    let port = env::var("PORT").expect("PORT not set in environment variables.");
    let host = format!("0.0.0.0:{}", port);
    println!("Listening on http://{}", host);

    features::watchers::start_watching(&pool, 10);

    Server::new(TcpListener::bind(host)).run(app).await
}
