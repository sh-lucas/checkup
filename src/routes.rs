use poem::{EndpointExt, Route, get, handler, middleware::Cors, web::Json};
use poem_openapi::OpenApiService;
use std::sync::Arc;

use crate::features::metrics::MetricsCache;
use crate::features::{metrics, pings, users, watchers};

pub fn with_routes(app: Route, _metrics_cache: Arc<MetricsCache>) -> Route {
    let api_service = OpenApiService::new(
        (
            pings::PingsApi,
            users::UserApi,
            watchers::WatchersApi,
            metrics::MetricsApi,
        ),
        "checkup Rest API",
        "1.0",
    );

    let swagger_ui = api_service.swagger_ui();
    let redoc_ui = api_service.redoc();

    app.at("/", get(healthz))
        .nest("/api", api_service.with(Cors::new()))
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
    use futures::StreamExt;
    use poem::{EndpointExt, test::TestClient};

    #[tokio::test]
    async fn test_metrics_route() {
        use crate::features::metrics::MetricsResponse;

        let metrics_cache = Arc::new(MetricsCache::new());
        metrics_cache.set_payload(MetricsResponse {
            uptime_percent: 100.0,
            ..MetricsResponse::default()
        });

        let app = with_routes(Route::new(), metrics_cache.clone())
            .with(poem::middleware::AddData::new(metrics_cache));

        let cli = TestClient::new(app);
        let resp = cli.get("/api/metrics").send().await;

        resp.assert_status(poem::http::StatusCode::OK);
        let body = resp.0.into_body().into_string().await.unwrap();
        assert!(body.contains(r#""uptime_percent":100.0"#));
    }

    #[tokio::test]
    async fn test_metrics_stream_route() {
        use crate::features::metrics::MetricsResponse;

        let metrics_cache = Arc::new(MetricsCache::new());
        metrics_cache.set_payload(MetricsResponse {
            uptime_percent: 100.0,
            ..MetricsResponse::default()
        });

        let app = with_routes(Route::new(), metrics_cache.clone())
            .with(poem::middleware::AddData::new(metrics_cache));

        let cli = TestClient::new(app);
        let resp = cli.get("/api/metrics/stream").send().await;

        resp.assert_status(poem::http::StatusCode::OK);
        resp.assert_header("content-type", "text/event-stream");

        let mut body = resp.0.into_body().into_bytes_stream();
        let first_chunk = body.next().await.unwrap().unwrap();
        let chunk_str = String::from_utf8(first_chunk.to_vec()).unwrap();
        assert!(chunk_str.contains("data:"));
        assert!(chunk_str.contains(r#""uptime_percent":100"#));
    }

    #[tokio::test]
    async fn test_cors_headers() {
        let metrics_cache = Arc::new(MetricsCache::new());
        let app = with_routes(Route::new(), metrics_cache.clone())
            .with(poem::middleware::AddData::new(metrics_cache));

        let cli = TestClient::new(app);

        // 1. Regular GET request with Origin should get Access-Control-Allow-Origin back
        let resp = cli
            .get("/api/metrics")
            .header("origin", "https://sh-lucas.dev")
            .send()
            .await;
        resp.assert_status(poem::http::StatusCode::OK);
        resp.assert_header("access-control-allow-origin", "https://sh-lucas.dev");

        // 2. Preflight OPTIONS request should succeed and return CORS headers
        let resp = cli
            .options("/api/metrics")
            .header("origin", "https://example.com")
            .header("access-control-request-method", "GET")
            .send()
            .await;
        resp.assert_status(poem::http::StatusCode::OK);
        resp.assert_header("access-control-allow-origin", "https://example.com");
    }
}
