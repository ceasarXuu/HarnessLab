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
    CORE-001) package="harnesslab-core"; test_name="model::tests::core_001_state_machine_allows_expected_lifecycle" ;;
    CORE-002) package="harnesslab-core"; test_name="model::tests::core_002_state_machine_rejects_terminal_to_running" ;;
    CORE-003) package="harnesslab-core"; test_name="model::tests::core_003_failure_classifier_maps_agent_timeout" ;;
    CORE-004) package="harnesslab-core"; test_name="model::tests::core_004_failure_classifier_maps_failed_verifier" ;;
    ORCH-003) package="harnesslab-core"; test_name="model::tests::orch_003_exit_code_priority_prefers_execution_over_benchmark" ;;
    CFG-001) package="harnesslab-core"; test_name="config::tests::cfg_001_valid_global_config_passes" ;;
    CFG-002) package="harnesslab-core"; test_name="config::tests::cfg_002_invalid_profile_name_fails" ;;
    CFG-003|SEC-001) package="harnesslab-core"; test_name="redaction::tests::cfg_003_redacts_secret_values_without_removing_names" ;;
    CFG-004) package="harnesslab-core"; test_name="config::tests::cfg_004_path_expands_home_and_relative_paths" ;;
    AGT-005) package="harnesslab-core"; test_name="config::tests::agt_005_docker_socket_requested_warns" ;;
    USE-001) package="harnesslab-core"; test_name="usage::tests::use_001_parser_none_is_unknown" ;;
    USE-002) package="harnesslab-core"; test_name="usage::tests::use_002_regex_parser_extracts_tokens" ;;
    USE-004) package="harnesslab-core"; test_name="usage::tests::use_004_attempts_aggregate_parsed_usage" ;;
    C-BENCH-001) package="harnesslab-adapters"; test_name="registry::tests::c_bench_001_built_in_descriptors_include_required_benchmarks" ;;
    C-BENCH-002) package="harnesslab-adapters"; test_name="fake_terminal::tests::c_bench_002_fake_terminal_task_plan_is_serializable" ;;
    C-BENCH-003) package="harnesslab-adapters"; test_name="fake_patch::tests::c_bench_003_fake_patch_plan_has_patch_spec" ;;
    ART-003) package="harnesslab-infra"; test_name="artifact::tests::art_003_atomic_json_write_produces_valid_json" ;;
    LOG-003) package="harnesslab-infra"; test_name="event::tests::log_003_events_are_redacted" ;;
    C-SBOX-001) package="harnesslab-infra"; test_name="docker::tests::c_sbox_001_health_check_is_structured" ;;
    C-SBOX-002) package="harnesslab-infra"; test_name="process::tests::c_sbox_002_host_exec_echo_captures_stdout" ;;
    C-SBOX-003) package="harnesslab-infra"; test_name="process::tests::c_sbox_003_host_exec_timeout_is_structured" ;;
    RPT-001) package="harnesslab-report"; test_name="tests::rpt_001_report_html_contains_summary_and_relative_links" ;;
    INT-001) package="harnesslab-cli"; test_name="int_001_init_empty_home_creates_config_and_profiles" ;;
    INT-003) package="harnesslab-cli"; test_name="int_003_fake_terminal_success_creates_report_and_results" ;;
    INT-004) package="harnesslab-cli"; test_name="int_004_fake_terminal_test_fail_exits_2" ;;
    INT-005) package="harnesslab-cli"; test_name="int_005_fake_terminal_timeout_exits_1" ;;
    INT-006) package="harnesslab-cli"; test_name="int_006_fake_patch_success_saves_diff" ;;
    INT-009) package="harnesslab-cli"; test_name="int_009_replay_success_creates_new_run" ;;
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
cargo run -p xtask -- check-coverage --lcov coverage/lcov.info --min-line 95 --min-branch 70
cargo +nightly-2026-05-26 llvm-cov report --cobertura --output-path coverage/cobertura.xml
cargo +nightly-2026-05-26 llvm-cov report --json --output-path coverage/coverage.json

echo "== new-file-coverage =="
scripts/check-new-file-coverage.sh

echo "PASS scripts/test-after-change.sh"
