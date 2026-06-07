#!/usr/bin/env bash
set -euo pipefail

cargo run -p xtask -- verify-test-registry
cargo run -p xtask -- verify-frozen-selector-manifest
exec cargo run -p xtask -- generate-test-traceability
