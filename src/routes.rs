use poem::{Route, get, handler, web::Json};
use poem_openapi::OpenApiService;
use std::sync::Arc;

use crate::features::stats::{get_stats, StatsCache};
use crate::features::{pings, users, watchers};

pub fn with_routes(app: Route, _stats_cache: Arc<StatsCache>) -> Route {
    let api_service = OpenApiService::new(
        (pings::PingsApi, users::UserApi, watchers::WatchersApi),
        "checkup Rest API",
        "1.0",
    );

    let swagger_ui = api_service.swagger_ui();
    let redoc_ui = api_service.redoc();

    app.at("/", get(healthz))
        .at("/stats", get(get_stats))
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

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use poem::{EndpointExt, test::TestClient};

    #[tokio::test]
    async fn test_stats_route() {
        let stats_cache = Arc::new(StatsCache::new());
        stats_cache.set_payload(r#"{"uptime_percent":100.0}"#.to_string());

        let app = with_routes(Route::new(), stats_cache.clone())
            .with(poem::middleware::AddData::new(stats_cache));

        let cli = TestClient::new(app);
        let resp = cli.get("/stats").send().await;

        resp.assert_status(poem::http::StatusCode::OK);
        let body = resp.0.into_body().into_string().await.unwrap();
        assert_eq!(body, r#"{"uptime_percent":100.0}"#);
    }
}
