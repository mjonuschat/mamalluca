//! Async WebSocket client for Moonraker's JSON-RPC 2.0 API.
//!
//! Provides automatic reconnection with exponential backoff,
//! subscription management, and channel-based event delivery.
//!
//! # Overview
//!
//! [`MoonrakerClient`] is the main entry point. Call [`MoonrakerClient::connect`]
//! to spawn a background connection task and receive an event stream:
//!
//! ```no_run
//! use moonraker_client::{MoonrakerClient, MoonrakerConfig, MoonrakerEvent};
//! use tokio_util::sync::CancellationToken;
//!
//! # async fn example() -> Result<(), moonraker_client::MoonrakerError> {
//! let cancel = CancellationToken::new();
//! let config = MoonrakerConfig::default();
//! let (client, mut events) = MoonrakerClient::connect(config, cancel).await?;
//!
//! // Subscribe to printer objects
//! client.subscribe(&["extruder".into(), "heater_bed".into()]).await?;
//!
//! // Process events
//! while let Some(event) = events.recv().await {
//!     match event {
//!         MoonrakerEvent::StatusUpdate { key, data } => {
//!             println!("{key}: {data}");
//!         }
//!         _ => {}
//!     }
//! }
//! # Ok(())
//! # }
//! ```

pub(crate) mod connection;
pub mod error;
pub mod jsonrpc;
pub(crate) mod reconnect;
pub(crate) mod subscription;

pub use error::MoonrakerError;

use std::time::Duration;

use tokio::sync::{mpsc, oneshot};
use tokio_util::sync::CancellationToken;
use url::Url;

// ---------------------------------------------------------------------------
// Configuration types
// ---------------------------------------------------------------------------

/// Configuration for connecting to a Moonraker instance.
///
/// Use [`Default::default()`] for sensible defaults, then override fields
/// as needed.
///
/// # Fields
/// - `url`: The WebSocket URL (default: `ws://localhost:7125/websocket`).
/// - `reconnect`: Backoff timing for reconnection attempts.
/// - `rpc_timeout`: How long to wait for a JSON-RPC response (default: 10s).
#[derive(Debug, Clone)]
pub struct MoonrakerConfig {
    /// The Moonraker WebSocket endpoint URL.
    pub url: Url,
    /// Reconnection backoff configuration.
    pub reconnect: ReconnectConfig,
    /// Maximum time to wait for an RPC response before timing out.
    pub rpc_timeout: Duration,
}

impl Default for MoonrakerConfig {
    fn default() -> Self {
        Self {
            // `Url::parse` on a well-known constant will always succeed.
            url: Url::parse("ws://localhost:7125/websocket").expect("default URL is valid"),
            reconnect: ReconnectConfig::default(),
            rpc_timeout: Duration::from_secs(10),
        }
    }
}

/// Configuration for the exponential backoff used during reconnection.
///
/// The delay between reconnect attempts starts at `initial_delay` and
/// grows by `multiplier` each time, capping at `max_delay`. Optionally,
/// a maximum number of attempts can be set.
#[derive(Debug, Clone, Copy)]
pub struct ReconnectConfig {
    /// The delay before the first reconnection attempt.
    pub initial_delay: Duration,
    /// The maximum delay between reconnection attempts.
    pub max_delay: Duration,
    /// The exponential growth factor (e.g. 2.0 for doubling).
    pub multiplier: f64,
    /// Optional cap on the number of reconnection attempts.
    /// `None` means retry indefinitely.
    pub max_attempts: Option<u32>,
}

impl Default for ReconnectConfig {
    fn default() -> Self {
        Self {
            initial_delay: Duration::from_secs(1),
            max_delay: Duration::from_secs(60),
            multiplier: 2.0,
            max_attempts: None,
        }
    }
}

// ---------------------------------------------------------------------------
// Event types
// ---------------------------------------------------------------------------

/// Events emitted by the Moonraker client to the consumer.
///
/// Receive these from the `mpsc::Receiver<MoonrakerEvent>` returned by
/// [`MoonrakerClient::connect`].
#[derive(Debug)]
pub enum MoonrakerEvent {
    /// The WebSocket connection has been established (or re-established).
    ///
    /// After reconnect, any previously tracked subscriptions are
    /// automatically re-subscribed before this event is emitted.
    Connected,

    /// The WebSocket connection was lost or closed.
    Disconnected {
        /// Why the connection ended.
        reason: DisconnectReason,
    },

    /// Klippy (the Klipper host software) changed state.
    KlippyStateChanged(KlippyState),

    /// A printer object status update was received.
    ///
    /// `key` is the Moonraker object name (e.g. `"extruder"`, `"heater_bed"`).
    /// `data` contains the updated fields as a JSON value.
    StatusUpdate {
        /// The Moonraker object name that was updated.
        key: String,
        /// The JSON payload containing updated fields.
        data: serde_json::Value,
    },

