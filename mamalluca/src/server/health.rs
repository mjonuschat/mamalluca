//! Health check endpoint.

use axum::Json;
use axum::extract::State;
use axum::http::StatusCode;
use axum::response::IntoResponse;
use serde_json::json;
use std::sync::atomic::Ordering;

use super::AppState;

/// `GET /health` — Returns connection status as JSON.
///
/// - 200 OK with `{"status": "ok", "moonraker_connected": true}` when connected
/// - 503 Service Unavailable with `{"status": "degraded", ...}` when disconnected
///
/// Useful for container health checks and load balancer probes.
pub async fn health_handler(State(state): State<AppState>) -> impl IntoResponse {
    let connected = state.connection_status.load(Ordering::Relaxed);
    let status = if connected {
        StatusCode::OK
    } else {
        StatusCode::SERVICE_UNAVAILABLE
    };
    let body = json!({
        "status": if connected { "ok" } else { "degraded" },
        "moonraker_connected": connected,
    });
    (status, Json(body))
}
