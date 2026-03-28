//! Klipper fan status types.
//!
//! Maps to `klippy/extras/fan.py` and related fan modules. Covers the
//! part cooling fan, controller fan, heater fan, generic fans, and
//! temperature-controlled fans.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status for a generic fan (part cooling, controller, heater, or fan_generic).
///
/// Reports current duty cycle and optional tachometer reading. Used for
/// any fan type that does not have a temperature target.
///
/// Source: `klippy/extras/fan.py`, `klippy/extras/fan_generic.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GenericFanStats {
    /// Current fan speed as a fraction between 0.0 (off) and 1.0 (full).
    #[serde(default)]
    pub speed: f64,

    /// Tachometer reading in revolutions per minute. Zero when no tach sensor
    /// is wired or the fan is stopped.
    #[serde(default)]
    pub rpm: f64,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Status for a temperature-controlled fan.
///
/// Extends [`GenericFanStats`] with a temperature target so Klipper can
/// regulate the fan speed to reach a set temperature.
///
/// Source: `klippy/extras/temperature_fan.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TemperatureFanStats {
    /// Current fan speed as a fraction between 0.0 (off) and 1.0 (full).
    #[serde(default)]
    pub speed: f64,

    /// Tachometer reading in revolutions per minute.
    #[serde(default)]
    pub rpm: f64,

    /// Target temperature in degrees Celsius that the fan tries to maintain.
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
    fn deserialize_generic_fan_full() {
        let json = serde_json::json!({
            "speed": 0.75,
            "rpm": 4200.0
        });
        let stats: GenericFanStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert!((stats.speed - 0.75).abs() < f64::EPSILON);
        assert!((stats.rpm - 4200.0).abs() < f64::EPSILON);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_generic_fan_unknown_fields() {
        let json = serde_json::json!({
            "speed": 1.0,
            "tach_pin": "PA3"
        });
        let stats: GenericFanStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!((stats.speed - 1.0).abs() < f64::EPSILON);
        assert!(stats.extra.contains_key("tach_pin"));
    }

    #[test]
    fn deserialize_temperature_fan_full() {
        let json = serde_json::json!({
            "speed": 0.5,
            "rpm": 3000.0,
            "target": 45.0,
            "temperature": 43.2
        });
        let stats: TemperatureFanStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert!((stats.target - 45.0).abs() < f64::EPSILON);
        assert!((stats.temperature - 43.2).abs() < f64::EPSILON);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_temperature_fan_unknown_fields() {
        let json = serde_json::json!({
            "speed": 0.0,
            "pid_kp": 1.5
        });
        let stats: TemperatureFanStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("pid_kp"));
    }
}
