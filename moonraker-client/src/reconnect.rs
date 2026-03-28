//! Reconnecting WebSocket supervision loop.
//!
//! This module contains the core connection lifecycle logic. An internal tokio
//! task runs [`run_connection_loop`], which:
//!
//! 1. Connects to Moonraker (with exponential backoff on failure)
//! 2. Processes WebSocket messages and commands from the public API
//! 3. Re-subscribes to previously tracked objects after reconnect
//! 4. Emits [`MoonrakerEvent`](crate::MoonrakerEvent) notifications to the consumer
//! 5. On disconnect, cleans up pending RPCs and retries from step 1
//!
//! Communication with the public API (`MoonrakerClient`) happens via channels:
//! - **Inbound**: `mpsc::Receiver<Command>` carries RPC requests and subscribe commands
//! - **Outbound**: `mpsc::Sender<MoonrakerEvent>` delivers events to the consumer

use std::collections::HashMap;
use std::time::{Duration, Instant};

use serde_json::json;
use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use tracing::{debug, info, warn};
use url::Url;

use crate::connection::Connection;
use crate::error::MoonrakerError;
use crate::jsonrpc::{IdGenerator, JsonRpcRequest, ParsedMessage, parse_message};
use crate::subscription::SubscriptionManager;
use crate::{DisconnectReason, KlippyState, MoonrakerEvent, ReconnectConfig};

/// Internal commands sent from [`crate::MoonrakerClient`] to the connection task.
///
/// These are the messages that flow over the `mpsc` channel from the public API
/// into [`run_connection_loop`].
pub(crate) enum Command {
    /// Send a JSON-RPC request and deliver the response (or error) via the
    /// oneshot channel.
    Rpc {
        /// The JSON-RPC method name (e.g. `"server.info"`).
        method: String,
        /// The parameters to include in the request.
        params: serde_json::Value,
        /// Channel to send the response back on.
        response_tx: oneshot::Sender<Result<serde_json::Value, MoonrakerError>>,
    },
    /// Subscribe to Moonraker status objects. The subscription is also tracked
    /// internally so it can be replayed after reconnect.
    Subscribe {
        /// The Moonraker object names to subscribe to.
        objects: Vec<String>,
        /// Channel to send the subscription response back on.
        response_tx: oneshot::Sender<Result<serde_json::Value, MoonrakerError>>,
    },
    /// Request a graceful shutdown of the connection loop.
    Close,
}

/// A pending RPC request awaiting its response from Moonraker.
struct PendingRpc {
    /// Channel to deliver the response on.
    response_tx: oneshot::Sender<Result<serde_json::Value, MoonrakerError>>,
    /// When this request was sent, for timeout tracking.
    sent_at: Instant,
}

/// Compute the next backoff delay using exponential growth with a ceiling.
///
/// Multiplies `current` by `multiplier` and caps the result at `max`.
/// Uses `Duration::from_secs_f64` which saturates rather than panicking
/// on overflow.
///
/// # Parameters
/// - `current`: The current delay duration.
/// - `max`: The maximum allowed delay.
/// - `multiplier`: The exponential growth factor (e.g. 2.0 for doubling).
fn next_delay(current: Duration, max: Duration, multiplier: f64) -> Duration {
    // `as_secs_f64()` returns a finite f64 for any valid Duration.
    // `from_secs_f64()` clamps to Duration::MAX on overflow.
    let next = Duration::from_secs_f64(current.as_secs_f64() * multiplier);
    next.min(max)
}

