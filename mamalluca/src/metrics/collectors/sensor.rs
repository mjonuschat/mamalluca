//! Collectors for Klipper filament runout sensors and Z thermal adjustment.

use mamalluca_macros::collector;
use metrics::gauge;

use super::labels_for;
use crate::metrics::MetricCollector;

/// Record filament runout sensor gauges from a [`klipper_types::sensor::FilamentRunoutSensorStats`] value.
///
/// Shared by [`FilamentSwitchSensorCollector`] and [`FilamentMotionSensorCollector`] to avoid
/// code duplication — both sensor types report identical fields.
///
/// # Arguments
/// * `name` - Instance name (e.g. `"runout_sensor"`), always present for named collectors
/// * `data` - Raw JSON value from the status update
///
/// # Errors
/// Returns an error if deserialization fails.
fn record_filament_sensor_stats(
    name: Option<&str>,
    data: &serde_json::Value,
) -> anyhow::Result<()> {
    let stats: klipper_types::sensor::FilamentRunoutSensorStats =
        serde_json::from_value(data.clone())?;
    let labels = labels_for(name);

    // Prometheus has no boolean type; represent as 0.0/1.0.
    gauge!("klipper.stats.filament_runout_sensor.enabled", &labels).set(stats.enabled as u8 as f64);
    gauge!(
        "klipper.stats.filament_runout_sensor.filament_detected",
        &labels
    )
    .set(stats.filament_detected as u8 as f64);

    Ok(())
}

/// Collects filament switch sensor statistics.
///
/// Named instances correspond to sensors declared in `printer.cfg` as
/// `[filament_switch_sensor runout_sensor]`, etc.
#[collector(prefix = "filament_switch_sensor", named)]
pub struct FilamentSwitchSensorCollector;

impl MetricCollector for FilamentSwitchSensorCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record filament switch sensor statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Sensor instance name (e.g. `"runout_sensor"`)
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
        record_filament_sensor_stats(name, data)
    }
}

/// Collects filament motion sensor statistics.
///
/// Named instances correspond to sensors declared in `printer.cfg` as
/// `[filament_motion_sensor runout_sensor]`, etc. Reports the same fields
/// as [`FilamentSwitchSensorCollector`].
#[collector(prefix = "filament_motion_sensor", named)]
pub struct FilamentMotionSensorCollector;

impl MetricCollector for FilamentMotionSensorCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record filament motion sensor statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Sensor instance name (e.g. `"runout_sensor"`)
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
        record_filament_sensor_stats(name, data)
    }
}

/// Collects Z thermal adjustment statistics.
///
/// Singleton — maps to the `"z_thermal_adjust"` Klipper status object.
/// Temperature is recorded under the shared `klipper.stats.temperature.current`
/// metric with a hardcoded `name="z_adjust"` label to distinguish it from other
/// temperature sources.
#[collector(prefix = "z_thermal_adjust")]
pub struct ZThermalAdjustCollector;

impl MetricCollector for ZThermalAdjustCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record Z thermal adjustment statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; exact match already happened)
    /// * `_name` - Always `None` for this singleton collector
    /// * `data` - Raw JSON value from the status update
    ///
    /// # Errors
    /// Returns an error if deserialization fails.
    fn record(
        &self,
        _key: &str,
        _name: Option<&str>,
        data: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let stats: klipper_types::sensor::ZThermalAdjustStats =
            serde_json::from_value(data.clone())?;

        // Hardcode the label so this temperature appears alongside other named
        // temperature sensors in dashboards.
        let labels: Vec<(&'static str, String)> = vec![("name", "z_adjust".to_owned())];

        gauge!("klipper.stats.temperature.current", &labels).set(stats.temperature);
        gauge!("klipper.stats.z_thermal_adjust.measured_min_temp", &labels)
            .set(stats.measured_min_temp);
        gauge!("klipper.stats.z_thermal_adjust.measured_max_temp", &labels)
            .set(stats.measured_max_temp);
        gauge!(
            "klipper.stats.z_thermal_adjust.z_adjust_ref_temperature",
            &labels
        )
        .set(stats.z_adjust_ref_temperature);
        gauge!("klipper.stats.z_thermal_adjust.current_z_adjust", &labels)
            .set(stats.current_z_adjust);

        Ok(())
    }
}
