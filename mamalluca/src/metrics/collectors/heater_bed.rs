//! Collector for Klipper heated bed stats.

use mamalluca_macros::collector;
use metrics::gauge;

use super::labels_for;
use crate::metrics::MetricCollector;

/// Collects heated bed statistics.
///
/// The heated bed is typically a singleton (`"heater_bed"`), but in theory
/// a printer could have multiple named beds — so this collector is `named`.
#[collector(prefix = "heater_bed", named)]
pub struct HeaterBedCollector;

impl MetricCollector for HeaterBedCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record heated bed statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Instance name if more than one bed is configured, `None` otherwise
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
        let stats: klipper_types::HeaterBedStats = serde_json::from_value(data.clone())?;
        let labels = labels_for(name);

        gauge!("klipper.stats.heater_bed.power", &labels).set(stats.power);
        gauge!("klipper.stats.heater_bed.target", &labels).set(stats.target);
        gauge!("klipper.stats.heater_bed.temperature", &labels).set(stats.temperature);

        Ok(())
    }
}

/// Collects generic heater statistics (e.g. enclosure or chamber heaters).
///
/// Named instances correspond to heaters declared in `printer.cfg`, e.g.
/// `[heater_generic heater_chamber]` produces the key `"heater_generic heater_chamber"`.
///
/// Source: `klippy/extras/heaters.py`
#[collector(prefix = "heater_generic", named)]
pub struct HeaterGenericCollector;

impl MetricCollector for HeaterGenericCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record generic heater statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Heater instance name (e.g. `"heater_chamber"`)
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
        let stats: klipper_types::HeaterBedStats = serde_json::from_value(data.clone())?;
        let labels = labels_for(name);

        gauge!("klipper.stats.heater_generic.power", &labels).set(stats.power);
        gauge!("klipper.stats.heater_generic.target", &labels).set(stats.target);
        gauge!("klipper.stats.heater_generic.temperature", &labels).set(stats.temperature);

        Ok(())
    }
}
