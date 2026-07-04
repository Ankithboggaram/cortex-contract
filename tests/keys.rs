//! The implementation checklist's own acceptance test for `keys` (README §Implementation checklist).

use cortex_contract::keys::{DEFAULT_KEY_PREFIX, feature_key, update_channel};

#[test]
fn matches_the_documented_examples() {
    assert_eq!(feature_key("features", "e1"), "features:e1");
    assert_eq!(update_channel("features", "e1"), "features:updates:e1");
}

#[test]
fn default_prefix_round_trips_through_both_functions() {
    assert_eq!(feature_key(DEFAULT_KEY_PREFIX, "e1"), "features:e1");
    assert_eq!(
        update_channel(DEFAULT_KEY_PREFIX, "e1"),
        "features:updates:e1"
    );
}
