//! The canonical, versioned feature schema (PRD.md §4.1).
//!
//! The ordered list of `[[feature]]` entries in `feature_schema.toml` **is**
//! the vector order: index N here == `FeatureRecord.features[N]`, in the
//! online store, the `features` Kafka topic, and the offline Parquet columns.
//! Dendrite writes in this order, Axon validates its model input width
//! against it, and Synapse reads these as its training column names.
//!
//! Rule (enforced by convention, not by this type): append-or-bump. Never
//! reorder or repurpose an index in place — any change to names, order, or
//! count is a new `version`.

use std::path::Path;

use serde::Deserialize;

use crate::error::SchemaError;

/// One entry in the ordered feature vector: a name and its declared dtype.
///
/// `dtype` is metadata for tooling/docs; every feature is `f32` on the wire
/// (`FeatureRecord.features` is protobuf `repeated float`) regardless of what
/// this field says.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FeatureSpec {
    /// The feature's name, e.g. `"txn_count_1m"`.
    pub name: String,
    /// The feature's declared dtype, e.g. `"f32"`.
    pub dtype: String,
}

/// The versioned, ordered feature schema loaded from a TOML file.
///
/// The `[[feature]]` array's order in the source TOML is preserved here —
/// `features[N]` is the schema's own index N, matching `FeatureRecord.features[N]`.
#[derive(Debug, Clone, PartialEq, Deserialize)]
pub struct FeatureSchema {
    /// Which feature set this is. Stamped into every `FeatureRecord.schema_version`
    /// produced under it; bump on any change to names, order, or count.
    pub version: u32,
    /// The ordered feature entries; TOML key is `[[feature]]` (singular).
    #[serde(rename = "feature")]
    pub features: Vec<FeatureSpec>,
}

impl FeatureSchema {
    /// Loads and parses a feature schema from a TOML file.
    ///
    /// # Errors
    /// [`SchemaError::Io`] if `path` cannot be read; [`SchemaError::Parse`] if
    /// the content is not valid TOML or does not match this shape.
    pub fn from_toml(path: impl AsRef<Path>) -> Result<Self, SchemaError> {
        let text = std::fs::read_to_string(path)?;
        let schema = toml::from_str(&text)?;
        Ok(schema)
    }

    /// The number of features in the vector — the required length of
    /// `FeatureRecord.features` under this schema.
    #[must_use]
    pub fn width(&self) -> usize {
        self.features.len()
    }

    /// The feature names, in canonical vector order.
    pub fn names(&self) -> impl Iterator<Item = &str> {
        self.features.iter().map(|f| f.name.as_str())
    }

    /// Validates a record's `schema_version` and vector length against this
    /// schema.
    ///
    /// Checks version before width, so a version mismatch is reported even
    /// when the vector happens to also be the wrong length — the more
    /// specific, more actionable error of the two (CORTEX.md §3.1).
    ///
    /// # Errors
    /// [`SchemaError::VersionMismatch`] or [`SchemaError::WidthMismatch`].
    pub fn validate(&self, record_version: u32, vector_len: usize) -> Result<(), SchemaError> {
        if record_version != self.version {
            return Err(SchemaError::VersionMismatch {
                expected: self.version,
                got: record_version,
            });
        }
        if vector_len != self.width() {
            return Err(SchemaError::WidthMismatch {
                expected: self.width(),
                got: vector_len,
            });
        }
        Ok(())
    }
}
