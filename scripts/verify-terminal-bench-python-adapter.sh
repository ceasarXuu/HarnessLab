#!/usr/bin/env bash
set -euo pipefail

PYTHONPATH="$PWD/integrations/terminal_bench" \
  uvx --from terminal-bench --with pytest python -m pytest \
  "$PWD/integrations/terminal_bench" \
  -q
