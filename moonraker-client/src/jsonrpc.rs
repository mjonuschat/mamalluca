//! JSON-RPC 2.0 protocol types for Moonraker WebSocket communication.
//!
//! This module provides serializable/deserializable types for the JSON-RPC 2.0
//! protocol used by Moonraker's WebSocket API. It includes request construction,
//! response/notification parsing, and an atomic ID generator for tracking
//! request-response pairs.
//!
//! # Message Discrimination
//!
//! Moonraker sends two kinds of JSON-RPC messages over WebSocket:
//! - **Responses** contain an `id` field matching a previous request
//! - **Notifications** have a `method` field but no `id`
//!
//! Use [`parse_message`] to automatically discriminate between them.

use std::fmt;
use std::sync::atomic::{AtomicU64, Ordering};

use serde::{Deserialize, Serialize};

/// A JSON-RPC 2.0 request to send to Moonraker.
///
/// Serializes to the standard JSON-RPC 2.0 request format. The `params` field
/// is omitted from serialization when it is `Value::Null`, keeping requests
/// compact for methods that take no arguments.
///
/// # Examples
///
/// ```
/// use moonraker_client::jsonrpc::JsonRpcRequest;
///
/// // Request without parameters
/// let req = JsonRpcRequest::new("server.info".to_string(), 1);
///
/// // Request with parameters
/// let req = JsonRpcRequest::with_params(
///     "printer.objects.subscribe".to_string(),
///     2,
///     serde_json::json!({"objects": {"extruder": null}}),
/// );
/// ```
#[derive(Clone, Debug, Serialize)]
pub struct JsonRpcRequest {
    /// Protocol version, always "2.0".
    jsonrpc: &'static str,
    /// The RPC method name (e.g. "server.info", "printer.objects.subscribe").
    method: String,
    /// Unique request identifier for matching responses.
    id: u64,
    /// Method parameters. Omitted from JSON when null.
    #[serde(skip_serializing_if = "serde_json::Value::is_null")]
    params: serde_json::Value,
}

impl JsonRpcRequest {
    /// Creates a new request with no parameters.
    ///
    /// # Parameters
    /// - `method`: The JSON-RPC method name to invoke.
    /// - `id`: A unique request ID for correlating the response.
    pub fn new(method: String, id: u64) -> Self {
        Self {
            jsonrpc: "2.0",
            method,
            id,
            params: serde_json::Value::Null,
        }
    }

    /// Creates a new request with the given parameters.
    ///
    /// # Parameters
    /// - `method`: The JSON-RPC method name to invoke.
    /// - `id`: A unique request ID for correlating the response.
    /// - `params`: A JSON value containing the method parameters.
    pub fn with_params(method: String, id: u64, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: "2.0",
            method,
            id,
            params,
        }
    }
}

/// A JSON-RPC 2.0 response received from Moonraker.
///
/// Exactly one of `result` or `error` will be `Some` in a well-formed response.
/// The `id` matches the request that triggered this response (or is `None` for
/// parse errors on the server side).
#[derive(Clone, Debug, Deserialize)]
pub struct JsonRpcResponse {
    /// The request ID this response corresponds to.
    pub id: Option<u64>,
    /// The successful result payload, if the call succeeded.
    pub result: Option<serde_json::Value>,
    /// The error object, if the call failed.
    pub error: Option<JsonRpcError>,
}

/// A JSON-RPC 2.0 error object.
///
/// Returned inside [`JsonRpcResponse`] when a request fails. The `code` field
/// uses standard JSON-RPC error codes (e.g. -32600 for invalid request,
/// -32601 for method not found).
#[derive(Clone, Debug, Deserialize)]
pub struct JsonRpcError {
    /// A numeric error code indicating the type of failure.
    pub code: i64,
    /// A short human-readable description of the error.
    pub message: String,
    /// Optional additional data about the error.
    pub data: Option<serde_json::Value>,
}

impl fmt::Display for JsonRpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "JSON-RPC error {}: {}", self.code, self.message)
    }
}

