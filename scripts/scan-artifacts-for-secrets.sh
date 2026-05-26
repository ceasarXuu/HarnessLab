#!/usr/bin/env bash
set -euo pipefail

secret="${HARNESSLAB_TEST_SECRET:-HARNESSLAB_TEST_SECRET_VALUE}"
exec cargo run -p xtask -- scan-secrets --secret "$secret"
