#!/usr/bin/env bash
set -euo pipefail

cargo run -p xtask -- verify-test-registry
exec cargo run -p xtask -- generate-test-traceability