/// A JSON-RPC 2.0 notification from Moonraker (no `id` field).
///
/// Notifications are server-initiated messages that do not expect a response.
/// Moonraker uses these for status updates (e.g. `notify_status_update`,
/// `notify_klippy_ready`).
#[derive(Clone, Debug, Deserialize)]
pub struct JsonRpcNotification {
    /// The notification method name (e.g. "notify_status_update").
    pub method: String,
    /// Parameters attached to the notification. Defaults to an empty array
    /// if the server omits the field.
    #[serde(default)]
    pub params: serde_json::Value,
}

/// Atomic counter for generating unique JSON-RPC request IDs.
///
/// Uses [`AtomicU64`] with relaxed ordering, which is sufficient because
/// uniqueness only requires monotonic increment — no cross-thread
/// synchronization of other data depends on the ID value.
///
/// IDs start at 1 (Moonraker treats 0 as "no id" in some contexts).
#[derive(Debug)]
pub struct IdGenerator {
    /// The next ID to hand out.
    next: AtomicU64,
}

impl IdGenerator {
    /// Creates a new generator that starts at 1.
    pub fn new() -> Self {
        Self {
            next: AtomicU64::new(1),
        }
    }

    /// Returns the next unique request ID.
    ///
    /// Each call increments the internal counter atomically, so this is safe
    /// to call from multiple threads concurrently.
    pub fn next_id(&self) -> u64 {
        self.next.fetch_add(1, Ordering::Relaxed)
    }
}

impl Default for IdGenerator {
    fn default() -> Self {
        Self::new()
    }
}

/// A parsed WebSocket message, discriminated into response or notification.
///
/// Use [`parse_message`] to produce this from raw JSON text.
#[derive(Clone, Debug)]
pub enum ParsedMessage {
    /// A response to a previously-sent request (has an `id` field).
    Response(JsonRpcResponse),
    /// A server-initiated notification (no `id` field).
    Notification(JsonRpcNotification),
}

