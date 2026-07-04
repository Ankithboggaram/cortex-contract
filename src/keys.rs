//! The online-store keyspace and freshness channel (PRD.md §4.2).
//!
//! The only place these string formats exist in the platform. Dendrite
//! (writer) and Axon (reader) both call through here instead of hardcoding
//! `{prefix}:{entity_id}` literals, which is what caused the historical
//! `features:updates:` vs `axon:updates:` drift this module resolves.

/// The default key prefix used when a deployment doesn't configure its own.
pub const DEFAULT_KEY_PREFIX: &str = "features";

/// The online-store key holding an entity's current `FeatureRecord`.
///
/// `feature_key("features", "e1") == "features:e1"`.
#[must_use]
pub fn feature_key(prefix: &str, entity_id: &str) -> String {
    format!("{prefix}:{entity_id}")
}

/// The pub/sub channel a writer publishes to after writing fresh features for
/// an entity, and a reader subscribes to for push-based freshness updates.
///
/// `update_channel("features", "e1") == "features:updates:e1"`.
#[must_use]
pub fn update_channel(prefix: &str, entity_id: &str) -> String {
    format!("{prefix}:updates:{entity_id}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feature_key_formats_prefix_and_entity() {
        assert_eq!(feature_key("features", "e1"), "features:e1");
    }

    #[test]
    fn update_channel_formats_prefix_and_entity() {
        assert_eq!(update_channel("features", "e1"), "features:updates:e1");
    }

    #[test]
    fn default_key_prefix_is_features() {
        assert_eq!(DEFAULT_KEY_PREFIX, "features");
    }

    #[test]
    fn feature_key_and_update_channel_use_default_prefix_consistently() {
        let key = feature_key(DEFAULT_KEY_PREFIX, "e1");
        let channel = update_channel(DEFAULT_KEY_PREFIX, "e1");
        assert_eq!(key, "features:e1");
        assert_eq!(channel, "features:updates:e1");
    }
}
