use poem::{Route, get, handler, post, web::Json};

use crate::features::pings;
use crate::features::users;
use crate::features::watchers;

pub fn with_routes(app: Route) -> Route {
    app.at("/", get(healthz))
        // watchers
        .at("/watchers/create", post(watchers::post_watcher))
        .at("/watchers/list", get(watchers::get_my_watchers))
        // users
        .at("/users/create", post(users::post_user))
        // pings
        .at("/pings", get(pings::get_down_pings))
}

#[derive(Debug, serde::Serialize)]
struct Healthz {
    message: String,
}

// ping handler
#[handler]
fn healthz() -> Json<Healthz> {
    Json(Healthz {
        message: "server online".to_string(),
    })
}
