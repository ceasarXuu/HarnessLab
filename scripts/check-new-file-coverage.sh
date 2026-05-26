#!/usr/bin/env bash
set -euo pipefail

exec cargo run -p xtask -- check-new-file-coverage
