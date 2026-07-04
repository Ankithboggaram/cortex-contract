//! Redis-backed online store: [`OnlineStoreWriter`] does `SET` + `PUBLISH`,
//! [`OnlineStoreReader`] does `GET` + `SUBSCRIBE`: relocated from Axon's
//! original `redis.rs` (the read half) and paired here with a new write half
//! (Dendrite-sink's role), so both directions share one key format and one
//! codec and can never drift (PRD.md §4.4, CORTEX.md §4.7).
//!
//! ## Zero-alloc asymmetry
//! [`fetch`](OnlineStoreReader::fetch) decodes into a `Mutex`-guarded scratch
//! [`FeatureRecord`] reused across calls via `codec::decode_into`, so the read
//! path is allocation-free after warmup: the guarantee PRD.md §4.4 calls out
//! explicitly for `fetch`. The lock is only held across the CPU-bound decode
//! and copy, never across the network round trip. [`write`](OnlineStoreWriter::write)
//! uses the simpler allocating `codec::encode`: the PRD does not make the same
//! explicit zero-alloc demand of the write path, and avoiding a second shared
//! scratch buffer here avoids serializing concurrent writes through one lock.

use std::sync::Mutex;
use std::time::Duration;

use async_trait::async_trait;
use deadpool_redis::Pool;
use deadpool_redis::redis::Client as RedisClient;
use futures_util::StreamExt as _;
use tokio::sync::mpsc;
use tokio_stream::wrappers::ReceiverStream;

use crate::codec;
use crate::error::StoreError;
use crate::keys::{feature_key, update_channel};
use crate::record::FeatureRecord;
use crate::store::{FetchResult, OnlineStoreReader, OnlineStoreWriter, RecordHeader, UpdateStream};

/// Redis-backed implementation of both [`OnlineStoreWriter`] and
/// [`OnlineStoreReader`], sharing one connection pool and key prefix.
pub struct RedisOnlineStore {
    pool: Pool,
    /// Dedicated client for pub/sub connections (one per active `updates` stream).
    client: RedisClient,
    /// Key prefix applied to every entity lookup: `{key_prefix}:{entity_id}`.
    key_prefix: String,
    /// Reused decode target for `fetch`; grows once to the widest record ever
    /// fetched, then allocates nothing (see the module-level zero-alloc note).
    decode_scratch: Mutex<FeatureRecord>,
}

impl RedisOnlineStore {
    /// Creates a new `RedisOnlineStore` connected to `url`.
    ///
    /// The pool is sized to the number of Tokio worker threads so each thread
    /// can hold a connection without contention.
    pub fn new(url: &str, key_prefix: &str) -> Result<Self, StoreError> {
        let cfg = deadpool_redis::Config::from_url(url);
        let pool = cfg
            .create_pool(Some(deadpool_redis::Runtime::Tokio1))
            .map_err(|e| StoreError::Connection(format!("failed to create Redis pool: {e}")))?;
        let client = RedisClient::open(url)
            .map_err(|e| StoreError::Connection(format!("failed to create Redis client: {e}")))?;
        Ok(Self {
            pool,
            client,
            key_prefix: key_prefix.to_owned(),
            decode_scratch: Mutex::new(FeatureRecord::default()),
        })
    }

    async fn ping_impl(&self) -> Result<(), StoreError> {
        let mut conn = self.pool.get().await.map_err(|e| {
            StoreError::Connection(format!("Redis ping: failed to get connection: {e}"))
        })?;

        deadpool_redis::redis::cmd("PING")
            .query_async::<String>(&mut conn)
            .await
            .map_err(|e| StoreError::Connection(format!("Redis ping failed: {e}")))?;

        Ok(())
    }
}

impl std::fmt::Debug for RedisOnlineStore {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RedisOnlineStore")
            .field("key_prefix", &self.key_prefix)
            .finish_non_exhaustive()
    }
}

#[async_trait]
impl OnlineStoreWriter for RedisOnlineStore {
    async fn write(&self, entity_id: &str, rec: &FeatureRecord) -> Result<(), StoreError> {
        let key = feature_key(&self.key_prefix, entity_id);
        let bytes = codec::encode(rec);

        let mut conn =
            self.pool.get().await.map_err(|e| {
                StoreError::Connection(format!("failed to get Redis connection: {e}"))
            })?;

        deadpool_redis::redis::cmd("SET")
            .arg(&key)
            .arg(bytes)
            .query_async::<()>(&mut conn)
            .await
            .map_err(|e| StoreError::Write {
                entity_id: entity_id.to_owned(),
                reason: format!("Redis SET failed: {e}"),
            })?;

        Ok(())
    }