/// The main connection supervision loop.
///
/// This function runs for the lifetime of the client. It manages:
/// - WebSocket connect/reconnect with exponential backoff
/// - Dispatching JSON-RPC requests from the public API
/// - Parsing incoming WebSocket messages into events
/// - Tracking and replaying subscriptions after reconnect
/// - Timeout enforcement for pending RPC requests
///
/// The loop exits when either the [`CancellationToken`] is cancelled or
/// a [`Command::Close`] is received.
///
/// # Parameters
/// - `url`: The Moonraker WebSocket endpoint URL.
/// - `reconnect_config`: Backoff timing configuration.
/// - `rpc_timeout`: How long to wait for an RPC response before timing out.
/// - `commands`: Channel receiving commands from the public API.
/// - `events`: Channel for sending events to the consumer.
/// - `cancel`: Token to signal graceful shutdown.
pub(crate) async fn run_connection_loop(
    url: Url,
    reconnect_config: ReconnectConfig,
    rpc_timeout: Duration,
    mut commands: mpsc::Receiver<Command>,
    events: mpsc::Sender<MoonrakerEvent>,
    cancel: CancellationToken,
) {
    let id_gen = IdGenerator::new();
    let mut subscriptions = SubscriptionManager::new();
    let mut backoff_delay = reconnect_config.initial_delay;
    let mut attempts: u32 = 0;

    // Outer loop: each iteration is one connection attempt + session.
    loop {
        if cancel.is_cancelled() {
            debug!("Connection loop cancelled before connect attempt");
            return;
        }

        // --- Step 1: Attempt to connect ---
        info!(url = %url, "Connecting to Moonraker");
        let mut conn = match Connection::connect(&url).await {
            Ok(conn) => {
                // Reset backoff on successful connection.
                backoff_delay = reconnect_config.initial_delay;
                attempts = 0;
                conn
            }
            Err(err) => {
                attempts += 1;
                warn!(
                    error = %err,
                    attempt = attempts,
                    delay_ms = backoff_delay.as_millis() as u64,
                    "Failed to connect to Moonraker, retrying"
                );

                // Check if we've exceeded the maximum number of attempts.
                if let Some(max) = reconnect_config.max_attempts
                    && attempts >= max
                {
                    warn!(
                        max_attempts = max,
                        "Maximum reconnect attempts reached, giving up"
                    );
                    return;
                }

                // Wait for the backoff delay, but bail if cancelled.
                tokio::select! {
                    () = cancel.cancelled() => {
                        debug!("Connection loop cancelled during backoff");
                        return;
                    }
                    () = tokio::time::sleep(backoff_delay) => {}
                }

                backoff_delay = next_delay(
                    backoff_delay,
                    reconnect_config.max_delay,
                    reconnect_config.multiplier,
                );
                continue;
            }
        };

        // --- Step 2: Re-subscribe if we had previous subscriptions ---
        let tracked = subscriptions.subscribed_objects().to_vec();
        if !tracked.is_empty() {
            info!(
                count = tracked.len(),
                "Re-subscribing to previously tracked objects"
            );
            if let Err(err) = send_subscribe_rpc(&mut conn, &id_gen, &tracked).await {
                warn!(
                    error = %err,
                    "Failed to re-subscribe after reconnect"
                );
                // Treat as a connection failure — go back to the top of the loop.
                let _ = events
                    .send(MoonrakerEvent::Disconnected {
                        reason: DisconnectReason::NetworkError(err.to_string()),
                    })
                    .await;
                backoff_delay = next_delay(
                    backoff_delay,
                    reconnect_config.max_delay,
                    reconnect_config.multiplier,
                );
                continue;
            }
        }

        // --- Step 3: Emit Connected event ---
        let _ = events.send(MoonrakerEvent::Connected).await;
        info!("Connected to Moonraker");

        // --- Step 4: Enter the message processing loop ---
        let mut pending: HashMap<u64, PendingRpc> = HashMap::new();
        let disconnect_reason = process_messages(
            &mut conn,
            &id_gen,
            &mut subscriptions,
            rpc_timeout,
            &mut commands,
            &events,
            &cancel,
            &mut pending,
        )
        .await;

        // --- Step 5: Clean up after disconnect ---
        // Fail all pending RPCs — their responses will never arrive.
        for (_id, rpc) in pending.drain() {
            let _ = rpc.response_tx.send(Err(MoonrakerError::ChannelClosed));
        }

        // Check whether we should exit before sending the event (which
        // moves `disconnect_reason`).
        let should_exit =
            matches!(disconnect_reason, DisconnectReason::ClientRequested) || cancel.is_cancelled();

        let _ = events
            .send(MoonrakerEvent::Disconnected {
                reason: disconnect_reason,
            })
            .await;

        if should_exit {
            debug!("Connection loop exiting after client-requested close");
            return;
        }

        // Apply backoff before reconnecting.
        info!(
            delay_ms = backoff_delay.as_millis() as u64,
            "Disconnected from Moonraker, reconnecting"
        );
        tokio::select! {
            () = cancel.cancelled() => {
                debug!("Connection loop cancelled during reconnect backoff");
                return;
            }
            () = tokio::time::sleep(backoff_delay) => {}
        }
        backoff_delay = next_delay(
            backoff_delay,
            reconnect_config.max_delay,
            reconnect_config.multiplier,
        );
    }
}

