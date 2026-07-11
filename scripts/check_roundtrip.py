#!/usr/bin/env python3
"""Proves Rust and the committed Python package decode/encode byte-identical Protobuf.

This is the "Rust <-> Python round-trip test" the implementation checklist
calls for: cortex-contract's payload is Protobuf specifically so that Synapse
(Python) and Axon/Dendrite (Rust) never drift on the wire format, in
particular on `float` (proto `float` is unambiguously 32-bit; naive Python
`msgpack` is not). This script imports the actual distributed package
(python/cortex_contract), not a fresh throwaway regeneration, so it also
catches the package being left stale after a proto change. It proves both
directions:

  1. Rust encodes the canonical records -> Python decodes them and checks
     every field, comparing floats by their 32-bit bit pattern (not decimal
     text, since a Python `float` widens to 64-bit on read).
  2. Python encodes the same canonical records -> Rust decodes them (via the
     `roundtrip_fixture` example) and prints a fixed text format; this script
     asserts that text matches exactly.

Requirements:
    cargo on PATH
    protobuf runtime: pip install -r requirements-dev.txt (or use .venv)

Run from the repository root:
    .venv/bin/python scripts/check_roundtrip.py
"""

import shutil
import struct
import subprocess
import sys
import tempfile
from pathlib import Path
from typing import Any

REPO_ROOT = Path(__file__).resolve().parent.parent
sys.path.insert(0, str(REPO_ROOT / "python"))

from cortex_contract import FeatureRecord, PredictionRecord  # noqa: E402

FEATURE_RECORD: dict[str, Any] = {
    "schema_version": 1,
    "event_time_ms": 1_719_000_000_123,
    "features": [0.1, 1.5, -2.0, 42.0],
}

PREDICTION_RECORD: dict[str, Any] = {
    "entity_id": "entity_0001",
    "model_name": "fraud_demo",
    "model_version": "3",
    "schema_version": 1,
    "event_time_ms": 1_719_000_000_123,
    "predict_time_ms": 1_719_000_000_456,
    "features": [0.1, 1.5, -2.0, 42.0],
    "output": [0.87],
    "request_id": "req-abc-123",
}


def f32_hex(value: float) -> str:
    """The 8-hex-digit IEEE-754 bit pattern of `value` narrowed to f32:
    matches the format `roundtrip_fixture`'s decode mode prints."""
    (bits,) = struct.unpack("<I", struct.pack("<f", value))
    return format(bits, "08x")


def csv_hex(values: list[float]) -> str:
    return ",".join(f32_hex(v) for v in values)


def read_framed(data: bytes, offset: int) -> tuple[bytes, int]:
    (length,) = struct.unpack_from("<I", data, offset)
    offset += 4
    return data[offset : offset + length], offset + length


def write_framed(buf: bytearray, message_bytes: bytes) -> None:
    buf.extend(struct.pack("<I", len(message_bytes)))
    buf.extend(message_bytes)


def run(cmd: list[str], **kwargs) -> subprocess.CompletedProcess:
    # All calls here are cargo invocations; run them from rust/, the crate root.
    result = subprocess.run(
        cmd, cwd=REPO_ROOT / "rust", capture_output=True, text=True, **kwargs
    )
    if result.returncode != 0:
        raise RuntimeError(
            f"command failed: {' '.join(cmd)}\nstdout:\n{result.stdout}\nstderr:\n{result.stderr}"
        )
    return result


def check_tool(name: str) -> None:
    if shutil.which(name) is None:
        sys.exit(f"error: '{name}' not found on PATH")


