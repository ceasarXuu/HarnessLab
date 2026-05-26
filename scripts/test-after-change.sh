#!/usr/bin/env bash
set -euo pipefail

if [[ "${1:-}" == "--select" ]]; then
  id="${2:?missing test id}"
  case "$id" in
    CLI-001) package="harnesslab-cli"; test_name="cli_001_help_lists_m0_commands" ;;
    CLI-002) package="harnesslab-cli"; test_name="cli_002_resume_and_replay_are_nested_under_run" ;;
    CLI-003) package="harnesslab-cli"; test_name="cli_003_m0_json_commands_have_stable_shape" ;;
    CLI-004) package="harnesslab-cli"; test_name="cli_004_m0_text_commands_succeed" ;;
    DOC-001) package="harnesslab-cli"; test_name="doc_001_doctor_json_has_stable_shape" ;;
    SEC-001) package="harnesslab-core"; test_name="tests::cfg_001_redacts_secret_values_without_removing_names" ;;
    META-002) exec scripts/verify-test-registry.sh ;;
    COV-005) package="xtask"; test_name="coverage::tests::coverage_001_module_thresholds_are_enforced" ;;
    COV-003) package="xtask"; test_name="coverage::tests::coverage_002_branch_threshold_requires_branch_data" ;;
    COV-007) package="xtask"; test_name="coverage::tests::coverage_003_new_files_must_appear_in_lcov" ;;
    *)
      echo "unknown test registry id: $id" >&2
      exit 2
      ;;
  esac
  output="$(cargo test -p "$package" --all-features "$test_name" -- --exact 2>&1)"
  printf '%s\n' "$output"
  if ! grep -q "running 1 test" <<<"$output"; then
    echo "selected test did not run exactly once: $id -> $test_name" >&2
    exit 1
  fi
  exit 0
fi

echo "== environment preflight =="
rustc --version
cargo --version
if ! cargo nextest --version | grep -q "cargo-nextest 0.9.136"; then
  echo "ERROR cargo-nextest: expected 0.9.136" >&2
  exit 1
fi
if ! cargo llvm-cov --version | grep -q "cargo-llvm-cov 0.8.7"; then
  echo "ERROR cargo-llvm-cov: expected 0.8.7" >&2
  exit 1
fi

echo "== format =="
cargo fmt --all --check

echo "== lint =="
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "== tests =="
if cargo nextest --version >/dev/null 2>&1; then
  cargo nextest run --workspace --all-features
else
  echo "SKIP cargo-nextest: not installed; using cargo test for M0 bootstrap"
  cargo test --workspace --all-features
fi

echo "== registry-check =="
scripts/verify-test-registry.sh

echo "== traceability-check =="
scripts/generate-test-traceability.sh

echo "== security-redaction =="
scripts/scan-artifacts-for-secrets.sh

echo "== coverage =="
mkdir -p coverage
cargo +nightly-2026-05-26 llvm-cov clean --workspace
cargo +nightly-2026-05-26 llvm-cov test --workspace --all-features --exclude xtask --branch --no-report
cargo +nightly-2026-05-26 llvm-cov report --lcov --output-path coverage/lcov.info
cargo run -p xtask -- check-coverage --lcov coverage/lcov.info --min-line 97 --min-branch 97
cargo +nightly-2026-05-26 llvm-cov report --cobertura --output-path coverage/cobertura.xml
cargo +nightly-2026-05-26 llvm-cov report --json --output-path coverage/coverage.json

echo "== new-file-coverage =="
scripts/check-new-file-coverage.sh

echo "PASS scripts/test-after-change.sh"
