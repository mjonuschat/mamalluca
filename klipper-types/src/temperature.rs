//! Klipper temperature sensor status types.
//!
//! Maps to `klippy/extras/temperature_sensor.py`. Generic temperature sensors
//! report their current reading along with min/max observed values.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status for a generic temperature sensor (read-only, no heater control).
///
/// Klipper supports many named sensors (e.g. `chamber`, `raspberry_pi`).
/// Source: `klippy/extras/temperature_sensor.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TemperatureSensorStats {
    /// Current measured temperature in degrees Celsius.
    #[serde(default)]
    pub temperature: f64,

    /// Lowest temperature recorded since Klipper started, in degrees Celsius.
    #[serde(default)]
    pub measured_min_temp: f64,

    /// Highest temperature recorded since Klipper started, in degrees Celsius.
    #[serde(default)]
    pub measured_max_temp: f64,

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
            "temperature": 42.5,
            "measured_min_temp": 21.0,
            "measured_max_temp": 55.3
        });

        let stats: TemperatureSensorStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert!((stats.temperature - 42.5).abs() < f64::EPSILON);
        assert!((stats.measured_min_temp - 21.0).abs() < f64::EPSILON);
        assert!((stats.measured_max_temp - 55.3).abs() < f64::EPSILON);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_with_unknown_fields() {
        let json = serde_json::json!({
            "temperature": 30.0,
            "measured_min_temp": 20.0,
            "measured_max_temp": 40.0,
            "sensor_type": "BME280"
        });

        let stats: TemperatureSensorStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("sensor_type"));
        assert_eq!(stats.extra["sensor_type"], serde_json::json!("BME280"));
    }

    #[test]
    fn deserialize_empty_json() {
        let json = serde_json::json!({});

        let stats: TemperatureSensorStats =
            serde_json::from_value(json).expect("should deserialize empty JSON via defaults");
        assert!((stats.temperature - 0.0).abs() < f64::EPSILON);
        assert!((stats.measured_min_temp - 0.0).abs() < f64::EPSILON);
        assert!((stats.measured_max_temp - 0.0).abs() < f64::EPSILON);
        assert!(stats.extra.is_empty());
    }
}
