//! Typed error enums shared across service boundaries (PRD.md §4.5).
//!
//! All enums are `#[non_exhaustive]`: always include a wildcard arm when
//! matching to remain compatible with future variants.
//!
//! Each enum corresponds to one subsystem:
//! - [`SchemaError`] - feature schema loading and validation
//! - [`CodecError`]  - protobuf encode/decode (see [`crate::codec`])
//! - [`StoreError`]  - the online store (see `crate::store`, `feature = "redis"`)

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
    /// `version`, the train/serve skew guard (CORTEX.md §3.1).
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

/// Errors from an online-store backend (`crate::store`, `feature = "redis"`).
///
/// Shared by both `OnlineStoreWriter` (Dendrite-sink) and `OnlineStoreReader`
/// (Axon) so a connection or shape failure means the same thing on either side.
#[non_exhaustive]
#[derive(Debug, Error)]
pub enum StoreError {
    /// A connection or pool could not be established.
    #[error("failed to connect to the online store: {0}")]
    Connection(String),

    /// A fetch (read) failed for the given entity.
    #[error("fetch failed for entity '{entity_id}': {reason}")]
    Fetch {
        /// Entity for which the fetch failed.
        entity_id: String,
        /// Underlying error from the backend.
        reason: String,
    },

    /// A write failed for the given entity.
    #[error("write failed for entity '{entity_id}': {reason}")]
    Write {
        /// Entity for which the write failed.
        entity_id: String,
        /// Underlying error from the backend.
        reason: String,
    },

    /// A freshness notification failed for the given entity.
    #[error("notify failed for entity '{entity_id}': {reason}")]
    Notify {
        /// Entity for which the notify failed.
        entity_id: String,
        /// Underlying error from the backend.
        reason: String,
    },

    /// The bytes fetched for the given entity could not be decoded.
    #[error("failed to decode stored record for entity '{entity_id}': {reason}")]
    Decode {
        /// Entity whose stored record could not be decoded.
        entity_id: String,
        /// Decode error detail.
        reason: String,
    },

    /// The decoded feature vector's length does not match the caller's buffer.
    #[error(
        "feature vector shape mismatch for entity '{entity_id}': expected {expected}, got {got}"
    )]
    ShapeMismatch {
        /// Entity whose feature vector had the wrong length.
        entity_id: String,
        /// Length the caller's destination buffer expects.
        expected: usize,
        /// Length of the vector actually stored.
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

    #[test]
    fn store_fetch_includes_entity_id() {
        let e = StoreError::Fetch {
            entity_id: "e1".into(),
            reason: "connection reset".into(),
        };
        let msg = format!("{e}");
        assert!(msg.contains("e1"));
        assert!(msg.contains("connection reset"));
    }

    #[test]
    fn store_shape_mismatch_includes_both_lengths() {
        let e = StoreError::ShapeMismatch {
            entity_id: "e1".into(),
            expected: 8,
            got: 6,
        };
        let msg = format!("{e}");
        assert!(msg.contains('8'));
        assert!(msg.contains('6'));
    }
}
