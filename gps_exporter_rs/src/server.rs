use crate::metrics::GPSMetricsCollector;
use axum::{extract::State, response::IntoResponse, routing::get, Router};
use std::sync::{Arc, Mutex};
use tracing::{debug, error};

pub type SharedCollector = Arc<Mutex<GPSMetricsCollector>>;

async fn metrics_handler(State(collector): State<SharedCollector>) -> impl IntoResponse {
    debug!("Received /metrics request");

    let metrics = collector
        .lock()
        .ok()
        .and_then(|c| c.render_metrics().ok())
        .unwrap_or_else(|| {
            error!("Failed to render metrics");
            "# Error rendering metrics\n".to_string()
        });

    debug!("Returning {} bytes of metrics", metrics.len());

    (
        [("Content-Type", "text/plain; version=0.0.4")],
        metrics,
    )
}

pub fn create_app(collector: SharedCollector) -> Router {
    Router::new()
        .route("/metrics", get(metrics_handler))
        .with_state(collector)
}
