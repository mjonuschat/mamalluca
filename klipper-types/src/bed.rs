//! Klipper bed probe and Z-tilt status types.
//!
//! Maps to `klippy/extras/probe.py` and `klippy/extras/z_tilt.py`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status for a Z-axis bed probe (e.g. BLTouch, inductive probe).
///
/// Source: `klippy/extras/probe.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ProbeStats {
    /// Probe name identifier.
    #[serde(default)]
    pub name: String,

    /// Whether the probe was triggered during the last query.
    #[serde(default)]
    pub last_query: bool,

    /// Z-axis result from the last probe measurement in millimeters.
    #[serde(default)]
    pub last_z_result: f64,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Status for Z-tilt adjustment (bed leveling via multiple Z steppers).
///
/// Source: `klippy/extras/z_tilt.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ZTiltStats {
    /// Whether the Z-tilt adjustment has been applied since last homing.
    #[serde(default)]
    pub applied: bool,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_probe_full() {
        let json = serde_json::json!({
            "name": "bltouch",
            "last_query": true,
            "last_z_result": 1.234
        });
        let stats: ProbeStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert_eq!(stats.name, "bltouch");
        assert!(stats.last_query);
        assert!((stats.last_z_result - 1.234).abs() < f64::EPSILON);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_probe_unknown_fields() {
        let json = serde_json::json!({
            "name": "probe",
            "pin": "PA1"
        });
        let stats: ProbeStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("pin"));
    }

    #[test]
    fn deserialize_z_tilt_full() {
        let json = serde_json::json!({"applied": true});
        let stats: ZTiltStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert!(stats.applied);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_z_tilt_unknown_fields() {
        let json = serde_json::json!({"applied": false, "z_positions": [0.0, 0.01]});
        let stats: ZTiltStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("z_positions"));
    }
}
