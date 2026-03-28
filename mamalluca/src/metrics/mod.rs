//! Metric collection framework.
//!
//! Provides the [`MetricCollector`] trait and [`CollectorRegistry`] for
//! dispatching Klipper/Moonraker status updates to typed collectors.

pub mod collectors;
pub mod registry;

/// A collector that deserializes a Klipper/Moonraker status object
/// and records its values as Prometheus metrics.
///
/// Implement `record()` manually. Use the `#[collector]` macro from
/// `mamalluca-macros` to auto-generate `key_prefix()`, `is_named()`,
/// and inventory registration.
pub trait MetricCollector: Send + Sync + 'static {
    /// The status key prefix this collector handles.
    ///
    /// For singleton collectors: the exact key (e.g. `"toolhead"`).
    /// For named collectors: the prefix before the space (e.g. `"tmc2209"`
    /// matches `"tmc2209 stepper_x"`).
    fn key_prefix(&self) -> &str;

    /// Whether this collector handles named instances.
    ///
    /// Named collectors match keys like `"prefix instance_name"`.
    /// Singleton collectors match the exact key.
    fn is_named(&self) -> bool;

    /// Deserialize and record metrics from a status update.
    ///
    /// # Arguments
    /// * `key` - The full status key (e.g. `"tmc2209 stepper_x"`)
    /// * `name` - The instance name for named collectors (`Some("stepper_x")`),
    ///   `None` for singletons
    /// * `data` - Raw JSON value from the status update
    fn record(&self, key: &str, name: Option<&str>, data: &serde_json::Value)
    -> anyhow::Result<()>;
}

/// Wrapper for `inventory` collection.
///
/// `inventory` requires a concrete `Collect` type. This wraps a
/// [`MetricCollector`] trait object for registration.
pub struct CollectorEntry {
    /// The wrapped collector trait object.
    collector: Box<dyn MetricCollector>,
}

impl CollectorEntry {
    /// Create a new entry wrapping a collector.
    pub fn new<T: MetricCollector>(collector: T) -> Self {
        Self {
            collector: Box::new(collector),
        }
    }

    /// Get a reference to the wrapped collector.
    pub fn collector(&self) -> &dyn MetricCollector {
        &*self.collector
    }
}

// Make CollectorEntry collectable by inventory.
// This allows `inventory::iter::<CollectorEntry>` to iterate over all
// collectors submitted via `inventory::submit!` at program startup.
inventory::collect!(CollectorEntry);