/// Inner message processing loop.
///
/// Uses `tokio::select!` to concurrently handle:
/// - Incoming WebSocket messages (responses and notifications)
/// - Commands from the public API (RPC requests, subscribe, close)
/// - Cancellation
/// - RPC timeout checks
///
/// Returns the reason the loop exited (so the outer loop can decide whether
/// to reconnect or shut down).
#[allow(clippy::too_many_arguments)]
async fn process_messages(
    conn: &mut Connection,
    id_gen: &IdGenerator,
    subscriptions: &mut SubscriptionManager,
    rpc_timeout: Duration,
    commands: &mut mpsc::Receiver<Command>,
    events: &mpsc::Sender<MoonrakerEvent>,
    cancel: &CancellationToken,
    pending: &mut HashMap<u64, PendingRpc>,
) -> DisconnectReason {
    // Check for timed-out RPCs every second.
    let mut timeout_interval = tokio::time::interval(Duration::from_secs(1));
    // The first tick completes immediately; skip it so we don't do a
    // spurious timeout check right away.
    timeout_interval.tick().await;

    loop {
        tokio::select! {
            // --- Branch 1: WebSocket message from Moonraker ---
            msg = conn.next_message() => {
                match msg {
                    Some(Ok(text)) => {
                        handle_ws_message(
                            &text, pending, events, subscriptions,
                        ).await;
                    }
                    Some(Err(err)) => {
                        warn!(error = %err, "WebSocket error, disconnecting");
                        return DisconnectReason::NetworkError(err.to_string());
                    }
                    None => {
                        // Stream ended (close frame or EOF).
                        info!("WebSocket connection closed by server");
                        return DisconnectReason::ServerClosed;
                    }
                }
            }

            // --- Branch 2: Command from the public API ---
            cmd = commands.recv() => {
                match cmd {
                    Some(Command::Rpc { method, params, response_tx }) => {
                        let id = id_gen.next_id();
                        let request = if params.is_null() {
                            JsonRpcRequest::new(method, id)
                        } else {
                            JsonRpcRequest::with_params(method, id, params)
                        };

                        // Serialize and send the request over the WebSocket.
                        let json = match serde_json::to_string(&request) {
                            Ok(j) => j,
                            Err(err) => {
                                let _ = response_tx.send(Err(MoonrakerError::from(err)));
                                continue;
                            }
                        };

                        if let Err(err) = conn.send_text(&json).await {
                            let _ = response_tx.send(Err(
                                MoonrakerError::WebSocket(err.to_string())
                            ));
                            return DisconnectReason::NetworkError(err.to_string());
                        }

                        // Track the pending response.
                        pending.insert(id, PendingRpc {
                            response_tx,
                            sent_at: Instant::now(),
                        });
                    }
                    Some(Command::Subscribe { objects, response_tx }) => {
                        // Build the subscription objects map: each object name
                        // maps to null (meaning "subscribe to all fields").
                        let id = id_gen.next_id();
                        let objects_map: serde_json::Map<String, serde_json::Value> =
                            objects.iter()
                                .map(|name| (name.clone(), serde_json::Value::Null))
                                .collect();
                        let params = json!({ "objects": objects_map });
                        let request = JsonRpcRequest::with_params(
                            "printer.objects.subscribe".to_owned(),
                            id,
                            params,
                        );

                        let json = match serde_json::to_string(&request) {
                            Ok(j) => j,
                            Err(err) => {
                                let _ = response_tx.send(Err(MoonrakerError::from(err)));
                                continue;
                            }
                        };

                        if let Err(err) = conn.send_text(&json).await {
                            let _ = response_tx.send(Err(
                                MoonrakerError::WebSocket(err.to_string())
                            ));
                            return DisconnectReason::NetworkError(err.to_string());
                        }

                        // Track the subscription for future reconnects.
                        subscriptions.track(&objects);

                        pending.insert(id, PendingRpc {
                            response_tx,
                            sent_at: Instant::now(),
                        });
                    }
                    Some(Command::Close) => {
                        info!("Close requested, shutting down connection");
                        // Best-effort: send a WebSocket close frame.
                        let _ = conn.close().await;
                        return DisconnectReason::ClientRequested;
                    }
                    None => {
                        // The command channel was dropped, meaning the
                        // MoonrakerClient was dropped. Shut down.
                        debug!("Command channel closed, shutting down");
                        let _ = conn.close().await;
                        return DisconnectReason::ClientRequested;
                    }
                }
            }

            // --- Branch 3: Cancellation token ---
            () = cancel.cancelled() => {
                info!("Cancellation requested, shutting down connection");
                let _ = conn.close().await;
                return DisconnectReason::ClientRequested;
            }

            // --- Branch 4: Periodic RPC timeout check ---
            _ = timeout_interval.tick() => {
                expire_timed_out_rpcs(pending, rpc_timeout);
            }
        }
    }
}

