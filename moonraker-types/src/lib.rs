//! Type definitions for Moonraker server status endpoints.
//!
//! These types map to Moonraker's own endpoints (`/machine/proc_stats`,
//! `/server/info`), separate from Klipper status objects.
//!
//! All structs use permissive deserialization (`#[serde(default)]` on every
//! field) so that older or newer Moonraker versions that omit or add fields
//! do not cause deserialization failures. Unknown fields are captured in
//! an `extra` HashMap via `#[serde(flatten)]`, except where a flatten is
//! already present for structural reasons (see [`SystemCpuUsageData`]).

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Top-level Moonraker process and system statistics.
///
/// Received as the `"server"` component of a Moonraker status subscription.
/// Contains host-level metrics (CPU temperature, memory, network) and
/// Moonraker service resource usage.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MoonrakerStats {
    /// CPU temperature of the host SoC/board in degrees Celsius.
    #[serde(default)]
    pub cpu_temp: f64,

    /// Resource usage of the Moonraker process itself.
    #[serde(default)]
    pub moonraker_stats: MoonrakerServiceData,

    /// Per-interface network statistics keyed by interface name (e.g. `"eth0"`).
    #[serde(default)]
    pub network: HashMap<String, NetworkInterfaceData>,

    /// System-wide CPU utilisation, broken down by logical core.
    #[serde(default)]
    pub system_cpu_usage: SystemCpuUsageData,

    /// System-wide memory utilisation in kibibytes.
    #[serde(default)]
    pub system_memory: SystemMemoryUsageData,

    /// Number of active WebSocket connections to Moonraker.
    #[serde(default)]
    pub websocket_connections: u64,

    /// Captures unknown keys added by future Moonraker versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Resource usage of the Moonraker server process.
