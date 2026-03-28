//! Klipper stepper motor and driver status types.
//!
//! Maps to TMC stepper driver modules (e.g. `klippy/extras/tmc2209.py`)
//! and the stepper enable tracking in `klippy/extras/stepper_enable.py`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Status for a TMC-family stepper motor driver (e.g. TMC2209, TMC2240).
///
/// Reports the configured current levels, phase offset calibration data,
/// and an optional on-driver temperature reading.
///
/// Source: `klippy/extras/tmc.py`, `klippy/extras/tmc2209.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct TMCStepperMotorDriver {
    /// Configured hold current in amps.
    #[serde(default)]
    pub hold_current: f64,

    /// MCU-level phase offset used for homing calibration.
    #[serde(default)]
    pub mcu_phase_offset: u64,

    /// Stepper position corresponding to the phase offset, in millimeters.
    #[serde(default)]
    pub phase_offset_position: f64,

    /// Configured run current in amps.
    #[serde(default)]
    pub run_current: f64,

    /// On-driver temperature in degrees Celsius.
    /// `None` when the driver does not report temperature.
    #[serde(default)]
    pub temperature: Option<f64>,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Tracks which stepper motors are currently enabled (energized).
///
/// The `steppers` map keys are stepper names (e.g. `"stepper_x"`) and
/// values indicate whether the driver is enabled.
///
/// Source: `klippy/extras/stepper_enable.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct StepperEnableStats {
    /// Map from stepper name to its enabled state.
    #[serde(default)]
    pub steppers: HashMap<String, bool>,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_tmc_driver_full() {
        let json = serde_json::json!({
            "hold_current": 0.6,
            "mcu_phase_offset": 42,
            "phase_offset_position": -0.015,
            "run_current": 1.2,
            "temperature": 55.3
        });
        let stats: TMCStepperMotorDriver =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert!((stats.hold_current - 0.6).abs() < f64::EPSILON);
        assert_eq!(stats.mcu_phase_offset, 42);
        assert_eq!(stats.temperature, Some(55.3));
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_tmc_driver_no_temperature() {
        let json = serde_json::json!({
            "hold_current": 0.8,
            "mcu_phase_offset": 0,
            "phase_offset_position": 0.0,
            "run_current": 1.0
        });
        let stats: TMCStepperMotorDriver =
            serde_json::from_value(json).expect("should deserialize without temperature");
        assert_eq!(stats.temperature, None);
    }

    #[test]
    fn deserialize_tmc_driver_unknown_fields() {
        let json = serde_json::json!({
            "hold_current": 0.5,
            "drv_status": "OK"
        });
        let stats: TMCStepperMotorDriver =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("drv_status"));
    }

    #[test]
    fn deserialize_stepper_enable_full() {
        let json = serde_json::json!({
            "steppers": {
                "stepper_x": true,
                "stepper_y": true,
                "stepper_z": false
            }
        });
        let stats: StepperEnableStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert_eq!(stats.steppers.len(), 3);
        assert!(stats.steppers["stepper_x"]);
        assert!(!stats.steppers["stepper_z"]);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_stepper_enable_unknown_fields() {
        let json = serde_json::json!({
            "steppers": {},
            "motor_count": 4
        });
        let stats: StepperEnableStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("motor_count"));
    }
}
