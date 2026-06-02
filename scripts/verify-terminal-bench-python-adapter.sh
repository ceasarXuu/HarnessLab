#!/usr/bin/env bash
set -euo pipefail

PYTHONPATH="$PWD/integrations/terminal_bench" \
  uvx --from terminal-bench python -m unittest \
  "$PWD/integrations/terminal_bench/harnesslab_tb_agent_test.py" \
  "$PWD/integrations/terminal_bench/harnesslab_tb_agent_extract_test.py" \
  "$PWD/integrations/terminal_bench/harnesslab_tb_process_test.py"
