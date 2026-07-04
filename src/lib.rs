//! `cortex-contract` — the shared contract for the Cortex real-time ML platform.
//!
//! Defines the canonical feature payload, the versioned feature schema, the
//! online-store keyspace, the serialization codec, and the online-store
//! traits that Axon (serving), Dendrite (features), and Synapse (training)
//! all depend on. No business logic — pure contract.

pub mod codec;
pub mod error;
pub mod keys;
pub mod record;
pub mod schema;

pub use error::{CodecError, SchemaError};
pub use record::{FeatureRecord, PredictionRecord};
pub use schema::{FeatureSchema, FeatureSpec};
