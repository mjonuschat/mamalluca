//! Type definitions for Klipper 3D printer firmware status objects.
//!
//! These types map to the Python dictionaries returned by Klipper's
//! `get_status()` methods. Source of truth: `klippy/` Python source.
//!
//! All structs use permissive deserialization (`#[serde(default)]` on every
//! field) so that older or newer Klipper versions that omit or add fields
//! do not cause deserialization failures. Unknown fields are captured in
//! an `extra` HashMap via `#[serde(flatten)]`.

pub mod extruder;
pub mod heater_bed;
pub mod mcu;
pub mod temperature;
pub mod webhooks;

// Re-export primary types at crate root for convenience.
pub use extruder::ExtruderStats;
pub use heater_bed::HeaterBedStats;
pub use mcu::McuStats;
pub use temperature::TemperatureSensorStats;
pub use webhooks::{KlippyState, WebhooksStats};
