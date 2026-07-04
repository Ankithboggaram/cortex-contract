//! Tests for `FeatureSchema` against the real reference `feature_schema.toml`
//! (not a fixture): this file *is* the contract every other test should load.

use cortex_contract::{FeatureSchema, SchemaError};

fn reference_schema_path() -> std::path::PathBuf {
    std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("feature_schema.toml")
}

fn load_reference() -> FeatureSchema {
    FeatureSchema::from_toml(reference_schema_path()).expect("load reference schema")
}

#[test]
fn loads_reference_schema_with_expected_shape() {
    let schema = load_reference();

    assert_eq!(schema.version, 1);
    assert_eq!(schema.width(), 8);

    let names: Vec<&str> = schema.names().collect();
    assert_eq!(
        names,
        vec![
            "txn_count_1m",
            "txn_count_5m",
            "txn_count_1h",
            "amount_sum_1m",
            "amount_sum_1h",
            "amount_avg_1h",
            "amount_min_1h",
            "amount_max_1h",
        ]
    );
}

#[test]
fn validate_accepts_matching_version_and_width() {
    let schema = load_reference();
    assert!(schema.validate(1, 8).is_ok());
}

#[test]
fn validate_rejects_version_mismatch() {
    let schema = load_reference();
    let err = schema.validate(2, 8).unwrap_err();
    assert!(matches!(
        err,
        SchemaError::VersionMismatch {
            expected: 1,
            got: 2
        }
    ));
}

#[test]
fn validate_rejects_width_mismatch() {
    let schema = load_reference();
    let err = schema.validate(1, 6).unwrap_err();
    assert!(matches!(
        err,
        SchemaError::WidthMismatch {
            expected: 8,
            got: 6
        }
    ));
}

/// Version is checked first: a record wrong on both counts should report the
/// more actionable train/serve-skew error, not the width error.
#[test]
fn validate_reports_version_mismatch_before_width_mismatch() {
    let schema = load_reference();
    let err = schema.validate(2, 6).unwrap_err();
    assert!(matches!(err, SchemaError::VersionMismatch { .. }));
}

#[test]
fn from_toml_missing_file_returns_io_error() {
    let err = FeatureSchema::from_toml("/nonexistent/does/not/exist.toml").unwrap_err();
    assert!(matches!(err, SchemaError::Io(_)));
}

#[test]
fn from_toml_malformed_content_returns_parse_error() {
    let dir = std::env::temp_dir();
    let path = dir.join(format!(
        "cortex_contract_bad_schema_{}.toml",
        std::process::id()
    ));
    std::fs::write(&path, "this is not [ valid toml").expect("write temp file");

    let result = FeatureSchema::from_toml(&path);
    let _ = std::fs::remove_file(&path);

    assert!(matches!(result.unwrap_err(), SchemaError::Parse(_)));
}
