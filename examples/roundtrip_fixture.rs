//! Fixture binary for the Rust <-> Python cross-language round-trip proof
//! (see `scripts/check_roundtrip.py`). Not part of the library API.
//!
//! `encode <path>` writes the canonical `FeatureRecord` and `PredictionRecord`
//! (see `canonical_*` below) to `<path>` as two length-prefixed protobuf
//! messages: a little-endian `u32` byte length followed by that many bytes,
//! `FeatureRecord` then `PredictionRecord`.
//!
//! `decode <path>` reads that same framing and prints one deterministic,
//! tab-separated line per message. Floats print as their 8-hex-digit IEEE-754
//! bit pattern (not decimal) so the comparison is bit-exact and immune to
//! decimal-formatting differences between Rust's and Python's float printers.

use std::env;
use std::fs;
use std::io::Write as _;

use cortex_contract::{FeatureRecord, PredictionRecord};
use prost::Message;

fn canonical_feature_record() -> FeatureRecord {
    FeatureRecord {
        schema_version: 1,
        event_time_ms: 1_719_000_000_123,
        features: vec![0.1, 1.5, -2.0, 42.0],
    }
}

fn canonical_prediction_record() -> PredictionRecord {
    PredictionRecord {
        entity_id: "entity_0001".to_owned(),
        model_name: "fraud_demo".to_owned(),
        model_version: "3".to_owned(),
        schema_version: 1,
        event_time_ms: 1_719_000_000_123,
        predict_time_ms: 1_719_000_000_456,
        features: vec![0.1, 1.5, -2.0, 42.0],
        output: vec![0.87],
        request_id: "req-abc-123".to_owned(),
    }
}

/// Renders each value as its 8-hex-digit IEEE-754 bit pattern, comma-joined.
fn csv(values: &[f32]) -> String {
    values
        .iter()
        .map(|v| format!("{:08x}", v.to_bits()))
        .collect::<Vec<_>>()
        .join(",")
}

fn write_framed(buf: &mut Vec<u8>, msg: &impl Message) {
    let encoded = msg.encode_to_vec();
    buf.extend_from_slice(&(encoded.len() as u32).to_le_bytes());
    buf.extend_from_slice(&encoded);
}

fn read_framed(bytes: &[u8], offset: &mut usize) -> Vec<u8> {
    let len = u32::from_le_bytes(
        bytes[*offset..*offset + 4]
            .try_into()
            .expect("length prefix"),
    );
    *offset += 4;
    let msg = bytes[*offset..*offset + len as usize].to_vec();
    *offset += len as usize;
    msg
}

fn encode(path: &str) {
    let mut buf = Vec::new();
    write_framed(&mut buf, &canonical_feature_record());
    write_framed(&mut buf, &canonical_prediction_record());
    fs::write(path, buf).expect("write fixture file");
}

fn decode(path: &str) {
    let bytes = fs::read(path).expect("read fixture file");
    let mut offset = 0;

    let fr_bytes = read_framed(&bytes, &mut offset);
    let fr = FeatureRecord::decode(fr_bytes.as_slice()).expect("decode FeatureRecord");

    let pr_bytes = read_framed(&bytes, &mut offset);
    let pr = PredictionRecord::decode(pr_bytes.as_slice()).expect("decode PredictionRecord");

    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    writeln!(
        out,
        "FR\t{}\t{}\t{}",
        fr.schema_version,
        fr.event_time_ms,
        csv(&fr.features)
    )
    .unwrap();
    writeln!(
        out,
        "PR\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}",
        pr.entity_id,
        pr.model_name,
        pr.model_version,
        pr.schema_version,
        pr.event_time_ms,
        pr.predict_time_ms,
        csv(&pr.features),
        csv(&pr.output),
        pr.request_id,
    )
    .unwrap();
}

fn main() {
    let args: Vec<String> = env::args().collect();
    let (mode, path) = match (args.get(1), args.get(2)) {
        (Some(mode), Some(path)) => (mode.as_str(), path.as_str()),
        _ => {
            eprintln!("usage: roundtrip_fixture <encode|decode> <path>");
            std::process::exit(2);
        }
    };

    match mode {
        "encode" => encode(path),
        "decode" => decode(path),
        other => {
            eprintln!("unknown mode: {other} (expected encode|decode)");
            std::process::exit(2);
        }
    }
}
