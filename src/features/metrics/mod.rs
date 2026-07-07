mod metrics_handlers;

pub use metrics_handlers::{MetricsState, update_metrics};

use futures::StreamExt;
use futures::stream::BoxStream;
use poem::web::Data;
use poem_openapi::{
    Object, OpenApi,
    payload::{EventStream, Json},
};
use std::sync::{Arc, RwLock};
use tokio::sync::watch;

#[derive(serde::Serialize, serde::Deserialize, Debug, Object, Clone)]
pub struct MetricsResponse {
    pub uptime_percent: f64,
    pub load_avg: f64,
    pub memory_percent: f64,
    pub cpu_percent: f64,
    pub cpu_usage: f64,
    pub io_percent: f64,
    pub disk_percent: f64,
}

impl Default for MetricsResponse {
    fn default() -> Self {
        Self {
            uptime_percent: 100.0,
            load_avg: 0.0,
            memory_percent: 0.0,
            cpu_percent: 0.0,
            cpu_usage: 0.0,
            io_percent: 0.0,
            disk_percent: 0.0,
        }
    }
}

pub struct MetricsCache {
    payload: RwLock<Arc<MetricsResponse>>,
    tx: watch::Sender<Arc<MetricsResponse>>,
}

impl MetricsCache {
    #[must_use]
    pub fn new() -> Self {
        let initial = Arc::new(MetricsResponse::default());
        let (tx, _rx) = watch::channel(initial.clone());
        Self {
            payload: RwLock::new(initial),
            tx,
        }
    }

    pub fn set_payload(&self, new_payload: MetricsResponse) {
        let arc_payload = Arc::new(new_payload);
        if let Ok(mut w) = self.payload.write() {
            *w = arc_payload.clone();
        }
        let _ = self.tx.send(arc_payload);
    }

    #[must_use]
    pub fn get_payload(&self) -> Arc<MetricsResponse> {
        if let Ok(r) = self.payload.read() {
            r.clone()
        } else {
            Arc::new(MetricsResponse::default())
        }
    }

    pub fn get_stream(&self) -> impl futures::Stream<Item = Arc<MetricsResponse>> + 'static {
        let rx = self.tx.subscribe();
        let initial = rx.borrow().clone();

        let first_stream = futures::stream::once(async move { initial });
        let rest_stream = futures::stream::unfold(rx, |mut rx| async move {
            if rx.changed().await.is_ok() {
                let val = rx.borrow().clone();
                Some((val, rx))
            } else {
                None
            }
        });

        futures::StreamExt::chain(first_stream, rest_stream)
    }
}

impl Default for MetricsCache {
    fn default() -> Self {
        Self::new()
    }
}

crate::api_response! {
    pub enum GetMetricsResponse {
        #[oai(status = 200)]
        Ok(Json<MetricsResponse>),
    }
}

pub struct MetricsApi;

#[OpenApi]
impl MetricsApi {
    /// Retrieve system metrics including uptime, CPU, load, memory, disk, and IO percent.
    #[oai(path = "/metrics", method = "get")]
    #[allow(clippy::unused_async)]
    pub async fn get_metrics(&self, cache: Data<&Arc<MetricsCache>>) -> GetMetricsResponse {
        let payload = cache.0.get_payload();
        GetMetricsResponse::Ok(Json((*payload).clone()))
    }

    /// Stream system metrics updates in real-time.
    #[oai(path = "/metrics/stream", method = "get")]
    #[allow(clippy::unused_async)]
    pub async fn stream_metrics(
        &self,
        cache: Data<&Arc<MetricsCache>>,
    ) -> EventStream<BoxStream<'static, MetricsResponse>> {
        let cache = (*cache.0).clone();
        let stream = cache.get_stream().map(|arc| (*arc).clone()).boxed();
        EventStream::new(stream)
    }
}
