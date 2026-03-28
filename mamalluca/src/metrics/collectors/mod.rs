//! Metric collector implementations.
//!
//! Each sub-module contains collectors for a group of related Klipper
//! or Moonraker status objects. Collectors self-register via the
//! `#[collector]` macro and `inventory` crate.

pub mod extruder;
pub mod heater_bed;
pub mod mcu;
pub mod temperature;

/// Build a Prometheus label vector with an optional instance name.
///
/// Named collectors (e.g. multiple extruders) pass the instance name as a label.
/// Singleton collectors pass `None` and get an empty label set.
pub(crate) fn labels_for(name: Option<&str>) -> Vec<(&'static str, String)> {
    name.map(|n| vec![("name", n.to_owned())])
        .unwrap_or_default()
}
