//! Klipper heated bed status types.
//!
//! Maps to `klippy/extras/heaters.py`. The heater bed reports its current
//! temperature, target, and power output.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status for the heated print bed.
///
/// Like a simplified extruder heater: target, measured temperature, and power
/// output — without extrusion-specific fields. Source: `klippy/extras/heaters.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HeaterBedStats {
    /// Current heater power output, between 0.0 (off) and 1.0 (full power).
    #[serde(default)]
    pub power: f64,

    /// Target temperature in degrees Celsius.
    #[serde(default)]
    pub target: f64,

    /// Current measured temperature in degrees Celsius.
    #[serde(default)]
    pub temperature: f64,

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
            "power": 0.8,
            "target": 60.0,
            "temperature": 58.5
        });

        let stats: HeaterBedStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert!((stats.power - 0.8).abs() < f64::EPSILON);
        assert!((stats.target - 60.0).abs() < f64::EPSILON);
        assert!((stats.temperature - 58.5).abs() < f64::EPSILON);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_with_unknown_fields() {
        let json = serde_json::json!({
            "power": 0.0,
            "target": 0.0,
            "temperature": 22.3,
            "pid_kp": 50.0
        });

        let stats: HeaterBedStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("pid_kp"));
        assert_eq!(stats.extra["pid_kp"], serde_json::json!(50.0));
    }

    #[test]
    fn deserialize_empty_json() {
        let json = serde_json::json!({});

        let stats: HeaterBedStats =
            serde_json::from_value(json).expect("should deserialize empty JSON via defaults");
        assert!((stats.power - 0.0).abs() < f64::EPSILON);
        assert!((stats.target - 0.0).abs() < f64::EPSILON);
        assert!((stats.temperature - 0.0).abs() < f64::EPSILON);
        assert!(stats.extra.is_empty());
    }
}
