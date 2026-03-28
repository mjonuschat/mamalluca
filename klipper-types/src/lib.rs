//! Type definitions for Klipper 3D printer firmware status objects.
//!
//! These types map to the Python dictionaries returned by Klipper's
//! `get_status()` methods. Source of truth: `klippy/` Python source.
//!
//! All structs use permissive deserialization (`#[serde(default)]` on every
//! field) so that older or newer Klipper versions that omit or add fields
//! do not cause deserialization failures. Unknown fields are captured in
//! an `extra` HashMap via `#[serde(flatten)]`.

pub mod bed;
pub mod extruder;
pub mod fan;
pub mod heater_bed;
pub mod mcu;
pub mod print;
pub mod sensor;
pub mod stepper;
pub mod system;
pub mod temperature;
pub mod toolhead;
pub mod webhooks;

// Re-export primary types at crate root for convenience.
pub use bed::{ProbeStats, ZTiltStats};
pub use extruder::ExtruderStats;
pub use fan::{GenericFanStats, TemperatureFanStats};
pub use heater_bed::HeaterBedStats;
pub use mcu::McuStats;
pub use print::{
    ExcludeObjectStats, PauseResumeStats, PrintJobInfo, PrintStats, VirtualSdCardStats,
};
pub use sensor::{FilamentRunoutSensorStats, ZThermalAdjustStats};
pub use stepper::{StepperEnableStats, TMCStepperMotorDriver};
pub use system::SystemStats;
pub use temperature::TemperatureSensorStats;
pub use toolhead::{GCodeMoveStats, MotionReportStats, ToolheadStats};
pub use webhooks::{KlippyState, WebhooksStats};
