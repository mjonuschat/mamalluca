//! Klipper extruder status types.
//!
//! Maps to `klippy/extras/extruder.py`. Each extruder reports its thermal
//! state, extrusion parameters, and whether it can currently extrude.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status for a single extruder (hotend).
///
/// Klipper supports multiple named extruders (e.g. `extruder`, `extruder1`).
/// Source: `klippy/extras/extruder.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ExtruderStats {
    /// Whether the extruder is hot enough to extrude filament.
    #[serde(default)]
    pub can_extrude: bool,

    /// Current heater power output, between 0.0 (off) and 1.0 (full power).
    #[serde(default)]
    pub power: f64,

    /// Active pressure advance coefficient.
    #[serde(default)]
    pub pressure_advance: f64,

    /// Pressure advance smoothing time window in seconds.
    #[serde(default)]
    pub smooth_time: f64,

    /// Target temperature in degrees Celsius.
    #[serde(default)]
    pub target: f64,

    /// Current measured temperature in degrees Celsius.
    #[serde(default)]
    pub temperature: f64,

    /// Time offset for temperature readings. May be absent in stock Klipper.
    #[serde(default)]
    pub time_offset: Option<f64>,

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
            "can_extrude": true,
            "power": 0.45,
            "pressure_advance": 0.05,
            "smooth_time": 0.04,
            "target": 220.0,
            "temperature": 219.8,
            "time_offset": -0.003
        });

        let stats: ExtruderStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert!(stats.can_extrude);
        assert!((stats.power - 0.45).abs() < f64::EPSILON);
        assert!((stats.target - 220.0).abs() < f64::EPSILON);
        assert_eq!(stats.time_offset, Some(-0.003));
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_with_unknown_fields() {
        let json = serde_json::json!({
            "can_extrude": false,
            "power": 0.0,
            "filament_diameter": 1.75
        });

        let stats: ExtruderStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(!stats.can_extrude);
        assert!(stats.extra.contains_key("filament_diameter"));
        assert_eq!(stats.extra["filament_diameter"], serde_json::json!(1.75));
    }

    #[test]
    fn deserialize_empty_json() {
        let json = serde_json::json!({});

        let stats: ExtruderStats =
            serde_json::from_value(json).expect("should deserialize empty JSON via defaults");
        assert!(!stats.can_extrude);
        assert!((stats.power - 0.0).abs() < f64::EPSILON);
        assert_eq!(stats.time_offset, None);
        assert!(stats.extra.is_empty());
    }
}
