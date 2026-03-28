//! Klipper toolhead, G-code move, and motion report status types.
//!
//! Maps to `klippy/toolhead.py`, `klippy/extras/gcode_move.py`, and
//! `klippy/extras/motion_report.py`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Toolhead kinematic and timing status.
///
/// Reports motion limits, print timing, and stall counts. The
/// `max_accel_to_decel` and `minimum_cruise_ratio` fields are
/// mutually exclusive across Klipper versions — newer firmware uses
/// `minimum_cruise_ratio` instead of `max_accel_to_decel`.
///
/// Source: `klippy/toolhead.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ToolheadStats {
    /// Maximum acceleration in mm/s^2.
    #[serde(default)]
    pub max_accel: f64,

    /// Maximum acceleration-to-deceleration rate in mm/s^2.
    /// Deprecated in newer Klipper versions in favor of `minimum_cruise_ratio`.
    #[serde(default)]
    pub max_accel_to_decel: Option<f64>,

    /// Maximum velocity in mm/s.
    #[serde(default)]
    pub max_velocity: f64,

    /// Square corner velocity limit in mm/s.
    #[serde(default)]
    pub square_corner_velocity: f64,

    /// Virtual print time of the toolhead in seconds.
    #[serde(default)]
    pub print_time: f64,

    /// Estimated wall-clock print time in seconds.
    #[serde(default)]
    pub estimated_print_time: f64,

    /// Number of toolhead stalls (buffer underruns).
    #[serde(default)]
    pub stalls: u64,

    /// Minimum cruise ratio, between 0.0 and 1.0. Replaces
    /// `max_accel_to_decel` in newer Klipper versions.
    #[serde(default)]
    pub minimum_cruise_ratio: Option<f64>,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// G-code move multiplier and speed status.
///
/// Tracks the current speed and extrusion/speed override factors that
/// the user can adjust via M220/M221 commands.
///
/// Source: `klippy/extras/gcode_move.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct GCodeMoveStats {
    /// Extrusion flow rate multiplier (1.0 = 100%).
    #[serde(default)]
    pub extrude_factor: f64,

    /// Speed override factor (1.0 = 100%).
    #[serde(default)]
    pub speed_factor: f64,

    /// Requested move speed in mm/s (before speed_factor is applied).
    #[serde(default)]
    pub speed: f64,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Live motion report with instantaneous velocities.
///
/// Updated at high frequency by the motion planner.
///
/// Source: `klippy/extras/motion_report.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MotionReportStats {
    /// Instantaneous extruder velocity in mm/s.
    #[serde(default)]
    pub live_extruder_velocity: f64,

    /// Instantaneous toolhead velocity in mm/s.
    #[serde(default)]
    pub live_velocity: f64,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_toolhead_full() {
        let json = serde_json::json!({
            "max_accel": 3000.0,
            "max_accel_to_decel": 1500.0,
            "max_velocity": 300.0,
            "square_corner_velocity": 5.0,
            "print_time": 123.456,
            "estimated_print_time": 124.0,
            "stalls": 0,
            "minimum_cruise_ratio": 0.5
        });
        let stats: ToolheadStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert!((stats.max_accel - 3000.0).abs() < f64::EPSILON);
        assert_eq!(stats.max_accel_to_decel, Some(1500.0));
        assert_eq!(stats.minimum_cruise_ratio, Some(0.5));
        assert_eq!(stats.stalls, 0);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_toolhead_unknown_fields() {
        let json = serde_json::json!({
            "max_accel": 5000.0,
            "input_shaper": "mzv"
        });
        let stats: ToolheadStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("input_shaper"));
    }

    #[test]
    fn deserialize_gcode_move_full() {
        let json = serde_json::json!({
            "extrude_factor": 1.0,
            "speed_factor": 0.8,
            "speed": 60.0
        });
        let stats: GCodeMoveStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert!((stats.speed_factor - 0.8).abs() < f64::EPSILON);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_gcode_move_unknown_fields() {
        let json = serde_json::json!({
            "speed": 100.0,
            "absolute_coordinates": true
        });
        let stats: GCodeMoveStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("absolute_coordinates"));
    }

    #[test]
    fn deserialize_motion_report_full() {
        let json = serde_json::json!({
            "live_extruder_velocity": 12.5,
            "live_velocity": 150.0
        });
        let stats: MotionReportStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert!((stats.live_velocity - 150.0).abs() < f64::EPSILON);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_motion_report_unknown_fields() {
        let json = serde_json::json!({
            "live_velocity": 0.0,
            "live_position": [0.0, 0.0, 0.0, 0.0]
        });
        let stats: MotionReportStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("live_position"));
    }
}