/// Parses a raw JSON string into either a [`JsonRpcResponse`] or a
/// [`JsonRpcNotification`].
///
/// The discrimination logic checks for the presence of an `id` field in the
/// JSON object:
/// - **Present** → deserialize as [`JsonRpcResponse`]
/// - **Absent** → deserialize as [`JsonRpcNotification`]
///
/// # Parameters
/// - `text`: A JSON-encoded string received from the Moonraker WebSocket.
///
/// # Errors
/// Returns [`serde_json::Error`] if the text is not valid JSON or does not
/// match the expected structure of either message type.
///
/// # Examples
///
/// ```
/// use moonraker_client::jsonrpc::parse_message;
///
/// let response_json = r#"{"jsonrpc":"2.0","id":1,"result":{}}"#;
/// let msg = parse_message(response_json).unwrap();
/// assert!(matches!(msg, moonraker_client::jsonrpc::ParsedMessage::Response(_)));
///
/// let notification_json = r#"{"jsonrpc":"2.0","method":"notify_klippy_ready","params":[]}"#;
/// let msg = parse_message(notification_json).unwrap();
/// assert!(matches!(msg, moonraker_client::jsonrpc::ParsedMessage::Notification(_)));
/// ```
pub fn parse_message(text: &str) -> Result<ParsedMessage, serde_json::Error> {
    // First do a lightweight parse to check for the "id" field, then
    // deserialize into the appropriate concrete type. This avoids needing
    // an untagged enum (which produces poor error messages on failure).
    let raw: serde_json::Value = serde_json::from_str(text)?;

    if raw.get("id").is_some() {
        let response: JsonRpcResponse = serde_json::from_value(raw)?;
        Ok(ParsedMessage::Response(response))
    } else {
        let notification: JsonRpcNotification = serde_json::from_value(raw)?;
        Ok(ParsedMessage::Notification(notification))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that serializing a request with no params omits the `params`
    /// field entirely, keeping the JSON compact.
    #[test]
    fn serialize_request_without_params() {
        let req = JsonRpcRequest::new("server.info".to_string(), 1);
        let json = serde_json::to_value(&req).expect("serialization should succeed");

        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["method"], "server.info");
        assert_eq!(json["id"], 1);
        // The `params` key must be absent (not null) due to skip_serializing_if
        assert!(
            json.get("params").is_none(),
            "params field should be absent when null"
        );
    }

    /// Verifies that serializing a request with params includes the `params`
    /// field with the provided value.
    #[test]
    fn serialize_request_with_params() {
        let params = serde_json::json!({"objects": {"extruder": null}});
        let req = JsonRpcRequest::with_params(
            "printer.objects.subscribe".to_string(),
            42,
            params.clone(),
        );
        let json = serde_json::to_value(&req).expect("serialization should succeed");

        assert_eq!(json["jsonrpc"], "2.0");
        assert_eq!(json["method"], "printer.objects.subscribe");
        assert_eq!(json["id"], 42);
        assert_eq!(json["params"], params);
    }

    /// Verifies that a JSON message with an `id` field is parsed as a Response.
    #[test]
    fn parse_response_message() {
        let text = r#"{"jsonrpc":"2.0","id":7,"result":{"state":"ready"}}"#;
        let msg = parse_message(text).expect("parse should succeed");

        match msg {
            ParsedMessage::Response(resp) => {
                assert_eq!(resp.id, Some(7));
                assert!(resp.result.is_some());
                assert!(resp.error.is_none());
                let result = resp.result.expect("result should be present");
                assert_eq!(result["state"], "ready");
            }
            ParsedMessage::Notification(_) => {
                panic!("expected Response, got Notification");
            }
        }
    }

    /// Verifies that a JSON message without an `id` field is parsed as a
    /// Notification.
    #[test]
    fn parse_notification_message() {
        let text = r#"{"jsonrpc":"2.0","method":"notify_klippy_ready","params":[]}"#;
        let msg = parse_message(text).expect("parse should succeed");

        match msg {
            ParsedMessage::Notification(notif) => {
                assert_eq!(notif.method, "notify_klippy_ready");
                assert!(notif.params.is_array());
            }
            ParsedMessage::Response(_) => {
                panic!("expected Notification, got Response");
            }
        }
    }

    /// Verifies that an error response is correctly parsed with code, message,
    /// and optional data fields.
    #[test]
    fn parse_error_response() {
        let text = r#"{
            "jsonrpc": "2.0",
            "id": 3,
            "error": {
                "code": -32601,
                "message": "Method not found",
                "data": "unknown_method"
            }
        }"#;
        let msg = parse_message(text).expect("parse should succeed");

        match msg {
            ParsedMessage::Response(resp) => {
                assert_eq!(resp.id, Some(3));
                assert!(resp.result.is_none());
                let err = resp.error.expect("error should be present");
                assert_eq!(err.code, -32601);
                assert_eq!(err.message, "Method not found");
                assert_eq!(err.data, Some(serde_json::json!("unknown_method")));
                // Verify Display implementation
                assert_eq!(err.to_string(), "JSON-RPC error -32601: Method not found");
            }
            ParsedMessage::Notification(_) => {
                panic!("expected Response, got Notification");
            }
        }
    }

    /// Verifies that the ID generator produces monotonically incrementing
    /// values starting from 1.
    #[test]
    fn id_generator_increments() {
        let id_gen = IdGenerator::new();

        assert_eq!(id_gen.next_id(), 1);
        assert_eq!(id_gen.next_id(), 2);
        assert_eq!(id_gen.next_id(), 3);
    }

    /// Verifies that `IdGenerator::default()` behaves identically to `new()`.
    #[test]
    fn id_generator_default_starts_at_one() {
        let id_gen = IdGenerator::default();
        assert_eq!(id_gen.next_id(), 1);
    }

    /// Verifies that a notification with no `params` field deserializes with
    /// the default value (null).
    #[test]
    fn parse_notification_without_params() {
        let text = r#"{"jsonrpc":"2.0","method":"notify_klippy_shutdown"}"#;
        let msg = parse_message(text).expect("parse should succeed");

        match msg {
            ParsedMessage::Notification(notif) => {
                assert_eq!(notif.method, "notify_klippy_shutdown");
                // serde default for Value is Null
                assert!(notif.params.is_null());
            }
            ParsedMessage::Response(_) => {
                panic!("expected Notification, got Response");
            }
        }
    }
}
