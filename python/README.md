# cortex-contract (Python)

[![CI](https://github.com/Ankithboggaram/cortex-contract/actions/workflows/ci.yml/badge.svg)](https://github.com/Ankithboggaram/cortex-contract/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](../LICENSE)
[![Python](https://img.shields.io/badge/python-3.11+-blue.svg)](pyproject.toml)

Generated Python bindings for the feature and prediction record contract shared
across the Cortex platform. The message definitions here (field names, types,
and field numbers) match the Rust crate exactly, since those are what determine
the wire format both languages have to agree on.

## Install

```toml
[tool.poetry.dependencies]
cortex-contract = { git = "https://github.com/Ankithboggaram/cortex-contract.git", tag = "v0.1.2", subdirectory = "python" }
```

or with plain pip:

```bash
pip install "git+https://github.com/Ankithboggaram/cortex-contract.git@v0.1.2#subdirectory=python"
```

## Usage

```python
from cortex_contract import FeatureRecord, PredictionRecord

record = FeatureRecord(schema_version=1, event_time_ms=1719000000123, features=[0.1, 1.5, -2.0, 42.0])
data = record.SerializeToString()
```

## Regenerating

The two `*_pb2.py` files in `cortex_contract/` are committed, not built at
install time, so installing this package needs nothing beyond `protobuf`
itself. Regenerate them after any change to `../proto/cortex/contract/v1/*.proto`
with `../scripts/gen_python_package.sh`, then bump the version in
`pyproject.toml` before tagging a new release.
