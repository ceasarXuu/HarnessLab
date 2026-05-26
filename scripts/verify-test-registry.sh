#!/usr/bin/env bash
set -euo pipefail

exec cargo run -p xtask -- verify-test-registry