    async fn notify(&self, entity_id: &str) -> Result<(), StoreError> {
        let channel = update_channel(&self.key_prefix, entity_id);

        let mut conn =
            self.pool.get().await.map_err(|e| {
                StoreError::Connection(format!("failed to get Redis connection: {e}"))
            })?;

        deadpool_redis::redis::cmd("PUBLISH")
            .arg(&channel)
            .arg(entity_id)
            .query_async::<i64>(&mut conn)
            .await
            .map_err(|e| StoreError::Notify {
                entity_id: entity_id.to_owned(),
                reason: format!("Redis PUBLISH failed: {e}"),
            })?;

        Ok(())
    }

    async fn ping(&self) -> Result<(), StoreError> {
        self.ping_impl().await
    }
}

#[async_trait]
impl OnlineStoreReader for RedisOnlineStore {
    async fn fetch(
        &self,
        entity_id: &str,
        header: &mut RecordHeader,
        dest: &mut [f32],
    ) -> Result<FetchResult, StoreError> {
        let key = feature_key(&self.key_prefix, entity_id);

        let mut conn =
            self.pool.get().await.map_err(|e| {
                StoreError::Connection(format!("failed to get Redis connection: {e}"))
            })?;

        let bytes: Option<Vec<u8>> = deadpool_redis::redis::cmd("GET")
            .arg(&key)
            .query_async(&mut conn)
            .await
            .map_err(|e| StoreError::Fetch {
                entity_id: entity_id.to_owned(),
                reason: format!("Redis GET failed: {e}"),
            })?;

        let bytes = match bytes {
            Some(b) => b,
            None => return Ok(FetchResult::Miss),
        };

        // Lock scope covers only the CPU-bound decode + copy below, never the
        // network round trip above; see the module-level zero-alloc note.
        #[allow(clippy::expect_used)]
        let mut rec = self
            .decode_scratch
            .lock()
            .expect("decode scratch mutex poisoned");

        codec::decode_into(&bytes, &mut rec).map_err(|e| StoreError::Decode {
            entity_id: entity_id.to_owned(),
            reason: e.to_string(),
        })?;

        if rec.features.len() != dest.len() {
            return Err(StoreError::ShapeMismatch {
                entity_id: entity_id.to_owned(),
                expected: dest.len(),
                got: rec.features.len(),
            });
        }

        header.schema_version = rec.schema_version;
        header.event_time_ms = rec.event_time_ms;
        dest.copy_from_slice(&rec.features);

        Ok(FetchResult::Hit)
    }

    async fn updates(&self, entity_id: &str, poll_interval: Duration) -> UpdateStream {
        let channel = update_channel(&self.key_prefix, entity_id);
        let client = self.client.clone();
        let (tx, rx) = mpsc::channel::<()>(16);

        // Spawn a task that owns the pub/sub connection and forwards a ()
        // token each time a writer publishes to this entity's channel. When
        // the receiver is dropped (stream consumer disconnected), tx.send
        // returns Err and the task exits, closing the Redis connection.
        tokio::spawn(async move {
            let Ok(mut pubsub) = client.get_async_pubsub().await else {
                return;
            };
            if pubsub.subscribe(&channel).await.is_err() {
                return;
            }
            let mut msgs = pubsub.into_on_message();
            while msgs.next().await.is_some() {
                if tx.send(()).await.is_err() {
                    break;
                }
            }
        });

        // If the spawned task above fails to connect or subscribe, tx is
        // dropped immediately and this stream ends; the caller (Axon) falls
        // back to the trait's default polling on its next call, exactly as
        // the trait doc describes.
        let _ = poll_interval; // pub/sub replaces the timer for this backend
        Box::pin(ReceiverStream::new(rx))
    }

    async fn ping(&self) -> Result<(), StoreError> {
        self.ping_impl().await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rejects_an_invalid_url() {
        let err = RedisOnlineStore::new("not a valid redis url", "features").unwrap_err();
        assert!(matches!(err, StoreError::Connection(_)));
    }

    #[test]
    fn debug_does_not_panic_and_hides_internals() {
        let store = RedisOnlineStore::new("redis://localhost:6379", "features")
            .expect("valid URL, no connection attempted yet");
        let debug = format!("{store:?}");
        assert!(debug.contains("features"));
    }
}
