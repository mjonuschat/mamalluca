//! Klipper webhooks status types.
//!
//! Maps to `klippy/webhooks.py`. Exposes the printer's high-level state
//! and an accompanying human-readable message.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// High-level printer state reported by Klipper.
///
/// Lifecycle: startup -> ready -> (error | shutdown).
/// Source: `klippy/webhooks.py`
#[derive(Clone, Debug, Default, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum KlippyState {
    /// Klipper is ready and accepting commands.
    Ready,
    /// An unrecoverable error has occurred.
    Error,
    /// Klipper has shut down (either requested or due to a fault).
    Shutdown,
    /// Klipper is still initializing.
    #[default]
    Startup,
}

/// Status from the Klipper webhooks module.
///
/// Contains the current state and a descriptive message. The message is
/// useful for surfacing error details. Source: `klippy/webhooks.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct WebhooksStats {
    /// The current high-level printer state (ready, error, shutdown, startup).
    #[serde(default)]
    pub state: KlippyState,

    /// Human-readable message describing the current state.
    #[serde(default)]
    pub state_message: String,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_full_payload() {
        let json = serde_json::json!({
            "state": "ready",
            "state_message": "Printer is ready"
        });

        let stats: WebhooksStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert_eq!(stats.state, KlippyState::Ready);
        assert_eq!(stats.state_message, "Printer is ready");
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_with_unknown_fields() {
        let json = serde_json::json!({
            "state": "error",
            "state_message": "MCU protocol error",
            "new_field": 42
        });

        let stats: WebhooksStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert_eq!(stats.state, KlippyState::Error);
        assert!(stats.extra.contains_key("new_field"));
        assert_eq!(stats.extra["new_field"], serde_json::json!(42));
    }

    #[test]
    fn deserialize_empty_json() {
        let json = serde_json::json!({});

        let stats: WebhooksStats =
            serde_json::from_value(json).expect("should deserialize empty JSON via defaults");
        assert_eq!(stats.state, KlippyState::Startup);
        assert_eq!(stats.state_message, "");
        assert!(stats.extra.is_empty());
    }
}
