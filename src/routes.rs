use poem::{Route, get, handler, post, web::Json};

use crate::features::bench;
use crate::features::pings;
use crate::features::users;
use crate::features::watchers;

pub fn with_routes(app: Route) -> Route {
    app.at("/", get(healthz))
        .at("/watchers/create", post(watchers::post_watcher))
        .at("/watchers/list", get(watchers::get_my_watchers))
        .at("/users/create", post(users::post_user))
        .at("/pings", get(pings::get_down_pings))
        .at("/bench/heavy", get(bench::bench_heavy))
        .at("/bench/light", get(bench::bench_light))
        .at("/bench/write", post(bench::bench_write))
}

#[derive(Debug, serde::Serialize)]
struct Healthz {
    message: String,
}

#[handler]
fn healthz() -> Json<Healthz> {
    Json(Healthz {
        message: "server online".to_string(),
    })
}
