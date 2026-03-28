//! Collectors for Klipper toolhead, G-code move, and motion report stats.

use mamalluca_macros::collector;
use metrics::gauge;

use super::labels_for;
use crate::metrics::MetricCollector;

/// Collects toolhead kinematic and timing statistics.
///
/// `toolhead` is a singleton in Klipper — there is exactly one toolhead
/// per printer.
#[collector(prefix = "toolhead")]
pub struct ToolheadCollector;

impl MetricCollector for ToolheadCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record toolhead statistics.
    ///
    /// The `max_accel_to_decel` and `minimum_cruise_ratio` fields are
    /// mutually exclusive across Klipper versions and are only recorded
    /// when present (`Some`).
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `_name` - Always `None` for this singleton collector
    /// * `data` - Raw JSON value from the status update
    ///
    /// # Errors
    /// Returns an error if deserialization fails.
    fn record(
        &self,
        _key: &str,
        name: Option<&str>,
        data: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let stats: klipper_types::ToolheadStats = serde_json::from_value(data.clone())?;
        let labels = labels_for(name);

        gauge!("klipper.stats.toolhead.print_time", &labels).set(stats.print_time);
        gauge!("klipper.stats.toolhead.estimated_print_time", &labels)
            .set(stats.estimated_print_time);
        gauge!("klipper.stats.toolhead.max_accel", &labels).set(stats.max_accel);
        gauge!("klipper.stats.toolhead.max_velocity", &labels).set(stats.max_velocity);
        gauge!("klipper.stats.toolhead.square_corner_velocity", &labels)
            .set(stats.square_corner_velocity);
        // Stalls is a monotonically increasing counter, but toolhead reports it as a
        // plain integer; record as a gauge to preserve the semantics from the source.
        gauge!("klipper.stats.toolhead.stalls", &labels).set(stats.stalls as f64);

        // `max_accel_to_decel` and `minimum_cruise_ratio` are version-dependent —
        // only record them when the firmware actually reports them.
        if let Some(val) = stats.max_accel_to_decel {
            gauge!("klipper.stats.toolhead.max_accel_to_decel", &labels).set(val);
        }
        if let Some(val) = stats.minimum_cruise_ratio {
            gauge!("klipper.stats.toolhead.minimum_cruise_ratio", &labels).set(val);
        }

        Ok(())
    }
}

/// Collects G-code move speed and extrusion factor statistics.
///
/// `gcode_move` is a singleton that tracks the current M220/M221 speed
/// and extrusion overrides plus the active move speed.
#[collector(prefix = "gcode_move")]
pub struct GCodeMoveCollector;

impl MetricCollector for GCodeMoveCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record G-code move statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `_name` - Always `None` for this singleton collector
    /// * `data` - Raw JSON value from the status update
    ///
    /// # Errors
    /// Returns an error if deserialization fails.
    fn record(
        &self,
        _key: &str,
        name: Option<&str>,
        data: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let stats: klipper_types::GCodeMoveStats = serde_json::from_value(data.clone())?;
        let labels = labels_for(name);

        gauge!("klipper.stats.gcode_move.speed_factor", &labels).set(stats.speed_factor);
        gauge!("klipper.stats.gcode_move.extrude_factor", &labels).set(stats.extrude_factor);
        gauge!("klipper.stats.gcode_move.speed", &labels).set(stats.speed);

        Ok(())
    }
}

/// Collects instantaneous motion report statistics.
///
/// `motion_report` is a singleton that provides live velocity readings
/// updated at high frequency by the motion planner.
#[collector(prefix = "motion_report")]
pub struct MotionReportCollector;

impl MetricCollector for MotionReportCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record motion report statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `_name` - Always `None` for this singleton collector
    /// * `data` - Raw JSON value from the status update
    ///
    /// # Errors
    /// Returns an error if deserialization fails.
    fn record(
        &self,
        _key: &str,
        name: Option<&str>,
        data: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let stats: klipper_types::MotionReportStats = serde_json::from_value(data.clone())?;
        let labels = labels_for(name);

        gauge!("klipper.stats.motion_report.extruder_velocity", &labels)
            .set(stats.live_extruder_velocity);
        gauge!("klipper.stats.motion_report.velocity", &labels).set(stats.live_velocity);

        Ok(())
    }
}
