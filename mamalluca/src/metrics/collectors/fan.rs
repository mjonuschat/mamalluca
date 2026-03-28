//! Collectors for Klipper fan types (part cooling, generic, heater, controller).

use mamalluca_macros::collector;
use metrics::gauge;

use super::labels_for;
use crate::metrics::MetricCollector;

/// Record speed and RPM gauges from a [`klipper_types::GenericFanStats`] value.
///
/// Shared by all four fan collectors that use the same underlying type.
///
/// # Arguments
/// * `name` - Instance name, or `None` for singleton fans
/// * `data` - Raw JSON value from the status update
///
/// # Errors
/// Returns an error if deserialization fails.
fn record_generic_fan_stats(name: Option<&str>, data: &serde_json::Value) -> anyhow::Result<()> {
    let stats: klipper_types::GenericFanStats = serde_json::from_value(data.clone())?;
    let labels = labels_for(name);

    gauge!("klipper.stats.fan.speed", &labels).set(stats.speed);
    gauge!("klipper.stats.fan.rpm", &labels).set(stats.rpm);

    Ok(())
}

/// Collects part-cooling fan statistics.
///
/// `fan` is a singleton in Klipper — there is at most one part-cooling fan
/// per printer, declared as `[fan]` in `printer.cfg`.
#[collector(prefix = "fan")]
pub struct FanCollector;

impl MetricCollector for FanCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record part-cooling fan statistics.
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
        record_generic_fan_stats(name, data)
    }
}

/// Collects generic fan statistics.
///
/// Named instances correspond to fans declared in `printer.cfg` as
/// `[fan_generic <name>]`, e.g. `[fan_generic exhaust]`.
#[collector(prefix = "fan_generic", named)]
pub struct FanGenericCollector;

impl MetricCollector for FanGenericCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record generic fan statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Fan instance name (e.g. `"exhaust"`)
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
        record_generic_fan_stats(name, data)
    }
}

/// Collects heater fan statistics.
///
/// Named instances correspond to fans declared in `printer.cfg` as
/// `[heater_fan <name>]`, e.g. `[heater_fan hotend_fan]`.
#[collector(prefix = "heater_fan", named)]
pub struct HeaterFanCollector;

impl MetricCollector for HeaterFanCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record heater fan statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Fan instance name (e.g. `"hotend_fan"`)
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
        record_generic_fan_stats(name, data)
    }
}

/// Collects controller fan statistics.
///
/// Named instances correspond to fans declared in `printer.cfg` as
/// `[controller_fan <name>]`, e.g. `[controller_fan electronics_bay]`.
#[collector(prefix = "controller_fan", named)]
pub struct ControllerFanCollector;

impl MetricCollector for ControllerFanCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record controller fan statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Fan instance name (e.g. `"electronics_bay"`)
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
        record_generic_fan_stats(name, data)
    }
}
