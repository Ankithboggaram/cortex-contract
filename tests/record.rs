//! Round-trip tests for the prost-generated `FeatureRecord` / `PredictionRecord`.
//!
//! These exercise plain `prost::Message` encode/decode directly. The `codec`
//! module (PRD.md §4.3, `src/codec.rs`) wraps the same calls with the
//! zero-alloc `decode_into` API; this test only proves the generated types
//! are correct.

use cortex_contract::{FeatureRecord, PredictionRecord};
use prost::Message;

#[test]
fn feature_record_round_trips() {
    let original = FeatureRecord {
        schema_version: 1,
        event_time_ms: 1_719_000_000_123,
        features: vec![0.1, 1.5, -2.0, 42.0],
    };

    let bytes = original.encode_to_vec();
    let decoded = FeatureRecord::decode(bytes.as_slice()).expect("decode FeatureRecord");

    assert_eq!(decoded, original);
}

#[test]
fn prediction_record_round_trips() {
    let original = PredictionRecord {
        entity_id: "entity_0001".to_owned(),
        model_name: "fraud_demo".to_owned(),
        model_version: "3".to_owned(),
        schema_version: 1,
        event_time_ms: 1_719_000_000_123,
        predict_time_ms: 1_719_000_000_456,
        features: vec![0.1, 1.5, -2.0, 42.0],
        output: vec![0.87],
        request_id: "req-abc-123".to_owned(),
    };

    let bytes = original.encode_to_vec();
    let decoded = PredictionRecord::decode(bytes.as_slice()).expect("decode PredictionRecord");

    assert_eq!(decoded, original);
}

/// proto3 `float` is unambiguously 32-bit; this pins that guarantee so a future
/// prost/protoc upgrade can't silently widen it (the exact drift class the
/// crate exists to prevent; see CORTEX.md §4).
#[test]
fn features_are_f32_not_f64() {
    let rec = FeatureRecord {
        schema_version: 1,
        event_time_ms: 0,
        features: vec![0.1],
    };
    let _: &Vec<f32> = &rec.features;
}
