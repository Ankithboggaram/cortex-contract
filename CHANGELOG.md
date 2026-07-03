# Changelog

All notable changes to this project will be documented in this file.

The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/).
This project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

> The crate version tracks the API and the Protobuf wire format. The _feature-set_
> version is separate: it lives in `feature_schema.toml` (`version`) and in every
> `FeatureRecord.schema_version`. The two are intentionally independent.

## [Unreleased]

### Added

- Repository tooling and conventions mirroring Axon: CI (`fmt`, `deny`, `clippy`,
  `test` on stable + beta, `test-msrv`, `docs`), `deny.toml`, `rustfmt.toml`,
  `rust-toolchain.toml`, MIT `LICENSE`, and `proto/buf.yaml`.
- Python virtual environment (`.venv`) with the `protobuf` runtime for the
  cross-language proof (Rust ↔ Python round-trip of the wire format).
