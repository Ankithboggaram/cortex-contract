# cortex-contract

[![CI](https://github.com/Ankithboggaram/cortex-contract/actions/workflows/ci.yml/badge.svg)](https://github.com/Ankithboggaram/cortex-contract/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Shared contract for the **Cortex** platform: the feature payload, schema, keyspace, codec, and
online-store traits that Axon, Dendrite, and Synapse all depend on. No business logic.

One `.proto` source of truth in [`proto/`](proto/), one implementation per language:

| Language | Location             | Role                                           |
| -------- | -------------------- | ---------------------------------------------- |
| Rust     | [`rust/`](rust/)     | The native crate: traits, codec, Redis backend |
| Python   | [`python/`](python/) | Generated bindings, installable package        |

See each directory's README for install and usage instructions. `scripts/check_roundtrip.py`
proves both languages encode/decode byte-identical Protobuf.

## License

MIT
