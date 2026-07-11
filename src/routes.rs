use poem::{EndpointExt, Route, middleware::Cors};
use poem_openapi::{Object, OpenApi, OpenApiService, payload::Json};

use crate::api_response;
use crate::features::{metrics, pings, users, watchers};

#[derive(Debug, serde::Serialize, Object)]
pub struct Healthz {
    pub message: String,
}

api_response! {
    pub enum HealthResponse {
        #[oai(status = 200)]
        Ok(Json<Healthz>),
    }
}

pub struct SystemApi;

#[OpenApi]
impl SystemApi {
    /// Server health check
    #[oai(path = "/", method = "get")]
    #[allow(clippy::unused_async)]
    async fn healthz(&self) -> HealthResponse {
        HealthResponse::Ok(Json(Healthz {
            message: "server online".to_string(),
        }))
    }
}

pub fn with_routes(app: Route) -> Route {
    let api_service = OpenApiService::new(
        (
            users::UserApi,
            pings::PingsApi,
            watchers::WatchersApi,
            metrics::MetricsApi,
            SystemApi,
        ),
        "checkup Rest API",
        "1.0",
    );

    let swagger_ui = api_service.swagger_ui();
    let redoc_ui = api_service.redoc();

    app.nest("/", api_service.with(Cors::new()))
        .nest("/docs", swagger_ui)
        .nest("/redoc", redoc_ui)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use futures::StreamExt;
    use poem::{EndpointExt, test::TestClient};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_healthz_route() {
        let app = with_routes(Route::new());
        let cli = TestClient::new(app);
        let resp = cli.get("/").send().await;

        resp.assert_status(poem::http::StatusCode::OK);
        let body = resp.0.into_body().into_string().await.unwrap();
        assert!(body.contains(r#""message":"server online""#));
    }

    #[tokio::test]
    async fn test_metrics_route() {
        use crate::features::metrics::MetricsResponse;

        let metrics_cache = Arc::new(metrics::MetricsCache::new());
        metrics_cache.set_payload(MetricsResponse {
            uptime_percent: 100.0,
            ..MetricsResponse::default()
        });

        let app = with_routes(Route::new()).with(poem::middleware::AddData::new(metrics_cache));

        let cli = TestClient::new(app);
        let resp = cli.get("/metrics").send().await;

        resp.assert_status(poem::http::StatusCode::OK);
        let body = resp.0.into_body().into_string().await.unwrap();
        assert!(body.contains(r#""uptime_percent":100.0"#));
    }

    #[tokio::test]
    async fn test_metrics_stream_route() {
        use crate::features::metrics::MetricsResponse;

        let metrics_cache = Arc::new(metrics::MetricsCache::new());
        metrics_cache.set_payload(MetricsResponse {
            uptime_percent: 100.0,
            ..MetricsResponse::default()
        });

        let app = with_routes(Route::new()).with(poem::middleware::AddData::new(metrics_cache));

        let cli = TestClient::new(app);
        let resp = cli.get("/metrics/stream").send().await;

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
        let metrics_cache = Arc::new(metrics::MetricsCache::new());
        let app = with_routes(Route::new()).with(poem::middleware::AddData::new(metrics_cache));

        let cli = TestClient::new(app);

        // 1. Regular GET request with Origin should get Access-Control-Allow-Origin back
        let resp = cli
            .get("/metrics")
            .header("origin", "https://sh-lucas.dev")
            .send()
            .await;
        resp.assert_status(poem::http::StatusCode::OK);
        resp.assert_header("access-control-allow-origin", "https://sh-lucas.dev");

        // 2. Preflight OPTIONS request should succeed and return CORS headers
        let resp = cli
            .options("/metrics")
            .header("origin", "https://example.com")
            .header("access-control-request-method", "GET")
            .send()
            .await;
        resp.assert_status(poem::http::StatusCode::OK);
        resp.assert_header("access-control-allow-origin", "https://example.com");
    }
}