    /// A Moonraker sensor update was received (`notify_sensor_update`).
    ///
    /// Moonraker sensors are user-defined in `moonraker.conf` (e.g. MQTT
    /// power monitors). Each sensor reports a flat map of named values.
    /// Values may be numeric, boolean, or string — consumers should handle
    /// all types gracefully.
    SensorUpdate {
        /// The sensor ID (e.g. `"hank-pm"`).
        sensor: String,
        /// The sensor's current values as a JSON object.
        values: serde_json::Value,
    },
}

/// The reason a WebSocket disconnect occurred.
#[derive(Debug)]
pub enum DisconnectReason {
    /// The server sent a close frame or the stream ended cleanly.
    ServerClosed,
    /// A network or protocol error caused the disconnect.
    NetworkError(String),
    /// The client requested the disconnect (via [`MoonrakerClient::close`]
    /// or cancellation token).
    ClientRequested,
}

/// The state of the Klippy host software.
///
/// Moonraker sends notifications when Klippy transitions between these states.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KlippyState {
    /// Klippy is fully initialized and ready.
    Ready,
    /// Klippy has shut down (firmware error or manual shutdown).
    Shutdown,
    /// Klippy has disconnected from Moonraker.
    Disconnected,
}

// ---------------------------------------------------------------------------
// Client handle
// ---------------------------------------------------------------------------

/// A handle to the Moonraker WebSocket client.
///
/// This is a lightweight, cloneable handle that communicates with a
/// background connection task via channels. Dropping all handles will
/// cause the background task to shut down.
///
/// # Thread Safety
///
/// `MoonrakerClient` is `Send + Sync` and can be shared across tasks.
/// Each method sends a command to the background task and awaits the
/// response via a oneshot channel.
#[derive(Debug, Clone)]
pub struct MoonrakerClient {
    /// Channel to send commands to the background connection task.
    command_tx: mpsc::Sender<reconnect::Command>,
}

impl MoonrakerClient {
    /// Connect to a Moonraker instance and start the background connection task.
    ///
    /// Returns a client handle and an event receiver. The background task
    /// manages the WebSocket connection lifecycle (connect, reconnect,
    /// subscription replay) until the cancellation token is triggered or
    /// [`close`](MoonrakerClient::close) is called.
    ///
    /// # Parameters
    /// - `config`: Connection and reconnection configuration.
    /// - `cancel`: Token to signal graceful shutdown of the background task.
    ///
    /// # Returns
    /// A tuple of `(client_handle, event_receiver)`. The event receiver
    /// delivers [`MoonrakerEvent`] values for the consumer to process.
    ///
    /// # Errors
    /// Currently infallible — the actual connection happens asynchronously
    /// in the background task. The first [`MoonrakerEvent::Connected`] (or
    /// a series of `Disconnected` events during retries) will indicate
    /// whether the connection succeeded.
    pub async fn connect(
        config: MoonrakerConfig,
        cancel: CancellationToken,
    ) -> Result<(Self, mpsc::Receiver<MoonrakerEvent>), MoonrakerError> {
        let (cmd_tx, cmd_rx) = mpsc::channel(32);
        let (event_tx, event_rx) = mpsc::channel(256);

        // Spawn the connection loop as a background task. It will run
        // until cancelled or until the command channel is dropped.
        tokio::spawn(reconnect::run_connection_loop(
            config.url,
            config.reconnect,
            config.rpc_timeout,
            cmd_rx,
            event_tx,
            cancel,
        ));

        Ok((Self { command_tx: cmd_tx }, event_rx))
    }

