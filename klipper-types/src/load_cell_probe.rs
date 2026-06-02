use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct LoadCellProbeStats {
    #[serde(default)]
    pub force_g: f64,

    #[serde(default)]
    pub min_force_g: f64,

    #[serde(default)]
    pub max_force_g: f64,

    #[serde(default)]
    pub is_calibrated: bool,

    #[serde(default)]
    pub counts_per_gram: f64,

    #[serde(default)]
    pub reference_tare_counts: f64,

    #[serde(default)]
    pub tare_counts: f64,

    #[serde(default)]
    pub tare_force: f64,

    #[serde(default)]
    pub last_trigger_time: f64,

    #[serde(default)]
    pub is_last_tap_valid: bool,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
