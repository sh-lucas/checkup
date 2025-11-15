use poem::{Route, get, handler, web::Json};

pub fn with_routes(app: Route) -> Route {
    app.at("/", get(healthz))
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
