//! Collectors for Klipper print job, virtual SD card, object exclusion, and pause/resume stats.

use mamalluca_macros::collector;
use metrics::gauge;

use super::labels_for;
use crate::metrics::MetricCollector;

/// Collects print job statistics.
///
/// Singleton — maps to the `"print_stats"` Klipper status object.
#[collector(prefix = "print_stats")]
pub struct PrintStatsCollector;

impl MetricCollector for PrintStatsCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record print job statistics.
    ///
    /// Layer counts are sourced from the nested `info` sub-object.
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
        let stats: klipper_types::print::PrintStats = serde_json::from_value(data.clone())?;
        // This collector has no named instances; always use an empty label set.
        let labels = labels_for(None);

        gauge!("klipper.stats.print_stats.filament_used", &labels).set(stats.filament_used);
        gauge!("klipper.stats.print_stats.print_duration", &labels).set(stats.print_duration);
        gauge!("klipper.stats.print_stats.total_duration", &labels).set(stats.total_duration);
        gauge!("klipper.stats.print_stats.current_layer", &labels)
            .set(stats.info.current_layer as f64);
        gauge!("klipper.stats.print_stats.total_layer", &labels).set(stats.info.total_layer as f64);

        Ok(())
    }
}

/// Collects virtual SD card statistics.
///
/// Singleton — maps to the `"virtual_sdcard"` Klipper status object.
#[collector(prefix = "virtual_sdcard")]
pub struct VirtualSdCardCollector;

impl MetricCollector for VirtualSdCardCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record virtual SD card statistics.
    ///
    /// `is_active` is a boolean recorded as `0.0`/`1.0` — Prometheus has no
    /// native boolean type.
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
        let stats: klipper_types::print::VirtualSdCardStats = serde_json::from_value(data.clone())?;
        let labels = labels_for(None);

        gauge!("klipper.stats.virtual_sdcard.file_size", &labels).set(stats.file_size as f64);
        gauge!("klipper.stats.virtual_sdcard.file_position", &labels)
            .set(stats.file_position as f64);
        gauge!("klipper.stats.virtual_sdcard.progress", &labels).set(stats.progress);
        // Prometheus has no boolean type; represent as 0.0/1.0.
        gauge!("klipper.stats.virtual_sdcard.is_active", &labels).set(stats.is_active as u8 as f64);

        Ok(())
    }
}

/// Collects object-exclusion statistics.
///
/// Singleton — maps to the `"exclude_object"` Klipper status object.
/// Records only aggregate counts (excluded vs. total objects).
#[collector(prefix = "exclude_object")]
pub struct ExcludeObjectCollector;

impl MetricCollector for ExcludeObjectCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record object exclusion statistics.
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
        let stats: klipper_types::print::ExcludeObjectStats = serde_json::from_value(data.clone())?;
        let labels = labels_for(None);

        gauge!("klipper.stats.exclude_objects.excluded", &labels)
            .set(stats.excluded_objects.len() as f64);
        gauge!("klipper.stats.exclude_objects.objects", &labels).set(stats.objects.len() as f64);

        Ok(())
    }
}

/// Collects pause/resume state for the current print job.
///
/// Singleton — maps to the `"pause_resume"` Klipper status object.
#[collector(prefix = "pause_resume")]
pub struct PauseResumeCollector;

impl MetricCollector for PauseResumeCollector {
    fn key_prefix(&self) -> &str {
        Self::KEY_PREFIX
    }

    fn is_named(&self) -> bool {
        Self::IS_NAMED
    }

    /// Deserialize and record pause/resume state.
    ///
    /// `is_paused` is recorded as `0.0`/`1.0` — Prometheus has no native
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
        let stats: klipper_types::print::PauseResumeStats = serde_json::from_value(data.clone())?;

        // Prometheus has no boolean type; represent as 0.0/1.0.
        gauge!("klipper.stats.pause_resume.paused").set(stats.is_paused as u8 as f64);

        Ok(())
    }
}
