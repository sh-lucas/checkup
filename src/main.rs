use std::env;

use dotenvy::dotenv;
use poem::{Route, Server, listener::TcpListener};

mod routes;
use routes::with_routes;

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    dotenv().ok();

    // creates the default handler and adds the routes =)
    let app = with_routes(Route::new());

    let port = env::var("PORT").expect("PORT not set in environment variables.");
    let host = format!("0.0.0.0:{}", port);
    println!("Listening on http://{}", host);

    Server::new(TcpListener::bind(host)).run(app).await
}
