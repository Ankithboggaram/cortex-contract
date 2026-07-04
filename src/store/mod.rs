//! Paired online-store traits: `OnlineStoreWriter` (Dendrite-sink) and
//! `OnlineStoreReader` (Axon), plus the `OnlineBackend` config enum and
//! feature-gated backend implementations (PRD.md §4.4).
//!
//! Paired by design: writer and reader cannot target different backends or
//! encodings, because both halves of a given backend are implemented in one
//! place and share this crate's `keys` and `codec` conventions. A backend
//! without pub/sub implements `notify` as a no-op; a reader without pub/sub
//! support falls back to `OnlineStoreReader::updates`'s default polling
//! implementation.

use std::pin::Pin;
use std::time::Duration;

use async_trait::async_trait;
use futures_util::{Stream, StreamExt as _};
use tokio_stream::wrappers::IntervalStream;

use crate::error::StoreError;
use crate::record::FeatureRecord;

#[cfg(feature = "redis")]
pub mod redis;

/// The non-feature metadata of a stored [`FeatureRecord`], filled in by
/// [`OnlineStoreReader::fetch`] alongside the feature vector itself: the
/// header Axon needs for the freshness and schema-version checks
/// (CORTEX.md §3.1) without decoding a whole `FeatureRecord` at the call site.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct RecordHeader {
    /// The `schema_version` the stored record was written under.
    pub schema_version: u32,
    /// The stored record's source event time, epoch milliseconds.
    pub event_time_ms: i64,
}

/// Outcome of a fetch: whether an entry existed for the entity.
#[non_exhaustive]
#[derive(Debug)]
pub enum FetchResult {
    /// A record was found; `header` and `dest` were fully overwritten.
    Hit,
    /// No entry exists for the entity; `header` and `dest` are unchanged.
    Miss,
}

/// A stream that yields `()` each time fresh features may be available for
/// an entity: either pushed by the backend (e.g. Redis pub/sub) or, via the
/// default polling fallback, on a fixed timer.
pub type UpdateStream = Pin<Box<dyn Stream<Item = ()> + Send>>;

/// Writes computed features to the online store (Dendrite-sink's role).
#[async_trait]
pub trait OnlineStoreWriter: Send + Sync {
    /// Writes `rec` for `entity_id`, overwriting any existing value.
    async fn write(&self, entity_id: &str, rec: &FeatureRecord) -> Result<(), StoreError>;

    /// Notifies subscribers that fresh features are available for `entity_id`.
    ///
    /// Backends without pub/sub support implement this as a no-op; readers
    /// then rely on [`OnlineStoreReader::updates`]'s default polling.
    async fn notify(&self, entity_id: &str) -> Result<(), StoreError>;

    /// Checks that the store is reachable. Must not modify any store data.
    async fn ping(&self) -> Result<(), StoreError>;
}

/// Reads computed features from the online store (Axon's role).
#[async_trait]
pub trait OnlineStoreReader: Send + Sync {
    /// Fetches the current record for `entity_id`, writing its header into
    /// `header` and its feature vector into `dest`.
    ///
    /// # Implementors
    /// On [`FetchResult::Hit`], `header` and `dest` must be fully overwritten;
    /// `dest`'s length must match the stored vector's length exactly (return
    /// [`StoreError::ShapeMismatch`] otherwise). On [`FetchResult::Miss`],
    /// neither is modified. Should decode via `crate::codec::decode_into` (or
    /// an equivalent reused-buffer decode) so the fetch path stays
    /// allocation-free after warmup (PRD.md §6).
    async fn fetch(
        &self,
        entity_id: &str,
        header: &mut RecordHeader,
        dest: &mut [f32],
    ) -> Result<FetchResult, StoreError>;

