use crate::features::users;
use crate::features::watchers;
use poem::{Route, get, handler, post, web::Json};

pub fn with_routes(app: Route) -> Route {
    app.at("/", get(healthz))
        .at("/watchers/create", post(watchers::post_watcher))
        .at("/users/create", post(users::post_user))
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
