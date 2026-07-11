#!/usr/bin/env bash
# Regenerates python/cortex_contract/*_pb2.{py,pyi} from proto/cortex/contract/v1/*.proto.
#
# Unlike scripts/check_roundtrip.py's throwaway codegen, the output here is
# committed: it is the actual content of the distributed Python package, so
# installing it needs nothing beyond the protobuf runtime, no protoc, no this
# script. Run this after any change to the proto files, then bump the version
# in python/pyproject.toml before tagging a release.
set -euo pipefail

cd "$(dirname "${BASH_SOURCE[0]}")/.."

OUT_DIR="python/cortex_contract"

# proto_path points directly at the v1 dir, not the "proto/" root, so protoc
# emits flat modules (feature_record_pb2.py) instead of mirroring the proto
# package's directory nesting into this package.
#
# --pyi_out generates a .pyi stub alongside each _pb2.py: the generated .py
# builds its message classes dynamically (via the descriptor pool), which
# static type checkers can't see through on their own, so without the stub
# mypy reports FeatureRecord/PredictionRecord as missing attributes.
python3 -m grpc_tools.protoc \
  --proto_path=proto/cortex/contract/v1 \
  --python_out="$OUT_DIR" \
  --pyi_out="$OUT_DIR" \
  feature_record.proto \
  prediction_record.proto

echo "generated:"
ls "$OUT_DIR"/*_pb2.py "$OUT_DIR"/*_pb2.pyi
