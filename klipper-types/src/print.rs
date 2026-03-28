//! Klipper print job, virtual SD card, and object exclusion status types.
//!
//! Maps to `klippy/extras/print_stats.py`, `klippy/extras/virtual_sdcard.py`,
//! `klippy/extras/exclude_object.py`, and `klippy/extras/pause_resume.py`.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Layer information embedded inside [`PrintStats`].
///
/// Tracks the current and total layer count as reported by the slicer
/// via embedded G-code comments.
///
/// Source: `klippy/extras/print_stats.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PrintJobInfo {
    /// Zero-indexed current layer number.
    #[serde(default)]
    pub current_layer: u64,

    /// Total number of layers in the print, as parsed from slicer metadata.
    #[serde(default)]
    pub total_layer: u64,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Aggregate print statistics for the active or most recent print job.
///
/// Source: `klippy/extras/print_stats.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PrintStats {
    /// Total filament extruded during the print in millimeters.
    #[serde(default)]
    pub filament_used: f64,

    /// Elapsed time actively printing in seconds (excludes pauses).
    #[serde(default)]
    pub print_duration: f64,

    /// Total elapsed time in seconds, including pauses and idle.
    #[serde(default)]
    pub total_duration: f64,

    /// Layer information from slicer metadata.
    #[serde(default)]
    pub info: PrintJobInfo,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Virtual SD card status for file-based printing.
///
/// Source: `klippy/extras/virtual_sdcard.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct VirtualSdCardStats {
    /// Total file size in bytes.
    #[serde(default)]
    pub file_size: u64,

    /// Current read position in bytes.
    #[serde(default)]
    pub file_position: u64,

    /// Print progress as a fraction between 0.0 and 1.0.
    #[serde(default)]
    pub progress: f64,

    /// Whether the virtual SD card is actively reading a file.
    #[serde(default)]
    pub is_active: bool,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Object exclusion status tracking which objects have been excluded.
///
/// Source: `klippy/extras/exclude_object.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ExcludeObjectStats {
    /// List of objects that have been excluded from the current print.
    #[serde(default)]
    pub excluded_objects: Vec<serde_json::Value>,

    /// List of all objects known in the current print.
    #[serde(default)]
    pub objects: Vec<serde_json::Value>,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

/// Pause/resume state for the current print job.
///
/// Source: `klippy/extras/pause_resume.py`
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PauseResumeStats {
    /// Whether the print is currently paused.
    #[serde(default)]
    pub is_paused: bool,

    /// Captures unknown keys from newer Klipper/Kalico firmware versions.
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deserialize_print_stats_full() {
        let json = serde_json::json!({
            "filament_used": 1234.56,
            "print_duration": 3600.0,
            "total_duration": 3700.0,
            "info": {
                "current_layer": 42,
                "total_layer": 200
            }
        });
        let stats: PrintStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert!((stats.filament_used - 1234.56).abs() < f64::EPSILON);
        assert_eq!(stats.info.current_layer, 42);
        assert_eq!(stats.info.total_layer, 200);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_print_stats_unknown_fields() {
        let json = serde_json::json!({
            "filament_used": 0.0,
            "state": "printing"
        });
        let stats: PrintStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("state"));
    }

    #[test]
    fn deserialize_virtual_sdcard_full() {
        let json = serde_json::json!({
            "file_size": 1048576,
            "file_position": 524288,
            "progress": 0.5,
            "is_active": true
        });
        let stats: VirtualSdCardStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert_eq!(stats.file_size, 1_048_576);
        assert!(stats.is_active);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_virtual_sdcard_unknown_fields() {
        let json = serde_json::json!({
            "progress": 0.0,
            "filename": "test.gcode"
        });
        let stats: VirtualSdCardStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("filename"));
    }

    #[test]
    fn deserialize_exclude_object_full() {
        let json = serde_json::json!({
            "excluded_objects": [{"name": "part1"}],
            "objects": [{"name": "part1"}, {"name": "part2"}]
        });
        let stats: ExcludeObjectStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert_eq!(stats.excluded_objects.len(), 1);
        assert_eq!(stats.objects.len(), 2);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_exclude_object_unknown_fields() {
        let json = serde_json::json!({
            "excluded_objects": [],
            "objects": [],
            "current_object": "part2"
        });
        let stats: ExcludeObjectStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("current_object"));
    }

    #[test]
    fn deserialize_pause_resume_full() {
        let json = serde_json::json!({"is_paused": true});
        let stats: PauseResumeStats =
            serde_json::from_value(json).expect("should deserialize full payload");
        assert!(stats.is_paused);
        assert!(stats.extra.is_empty());
    }

    #[test]
    fn deserialize_pause_resume_unknown_fields() {
        let json = serde_json::json!({"is_paused": false, "pause_reason": "user"});
        let stats: PauseResumeStats =
            serde_json::from_value(json).expect("should deserialize with unknown fields");
        assert!(stats.extra.contains_key("pause_reason"));
    }
}
