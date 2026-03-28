//! Collectors for Klipper TMC stepper motor drivers and stepper enable tracking.

use mamalluca_macros::collector;
use metrics::gauge;

use super::labels_for;
use crate::metrics::MetricCollector;

/// Record TMC stepper driver gauges from a [`klipper_types::TMCStepperMotorDriver`] value.
///
/// Shared by all six TMC variant collectors to avoid code duplication.
///
/// # Arguments
/// * `name` - Instance name (e.g. `"stepper_x"`), or `None` for singleton drivers
/// * `data` - Raw JSON value from the status update
///
/// # Errors
/// Returns an error if deserialization fails.
fn record_tmc_stats(name: Option<&str>, data: &serde_json::Value) -> anyhow::Result<()> {
    let stats: klipper_types::TMCStepperMotorDriver = serde_json::from_value(data.clone())?;
    let labels = labels_for(name);

    gauge!("klipper.stats.stepper_driver.hold_current", &labels).set(stats.hold_current);
    gauge!("klipper.stats.stepper_driver.run_current", &labels).set(stats.run_current);

    // Temperature is optional — not all TMC variants report it.
    if let Some(temp) = stats.temperature {
        gauge!("klipper.stats.temperature.current", &labels).set(temp);
    }

    Ok(())
}

/// Collects TMC2130 stepper driver statistics.
///
/// Named instances correspond to drivers declared in `printer.cfg` as
/// `[tmc2130 stepper_x]`, etc.
#[collector(prefix = "tmc2130", named)]
pub struct Tmc2130Collector;

impl MetricCollector for Tmc2130Collector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record TMC2130 driver statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Stepper name (e.g. `"stepper_x"`)
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
        record_tmc_stats(name, data)
    }
}

/// Collects TMC2208 stepper driver statistics.
///
/// Named instances correspond to drivers declared in `printer.cfg` as
/// `[tmc2208 stepper_x]`, etc.
#[collector(prefix = "tmc2208", named)]
pub struct Tmc2208Collector;

impl MetricCollector for Tmc2208Collector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record TMC2208 driver statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Stepper name (e.g. `"stepper_x"`)
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
        record_tmc_stats(name, data)
    }
}

/// Collects TMC2209 stepper driver statistics.
///
/// Named instances correspond to drivers declared in `printer.cfg` as
/// `[tmc2209 stepper_x]`, etc.
#[collector(prefix = "tmc2209", named)]
pub struct Tmc2209Collector;

impl MetricCollector for Tmc2209Collector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record TMC2209 driver statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Stepper name (e.g. `"stepper_x"`)
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
        record_tmc_stats(name, data)
    }
}

/// Collects TMC2240 stepper driver statistics.
///
/// Named instances correspond to drivers declared in `printer.cfg` as
/// `[tmc2240 stepper_x]`, etc.
#[collector(prefix = "tmc2240", named)]
pub struct Tmc2240Collector;

impl MetricCollector for Tmc2240Collector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record TMC2240 driver statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Stepper name (e.g. `"stepper_x"`)
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
        record_tmc_stats(name, data)
    }
}

/// Collects TMC2660 stepper driver statistics.
///
/// Named instances correspond to drivers declared in `printer.cfg` as
/// `[tmc2660 stepper_x]`, etc.
#[collector(prefix = "tmc2660", named)]
pub struct Tmc2660Collector;

impl MetricCollector for Tmc2660Collector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record TMC2660 driver statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Stepper name (e.g. `"stepper_x"`)
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
        record_tmc_stats(name, data)
    }
}

/// Collects TMC5160 stepper driver statistics.
///
/// Named instances correspond to drivers declared in `printer.cfg` as
/// `[tmc5160 stepper_x]`, etc.
#[collector(prefix = "tmc5160", named)]
pub struct Tmc5160Collector;

impl MetricCollector for Tmc5160Collector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record TMC5160 driver statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Stepper name (e.g. `"stepper_x"`)
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
        record_tmc_stats(name, data)
    }
}

/// Collects stepper enable/disable state for all steppers.
///
/// `stepper_enable` is a singleton that reports the energized state of every
/// configured stepper motor. One gauge is emitted per stepper, with the
/// stepper name added as a `name` label.
#[collector(prefix = "stepper_enable")]
pub struct StepperEnableCollector;

impl MetricCollector for StepperEnableCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record stepper enable state for all steppers.
    ///
    /// Iterates the `steppers` map and emits one gauge per entry, labelled
    /// with the stepper name. Enabled is recorded as `1.0`, disabled as `0.0`.
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
        _name: Option<&str>,
        data: &serde_json::Value,
    ) -> anyhow::Result<()> {
        let stats: klipper_types::StepperEnableStats = serde_json::from_value(data.clone())?;

        for (stepper_name, enabled) in &stats.steppers {
            // Prometheus has no boolean type; represent as 0.0/1.0.
            let labels = labels_for(Some(stepper_name.as_str()));
            gauge!("klipper.stats.stepper_driver.enabled", &labels).set(*enabled as u8 as f64);
        }

        Ok(())
    }
}
