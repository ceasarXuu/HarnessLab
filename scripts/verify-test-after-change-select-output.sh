#!/usr/bin/env bash
set -euo pipefail

sentinel="HARNESSLAB_META_001_FAILURE_SENTINEL"

set +e
output="$(HARNESSLAB_META_001_FORCE_FAILURE=1 scripts/test-after-change.sh --select META-001-FAIL 2>&1)"
status=$?
set -e

printf '%s\n' "$output"

if [[ "$status" -eq 0 ]]; then
  echo "expected selected test failure with forced assertion panic" >&2
  exit 1
fi

if ! grep -Fq "$sentinel" <<<"$output"; then
  echo "selected test failure did not print assertion failure context" >&2
  exit 1
fi

if ! grep -Fq "meta_001_selected_failure_outputs_assertion_context" <<<"$output"; then
  echo "selected test failure did not print failing test name" >&2
  exit 1
fi

echo "PASS verify-test-after-change-select-output"
