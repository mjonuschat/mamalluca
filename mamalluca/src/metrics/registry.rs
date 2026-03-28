//! Registry that maps status keys to collectors.

use super::{CollectorEntry, MetricCollector};
use std::collections::HashMap;

/// Routes status updates to the appropriate collector.
///
/// Built at startup from all `inventory`-registered [`CollectorEntry`] instances.
/// Singletons use exact-match HashMap lookup (O(1)).
/// Named collectors use prefix matching (O(n), n ~ 15 named prefixes).
pub struct CollectorRegistry {
    /// Exact-match: key -> collector (e.g. "toolhead" -> ToolheadCollector).
    singletons: HashMap<String, &'static dyn MetricCollector>,
    /// Prefix-match: collectors for named instances (e.g. "tmc2209" matches "tmc2209 stepper_x").
    named: Vec<&'static dyn MetricCollector>,
}

impl CollectorRegistry {
    /// Build the registry from all inventory-registered collectors.
    ///
    /// Iterates over every [`CollectorEntry`] submitted via `inventory::submit!`,
    /// sorting each into either the singleton map or the named-prefix list.
    pub fn from_inventory() -> Self {
        let mut singletons = HashMap::new();
        let mut named = Vec::new();

        for entry in inventory::iter::<CollectorEntry> {
            let collector = entry.collector();
            if collector.is_named() {
                named.push(collector);
            } else {
                singletons.insert(collector.key_prefix().to_owned(), collector);
            }
        }

        tracing::info!(
            singletons = singletons.len(),
            named = named.len(),
            "Built collector registry"
        );

        Self { singletons, named }
    }

    /// Dispatch a status update to the matching collector.
    ///
    /// Returns `Ok(true)` if a collector handled it, `Ok(false)` if no
    /// collector matched (unknown status key -- not an error).
    ///
    /// # Arguments
    /// * `key` - The full status key (e.g. `"toolhead"` or `"tmc2209 stepper_x"`)
    /// * `data` - Raw JSON value from the status update
    ///
    /// # Errors
    /// Propagates any error from the matched collector's `record()` method.
    pub fn dispatch(&self, key: &str, data: &serde_json::Value) -> anyhow::Result<bool> {
        // Try exact match first (singletons like "toolhead", "print_stats").
        if let Some(collector) = self.singletons.get(key) {
            collector.record(key, None, data)?;
            return Ok(true);
        }

        // Try prefix match (named instances like "tmc2209 stepper_x").
        // Split on first space: prefix = "tmc2209", name = "stepper_x".
        let (prefix, name) = key
            .split_once(' ')
            .map(|(p, n)| (p, Some(n)))
            .unwrap_or((key, None));

        for collector in &self.named {
            if collector.key_prefix() == prefix {
                collector.record(key, name, data)?;
                return Ok(true);
            }
        }

        tracing::debug!(key, "No collector registered for status key");
        Ok(false)
    }
}