    /// Returns a stream that yields `()` each time new features may be
    /// available for `entity_id`.
    ///
    /// The default implementation polls on `poll_interval`: correct for any
    /// backend, but wasteful for one with push notifications. A backend with
    /// pub/sub (e.g. Redis) should override this to yield only on an actual
    /// update.
    async fn updates(&self, _entity_id: &str, poll_interval: Duration) -> UpdateStream {
        Box::pin(IntervalStream::new(tokio::time::interval(poll_interval)).map(|_| ()))
    }

    /// Checks that the store is reachable. Must not modify any store data.
    async fn ping(&self) -> Result<(), StoreError>;
}

/// Selects an online-store backend and its connection config.
///
/// Deserializes from the `[store]` / `[online_store]` config section Axon and
/// Dendrite each own; this enum is the one place that config's shape is
/// defined, so both sides agree on it (PRD.md §4.4, Axon PRD §A5).
#[non_exhaustive]
#[derive(Debug, Clone, serde::Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum OnlineBackend {
    /// Redis (or a Redis-compatible store: KeyDB, Dragonfly).
    Redis {
        /// Connection URL, e.g. `redis://localhost:6379`.
        url: String,
        /// Key prefix; defaults to `crate::keys::DEFAULT_KEY_PREFIX`.
        #[serde(default = "default_key_prefix")]
        key_prefix: String,
    },
}

fn default_key_prefix() -> String {
    crate::keys::DEFAULT_KEY_PREFIX.to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn online_backend_deserializes_from_toml_with_default_prefix() {
        let backend: OnlineBackend = toml::from_str(
            r#"
            type = "redis"
            url = "redis://localhost:6379"
            "#,
        )
        .expect("deserialize");

        match backend {
            OnlineBackend::Redis { url, key_prefix } => {
                assert_eq!(url, "redis://localhost:6379");
                assert_eq!(key_prefix, "features");
            }
        }
    }

    #[test]
    fn online_backend_deserializes_with_explicit_prefix() {
        let backend: OnlineBackend = toml::from_str(
            r#"
            type = "redis"
            url = "redis://localhost:6379"
            key_prefix = "custom"
            "#,
        )
        .expect("deserialize");

        match backend {
            OnlineBackend::Redis { key_prefix, .. } => assert_eq!(key_prefix, "custom"),
        }
    }

    #[test]
    fn record_header_default_is_zeroed() {
        let header = RecordHeader::default();
        assert_eq!(header.schema_version, 0);
        assert_eq!(header.event_time_ms, 0);
    }

    /// A minimal in-memory reader exercising only the trait's default
    /// `updates` polling fallback: no backend, no network.
    struct PollingOnlyReader;

    #[async_trait]
    impl OnlineStoreReader for PollingOnlyReader {
        async fn fetch(
            &self,
            _entity_id: &str,
            _header: &mut RecordHeader,
            _dest: &mut [f32],
        ) -> Result<FetchResult, StoreError> {
            Ok(FetchResult::Miss)
        }

        async fn ping(&self) -> Result<(), StoreError> {
            Ok(())
        }
    }

    #[tokio::test(start_paused = true)]
    async fn default_updates_polls_at_the_given_interval() {
        let reader = PollingOnlyReader;
        let mut stream = reader.updates("e1", Duration::from_millis(100)).await;

        for _ in 0..3 {
            let tick = tokio::time::timeout(Duration::from_secs(1), stream.next()).await;
            assert!(tick.is_ok(), "expected a tick within the timeout");
            assert!(tick.unwrap().is_some(), "stream ended unexpectedly");
        }
    }

    /// Both traits must be dyn-compatible: Axon holds `Arc<dyn OnlineStoreReader>`,
    /// Dendrite-sink holds `Arc<dyn OnlineStoreWriter>` (PRD.md §4.4).
    #[test]
    fn traits_are_dyn_compatible() {
        fn assert_reader_object_safe(_: &dyn OnlineStoreReader) {}
        fn assert_writer_object_safe(_: &dyn OnlineStoreWriter) {}
        let _ = assert_reader_object_safe;
        let _ = assert_writer_object_safe;
    }
}
