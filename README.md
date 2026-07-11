# cortex-contract

[![CI](https://github.com/Ankithboggaram/cortex-contract/actions/workflows/ci.yml/badge.svg)](https://github.com/Ankithboggaram/cortex-contract/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![MSRV](https://img.shields.io/badge/rustc-1.85+-orange.svg)](rust-toolchain.toml)

Shared contract crate for the **Cortex** platform: the feature payload, schema, keyspace, codec,
and online-store traits that Axon, Dendrite, and Synapse all depend on. No business logic.

---

## Install

```toml
[dependencies]
cortex-contract = { git = "https://github.com/Ankithboggaram/cortex-contract.git", rev = "<commit-sha>" }
```

`default = ["redis"]` for the store traits and `RedisOnlineStore`; `default-features = false` for
just the data types (Synapse's build).

---

## Modules

| Module                        | Provides                                                                |
| ----------------------------- | ----------------------------------------------------------------------- |
| `record`                      | `FeatureRecord`, `PredictionRecord`                                     |
| `schema`                      | `FeatureSchema`: loads `feature_schema.toml`, validates version + width |
| `keys`                        | `feature_key`, `update_channel`, `DEFAULT_KEY_PREFIX`                   |
| `codec`                       | `encode`/`decode`, zero-alloc `encode_into`/`decode_into`               |
| `store` (`feature = "redis"`) | `OnlineStoreReader`/`Writer` traits, `RedisOnlineStore`                 |
| `error`                       | `SchemaError`, `CodecError`, `StoreError`                               |

Python bindings are generated from the same `.proto` files and distributed as an installable
package at `python/`; see `scripts/check_roundtrip.py` for the cross-language proof.

---

## Install (Python)

```toml
[tool.poetry.dependencies]
cortex-contract = { git = "https://github.com/Ankithboggaram/cortex-contract.git", tag = "v0.1.2", subdirectory = "python" }
```

```python
from cortex_contract import FeatureRecord, PredictionRecord
```

No `protoc` needed to consume it: the generated bindings are committed inside `python/cortex_contract/`.
See `python/README.md` for regeneration instructions if the schema changes.

---

## Development

```bash
cargo test --all-features
cargo test --no-default-features
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
```

Requires Rust 1.85+ and `protoc` on `PATH`.

---

## License

MIT
