//! Public error types for the Moonraker client.
//!
//! [`MoonrakerError`] is the single error type exposed to consumers. It covers
//! connection failures, RPC timeouts, protocol errors, and internal channel
//! issues. Lower-level errors from `connection.rs` and `jsonrpc.rs` are
//! mapped into these variants by the reconnect loop.

use crate::jsonrpc::JsonRpcError;

/// Errors produced by the Moonraker client.
///
/// This is the crate's public error type. All methods on [`crate::MoonrakerClient`]
/// return `Result<T, MoonrakerError>`.
#[derive(Debug, thiserror::Error)]
pub enum MoonrakerError {
    /// Failed to establish a WebSocket connection to Moonraker.
    ///
    /// The inner string contains the underlying error message (DNS failure,
    /// connection refused, TLS error, etc.).
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),

    /// An RPC response was not received within the configured timeout.
    ///
    /// This can happen if Moonraker is overloaded or if a network partition
    /// causes the response to be lost.
    #[error("RPC request timed out")]
    RpcTimeout,

    /// Moonraker returned a JSON-RPC error response.
    ///
    /// The inner [`JsonRpcError`] contains the numeric error code, a
    /// human-readable message, and optional additional data.
    #[error("RPC error: {0}")]
    RpcError(JsonRpcError),

    /// JSON serialization or deserialization failed.
    ///
    /// Typically indicates a protocol mismatch — either we sent malformed JSON
    /// or Moonraker responded with an unexpected structure.
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    /// A WebSocket protocol error occurred.
    ///
    /// The inner string contains the underlying tungstenite error message.
    #[error("WebSocket error: {0}")]
    WebSocket(String),

    /// An internal channel was closed unexpectedly.
    ///
    /// This usually means the connection task has exited (due to cancellation
    /// or a fatal error) and can no longer process commands.
    #[error("Internal channel closed")]
    ChannelClosed,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies the Display output for each error variant.
    #[test]
    fn error_display_messages() {
        let err = MoonrakerError::ConnectionFailed("refused".to_owned());
        assert_eq!(err.to_string(), "Connection failed: refused");

        let err = MoonrakerError::RpcTimeout;
        assert_eq!(err.to_string(), "RPC request timed out");

        let err = MoonrakerError::RpcError(JsonRpcError {
            code: -32601,
            message: "Method not found".to_owned(),
            data: None,
        });
        assert!(err.to_string().contains("Method not found"));

        let err = MoonrakerError::WebSocket("broken pipe".to_owned());
        assert_eq!(err.to_string(), "WebSocket error: broken pipe");

        let err = MoonrakerError::ChannelClosed;
        assert_eq!(err.to_string(), "Internal channel closed");
    }

    /// Verifies that serde_json errors convert into the Serialization variant.
    #[test]
    fn serialization_error_from_serde() {
        let serde_err = serde_json::from_str::<serde_json::Value>("{{bad json")
            .expect_err("should fail to parse");
        let err = MoonrakerError::from(serde_err);
        assert!(
            matches!(err, MoonrakerError::Serialization(_)),
            "expected Serialization variant, got: {err:?}"
        );
    }
}