///
/// Reported under `moonraker_stats` inside [`MoonrakerStats`].
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MoonrakerServiceData {
    /// Moonraker process CPU utilisation as a percentage (0.0–100.0).
    #[serde(default)]
    pub cpu_usage: f64,

    /// Unit label for the [`memory`][Self::memory] field (typically `"kB"`).
    #[serde(default)]
    pub mem_units: String,

    /// Moonraker process resident-set size in the units reported by [`mem_units`][Self::mem_units].
    #[serde(default)]
    pub memory: u64,

    /// Unix timestamp (seconds since epoch) of the sample.
    #[serde(default)]
    pub time: f64,

    /// Captures unknown keys added by future Moonraker versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Statistics for a single network interface.
///
/// Reported as values inside the `network` map in [`MoonrakerStats`].
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct NetworkInterfaceData {
    /// Estimated current bandwidth in kilobits per second.
    #[serde(default)]
    pub bandwidth: f64,

    /// Cumulative bytes received.
    #[serde(default)]
    pub rx_bytes: u64,

    /// Cumulative inbound packets dropped.
    #[serde(default)]
    pub rx_drop: u64,

    /// Cumulative inbound packet errors.
    #[serde(default)]
    pub rx_errs: u64,

    /// Cumulative inbound packets received.
    #[serde(default)]
    pub rx_packets: u64,

    /// Cumulative bytes transmitted.
    #[serde(default)]
    pub tx_bytes: u64,

    /// Cumulative outbound packets dropped.
    #[serde(default)]
    pub tx_drop: u64,

    /// Cumulative outbound packet errors.
    #[serde(default)]
    pub tx_errs: u64,

    /// Cumulative outbound packets transmitted.
    #[serde(default)]
    pub tx_packets: u64,

    /// Captures unknown keys added by future Moonraker versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// System-wide CPU utilisation, including an aggregate and per-core breakdown.
///
/// Reported under `system_cpu_usage` inside [`MoonrakerStats`].
///
/// # Serde flattening note
///
/// Moonraker sends per-core values as sibling keys of `cpu` (e.g. `"cpu0"`,
/// `"cpu1"`, …). These are captured by the flattened `cores` map. Because
/// `serde` does not support two `#[serde(flatten)]` fields on the same struct,
/// this struct omits a separate `extra` catch-all — the `cores` map already
/// absorbs all unknown string-keyed numeric values at this level.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SystemCpuUsageData {
    /// Aggregate CPU utilisation across all cores as a percentage (0.0–100.0).
    #[serde(default)]
    pub cpu: f64,

    /// Per-core CPU utilisation keyed by core label (e.g. `"cpu0"`, `"cpu1"`).
    ///
    /// Captures all additional numeric keys present in the JSON object. A
    /// separate `extra` field is intentionally omitted because `serde` only
    /// allows one `#[serde(flatten)]` per struct.
    #[serde(flatten)]
    pub cores: HashMap<String, f64>,
}

/// System-wide memory statistics in kibibytes.
///
/// Reported under `system_memory` inside [`MoonrakerStats`].
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SystemMemoryUsageData {
    /// Memory available for new allocations (free + reclaimable), in kibibytes.
    #[serde(default)]
    pub available: u64,

    /// Total physical memory installed, in kibibytes.
    #[serde(default)]
    pub total: u64,

    /// Memory currently in use, in kibibytes.
    #[serde(default)]
    pub used: u64,

    /// Captures unknown keys added by future Moonraker versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A realistic snapshot produced by Moonraker on a Raspberry Pi with two
    /// network interfaces and a quad-core CPU.
    fn realistic_moonraker_json() -> serde_json::Value {
        serde_json::json!({
            "cpu_temp": 51.25,
            "moonraker_stats": {
                "cpu_usage": 2.34,
                "mem_units": "kB",
                "memory": 65536,
                "time": 1711584000.0
            },
            "network": {
                "eth0": {
                    "bandwidth": 125.0,
                    "rx_bytes": 1048576,
                    "rx_drop": 0,
                    "rx_errs": 0,
                    "rx_packets": 8192,
                    "tx_bytes": 524288,
                    "tx_drop": 0,
                    "tx_errs": 0,
                    "tx_packets": 4096
                },
                "wlan0": {
                    "bandwidth": 54.0,
                    "rx_bytes": 2097152,
                    "rx_drop": 2,
                    "rx_errs": 1,
                    "rx_packets": 16384,
                    "tx_bytes": 1048576,
                    "tx_drop": 0,
                    "tx_errs": 0,
                    "tx_packets": 8192
                }
            },
            "system_cpu_usage": {
                "cpu": 12.5,
                "cpu0": 10.0,
                "cpu1": 14.0,
                "cpu2": 11.0,
                "cpu3": 14.5
            },
            "system_memory": {
                "available": 3145728,
                "total": 4194304,
                "used": 1048576
            },
            "websocket_connections": 3
        })
    }

    #[test]
    fn deserialize_realistic_payload() {
        let stats: MoonrakerStats = serde_json::from_value(realistic_moonraker_json())
            .expect("should deserialize realistic Moonraker stats payload");

        assert!((stats.cpu_temp - 51.25).abs() < f64::EPSILON);
        assert_eq!(stats.websocket_connections, 3);

        assert!((stats.moonraker_stats.cpu_usage - 2.34).abs() < f64::EPSILON);
        assert_eq!(stats.moonraker_stats.mem_units, "kB");
        assert_eq!(stats.moonraker_stats.memory, 65_536);

        let eth0 = stats
            .network
            .get("eth0")
            .expect("eth0 interface should be present");
        assert_eq!(eth0.rx_bytes, 1_048_576);
        assert!((eth0.bandwidth - 125.0).abs() < f64::EPSILON);

        let wlan0 = stats
            .network
            .get("wlan0")
            .expect("wlan0 interface should be present");
        assert_eq!(wlan0.rx_drop, 2);

        assert!((stats.system_cpu_usage.cpu - 12.5).abs() < f64::EPSILON);
        assert_eq!(stats.system_memory.total, 4_194_304);

        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_with_unknown_fields() {
        let json = serde_json::json!({
            "cpu_temp": 45.0,
            "websocket_connections": 1,
            "moonraker_stats": {
                "cpu_usage": 1.0,
                "mem_units": "kB",
                "memory": 32768,
                "time": 1711584000.0,
                "future_field": "some_value"
            },
            "network": {},
            "system_cpu_usage": { "cpu": 5.0 },
            "system_memory": {
                "available": 1000000,
                "total": 2000000,
                "used": 1000000,
                "swap_total": 500000
            },
            "new_top_level_key": true
        });

        let stats: MoonrakerStats = serde_json::from_value(json)
            .expect("should deserialize JSON that contains unknown fields");

        assert!(
            stats.extra.contains_key("new_top_level_key"),
            "unknown top-level key should appear in extra"
        );
        assert!(
            stats.moonraker_stats.extra.contains_key("future_field"),
            "unknown moonraker_stats key should appear in extra"
        );
        assert!(
            stats.system_memory.extra.contains_key("swap_total"),
            "unknown system_memory key should appear in extra"
        );
    }

    #[test]
    fn deserialize_cpu_usage_varying_core_counts() {
        // Single-core (no per-core breakdown)
        let single_core = serde_json::json!({ "cpu": 8.0 });
        let usage: SystemCpuUsageData =
            serde_json::from_value(single_core).expect("should deserialize single-core CPU usage");
        assert!((usage.cpu - 8.0).abs() < f64::EPSILON);
        // "cpu" is consumed as the named field; cores map should be empty.
        assert!(
            usage.cores.is_empty(),
            "cores should be empty for single-core payload"
        );

        // Dual-core
        let dual_core = serde_json::json!({ "cpu": 15.0, "cpu0": 12.0, "cpu1": 18.0 });
        let usage: SystemCpuUsageData =
            serde_json::from_value(dual_core).expect("should deserialize dual-core CPU usage");
        assert_eq!(usage.cores.len(), 2);
        assert!((usage.cores["cpu0"] - 12.0).abs() < f64::EPSILON);
        assert!((usage.cores["cpu1"] - 18.0).abs() < f64::EPSILON);

        // Eight-core (stress test for flatten behaviour)
        let json = serde_json::json!({
            "cpu": 30.0,
            "cpu0": 25.0, "cpu1": 28.0, "cpu2": 31.0, "cpu3": 33.0,
            "cpu4": 27.0, "cpu5": 29.0, "cpu6": 32.0, "cpu7": 35.0
        });
        let usage: SystemCpuUsageData =
            serde_json::from_value(json).expect("should deserialize eight-core CPU usage");
        assert!((usage.cpu - 30.0).abs() < f64::EPSILON);
        assert_eq!(usage.cores.len(), 8);
    }
}
