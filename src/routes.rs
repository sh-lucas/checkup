use crate::handlers::add_watcher::add_watcher;
use poem::{Route, get, handler, post, web::Json};

pub fn with_routes(app: Route) -> Route {
    return app
        .at("/", get(healthz))
        .at("/watchers/create", post(add_watcher));
}

#[derive(Debug, serde::Serialize)]
struct Healthz {
    message: String,
}

// ping handler
#[handler]
fn healthz() -> Json<Healthz> {
    Json(Healthz {
        message: "server online.".to_string(),
    })
}