    /// Query the list of available printer objects from Moonraker.
    ///
    /// Sends a `printer.objects.list` RPC and parses the response into
    /// a list of object names (e.g. `["extruder", "heater_bed", "toolhead"]`).
    ///
    /// # Errors
    /// Returns [`MoonrakerError`] if the RPC fails, times out, or the
    /// response cannot be parsed.
    pub async fn get_object_list(&self) -> Result<Vec<String>, MoonrakerError> {
        let result = self
            .rpc("printer.objects.list", serde_json::Value::Null)
            .await?;

        // Moonraker returns: {"objects": ["extruder", "heater_bed", ...]}
        // Use `pointer` for safe nested access without panicking.
        Ok(result
            .pointer("/objects")
            .and_then(|v| v.as_array())
            .map(|arr| {
                arr.iter()
                    .filter_map(|v| v.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default())
    }

    /// Subscribe to status updates for the given printer objects.
    ///
    /// After subscribing, the event receiver will deliver
    /// [`MoonrakerEvent::StatusUpdate`] events whenever the subscribed
    /// objects change. Subscriptions are automatically re-established
    /// after a reconnect.
    ///
    /// # Parameters
    /// - `objects`: The Moonraker object names to subscribe to
    ///   (e.g. `["extruder", "heater_bed"]`).
    ///
    /// # Returns
    /// The initial status snapshot returned by Moonraker for the
    /// subscribed objects.
    ///
    /// # Errors
    /// Returns [`MoonrakerError`] if the subscribe RPC fails, times out,
    /// or the channel to the background task is closed.
    pub async fn subscribe(&self, objects: &[String]) -> Result<serde_json::Value, MoonrakerError> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(reconnect::Command::Subscribe {
                objects: objects.to_vec(),
                response_tx: tx,
            })
            .await
            .map_err(|_| MoonrakerError::ChannelClosed)?;
        rx.await.map_err(|_| MoonrakerError::ChannelClosed)?
    }

    /// Send an arbitrary JSON-RPC request to Moonraker.
    ///
    /// This is the low-level RPC method. Higher-level methods like
    /// [`get_object_list`](MoonrakerClient::get_object_list) and
    /// [`subscribe`](MoonrakerClient::subscribe) are built on top of it.
    ///
    /// # Parameters
    /// - `method`: The JSON-RPC method name (e.g. `"server.info"`).
    /// - `params`: The method parameters. Pass `Value::Null` for no params.
    ///
    /// # Returns
    /// The `result` field from the JSON-RPC response.
    ///
    /// # Errors
    /// Returns [`MoonrakerError`] if the RPC fails, times out, or the
    /// channel to the background task is closed.
    pub async fn rpc(
        &self,
        method: &str,
        params: serde_json::Value,
    ) -> Result<serde_json::Value, MoonrakerError> {
        let (tx, rx) = oneshot::channel();
        self.command_tx
            .send(reconnect::Command::Rpc {
                method: method.to_owned(),
                params,
                response_tx: tx,
            })
            .await
            .map_err(|_| MoonrakerError::ChannelClosed)?;

        // The background task enforces per-RPC timeouts, but add a safety
        // timeout here as well in case the background task is stuck.
        tokio::time::timeout(Duration::from_secs(30), rx)
            .await
            .map_err(|_| MoonrakerError::RpcTimeout)?
            .map_err(|_| MoonrakerError::ChannelClosed)?
    }

    /// Request a graceful shutdown of the background connection task.
    ///
    /// Sends a close command and returns immediately. The background task
    /// will send a WebSocket close frame and exit. The event receiver
    /// will deliver a final [`MoonrakerEvent::Disconnected`] with
    /// [`DisconnectReason::ClientRequested`].
    pub async fn close(&self) {
        let _ = self.command_tx.send(reconnect::Command::Close).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that the default config has sensible values.
    #[test]
    fn default_config_values() {
        let config = MoonrakerConfig::default();
        assert_eq!(config.url.as_str(), "ws://localhost:7125/websocket");
        assert_eq!(config.rpc_timeout, Duration::from_secs(10));
    }

    /// Verifies the default reconnect config.
    #[test]
    fn default_reconnect_config_values() {
        let config = ReconnectConfig::default();
        assert_eq!(config.initial_delay, Duration::from_secs(1));
        assert_eq!(config.max_delay, Duration::from_secs(60));
        assert_eq!(config.multiplier, 2.0);
        assert!(config.max_attempts.is_none());
    }

    /// Verifies that DisconnectReason variants have Debug output.
    #[test]
    fn disconnect_reason_debug() {
        let reason = DisconnectReason::ServerClosed;
        let debug_str = format!("{reason:?}");
        assert!(debug_str.contains("ServerClosed"));

        let reason = DisconnectReason::NetworkError("timeout".to_owned());
        let debug_str = format!("{reason:?}");
        assert!(debug_str.contains("NetworkError"));
        assert!(debug_str.contains("timeout"));

        let reason = DisconnectReason::ClientRequested;
        let debug_str = format!("{reason:?}");
        assert!(debug_str.contains("ClientRequested"));
    }

    /// Verifies that KlippyState variants have expected Debug and equality.
    #[test]
    fn klippy_state_debug_and_eq() {
        assert_eq!(KlippyState::Ready, KlippyState::Ready);
        assert_ne!(KlippyState::Ready, KlippyState::Shutdown);
        assert_ne!(KlippyState::Shutdown, KlippyState::Disconnected);

        let debug_str = format!("{:?}", KlippyState::Ready);
        assert_eq!(debug_str, "Ready");
    }

    /// Verifies that MoonrakerClient is Clone (it's a lightweight handle).
    #[test]
    fn client_is_clone() {
        // We can't easily construct a real client without a runtime, but
        // we can verify the Clone bound is satisfied at compile time.
        fn assert_clone<T: Clone>() {}
        assert_clone::<MoonrakerClient>();
    }
}
