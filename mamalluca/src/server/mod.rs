//! HTTP server for Prometheus metrics and health checks.

pub mod health;
pub mod metrics;

use axum::Router;
use axum::routing::get;
use metrics_exporter_prometheus::PrometheusHandle;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;
use tower_http::trace::TraceLayer;

/// Shared state accessible by all HTTP handlers.
#[derive(Clone)]
pub struct AppState {
    /// Handle to render Prometheus metrics as text
    pub metrics_handle: PrometheusHandle,
    /// Whether the Moonraker WebSocket connection is active
    pub connection_status: Arc<AtomicBool>,
}

/// Build the axum router with all routes.
///
/// Routes:
/// - `GET /health` — JSON health check with connection status
/// - `GET /` and `GET /metrics` — Prometheus text metrics
pub fn app(state: AppState) -> Router {
    Router::new()
        .route("/", get(metrics::metrics_handler))
        .route("/metrics", get(metrics::metrics_handler))
        .route("/health", get(health::health_handler))
        .with_state(state)
        .layer(TraceLayer::new_for_http())
}
