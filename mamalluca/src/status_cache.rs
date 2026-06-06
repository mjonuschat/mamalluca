//! Full-object status cache for Moonraker partial status updates.

use serde_json::Value;
use std::collections::HashMap;

/// Caches the last full status object for each Moonraker object key.
#[derive(Debug, Default)]
pub(crate) struct StatusCache {
    objects: HashMap<String, Value>,
}

impl StatusCache {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    /// Seed the cache from a `printer.objects.subscribe` response.
    ///
    /// Returns cloned full objects so the caller can dispatch the initial snapshot
    /// immediately after seeding the cache.
    pub(crate) fn seed_from_subscribe_response(
        &mut self,
        response: &Value,
    ) -> Option<Vec<(String, Value)>> {
        let status = response.get("status")?.as_object()?;
        let mut seeded = Vec::with_capacity(status.len());

        for (key, value) in status {
            self.objects.insert(key.clone(), value.clone());
            seeded.push((key.clone(), value.clone()));
        }

        Some(seeded)
    }

    /// Merge a partial status update into the cached object and return the cache entry.
    pub(crate) fn update(&mut self, key: &str, data: Value) -> &Value {
        let cached = self.objects.entry(key.to_owned()).or_insert(Value::Null);
        merge_value(cached, data);
        cached
    }
}

fn merge_value(existing: &mut Value, incoming: Value) {
    match (existing, incoming) {
        (Value::Object(existing), Value::Object(incoming)) => {
            for (key, value) in incoming {
                match existing.get_mut(&key) {
                    Some(existing_value) => merge_value(existing_value, value),
                    None => {
                        existing.insert(key, value);
                    }
                }
            }
        }
        (existing, incoming) => {
            *existing = incoming;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn seeded_heater_bed_partial_update_preserves_temperature() {
        let mut cache = StatusCache::new();
        cache.update(
            "heater_bed",
            json!({
                "power": 0.2,
                "target": 60.0,
                "temperature": 58.5
            }),
        );

        let updated = cache.update("heater_bed", json!({ "power": 0.3 }));
        let stats: klipper_types::HeaterBedStats =
            serde_json::from_value(updated.clone()).expect("cached heater_bed should deserialize");

        assert_eq!(stats.power, 0.3);
        assert_eq!(stats.target, 60.0);
        assert_eq!(stats.temperature, 58.5);
    }

    #[test]
    fn seeded_extruder_partial_update_preserves_temperature() {
        let mut cache = StatusCache::new();
        cache.update(
            "extruder",
            json!({
                "can_extrude": true,
                "power": 0.4,
                "pressure_advance": 0.05,
                "smooth_time": 0.04,
                "target": 215.0,
                "temperature": 214.2
            }),
        );

        let updated = cache.update("extruder", json!({ "target": 220.0 }));
        let stats: klipper_types::ExtruderStats =
            serde_json::from_value(updated.clone()).expect("cached extruder should deserialize");

        assert_eq!(stats.target, 220.0);
        assert_eq!(stats.temperature, 214.2);
        assert_eq!(stats.power, 0.4);
    }

    #[test]
    fn recursive_update_preserves_nested_sibling_fields() {
        let mut cache = StatusCache::new();
        cache.update(
            "print_stats",
            json!({
                "print_duration": 10.0,
                "info": {
                    "current_layer": 3,
                    "total_layer": 120
                }
            }),
        );

        let updated = cache.update(
            "print_stats",
            json!({
                "info": {
                    "current_layer": 4
                }
            }),
        );

        assert_eq!(updated["info"]["current_layer"], json!(4));
        assert_eq!(updated["info"]["total_layer"], json!(120));
        assert_eq!(updated["print_duration"], json!(10.0));
    }

    #[test]
    fn incoming_null_is_stored_instead_of_deleting_the_key() {
        let mut cache = StatusCache::new();
        cache.update(
            "heater_bed",
            json!({
                "power": 0.2,
                "target": 60.0,
                "temperature": 58.5
            }),
        );

        let updated = cache.update("heater_bed", json!({ "temperature": null }));

        assert!(updated.get("temperature").is_some());
        assert!(updated["temperature"].is_null());
        serde_json::from_value::<klipper_types::HeaterBedStats>(updated.clone())
            .expect_err("null numeric fields should fail instead of defaulting to zero");
    }

    #[test]
    fn cold_cache_partial_update_preserves_current_race_behavior() {
        let mut cache = StatusCache::new();

        let updated = cache.update("heater_bed", json!({ "power": 0.3 }));
        let stats: klipper_types::HeaterBedStats =
            serde_json::from_value(updated.clone()).expect("partial heater_bed should deserialize");

        assert_eq!(stats.power, 0.3);
        assert_eq!(stats.target, 0.0);
        assert_eq!(stats.temperature, 0.0);
    }

    #[test]
    fn seed_from_subscribe_response_extracts_full_status_objects() {
        let mut cache = StatusCache::new();
        let response = json!({
            "status": {
                "heater_bed": {
                    "power": 0.2,
                    "target": 60.0,
                    "temperature": 58.5
                },
                "extruder": {
                    "target": 215.0,
                    "temperature": 214.2
                }
            }
        });

        let seeded = cache
            .seed_from_subscribe_response(&response)
            .expect("subscribe response should contain status");

        assert_eq!(seeded.len(), 2);
        assert!(
            seeded
                .iter()
                .any(|(key, data)| key == "heater_bed" && data["temperature"] == json!(58.5))
        );
        assert!(
            seeded
                .iter()
                .any(|(key, data)| key == "extruder" && data["temperature"] == json!(214.2))
        );

        let updated = cache.update("heater_bed", json!({ "power": 0.3 }));
        let stats: klipper_types::HeaterBedStats =
            serde_json::from_value(updated.clone()).expect("cached heater_bed should deserialize");
        assert_eq!(stats.temperature, 58.5);
        assert_eq!(stats.power, 0.3);
    }

    #[test]
    fn seed_from_subscribe_response_overwrites_existing_entries() {
        let mut cache = StatusCache::new();
        cache.update(
            "heater_bed",
            json!({
                "power": 0.1,
                "target": 50.0,
                "temperature": 49.0
            }),
        );

        let response = json!({
            "status": {
                "heater_bed": {
                    "power": 0.0,
                    "target": 0.0,
                    "temperature": 22.0
                }
            }
        });

        cache
            .seed_from_subscribe_response(&response)
            .expect("subscribe response should contain status");
        let updated = cache.update("heater_bed", json!({ "power": 0.2 }));
        let stats: klipper_types::HeaterBedStats =
            serde_json::from_value(updated.clone()).expect("cached heater_bed should deserialize");

        assert_eq!(stats.temperature, 22.0);
        assert_eq!(stats.target, 0.0);
        assert_eq!(stats.power, 0.2);
    }

    #[test]
    fn seed_from_subscribe_response_returns_none_without_status_object() {
        let mut cache = StatusCache::new();

        assert!(cache.seed_from_subscribe_response(&json!({})).is_none());
        assert!(
            cache
                .seed_from_subscribe_response(&json!({ "status": null }))
                .is_none()
        );
        assert!(
            cache
                .seed_from_subscribe_response(&json!({ "status": [] }))
                .is_none()
        );
    }
}