def rust_to_python(fixture_path: Path) -> None:
    """Direction 1: Rust encodes -> Python decodes and checks every field."""
    run(
        [
            "cargo",
            "run",
            "--quiet",
            "--example",
            "roundtrip_fixture",
            "--",
            "encode",
            str(fixture_path),
        ]
    )

    data = fixture_path.read_bytes()
    fr_bytes, offset = read_framed(data, 0)
    pr_bytes, offset = read_framed(data, offset)

    fr = FeatureRecord()
    fr.ParseFromString(fr_bytes)
    assert fr.schema_version == FEATURE_RECORD["schema_version"]
    assert fr.event_time_ms == FEATURE_RECORD["event_time_ms"]
    assert [f32_hex(v) for v in fr.features] == [
        f32_hex(v) for v in FEATURE_RECORD["features"]
    ]

    pr = PredictionRecord()
    pr.ParseFromString(pr_bytes)
    assert pr.entity_id == PREDICTION_RECORD["entity_id"]
    assert pr.model_name == PREDICTION_RECORD["model_name"]
    assert pr.model_version == PREDICTION_RECORD["model_version"]
    assert pr.schema_version == PREDICTION_RECORD["schema_version"]
    assert pr.event_time_ms == PREDICTION_RECORD["event_time_ms"]
    assert pr.predict_time_ms == PREDICTION_RECORD["predict_time_ms"]
    assert [f32_hex(v) for v in pr.features] == [
        f32_hex(v) for v in PREDICTION_RECORD["features"]
    ]
    assert [f32_hex(v) for v in pr.output] == [
        f32_hex(v) for v in PREDICTION_RECORD["output"]
    ]
    assert pr.request_id == PREDICTION_RECORD["request_id"]

    print("  Rust -> Python: FeatureRecord and PredictionRecord fields match")


def python_to_rust(fixture_path: Path) -> None:
    """Direction 2: Python encodes -> Rust decodes and prints must match exactly."""
    fr = FeatureRecord(
        schema_version=FEATURE_RECORD["schema_version"],
        event_time_ms=FEATURE_RECORD["event_time_ms"],
        features=FEATURE_RECORD["features"],
    )
    pr = PredictionRecord(
        entity_id=PREDICTION_RECORD["entity_id"],
        model_name=PREDICTION_RECORD["model_name"],
        model_version=PREDICTION_RECORD["model_version"],
        schema_version=PREDICTION_RECORD["schema_version"],
        event_time_ms=PREDICTION_RECORD["event_time_ms"],
        predict_time_ms=PREDICTION_RECORD["predict_time_ms"],
        features=PREDICTION_RECORD["features"],
        output=PREDICTION_RECORD["output"],
        request_id=PREDICTION_RECORD["request_id"],
    )

    buf = bytearray()
    write_framed(buf, fr.SerializeToString())
    write_framed(buf, pr.SerializeToString())
    fixture_path.write_bytes(bytes(buf))

    result = run(
        [
            "cargo",
            "run",
            "--quiet",
            "--example",
            "roundtrip_fixture",
            "--",
            "decode",
            str(fixture_path),
        ]
    )

    expected_fr_line = "FR\t{}\t{}\t{}".format(
        FEATURE_RECORD["schema_version"],
        FEATURE_RECORD["event_time_ms"],
        csv_hex(FEATURE_RECORD["features"]),
    )
    expected_pr_line = "PR\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}".format(
        PREDICTION_RECORD["entity_id"],
        PREDICTION_RECORD["model_name"],
        PREDICTION_RECORD["model_version"],
        PREDICTION_RECORD["schema_version"],
        PREDICTION_RECORD["event_time_ms"],
        PREDICTION_RECORD["predict_time_ms"],
        csv_hex(PREDICTION_RECORD["features"]),
        csv_hex(PREDICTION_RECORD["output"]),
        PREDICTION_RECORD["request_id"],
    )

    actual_lines = result.stdout.strip("\n").split("\n")
    assert actual_lines == [expected_fr_line, expected_pr_line], (
        f"mismatch:\nexpected: {[expected_fr_line, expected_pr_line]}\nactual:   {actual_lines}"
    )

    print("  Python -> Rust: FeatureRecord and PredictionRecord text matches exactly")


def main() -> None:
    check_tool("cargo")

    with tempfile.TemporaryDirectory() as tmp:
        tmp_path = Path(tmp)
        rust_to_python(tmp_path / "rust_encoded.bin")
        python_to_rust(tmp_path / "python_encoded.bin")

    print("cross-language round-trip OK")


if __name__ == "__main__":
    main()
