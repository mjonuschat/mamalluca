//! Collector for Moonraker server process and system resource stats.
//!
//! Moonraker server stats arrive via `notify_proc_stat_update`. The reconnect
//! loop re-emits these as a `StatusUpdate` with key `"moonraker"` and the raw
//! JSON params array as the data payload. This collector extracts element 0
//! from that array before deserializing.

use mamalluca_macros::collector;
use metrics::{counter, gauge};

use crate::metrics::MetricCollector;

/// Collects Moonraker server process and system statistics.
///
/// Singleton — keyed as `"moonraker"` in the status update stream.
///
/// The payload arrives as a JSON array (the raw `notify_proc_stat_update`
/// params). This collector extracts element `[0]` before deserializing into
/// [`moonraker_types::MoonrakerStats`].
#[collector(prefix = "moonraker")]
pub struct MoonrakerStatsCollector;

impl MetricCollector for MoonrakerStatsCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record Moonraker server resource statistics.
    ///
    /// The `data` value is the raw `notify_proc_stat_update` params, which is
    /// a JSON array. Element `[0]` contains the actual stats object.
    ///
    /// Records:
    /// - Service: `memory`, `cpu_usage`, `websocket_connections`
    /// - Network: per-interface `bandwidth`, and cumulative rx/tx bytes,
    ///   packets, errors, and drops (as counters)
    /// - CPU: overall `cpu_usage` and per-core usage, plus `cpu_temp`
    /// - Memory: `total`, `available`, `used`
    ///
    /// # Arguments
    /// * `_key` - The full status key (unused; exact match already happened)
    /// * `_name` - Always `None` for this singleton collector
    /// * `data` - The raw `notify_proc_stat_update` params (a JSON array)
    ///
    /// # Errors
    /// Returns an error if `data[0]` is absent or if deserialization fails.
    fn record(
        &self,
        _key: &str,
        _name: Option<&str>,
        data: &serde_json::Value,
    ) -> anyhow::Result<()> {
        // `notify_proc_stat_update` params are wrapped in an array; extract [0].
        let stats_data = data
            .get(0)
            .ok_or_else(|| anyhow::anyhow!("Missing element [0] in moonraker proc_stat payload"))?;
        let stats: moonraker_types::MoonrakerStats = serde_json::from_value(stats_data.clone())?;

        // --- Moonraker service metrics ---
        gauge!("moonraker.stats.service.memory").set(stats.moonraker_stats.memory as f64);
        gauge!("moonraker.stats.service.cpu_usage").set(stats.moonraker_stats.cpu_usage);
        gauge!("moonraker.stats.service.websocket_connections")
            .set(stats.websocket_connections as f64);

        // --- Per-network-interface metrics ---
        for (iface, net) in &stats.network {
            // Each interface gets a label so time series can be distinguished.
            let intf_labels: Vec<(&'static str, String)> = vec![("interface", iface.clone())];

            gauge!("moonraker.stats.network.bandwidth", &intf_labels).set(net.bandwidth);

            // Cumulative byte/packet counters use `absolute` so the value is
            // treated as a monotonically increasing counter by Prometheus.
            counter!("moonraker.stats.network.rx_bytes", &intf_labels).absolute(net.rx_bytes);
            counter!("moonraker.stats.network.rx_packets", &intf_labels).absolute(net.rx_packets);
            counter!("moonraker.stats.network.rx_errs", &intf_labels).absolute(net.rx_errs);
            counter!("moonraker.stats.network.rx_drop", &intf_labels).absolute(net.rx_drop);

            counter!("moonraker.stats.network.tx_bytes", &intf_labels).absolute(net.tx_bytes);
            counter!("moonraker.stats.network.tx_packets", &intf_labels).absolute(net.tx_packets);
            counter!("moonraker.stats.network.tx_errs", &intf_labels).absolute(net.tx_errs);
            counter!("moonraker.stats.network.tx_drop", &intf_labels).absolute(net.tx_drop);
        }

        // --- CPU metrics ---
        // Overall (aggregate) CPU usage is labelled cpu="cpu" to match the
        // per-core labels below (cpu="cpu0", cpu="cpu1", …).
        {
            let cpu_labels: Vec<(&'static str, String)> = vec![("cpu", "cpu".to_owned())];
            gauge!("moonraker.stats.system.cpu_usage", &cpu_labels).set(stats.system_cpu_usage.cpu);
            gauge!("moonraker.stats.system.cpu_temp", &cpu_labels).set(stats.cpu_temp);
        }

        // Per-core CPU usage — `cores` map keys are e.g. "cpu0", "cpu1", …
        for (core, value) in &stats.system_cpu_usage.cores {
            let core_labels: Vec<(&'static str, String)> = vec![("cpu", core.clone())];
            gauge!("moonraker.stats.system.cpu_usage", &core_labels).set(*value);
        }

        // --- Memory metrics ---
        gauge!("moonraker.stats.system.memory_total").set(stats.system_memory.total as f64);
        gauge!("moonraker.stats.system.memory_available").set(stats.system_memory.available as f64);
        gauge!("moonraker.stats.system.memory_used").set(stats.system_memory.used as f64);

        Ok(())
    }
}
