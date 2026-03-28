//! Klipper host system resource status types.
//!
//! Maps to `klippy/extras/system_stats.py`. Reports CPU, memory, and
//! load metrics from the host machine running Klipper.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Host system resource statistics as reported by Klipper.
///
/// Source: `klippy/extras/system_stats.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SystemStats {
    /// Cumulative CPU time consumed by the Klipper process in seconds.
    #[serde(default)]
    pub cputime: f64,

    /// Available memory on the host in kilobytes.
    #[serde(default)]
    pub memavail: u64,

    /// System load average (1-minute) as reported by the OS.
    #[serde(default)]
    pub sysload: f64,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_system_stats_full() {
        let json = serde_json::json!({
            "cputime": 45.67,
            "memavail": 1048576,
            "sysload": 0.42
        });
        let stats: SystemStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert!((stats.cputime - 45.67).abs() < f64::EPSILON);
        assert_eq!(stats.memavail, 1_048_576);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_system_stats_unknown_fields() {
        let json = serde_json::json!({
            "cputime": 0.0,
            "uptime": 86400
        });
        let stats: SystemStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("uptime"));
    }
}
