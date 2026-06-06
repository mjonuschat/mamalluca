//! Collector for Kalico load_cell_probe statistics

use mamalluca_macros::collector;
use metrics::gauge;

use super::labels_for;
use crate::metrics::MetricCollector;

#[collector(prefix = "load_cell_probe", named)]
pub struct LoadCellProbeCollector;

impl MetricCollector for LoadCellProbeCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record load cell probe statistics.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Instance name (e.g. `"nextruder"`), `None` for the primary probe
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
        let stats: klipper_types::LoadCellProbeStats = serde_json::from_value(data.clone())?;
        let labels = labels_for(name);

        gauge_if_some!(
            "klipper.stats.load_cell_probe.force_g",
            &labels,
            stats.force_g
        );
        gauge_if_some!(
            "klipper.stats.load_cell_probe.min_force_g",
            &labels,
            stats.min_force_g
        );
        gauge_if_some!(
            "klipper.stats.load_cell_probe.max_force_g",
            &labels,
            stats.max_force_g
        );
        // Prometheus has no boolean type; represent as 0.0/1.0.
        gauge!("klipper.stats.load_cell_probe.is_calibrated", &labels)
            .set(stats.is_calibrated as u8 as f64);
        gauge_if_some!(
            "klipper.stats.load_cell_probe.counts_per_gram",
            &labels,
            stats.counts_per_gram
        );
        gauge_if_some!(
            "klipper.stats.load_cell_probe.reference_tare_counts",
            &labels,
            stats.reference_tare_counts
        );
        gauge_if_some!(
            "klipper.stats.load_cell_probe.tare_counts",
            &labels,
            stats.tare_counts
        );
        gauge_if_some!(
            "klipper.stats.load_cell_probe.tare_force",
            &labels,
            stats.tare_force
        );
        gauge!("klipper.stats.load_cell_probe.last_trigger_time", &labels)
            .set(stats.last_trigger_time);
        gauge!("klipper.stats.load_cell_probe.is_last_tap_valid", &labels)
            .set(stats.is_last_tap_valid as u8 as f64);

        Ok(())
    }
}
