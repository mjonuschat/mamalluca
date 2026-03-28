//! Klipper MCU (micro-controller unit) status types.
//!
//! Maps to `klippy/mcu.py`. Each MCU reports communication statistics
//! such as byte counts, sequence numbers, and timing information.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Communication statistics for a single MCU connected to Klipper.
///
/// Klipper can drive multiple MCUs (e.g. main board + toolhead board).
/// Each MCU independently reports these counters and timing metrics.
///
/// Source: `klippy/mcu.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct McuStats {
    /// Clock frequency adjustment value.
    #[serde(default)]
    pub adj: u64,

    /// Total bytes received that failed validation.
    #[serde(default)]
    pub bytes_invalid: u64,

    /// Total bytes read from the MCU serial connection.
    #[serde(default)]
    pub bytes_read: u64,

    /// Total bytes that had to be retransmitted.
    #[serde(default)]
    pub bytes_retransmit: u64,

    /// Total bytes written to the MCU serial connection.
    #[serde(default)]
    pub bytes_write: u64,

    /// MCU clock frequency in Hz.
    #[serde(default)]
    pub freq: u64,

    /// Fraction of time the MCU was awake (not idle), between 0.0 and 1.0.
    #[serde(default)]
    pub mcu_awake: f64,

    /// Average MCU task execution time in seconds.
    #[serde(default)]
    pub mcu_task_avg: f64,

    /// Standard deviation of MCU task execution time in seconds.
    #[serde(default)]
    pub mcu_task_stddev: f64,

    /// Number of bytes queued and ready to send.
    #[serde(default)]
    pub ready_bytes: u64,

    /// Number of bytes queued for upcoming transmission.
    #[serde(default)]
    pub upcoming_bytes: u64,

    /// Sequence number of the last message sent to the MCU.
    #[serde(default)]
    pub send_seq: u64,

    /// Sequence number of the last message received from the MCU.
    #[serde(default)]
    pub receive_seq: u64,

    /// Sequence number of the last retransmitted message.
    #[serde(default)]
    pub retransmit_seq: u64,

    /// Smoothed round-trip time estimate in seconds (TCP-style SRTT).
    #[serde(default)]
    pub srtt: f64,

    /// Retransmission timeout in seconds (TCP-style RTO).
    #[serde(default)]
    pub rto: f64,

    /// Round-trip time variance in seconds (TCP-style RTTVAR).
    #[serde(default)]
    pub rttvar: f64,

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
            "adj": 100, "bytes_invalid": 0, "bytes_read": 524288,
            "bytes_retransmit": 64, "bytes_write": 262144, "freq": 72000000_u64,
            "mcu_awake": 0.002, "mcu_task_avg": 0.000015, "mcu_task_stddev": 0.000003,
            "ready_bytes": 128, "upcoming_bytes": 256, "send_seq": 9000,
            "receive_seq": 8999, "retransmit_seq": 1,
            "srtt": 0.001, "rto": 0.025, "rttvar": 0.0005
        });
        let stats: McuStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert_eq!(stats.freq, 72_000_000);
        assert_eq!(stats.bytes_read, 524_288);
        assert!((stats.srtt - 0.001).abs() < f64::EPSILON);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_with_unknown_fields() {
        let json = serde_json::json!({"freq": 48000000_u64, "firmware_version": "v0.12.0-100"});
        let stats: McuStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert_eq!(stats.freq, 48_000_000);
        assert!(stats.extra.contains_key("firmware_version"));
    }

    #[test]
    fn deserialize_empty_json() {
        let json = serde_json::json!({});
        let stats: McuStats =
            serde_json::from_value(json).expect("should deserialize empty JSON via defaults");
        assert_eq!(stats.freq, 0);
        assert_eq!(stats.bytes_read, 0);
        assert!(stats.extra.is_empty());
    }
}
