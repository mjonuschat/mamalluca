//! Klipper sensor status types for Z thermal adjustment and filament runout.
//!
//! Maps to `klippy/extras/z_thermal_adjust.py` and
//! `klippy/extras/filament_switch_sensor.py`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status for the Z thermal adjustment module.
///
/// Compensates Z-axis position drift caused by frame thermal expansion.
/// Tracks the current compensation amount and the temperature readings
/// used to compute it.
///
/// Source: `klippy/extras/z_thermal_adjust.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ZThermalAdjustStats {
    /// Current Z-axis compensation offset in millimeters.
    #[serde(default)]
    pub current_z_adjust: f64,

    /// Whether thermal Z adjustment is currently active.
    #[serde(default)]
    pub enabled: bool,

    /// Maximum temperature observed since the module was enabled, in Celsius.
    #[serde(default)]
    pub measured_max_temp: f64,

    /// Minimum temperature observed since the module was enabled, in Celsius.
    #[serde(default)]
    pub measured_min_temp: f64,

    /// Current frame temperature reading in degrees Celsius.
    #[serde(default)]
    pub temperature: f64,

    /// Reference temperature used as the zero-offset baseline, in Celsius.
    #[serde(default)]
    pub z_adjust_ref_temperature: f64,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Status for a filament runout (switch-based) sensor.
///
/// Source: `klippy/extras/filament_switch_sensor.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FilamentRunoutSensorStats {
    /// Whether the sensor is currently monitoring for runout events.
    #[serde(default)]
    pub enabled: bool,

    /// Whether filament is currently detected by the switch.
    #[serde(default)]
    pub filament_detected: bool,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_z_thermal_adjust_full() {
        let json = serde_json::json!({
            "current_z_adjust": 0.015,
            "enabled": true,
            "measured_max_temp": 32.5,
            "measured_min_temp": 22.0,
            "temperature": 28.3,
            "z_adjust_ref_temperature": 25.0
        });
        let stats: ZThermalAdjustStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert!((stats.current_z_adjust - 0.015).abs() < f64::EPSILON);
        assert!(stats.enabled);
        assert!((stats.z_adjust_ref_temperature - 25.0).abs() < f64::EPSILON);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_z_thermal_adjust_unknown_fields() {
        let json = serde_json::json!({
            "enabled": false,
            "coefficient": 0.001
        });
        let stats: ZThermalAdjustStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("coefficient"));
    }

    #[test]
    fn deserialize_filament_runout_full() {
        let json = serde_json::json!({
            "enabled": true,
            "filament_detected": true
        });
        let stats: FilamentRunoutSensorStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert!(stats.enabled);
        assert!(stats.filament_detected);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_filament_runout_unknown_fields() {
        let json = serde_json::json!({
            "enabled": true,
            "switch_pin": "PA5"
        });
        let stats: FilamentRunoutSensorStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("switch_pin"));
    }
}
