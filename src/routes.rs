use poem::{Route, get, handler, web::Json};
use poem_openapi::OpenApiService;

use crate::features::{pings, users, watchers};

pub fn with_routes(app: Route) -> Route {
    let api_service = OpenApiService::new(
        (pings::PingsApi, users::UserApi, watchers::WatchersApi),
        "checkup Rest API",
        "1.0",
    );

    let swagger_ui = api_service.swagger_ui();
    let redoc_ui = api_service.redoc();

    app.at("/", get(healthz))
        .nest("/api", api_service)
        .nest("/docs", swagger_ui)
        .nest("/redoc", redoc_ui)
}

#[derive(Debug, serde::Serialize)]
struct Healthz {
    message: String,
}

// health check handler
#[handler]
fn healthz() -> Json<Healthz> {
    Json(Healthz {
        message: "server online".to_string(),
    })
}