/// Handle a parsed WebSocket text message from Moonraker.
///
/// Dispatches responses to their pending RPC oneshot channels and converts
/// notifications into [`MoonrakerEvent`] values sent to the consumer.
async fn handle_ws_message(
    text: &str,
    pending: &mut HashMap<u64, PendingRpc>,
    events: &mpsc::Sender<MoonrakerEvent>,
    subscriptions: &mut SubscriptionManager,
) {
    let parsed = match parse_message(text) {
        Ok(p) => p,
        Err(err) => {
            warn!(error = %err, "Failed to parse WebSocket message, skipping");
            return;
        }
    };

    match parsed {
        ParsedMessage::Response(resp) => {
            // Match response to a pending request by its ID.
            let id = match resp.id {
                Some(id) => id,
                None => {
                    debug!("Received response with no ID, skipping");
                    return;
                }
            };

            if let Some(rpc) = pending.remove(&id) {
                let result = if let Some(err) = resp.error {
                    Err(MoonrakerError::RpcError(err))
                } else {
                    // `result` is None for successful responses with no payload;
                    // normalize to `Value::Null` so callers always get a value.
                    Ok(resp.result.unwrap_or(serde_json::Value::Null))
                };
                // The receiver may have been dropped (timeout or caller gave up).
                // That's fine — we just discard the result.
                let _ = rpc.response_tx.send(result);
            } else {
                debug!(id, "Received response for unknown request ID");
            }
        }
        ParsedMessage::Notification(notif) => {
            handle_notification(&notif.method, &notif.params, events, subscriptions).await;
        }
    }
}

/// Map a Moonraker notification to one or more [`MoonrakerEvent`] values.
///
/// Moonraker sends several notification types:
/// - `notify_status_update` — printer object status changes
/// - `notify_klippy_ready` / `notify_klippy_shutdown` / `notify_klippy_disconnected`
/// - `notify_proc_stat_update` — Moonraker server resource stats
///
/// Unknown methods are logged at debug level and ignored.
async fn handle_notification(
    method: &str,
    params: &serde_json::Value,
    events: &mpsc::Sender<MoonrakerEvent>,
    _subscriptions: &mut SubscriptionManager,
) {
    match method {
        "notify_status_update" => {
            // Params structure: `[{"toolhead": {...}, "extruder": {...}}, <timestamp>]`
            // The first element is an object whose keys are printer object names.
            if let Some(objects) = params.as_array().and_then(|arr| arr.first())
                && let Some(map) = objects.as_object()
            {
                for (key, data) in map {
                    let _ = events
                        .send(MoonrakerEvent::StatusUpdate {
                            key: key.clone(),
                            data: data.clone(),
                        })
                        .await;
                }
            }
        }
        "notify_klippy_ready" => {
            let _ = events
                .send(MoonrakerEvent::KlippyStateChanged(KlippyState::Ready))
                .await;
        }
        "notify_klippy_shutdown" => {
            let _ = events
                .send(MoonrakerEvent::KlippyStateChanged(KlippyState::Shutdown))
                .await;
        }
        "notify_klippy_disconnected" => {
            let _ = events
                .send(MoonrakerEvent::KlippyStateChanged(
                    KlippyState::Disconnected,
                ))
                .await;
        }
        "notify_proc_stat_update" => {
            // Moonraker server stats — emit as a StatusUpdate under the
            // "moonraker" key. The entire params array is the data payload.
            let _ = events
                .send(MoonrakerEvent::StatusUpdate {
                    key: "moonraker".to_owned(),
                    data: params.clone(),
                })
                .await;
        }
        "notify_sensor_update" => {
            // User-defined Moonraker sensors (e.g. MQTT power monitors).
            // Params structure: `[{"sensor_id": {"field": value, ...}, ...}]`
            if let Some(sensors) = params.as_array().and_then(|arr| arr.first())
                && let Some(map) = sensors.as_object()
            {
                for (sensor, values) in map {
                    let _ = events
                        .send(MoonrakerEvent::SensorUpdate {
                            sensor: sensor.clone(),
                            values: values.clone(),
                        })
                        .await;
                }
            }
        }
        _ => {
            debug!(method, payload = %params, "Unknown notification method, skipping");
        }
    }
}

