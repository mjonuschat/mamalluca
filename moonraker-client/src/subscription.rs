//! Subscription tracking for Moonraker printer objects.
//!
//! The [`SubscriptionManager`] records which Moonraker objects (e.g. `"extruder"`,
//! `"heater_bed"`) have been subscribed to. After a reconnect, the connection
//! loop uses this to automatically re-subscribe without the consumer needing
//! to do anything.

/// Tracks which Moonraker objects the client has subscribed to.
///
/// This is an internal bookkeeping structure used by the reconnect loop.
/// When the WebSocket connection drops and is re-established, the loop
/// calls [`subscribed_objects`](SubscriptionManager::subscribed_objects)
/// to replay the subscription request.
pub(crate) struct SubscriptionManager {
    /// The list of Moonraker object names we are subscribed to.
    objects: Vec<String>,
}

impl SubscriptionManager {
    /// Creates a new manager with no tracked subscriptions.
    pub fn new() -> Self {
        Self {
            objects: Vec::new(),
        }
    }

    /// Record that these objects are now subscribed.
    ///
    /// Replaces any previously tracked subscription list with the new one.
    ///
    /// # Parameters
    /// - `objects`: The Moonraker object names to track (e.g. `["extruder", "heater_bed"]`).
    pub fn track(&mut self, objects: &[String]) {
        self.objects = objects.to_vec();
    }

    /// Get the list of currently tracked subscriptions.
    ///
    /// Returns an empty slice if no subscriptions have been tracked yet
    /// (or if [`clear`](SubscriptionManager::clear) was called).
    pub fn subscribed_objects(&self) -> &[String] {
        &self.objects
    }

    /// Clear all tracked subscriptions.
    ///
    /// Useful if the consumer wants to reset the subscription list
    /// entirely (e.g. before re-subscribing to a different set of
    /// objects). Not currently called by the reconnect loop — it
    /// intentionally preserves subscriptions across reconnects so
    /// they can be replayed automatically.
    #[allow(dead_code)]
    pub fn clear(&mut self) {
        self.objects.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Verifies that a new manager starts with no subscriptions.
    #[test]
    fn new_manager_is_empty() {
        let mgr = SubscriptionManager::new();
        assert!(mgr.subscribed_objects().is_empty());
    }

    /// Verifies that tracking objects records them correctly.
    #[test]
    fn track_records_objects() {
        let mut mgr = SubscriptionManager::new();
        let objects = vec!["extruder".to_owned(), "heater_bed".to_owned()];
        mgr.track(&objects);
        assert_eq!(mgr.subscribed_objects(), &["extruder", "heater_bed"]);
    }

    /// Verifies that tracking replaces (not appends) the previous list.
    #[test]
    fn track_replaces_previous() {
        let mut mgr = SubscriptionManager::new();
        mgr.track(&["extruder".to_owned()]);
        mgr.track(&["heater_bed".to_owned(), "toolhead".to_owned()]);
        assert_eq!(mgr.subscribed_objects(), &["heater_bed", "toolhead"]);
    }

    /// Verifies that clear removes all tracked subscriptions.
    #[test]
    fn clear_removes_all() {
        let mut mgr = SubscriptionManager::new();
        mgr.track(&["extruder".to_owned()]);
        mgr.clear();
        assert!(mgr.subscribed_objects().is_empty());
    }
}
