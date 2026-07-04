//! Protobuf encode/decode: the platform's single serialization chokepoint
//! (PRD.md §4.3). Every feature payload crosses this module exactly once, on
//! both Axon's read hot path and Dendrite's write hot path, so its two hot
//! functions ([`encode_into`], [`decode_into`]) are allocation-free after warmup.

use prost::Message as _;

use crate::error::CodecError;
use crate::record::FeatureRecord;

/// Encodes `rec` into a new buffer.
///
/// Convenience for cold paths; hot paths use [`encode_into`] to reuse a buffer
/// instead of allocating one per call.
#[must_use]
pub fn encode(rec: &FeatureRecord) -> Vec<u8> {
    rec.encode_to_vec()
}

/// Encodes `rec` into `buf`, reusing its allocation.
///
/// `buf` is cleared first (capacity retained); once `buf` has grown to fit
/// the largest record ever encoded into it, further calls allocate nothing.
pub fn encode_into(rec: &FeatureRecord, buf: &mut Vec<u8>) {
    buf.clear();
    buf.reserve(rec.encoded_len());
    #[allow(clippy::expect_used)]
    rec.encode(buf)
        .expect("buf reserved to encoded_len() has sufficient capacity");
}

/// Decodes a new [`FeatureRecord`] from `bytes`.
///
/// Convenience for cold paths; hot paths use [`decode_into`] to reuse a
/// `FeatureRecord` instead of allocating one per call.
///
/// # Errors
/// [`CodecError::Decode`] if `bytes` is not a valid encoding of `FeatureRecord`.
pub fn decode(bytes: &[u8]) -> Result<FeatureRecord, CodecError> {
    Ok(FeatureRecord::decode(bytes)?)
}

/// Decodes `bytes` into `rec` in place: the hot-path decode.
///
/// Clears `rec` (retaining its `features` `Vec`'s capacity) and merges
/// `bytes` into it. Once `rec` has been warmed up by a decode at least as
/// wide as every subsequent one, this allocates nothing: the guarantee that
/// keeps Axon's read path allocation-free (PRD.md §6).
///
/// # Errors
/// [`CodecError::Decode`] if `bytes` is not a valid encoding of `FeatureRecord`.
/// On error, `rec`'s contents are unspecified (partially merged).
pub fn decode_into(bytes: &[u8], rec: &mut FeatureRecord) -> Result<(), CodecError> {
    rec.clear();
    rec.merge(bytes)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> FeatureRecord {
        FeatureRecord {
            schema_version: 1,
            event_time_ms: 1_719_000_000_123,
            features: vec![0.1, 1.5, -2.0, 42.0],
        }
    }

    #[test]
    fn encode_decode_round_trips() {
        let rec = sample();
        let bytes = encode(&rec);
        let decoded = decode(&bytes).expect("decode");
        assert_eq!(decoded, rec);
    }

    #[test]
    fn encode_into_decode_into_round_trips() {
        let rec = sample();
        let mut buf = Vec::new();
        encode_into(&rec, &mut buf);

        let mut decoded = FeatureRecord::default();
        decode_into(&buf, &mut decoded).expect("decode_into");
        assert_eq!(decoded, rec);
    }

    #[test]
    fn decode_into_clears_stale_contents_before_merge() {
        let mut rec = FeatureRecord {
            schema_version: 99,
            event_time_ms: 999,
            features: vec![1.0, 2.0, 3.0, 4.0, 5.0],
        };
        let fresh = sample();
        let bytes = encode(&fresh);

        decode_into(&bytes, &mut rec).expect("decode_into");
        assert_eq!(rec, fresh);
    }

    /// The zero-alloc guarantee (PRD.md §6): once `rec` is warmed up, decoding
    /// a same-or-narrower record must not grow capacity or move the allocation.
    #[test]
    fn decode_into_reuses_capacity_after_warmup() {
        let rec = sample();
        let mut buf = Vec::new();
        encode_into(&rec, &mut buf);

        let mut warm = FeatureRecord::default();
        decode_into(&buf, &mut warm).expect("warmup decode");
        let warmed_capacity = warm.features.capacity();
        let warmed_ptr = warm.features.as_ptr();
        assert!(warmed_capacity >= rec.features.len());

        decode_into(&buf, &mut warm).expect("second decode");

        assert_eq!(warm.features.capacity(), warmed_capacity);
        assert_eq!(warm.features.as_ptr(), warmed_ptr);
        assert_eq!(warm, rec);
    }

    #[test]
    fn encode_into_reuses_buffer_capacity_after_warmup() {
        let rec = sample();
        let mut buf = Vec::with_capacity(0);

        encode_into(&rec, &mut buf);
        let warmed_capacity = buf.capacity();
        let warmed_ptr = buf.as_ptr();

        encode_into(&rec, &mut buf);

        assert_eq!(buf.capacity(), warmed_capacity);
        assert_eq!(buf.as_ptr(), warmed_ptr);
    }

    #[test]
    fn decode_rejects_garbage_bytes() {
        let garbage = [0xFFu8; 10];
        assert!(decode(&garbage).is_err());
    }

    #[test]
    fn decode_into_rejects_garbage_bytes() {
        let garbage = [0xFFu8; 10];
        let mut rec = FeatureRecord::default();
        assert!(decode_into(&garbage, &mut rec).is_err());
    }
}
