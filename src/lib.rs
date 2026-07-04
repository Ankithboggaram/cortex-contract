//! `cortex-contract` — the shared contract for the Cortex real-time ML platform.
//!
//! Defines the canonical feature payload, the versioned feature schema, the
//! online-store keyspace, the serialization codec, and the online-store
//! traits that Axon (serving), Dendrite (features), and Synapse (training)
//! all depend on. No business logic — pure contract.

pub mod record;

pub use record::{FeatureRecord, PredictionRecord};
