# cortex-contract (Rust)

[![CI](https://github.com/Ankithboggaram/cortex-contract/actions/workflows/ci.yml/badge.svg)](https://github.com/Ankithboggaram/cortex-contract/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../LICENSE)
[![MSRV](https://img.shields.io/badge/rustc-1.85+-orange.svg)](rust-toolchain.toml)

The Rust crate: the feature payload, schema, keyspace, codec, and online-store traits that
Axon, Dendrite, and Synapse's Rust tooling depend on. No business logic.

## Install

```toml
[dependencies]
cortex-contract = { git = "https://github.com/Ankithboggaram/cortex-contract.git", rev = "<commit-sha>" }
```

`default = ["redis"]` for the store traits and `RedisOnlineStore`; `default-features = false` for
just the data types (Synapse's build).

## Modules

| Module                        | Provides                                                                |
| ----------------------------- | ----------------------------------------------------------------------- |
| `record`                      | `FeatureRecord`, `PredictionRecord`                                     |
| `schema`                      | `FeatureSchema`: loads `feature_schema.toml`, validates version + width |
| `keys`                        | `feature_key`, `update_channel`, `DEFAULT_KEY_PREFIX`                   |
| `codec`                       | `encode`/`decode`, zero-alloc `encode_into`/`decode_into`               |
| `store` (`feature = "redis"`) | `OnlineStoreReader`/`Writer` traits, `RedisOnlineStore`                 |
| `error`                       | `SchemaError`, `CodecError`, `StoreError`                               |

## Development

Run from this directory (`rust/`):

```bash
cargo test --all-features
cargo test --no-default-features
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

Requires Rust 1.85+ and `protoc` on `PATH`.
