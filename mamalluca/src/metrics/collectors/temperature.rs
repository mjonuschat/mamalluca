//! Collectors for Klipper temperature sensors and temperature-controlled fans.

use mamalluca_macros::collector;
use metrics::gauge;

use super::labels_for;
use crate::metrics::MetricCollector;

/// Collects generic temperature sensor statistics.
///
/// Named instances correspond to sensors declared in `printer.cfg`, e.g.
/// `[temperature_sensor chamber]` produces the key `"temperature_sensor chamber"`.
#[collector(prefix = "temperature_sensor", named)]
pub struct TemperatureSensorCollector;

impl MetricCollector for TemperatureSensorCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record temperature sensor statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Sensor instance name (e.g. `"chamber"`)
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
        let stats: klipper_types::TemperatureSensorStats = serde_json::from_value(data.clone())?;
        let labels = labels_for(name);

        gauge!("klipper.stats.temperature_sensor.temperature", &labels).set(stats.temperature);
        gauge!(
            "klipper.stats.temperature_sensor.measured_min_temp",
            &labels
        )
        .set(stats.measured_min_temp);
        gauge!(
            "klipper.stats.temperature_sensor.measured_max_temp",
            &labels
        )
        .set(stats.measured_max_temp);

        Ok(())
    }
}

/// Collects temperature-controlled fan statistics.
///
/// Named instances correspond to fans declared in `printer.cfg`, e.g.
/// `[temperature_fan controller_fan]` produces the key `"temperature_fan controller_fan"`.
#[collector(prefix = "temperature_fan", named)]
pub struct TemperatureFanCollector;

impl MetricCollector for TemperatureFanCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record temperature-controlled fan statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Fan instance name (e.g. `"controller_fan"`)
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
        let stats: klipper_types::TemperatureFanStats = serde_json::from_value(data.clone())?;
        let labels = labels_for(name);

        gauge!("klipper.stats.temperature_fan.speed", &labels).set(stats.speed);
        gauge!("klipper.stats.temperature_fan.rpm", &labels).set(stats.rpm);
        gauge!("klipper.stats.temperature_fan.target", &labels).set(stats.target);
        gauge!("klipper.stats.temperature_fan.temperature", &labels).set(stats.temperature);

        Ok(())
    }
}
