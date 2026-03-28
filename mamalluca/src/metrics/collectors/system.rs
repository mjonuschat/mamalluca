//! Collector for Klipper host system resource stats.

use mamalluca_macros::collector;
use metrics::gauge;

use crate::metrics::MetricCollector;

/// Collects Klipper host system resource statistics.
///
/// Singleton — maps to the `"system_stats"` Klipper status object.
/// Reports CPU time consumed by the Klipper process, available host memory,
/// and the 1-minute system load average.
#[collector(prefix = "system_stats")]
pub struct SystemStatsCollector;

impl MetricCollector for SystemStatsCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record Klipper host system resource statistics.
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
        let stats: klipper_types::system::SystemStats = serde_json::from_value(data.clone())?;

        gauge!("klipper.stats.system.cpu_time").set(stats.cputime);
        gauge!("klipper.stats.system.mem_avail").set(stats.memavail as f64);
        gauge!("klipper.stats.system.sys_load").set(stats.sysload);

        Ok(())
    }
}
