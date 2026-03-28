//! Collectors for Klipper bed probe and Z-tilt adjustment stats.

use mamalluca_macros::collector;
use metrics::gauge;

use crate::metrics::MetricCollector;

/// Collects probe statistics.
///
/// Singleton — maps to the `"probe"` Klipper status object.
/// The `last_z_result` is labelled with the probe's own `name` field so that
/// systems with multiple probes can be distinguished in dashboards.
#[collector(prefix = "probe")]
pub struct ProbeCollector;

impl MetricCollector for ProbeCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record probe statistics.
    ///
    /// The `name` label is taken from [`klipper_types::bed::ProbeStats::name`] so
    /// that the metric remains unambiguous when multiple probe modules coexist.
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
        let stats: klipper_types::bed::ProbeStats = serde_json::from_value(data.clone())?;

        // Use the probe's own name field as the label value so dashboards can
        // distinguish between different probe modules (e.g. "bltouch", "probe").
        let labels: Vec<(&'static str, String)> = vec![("name", stats.name.clone())];

        gauge!("klipper.stats.probe.last_z_result", &labels).set(stats.last_z_result);

        Ok(())
    }
}

/// Collects Z-tilt adjustment statistics.
///
/// Singleton — maps to the `"z_tilt"` Klipper status object.
#[collector(prefix = "z_tilt")]
pub struct ZTiltCollector;

impl MetricCollector for ZTiltCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record Z-tilt adjustment statistics.
    ///
    /// `applied` is recorded as `0.0`/`1.0` — Prometheus has no native
    /// boolean type.
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
        let stats: klipper_types::bed::ZTiltStats = serde_json::from_value(data.clone())?;

        // Prometheus has no boolean type; represent as 0.0/1.0.
        gauge!("klipper.stats.z_tilt.applied").set(stats.applied as u8 as f64);

        Ok(())
    }
}
