//! Prometheus metrics endpoint.

use axum::extract::State;
use axum::response::IntoResponse;

use super::AppState;

/// `GET /` and `GET /metrics` — Returns Prometheus text format.
///
/// Always responds, even if Moonraker is disconnected (serves stale/zero metrics).
/// This decouples the HTTP server from the WebSocket lifecycle.
pub async fn metrics_handler(State(state): State<AppState>) -> impl IntoResponse {
    state.metrics_handle.render()
}
