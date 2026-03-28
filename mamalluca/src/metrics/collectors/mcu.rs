//! Collector for Klipper MCU (microcontroller) stats.

use mamalluca_macros::collector;
use metrics::{counter, gauge};

use super::labels_for;
use crate::metrics::MetricCollector;

/// Collects MCU statistics.
///
/// MCU status is a named instance — `"mcu"` for the main board,
/// `"mcu toolhead"` for additional MCUs.
#[collector(prefix = "mcu", named)]
pub struct McuCollector;

impl MetricCollector for McuCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record MCU communication statistics.
    ///
    /// MCU stats are nested under `last_stats` in the raw status object.
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; prefix matching already happened)
    /// * `name` - Instance name (e.g. `"toolhead"` for a secondary MCU), `None` for the main MCU
    /// * `data` - Raw JSON value from the status update
    ///
    /// # Errors
    /// Returns an error if `last_stats` is absent or if deserialization fails.
    fn record(
        &self,
        _key: &str,
        name: Option<&str>,
        data: &serde_json::Value,
    ) -> anyhow::Result<()> {
        // MCU stats are nested under `last_stats` in the raw status object.
        let stats_data = data
            .get("last_stats")
            .ok_or_else(|| anyhow::anyhow!("Missing last_stats in MCU data"))?;
        let stats: klipper_types::McuStats = serde_json::from_value(stats_data.clone())?;
        let labels = labels_for(name);

        gauge!("klipper.stats.mcu.adj", &labels).set(stats.adj as f64);
        gauge!("klipper.stats.mcu.freq", &labels).set(stats.freq as f64);
        gauge!("klipper.stats.mcu.mcu_awake", &labels).set(stats.mcu_awake);
        gauge!("klipper.stats.mcu.mcu_task_avg", &labels).set(stats.mcu_task_avg);
        gauge!("klipper.stats.mcu.mcu_task_stddev", &labels).set(stats.mcu_task_stddev);
        gauge!("klipper.stats.mcu.ready_bytes", &labels).set(stats.ready_bytes as f64);
        gauge!("klipper.stats.mcu.upcoming_bytes", &labels).set(stats.upcoming_bytes as f64);

        counter!("klipper.stats.mcu.bytes_read", &labels).absolute(stats.bytes_read);
        counter!("klipper.stats.mcu.bytes_write", &labels).absolute(stats.bytes_write);
        counter!("klipper.stats.mcu.bytes_invalid", &labels).absolute(stats.bytes_invalid);
        counter!("klipper.stats.mcu.bytes_retransmit", &labels).absolute(stats.bytes_retransmit);

        counter!("klipper.stats.mcu.receive_seq", &labels).absolute(stats.receive_seq);
        counter!("klipper.stats.mcu.send_seq", &labels).absolute(stats.send_seq);
        counter!("klipper.stats.mcu.retransmit_seq", &labels).absolute(stats.retransmit_seq);

        gauge!("klipper.stats.mcu.rto", &labels).set(stats.rto);
        gauge!("klipper.stats.mcu.rttvar", &labels).set(stats.rttvar);
        gauge!("klipper.stats.mcu.srtt", &labels).set(stats.srtt);

        Ok(())
    }
}