/// Send a `printer.objects.subscribe` RPC directly on the connection.
///
/// Used during reconnect to re-subscribe to previously tracked objects
/// before the message processing loop starts. Does **not** go through
/// the command channel — it writes directly to the WebSocket.
///
/// # Parameters
/// - `conn`: The active WebSocket connection.
/// - `id_gen`: ID generator for the JSON-RPC request.
/// - `objects`: The object names to subscribe to.
///
/// # Errors
/// Returns [`MoonrakerError`] if serialization or sending fails.
async fn send_subscribe_rpc(
    conn: &mut Connection,
    id_gen: &IdGenerator,
    objects: &[String],
) -> Result<(), MoonrakerError> {
    let objects_map: serde_json::Map<String, serde_json::Value> = objects
        .iter()
        .map(|name| (name.clone(), serde_json::Value::Null))
        .collect();
    let params = json!({ "objects": objects_map });
    let id = id_gen.next_id();
    let request = JsonRpcRequest::with_params("printer.objects.subscribe".to_owned(), id, params);
    let json_text = serde_json::to_string(&request)?;
    conn.send_text(&json_text)
        .await
        .map_err(|err| MoonrakerError::WebSocket(err.to_string()))?;

    // We don't wait for the response here — the subscription will take effect
    // and status updates will arrive as notifications. The reconnect path
    // is fire-and-forget for simplicity.
    Ok(())
}

/// Check all pending RPCs and fail any that have exceeded the timeout.
///
/// Collects the IDs of expired entries first, then removes them one by one
/// so we can send [`MoonrakerError::RpcTimeout`] on each response channel
/// before it is dropped.
fn expire_timed_out_rpcs(pending: &mut HashMap<u64, PendingRpc>, rpc_timeout: Duration) {
    let now = Instant::now();

    // Collect expired IDs first. We can't remove entries while iterating
    // because `HashMap::retain` doesn't let us move out of the value.
    let expired_ids: Vec<u64> = pending
        .iter()
        .filter(|(_id, rpc)| now.duration_since(rpc.sent_at) > rpc_timeout)
        .map(|(id, _rpc)| *id)
        .collect();

    for id in expired_ids {
        if let Some(rpc) = pending.remove(&id) {
            debug!(id, "RPC request timed out");
            let _ = rpc.response_tx.send(Err(MoonrakerError::RpcTimeout));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that the backoff delay doubles correctly.
    #[test]
    fn next_delay_doubles() {
        let current = Duration::from_secs(1);
        let max = Duration::from_secs(60);
        let result = next_delay(current, max, 2.0);
        assert_eq!(result, Duration::from_secs(2));
    }

    /// Verifies that the delay is capped at the maximum.
    #[test]
    fn next_delay_caps_at_max() {
        let current = Duration::from_secs(40);
        let max = Duration::from_secs(60);
        let result = next_delay(current, max, 2.0);
        assert_eq!(result, Duration::from_secs(60));
    }

    /// Verifies that a delay already at the maximum stays there.
    #[test]
    fn next_delay_at_max_stays() {
        let current = Duration::from_secs(60);
        let max = Duration::from_secs(60);
        let result = next_delay(current, max, 2.0);
        assert_eq!(result, Duration::from_secs(60));
    }

    /// Verifies that a fractional multiplier works correctly.
    #[test]
    fn next_delay_fractional_multiplier() {
        let current = Duration::from_secs(2);
        let max = Duration::from_secs(60);
        let result = next_delay(current, max, 1.5);
        assert_eq!(result, Duration::from_secs(3));
    }
}
