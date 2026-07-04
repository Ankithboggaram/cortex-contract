//! Typed error enums shared across service boundaries (PRD.md §4.5).
//!
//! All enums are `#[non_exhaustive]`: always include a wildcard arm when
//! matching to remain compatible with future variants.
//!
//! Each enum corresponds to one subsystem:
//! - [`SchemaError`] - feature schema loading and validation
//! - [`CodecError`]  - protobuf encode/decode (see [`crate::codec`])

use thiserror::Error;

/// Errors from encoding or decoding a message via [`crate::codec`].
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum CodecError {
    /// `bytes` was not a valid protobuf encoding of the target message.
    #[error("failed to decode protobuf message: {0}")]
    Decode(#[from] prost::DecodeError),
}

/// Errors from loading or validating a [`FeatureSchema`](crate::schema::FeatureSchema).
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum SchemaError {
    /// The schema file could not be read from disk.
    #[error("failed to read schema file: {0}")]
    Io(#[from] std::io::Error),

    /// The schema file content could not be parsed as valid TOML.
    #[error("failed to parse schema: {0}")]
    Parse(#[from] toml::de::Error),

    /// A `FeatureRecord`'s `schema_version` does not match this schema's
    /// `version` — the train/serve skew guard (CORTEX.md §3.1).
    #[error("schema version mismatch: expected {expected}, got {got}")]
    VersionMismatch {
        /// The schema's own `version`.
        expected: u32,
        /// The `schema_version` carried by the record being checked.
        got: u32,
    },

    /// A feature vector's length does not match [`FeatureSchema::width`](crate::schema::FeatureSchema::width).
    #[error("feature vector width mismatch: expected {expected}, got {got}")]
    WidthMismatch {
        /// The schema's width (number of `[[feature]]` entries).
        expected: usize,
        /// The length of the vector being checked.
        got: usize,
    },
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn version_mismatch_includes_both_versions() {
        let e = SchemaError::VersionMismatch {
            expected: 1,
            got: 2,
        };
        let msg = format!("{e}");
        assert!(msg.contains('1'));
        assert!(msg.contains('2'));
    }

    #[test]
    fn width_mismatch_includes_both_widths() {
        let e = SchemaError::WidthMismatch {
            expected: 8,
            got: 6,
        };
        let msg = format!("{e}");
        assert!(msg.contains('8'));
        assert!(msg.contains('6'));
    }

    #[test]
    fn io_error_is_source_chained() {
        let io = std::io::Error::new(std::io::ErrorKind::NotFound, "no such file");
        let e = SchemaError::Io(io);
        assert!(std::error::Error::source(&e).is_some());
    }

    #[test]
    fn codec_decode_error_is_source_chained() {
        use prost::Message as _;

        let garbage = [0xFFu8; 10];
        let decode_err = crate::record::FeatureRecord::decode(garbage.as_slice()).unwrap_err();
        let e = CodecError::Decode(decode_err);
        assert!(std::error::Error::source(&e).is_some());
    }
}
