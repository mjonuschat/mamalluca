//! Collector for Klipper extruder (hotend) stats.

use mamalluca_macros::collector;
use metrics::gauge;

use super::labels_for;
use crate::metrics::MetricCollector;

/// Collects extruder (hotend) statistics.
///
/// Extruder status is a named instance — `"extruder"` for the primary extruder,
/// `"extruder1"`, `"extruder2"`, etc. for additional extruders.
#[collector(prefix = "extruder", named)]
pub struct ExtruderCollector;

impl MetricCollector for ExtruderCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record extruder statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Instance name (e.g. `"extruder1"`), `None` for the primary extruder
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
        let stats: klipper_types::ExtruderStats = serde_json::from_value(data.clone())?;
        let labels = labels_for(name);

        // Prometheus has no boolean type; represent as 0.0/1.0.
        gauge!("klipper.stats.extruder.can_extrude", &labels).set(stats.can_extrude as u8 as f64);
        gauge!("klipper.stats.extruder.power", &labels).set(stats.power);
        gauge!("klipper.stats.extruder.pressure_advance", &labels).set(stats.pressure_advance);
        gauge!("klipper.stats.extruder.smooth_time", &labels).set(stats.smooth_time);
        gauge!("klipper.stats.extruder.target", &labels).set(stats.target);
        gauge!("klipper.stats.extruder.temperature", &labels).set(stats.temperature);

        // `time_offset` is optional — only present in Kalico and some Klipper forks.
        if let Some(offset) = stats.time_offset {
            gauge!("klipper.stats.extruder.time_offset", &labels).set(offset);
        }

        Ok(())
    }
}
