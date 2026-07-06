mod stats_handlers;

pub use stats_handlers::{update_stats, StatsState};

use poem::web::Data;
use poem::{handler, Response, http::header};
use std::sync::{Arc, RwLock};

pub struct StatsCache {
    payload: RwLock<Arc<String>>,
}

impl StatsCache {
    #[must_use]
    pub fn new() -> Self {
        Self {
            payload: RwLock::new(Arc::new("{}".to_string())),
        }
    }

    pub fn set_payload(&self, new_payload: String) {
        if let Ok(mut w) = self.payload.write() {
            *w = Arc::new(new_payload);
        }
    }

    #[must_use]
    pub fn get_payload(&self) -> Arc<String> {
        if let Ok(r) = self.payload.read() {
            r.clone()
        } else {
            Arc::new("{}".to_string())
        }
    }
}

impl Default for StatsCache {
    fn default() -> Self {
        Self::new()
    }
}

#[handler]
#[allow(clippy::needless_pass_by_value)]
pub fn get_stats(cache: Data<&Arc<StatsCache>>) -> Response {
    let payload = cache.0.get_payload();
    Response::builder()
        .header(header::CONTENT_TYPE, "application/json")
        .body(payload.as_ref().clone())
}
