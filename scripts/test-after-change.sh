#!/usr/bin/env bash
set -euo pipefail

run_filtered_tests() {
  local selected_id="$1"
  local package="$2"
  local target="$3"
  local test_filter="$4"
  local expected_count="$5"
  local -a cargo_args=(-p "$package" --all-features)
  if [[ "$target" == "lib" ]]; then
    cargo_args+=(--lib)
  elif [[ "$target" == test:* ]]; then
    cargo_args+=(--test "${target#test:}")
  fi

  set +e
  output="$(cargo test "${cargo_args[@]}" "$test_filter" -- --nocapture 2>&1)"
  cargo_status=$?
  set -e
  printf '%s\n' "$output"
  if [[ "$cargo_status" -ne 0 ]]; then
    exit "$cargo_status"
  fi
  if ! grep -Eq "^running ${expected_count} tests?$" <<<"$output"; then
    echo "selected test group did not run expected count: ${selected_id} -> ${test_filter}, expected ${expected_count}" >&2
    exit 1
  fi
  ok_count="$(grep -Ec "^test .*${test_filter//./\\.}.*\\.\\.\\. ok$" <<<"$output" || true)"
  if [[ "$ok_count" -ne "$expected_count" ]]; then
    echo "selected test group did not report expected ok lines: ${selected_id} -> ${test_filter}, expected ${expected_count}, got ${ok_count}" >&2
    exit 1
  fi
}

planned_adapter_proof() {
  local selected_id="$1"
  local phase="$2"
  echo "planned adapter proof is registered but not implemented yet: ${selected_id} (${phase})" >&2
  exit 64
}

if [[ "${1:-}" == "--select" ]]; then
  id="${2:?missing test id}"
  test_target="${HARNESSLAB_TEST_TARGET:-}"
  case "$id" in
    CLI-001) package="harnesslab-cli"; test_name="cli_001_help_lists_m0_commands" ;;
    CLI-002) package="harnesslab-cli"; test_name="cli_002_resume_and_replay_are_nested_under_run" ;;
    CLI-003) package="harnesslab-cli"; test_name="cli_003_m0_json_commands_have_stable_shape" ;;
    CLI-004) package="harnesslab-cli"; test_name="cli_004_m0_text_commands_succeed" ;;
    CLI-008) package="harnesslab-cli"; test_name="doctor::tests::doc_008_overall_status_prioritizes_error_then_warning" ;;
    DOC-001) package="harnesslab-cli"; test_name="doc_001_doctor_json_has_stable_shape" ;;
    DOC-003) package="harnesslab-cli"; test_name="doc_003_doctor_reports_semantically_invalid_agent_profiles" ;;
    DOC-004) package="harnesslab-cli"; test_name="doc_004_doctor_reports_builtin_benchmark_readiness" ;;
    DOC-005) package="harnesslab-cli"; test_name="doc_005_doctor_reports_agent_profile_warnings" ;;
    DOC-006) package="harnesslab-cli"; test_name="doc_006_doctor_reports_agent_profile_load_errors" ;;
    DOC-007) package="harnesslab-cli"; test_name="doc_007_doctor_reports_auth_and_usage_configuration_problems" ;;
    CORE-001) package="harnesslab-core"; test_name="model::tests::core_001_state_machine_allows_expected_lifecycle" ;;
    CORE-002) package="harnesslab-core"; test_name="model::tests::core_002_state_machine_rejects_terminal_to_running" ;;
    CORE-003) package="harnesslab-core"; test_name="model::tests::core_003_failure_classifier_maps_agent_timeout" ;;
    CORE-004) package="harnesslab-core"; test_name="model::tests::core_004_failure_classifier_maps_failed_verifier" ;;
    ORCH-003) package="harnesslab-core"; test_name="model::tests::orch_003_exit_code_mapping_covers_command_health" ;;
    CFG-001) package="harnesslab-core"; test_name="config::tests::cfg_001_valid_global_config_passes" ;;
    CFG-002) package="harnesslab-core"; test_name="config::tests::cfg_002_invalid_profile_name_fails" ;;
    CFG-003|SEC-001) package="harnesslab-core"; test_name="redaction::tests::cfg_003_redacts_secret_values_without_removing_names" ;;
    CFG-004) package="harnesslab-core"; test_name="config::tests::cfg_004_path_expands_home_and_relative_paths" ;;
    CFG-006) package="harnesslab-core"; test_name="config::tests::cfg_004_effective_auth_mount_specs_match_runtime_rules" ;;
    AGT-005) package="harnesslab-core"; test_name="config::tests::agt_005_docker_socket_requested_warns" ;;
    USE-001) package="harnesslab-core"; test_name="usage::tests::use_001_parser_none_is_unknown" ;;
    USE-002) package="harnesslab-core"; test_name="usage::tests::use_002_regex_parser_extracts_tokens" ;;
    USE-004) package="harnesslab-core"; test_name="usage::tests::use_004_attempts_aggregate_parsed_usage" ;;
    USE-005) package="harnesslab-cli"; test_name="use_005_usage_regex_parser_records_tokens_and_report_text" ;;
    USE-006) package="harnesslab-cli"; test_name="use_005_usage_json_path_parser_records_cost_and_report_text" ;;
    USE-007) package="harnesslab-cli"; test_name="use_005_usage_parser_failure_is_persisted_and_reported" ;;
    C-BENCH-001) package="harnesslab-adapters"; test_name="registry::tests::c_bench_001_built_in_descriptors_include_required_benchmarks" ;;
    C-BENCH-002) package="harnesslab-adapters"; test_name="fake_terminal::tests::c_bench_002_fake_terminal_task_plan_is_serializable" ;;
    C-BENCH-003) package="harnesslab-adapters"; test_name="fake_patch::tests::c_bench_003_fake_patch_plan_has_patch_spec" ;;
    C-BENCH-004) package="harnesslab-adapters"; test_name="registry::tests::c_bench_004_required_external_smoke_adapters_are_available" ;;
    C-BENCH-005) package="harnesslab-cli"; test_name="bench_001_terminal_bench_info_uses_local_data_root" ;;
    C-BENCH-006) package="harnesslab-cli"; test_name="bench_002_swe_bench_pro_info_uses_local_data_root" ;;
    C-BENCH-007) package="harnesslab-cli"; test_name="bench_003_run_blocks_unsupported_local_full_split_before_planning" ;;
    C-BENCH-008) package="harnesslab-cli"; test_name="bench_004_run_blocks_swe_bench_pro_full_before_planning" ;;
    C-BENCH-009) package="harnesslab-adapters"; test_name="terminal_bench::tests::c_bench_006_terminal_bench_maps_task_test_timeout"; test_target="lib" ;;
    C-BENCH-010) package="harnesslab-adapters"; test_name="terminal_bench::tests::c_bench_006_terminal_bench_ignores_timeout_text_inside_block_scalars"; test_target="lib" ;;
    ART-003) package="harnesslab-infra"; test_name="artifact::tests::art_003_atomic_json_write_produces_valid_json" ;;
    LOG-003) package="harnesslab-infra"; test_name="event::tests::log_003_events_are_redacted" ;;
    LOG-005) package="harnesslab-infra"; test_name="event::tests::log_005_concurrent_process_appends_preserve_jsonl" ;;
    LOG-006) package="harnesslab-infra"; test_name="event::tests::log_006_event_log_integrity_rejects_malformed_line" ;;
    META-001-FAIL) package="harnesslab-infra"; test_name="event::tests::meta_001_selected_failure_outputs_assertion_context" ;;
    C-SBOX-001) package="harnesslab-infra"; test_name="docker::tests::c_sbox_001_health_check_is_structured" ;;
    C-SBOX-002) package="harnesslab-infra"; test_name="process::tests::c_sbox_002_host_exec_echo_captures_stdout" ;;
    C-SBOX-003) package="harnesslab-infra"; test_name="process::tests::c_sbox_003_host_exec_timeout_is_structured" ;;
    C-SBOX-004) package="harnesslab-infra"; test_name="docker::tests::c_sbox_004_create_args_include_labels_mounts_and_network_policy" ;;
    C-SBOX-005) package="harnesslab-infra"; test_name="docker::tests::c_sbox_005_exec_copy_destroy_and_cleanup_args_are_stable" ;;
    C-SBOX-006) package="harnesslab-infra"; test_name="docker::tests::c_sbox_006_create_copy_and_destroy_use_runner_outputs" ;;
    C-SBOX-007) package="harnesslab-infra"; test_name="docker::tests::c_sbox_007_create_rejects_failed_or_empty_container_id" ;;
    C-SBOX-008) package="harnesslab-infra"; test_name="docker::tests::c_sbox_008_cleanup_orphans_removes_listed_containers" ;;
    C-SBOX-009) package="harnesslab-infra"; test_name="docker::tests::c_sbox_009_error_paths_are_structured" ;;
    C-SBOX-010) package="harnesslab-infra"; test_name="docker::tests::c_sbox_010_exec_without_docker_returns_process_record" ;;
    C-SBOX-011) package="harnesslab-infra"; test_name="docker::tests::c_sbox_011_create_args_cover_privileged_full_network_and_sanitized_names" ;;
    C-SBOX-012) package="harnesslab-infra"; test_name="docker::tests::c_sbox_012_mount_check_reports_dry_run_status" ;;
    C-SBOX-013) package="harnesslab-infra"; test_name="process::tests::c_sbox_003_host_exec_no_output_timeout_is_structured" ;;
    C-SBOX-014) package="harnesslab-infra"; test_name="c_sbox_014_sigterm_kills_registered_process_group" ;;
    C-SBOX-015) package="harnesslab-infra"; test_name="process::tests::c_sbox_003_no_output_activity_pattern_defers_to_hard_timeout" ;;
    C-SBOX-016) package="harnesslab-infra"; test_name="process::tests::c_sbox_003_no_output_activity_disappearing_kills_promptly" ;;
    C-SBOX-017) package="harnesslab-infra"; test_name="process::tests::c_sbox_017_no_output_progress_file_resets_watchdog_window" ;;
    C-SBOX-018) package="harnesslab-infra"; test_name="process::tests::c_sbox_018_no_output_activity_has_bounded_grace" ;;
    C-SBOX-019) package="harnesslab-infra"; test_name="process::tests::c_sbox_019_activity_event_emits_after_output_reset" ;;
    C-SBOX-020) package="harnesslab-infra"; test_name="process::tests::c_sbox_020_progress_file_is_sampled_before_watchdog_boundary" ;;
    C-SBOX-021) package="harnesslab-infra"; test_name="process::tests::c_sbox_018_progress_growth_resets_activity_grace" ;;
    RPT-001) package="harnesslab-report"; test_name="tests::rpt_001_report_html_contains_summary_and_relative_links" ;;
    RPT-002) package="harnesslab-report"; test_name="tests::rpt_001_report_encodes_task_ids_and_rejects_unsafe_patch_links" ;;
    ORCH-004) package="harnesslab-cli"; test_name="runner::tests::run_004_planned_attempts_repeat_each_task_by_configured_attempts" ;;
    ORCH-005) package="harnesslab-cli"; test_name="runner::tests::run_005_docker_request_uses_run_network_and_task_sandbox_spec" ;;
    ORCH-006) package="harnesslab-cli"; test_name="runner::tests::run_005_host_fixture_does_not_request_docker" ;;
    ORCH-007) package="harnesslab-cli"; test_name="runner::tests::run_006_run_agent_host_executes_inside_workspace" ;;
    ORCH-008) package="harnesslab-cli"; test_name="runner::sandbox::tests::sandbox_failure_records_logs_and_failure_code" ;;
    ORCH-009) package="harnesslab-cli"; test_name="runner::sandbox::tests::render_command_covers_stdin_file_and_argument_modes" ;;
    ORCH-010) package="harnesslab-cli"; test_name="runner::sandbox::tests::agent_timeout_uses_task_override_marker" ;;
    ORCH-011) package="harnesslab-cli"; test_name="runner::tests::run_007_run_shell_reports_failed_command" ;;
    ORCH-012) package="harnesslab-cli"; test_name="runner::tests::run_008_panic_message_preserves_string_payloads" ;;
    ORCH-013) package="harnesslab-cli"; test_name="runner::cleanup::tests::cleanup_001_plan_requires_docker_only_for_container_tasks" ;;
    ORCH-014) package="harnesslab-cli"; test_name="runner::sandbox::tests::docker_guard_exposes_handle_and_ignores_destroy_errors_on_drop" ;;
    ORCH-015) package="harnesslab-cli"; test_name="runner::attempts::tests::run_004_attempt_scheduler_refills_slot_before_slow_task_finishes" ;;
    ORCH-016) package="harnesslab-cli"; test_name="runner::attempts::tests::run_004_attempt_scheduler_stops_refill_after_run_health_abort" ;;
    ORCH-017) package="harnesslab-cli"; test_name="runner::attempts::tests::run_004_attempt_scheduler_stops_refill_after_worker_error" ;;
    ORCH-018) package="harnesslab-cli"; test_name="runner::attempts::tests::run_004_attempt_scheduler_stops_refill_after_worker_panic" ;;
    ORCH-019) package="harnesslab-cli"; test_name="runner::monitor::tests::monitor_aborts_immediately_on_external_runner_timeout" ;;
    ORCH-020) package="harnesslab-cli"; test_name="runner::cleanup::tests::cleanup_007_terminal_bench_pre_run_considers_stale_run_without_snapshot" ;;
    ORCH-021) package="harnesslab-cli"; test_name="runner::cleanup::tests::cleanup_008_terminal_bench_pre_run_uses_stale_run_json_id" ;;
    ORCH-022) package="harnesslab-cli"; test_name="runner::cleanup::tests::cleanup_009_terminal_bench_pre_run_ignores_loose_name_match" ;;
    ORCH-023) package="harnesslab-cli"; test_name="runner::monitor::tests::monitor_aborts_immediately_on_external_runner_setup_failure"; test_target="lib" ;;
    REPLAY-002) package="harnesslab-cli"; test_name="runner::tests::replay_002_resume_keeps_completed_attempts_and_schedules_missing_only" ;;
    REPLAY-004) package="harnesslab-cli"; test_name="runner::tests::replay_002_resume_failed_completed_attempt_schedules_recovery_attempt" ;;
    REPLAY-005) package="harnesslab-cli"; test_name="runner::tests::replay_002_resume_does_not_create_unbounded_recovery_attempts" ;;
    REPLAY-006) package="harnesslab-cli"; test_name="runner::tests::replay_002_resume_uses_encoded_task_dir_for_slash_bearing_task_ids" ;;
    REPLAY-003) package="harnesslab-cli"; test_name="runner::tests::replay_003_replay_spec_preserves_execution_config_and_links_source" ;;
    INT-001) package="harnesslab-cli"; test_name="int_001_init_empty_home_creates_config_and_profiles" ;;
    INT-003) package="harnesslab-cli"; test_name="int_003_fake_terminal_success_creates_report_and_results" ;;
    INT-004) package="harnesslab-cli"; test_name="int_004_fake_terminal_test_fail_exits_0_with_benchmark_verdict" ;;
    INT-005) package="harnesslab-cli"; test_name="int_005_fake_terminal_timeout_exits_1" ;;
    INT-006) package="harnesslab-cli"; test_name="int_006_fake_patch_success_saves_diff" ;;
    INT-009) package="harnesslab-cli"; test_name="int_009_replay_success_creates_new_run" ;;
    INT-011) run_filtered_tests "$id" "harnesslab-cli" "test:external_smoke_contract" "int_011_swe_bench_pro" 10; exit 0 ;;
    INT-012) package="harnesslab-cli"; test_name="int_012_replay_text_output_succeeds" ;;
    INT-013) package="harnesslab-cli"; test_name="int_013_replay_falls_back_when_benchmark_snapshot_is_missing" ;;
    INT-014) package="harnesslab-cli"; test_name="int_014_resume_rejects_invalid_profile_snapshot" ;;
    INT-015) package="harnesslab-cli"; test_name="int_008_resume_failed_run_recovers_once_and_reports_latest_attempt" ;;
    INT-016) package="harnesslab-cli"; test_name="int_016_resume_interrupted_attempt_schedules_recovery_attempt" ;;
    INT-017) package="harnesslab-cli"; test_name="int_017_replay_redacts_public_artifacts_without_current_env" ;;
    INT-018) package="harnesslab-cli"; test_name="int_018_replay_rejects_redacted_legacy_profile_without_runtime_snapshot" ;;
    INT-019) package="harnesslab-cli"; test_name="int_019_resume_report_marks_missing_original_command" ;;
    INT-020) package="harnesslab-cli"; test_name="int_020_resume_redacts_public_artifacts_without_current_env" ;;
    INT-021) package="harnesslab-cli"; test_name="int_021_terminal_bench_silent_official_runner_is_no_progress" ;;
    INT-022) package="harnesslab-cli"; test_name="int_022_terminal_bench_official_agent_timeout_is_benchmark_verdict" ;;
    INT-023) package="harnesslab-cli"; test_name="int_023_terminal_bench_repeated_official_agent_timeouts_do_not_abort_run" ;;
    INT-024) package="harnesslab-cli"; test_name="int_024_terminal_bench_success_with_agent_timeout_gets_warning" ;;
    INT-025) package="harnesslab-cli"; test_name="int_025_terminal_bench_default_no_output_watchdog_is_enabled" ;;
    INT-026) package="harnesslab-cli"; test_name="int_026_terminal_bench_no_progress_overrides_official_result" ;;
    INT-027) package="harnesslab-cli"; test_name="int_027_terminal_bench_repeated_no_progress_aborts_run" ;;
    INT-028) package="harnesslab-cli"; test_name="int_028_terminal_bench_hard_timeout_overrides_official_result" ;;
    INT-029) exec scripts/verify-terminal-bench-docker-activity-watchdog.sh ;;
    INT-031) package="harnesslab-cli"; test_name="int_031_terminal_bench_progress_deferral_still_hard_times_out" ;;
    INT-032) package="harnesslab-cli"; test_name="int_032_resume_rejects_malformed_event_log_before_reuse" ;;
    INT-033) package="harnesslab-cli"; test_name="int_033_replay_rejects_malformed_source_event_log" ;;
    INT-034) package="harnesslab-cli"; test_name="int_034_report_open_rejects_malformed_event_log" ;;
    INT-035) package="harnesslab-cli"; test_name="int_035_terminal_bench_stale_docker_activity_becomes_no_progress" ;;
    INT-036) exec scripts/verify-terminal-bench-docker-activity-grace-expiry.sh ;;
    INT-037) exec scripts/verify-terminal-bench-import-success-cleanup.sh ;;
    INT-038) exec scripts/verify-terminal-bench-import-timeout-cleanup.sh ;;
    INT-039) package="harnesslab-cli"; test_name="int_039_terminal_bench_stale_early_run_log_becomes_no_progress" ;;
    INT-040) package="harnesslab-cli"; test_name="int_040_terminal_bench_post_task_cleanup_failure_is_execution_failure" ;;
    INT-041) package="harnesslab-cli"; test_name="int_041_cleanup_failure_does_not_mask_no_progress_health" ;;
    INT-042) package="harnesslab-cli"; test_name="int_042_cleanup_failure_overrides_benchmark_failure_with_warning" ;;
    INT-043) package="harnesslab-cli"; test_name="int_043_cleanup_failure_does_not_mask_runner_timeout_health" ;;
    INT-044) package="harnesslab-cli"; test_name="int_044_terminal_bench_runtime_exports_amd64_platform_by_default"; test_target="test:terminal_bench_runtime_contract" ;;
    INT-045) package="harnesslab-cli"; test_name="int_011_terminal_bench_run_timeout_override_does_not_inflate_test_timeout"; test_target="test:terminal_bench_contract" ;;
    INT-046) package="harnesslab-cli"; test_name="int_046_terminal_bench_bridge_setup_failure_drops_stale_benchmark_warning"; test_target="test:terminal_bench_setup_failure_contract" ;;
    TB-001) package="harnesslab-cli"; test_name="runner::external::tests::terminal_bench_result_failed_adapter_cleanup_overrides_success_score" ;;
    TB-002) package="harnesslab-cli"; test_name="runner::external::tests::terminal_bench_result_live_child_cleanup_error_is_execution_failure" ;;
    TB-003) package="harnesslab-cli"; test_name="runner::external::tests::terminal_bench_result_live_child_cleanup_log_is_execution_failure" ;;
    TB-004) package="harnesslab-cli"; test_name="runner::external::tests::terminal_bench_hard_timeout_maps_to_external_runner_timeout" ;;
    TB-005) package="harnesslab-cli"; test_name="runner::external::tests::terminal_bench_result_maps_parse_error_to_agent_output_parse_error"; test_target="lib" ;;
    TB-006) package="harnesslab-cli"; test_name="runner::external::tests::terminal_bench_no_output_timeout_defaults_to_setup_watchdog"; test_target="lib" ;;
    TB-007) package="harnesslab-cli"; test_name="runner::external::log_scan::tests::detects_terminal_bench_compose_setup_failure"; test_target="lib" ;;
    TB-008) package="harnesslab-cli"; test_name="runner::external::terminal_bench_runtime::tests::terminal_bench_runtime_prepares_qemu_dataset_without_mutating_source"; test_target="lib" ;;
    TB-009) package="harnesslab-cli"; test_name="runner::external::terminal_bench_runtime::tests::terminal_bench_runtime_prepares_forced_amd64_qemu_dataset"; test_target="lib" ;;
    TB-010) package="harnesslab-cli"; test_name="runner::external::terminal_bench_runtime::tests::terminal_bench_runtime_prep_failure_is_structured_task_result"; test_target="lib" ;;
    TB-011) package="harnesslab-cli"; test_name="runner::external::log_scan::tests::ignores_verifier_logs_with_docker_error_text"; test_target="lib" ;;
    AGT-REG-005) cargo test -p harnesslab-cli agt_reg_005 -- --nocapture; exec scripts/verify-terminal-bench-registered-setup.sh ;;
    AGT-REG-007) run_filtered_tests "$id" "harnesslab-cli" "test:agent_registry_contract" "agt_reg_007" 2; exit 0 ;;
    AGT-REG-008) run_filtered_tests "$id" "harnesslab-core" "lib" "agt_reg_008" 5; run_filtered_tests "$id" "harnesslab-cli" "lib" "agt_reg_008" 1; run_filtered_tests "$id" "harnesslab-cli" "test:doctor_contract" "agt_reg_008" 4; exit 0 ;;
    AGT-REG-009) run_filtered_tests "$id" "harnesslab-cli" "test:doctor_setup_contract" "agt_reg_009" 4; exit 0 ;;
    AGT-REG-010) run_filtered_tests "$id" "harnesslab-cli" "lib" "agt_reg_010" 4; run_filtered_tests "$id" "harnesslab-cli" "test:doctor_contract" "agt_reg_010" 2; run_filtered_tests "$id" "harnesslab-cli" "test:replay_contract" "agt_reg_010" 5; exit 0 ;;
    AGT-REG-011) run_filtered_tests "$id" "harnesslab-cli" "test:host_auth_contract" "agt_reg_011" 3; exit 0 ;;
    AGT-REG-012) run_filtered_tests "$id" "harnesslab-cli" "test:agent_registry_contract" "agt_reg_012" 2; run_filtered_tests "$id" "harnesslab-cli" "test:doctor_run_as_contract" "agt_reg_012" 4; run_filtered_tests "$id" "harnesslab-cli" "test:terminal_bench_run_as_contract" "agt_reg_012" 1; run_filtered_tests "$id" "harnesslab-cli" "test:external_smoke_contract" "agt_reg_012" 1; exit 0 ;;
    PY-TB-001) exec scripts/verify-terminal-bench-python-adapter.sh ;;
    ADAPT-DATA-000) planned_adapter_proof "$id" "Phase 0: retired gap sentinel" ;;
    ADAPT-DATA-001) package="harnesslab-adapters"; test_name="data_contract_tests::adapt_data_001_descriptor_and_inspect_data_do_not_mutate_cache"; test_target="lib" ;;
    ADAPT-DATA-002) package="harnesslab-adapters"; test_name="data_contract_tests::adapt_data_002_prepare_is_idempotent_and_rejects_unready_data"; test_target="lib" ;;
    ADAPT-DATA-003) package="harnesslab-adapters"; test_name="data_contract_tests::adapt_data_003_list_tasks_returns_stable_task_ids_and_source_refs"; test_target="lib" ;;
    ADAPT-DATA-004) package="harnesslab-adapters"; test_name="data_contract_tests::adapt_data_004_snapshot_task_captures_replay_sufficient_identity"; test_target="lib" ;;
    ADAPT-DATA-005) package="harnesslab-adapters"; test_name="data_contract_tests::adapt_data_005_create_task_plan_is_stable_and_plan_is_wrapper"; test_target="lib" ;;
    ADAPT-RUNTIME-001) planned_adapter_proof "$id" "Phase 3: runtime registry dispatch" ;;
    ADAPT-RUNTIME-002) planned_adapter_proof "$id" "Phase 3: runtime preflight ownership" ;;
    ADAPT-RUNTIME-003) planned_adapter_proof "$id" "Phase 6: runtime snapshots" ;;
    ADAPT-RUNTIME-004) planned_adapter_proof "$id" "Phase 6: cleanup report" ;;
    ADAPT-RUNTIME-005) planned_adapter_proof "$id" "Phase 6: runtime event taxonomy" ;;
    SWEPRO-001) planned_adapter_proof "$id" "Phase 5: metadata failure classification" ;;
    SWEPRO-002) planned_adapter_proof "$id" "Phase 5: workspace failure classification" ;;
    SWEPRO-003) planned_adapter_proof "$id" "Phase 5: patch failure classification" ;;
    SWEPRO-004) planned_adapter_proof "$id" "Phase 5: evaluator parse classification" ;;
    SWEPRO-005) planned_adapter_proof "$id" "Phase 6: replay runtime materials" ;;
    META-002) exec scripts/verify-test-registry.sh ;;
    META-008) exec scripts/verify-planned-adapter-selectors.sh ;;
    COV-005) package="xtask"; test_name="coverage::tests::coverage_001_module_thresholds_are_enforced" ;;
    COV-003) package="xtask"; test_name="coverage::tests::coverage_002_branch_threshold_requires_branch_data" ;;
    COV-007) package="xtask"; test_name="coverage::tests::coverage_003_new_files_must_appear_in_lcov" ;;
    *)
      echo "unknown test registry id: $id" >&2
      exit 2
      ;;
  esac
  if [[ -z "$test_target" && "$test_name" != *::* ]]; then
    crate_tests_dir="crates/$package/tests"
    if [[ -d "$crate_tests_dir" ]]; then
      matched_test_files=()
      while IFS= read -r matched_test_file; do
        matched_test_files+=("$matched_test_file")
      done < <(rg -l "fn[[:space:]]+$test_name\\b" "$crate_tests_dir" 2>/dev/null || true)
      if [[ "${#matched_test_files[@]}" -eq 1 ]]; then
        test_target="test:$(basename "${matched_test_files[0]}" .rs)"
      fi
    fi
  fi

  cargo_args=(-p "$package" --all-features)
  if [[ "$test_target" == "lib" ]]; then
    cargo_args+=(--lib)
  elif [[ "$test_target" == test:* ]]; then
    cargo_args+=(--test "${test_target#test:}")
  fi
  set +e
  output="$(cargo test "${cargo_args[@]}" "$test_name" -- --exact 2>&1)"
  cargo_status=$?
  set -e
  printf '%s\n' "$output"
  if [[ "$cargo_status" -ne 0 ]]; then
    exit "$cargo_status"
  fi
  running_one_count="$(grep -c '^running 1 test$' <<<"$output" || true)"
  if [[ "$running_one_count" -ne 1 ]]; then
    echo "selected test did not run exactly once: $id -> $test_name" >&2
    exit 1
  fi
  test_basename="${test_name##*::}"
  if ! grep -Eq "^test .*${test_basename//./\\.} .*\\.\\.\\. ok$" <<<"$output"; then
    echo "selected test output did not contain target ok line: $id -> $test_name" >&2
    exit 1
  fi
  exit 0
fi

echo "== environment preflight =="
rustc --version
cargo --version
if ! cargo llvm-cov --version | grep -q "cargo-llvm-cov 0.8.7"; then
  echo "ERROR cargo-llvm-cov: expected 0.8.7" >&2
  exit 1
fi
test_runner="${HARNESSLAB_TEST_RUNNER:-cargo}"
case "$test_runner" in
  cargo)
    echo "test runner: cargo"
    ;;
  nextest)
    if ! cargo nextest --version | grep -q "cargo-nextest 0.9.136"; then
      echo "ERROR cargo-nextest: expected 0.9.136" >&2
      exit 1
    fi
    echo "test runner: nextest"
    ;;
  *)
    echo "ERROR HARNESSLAB_TEST_RUNNER must be cargo or nextest" >&2
    exit 1
    ;;
esac

echo "== format =="
cargo fmt --all --check

echo "== lint =="
cargo clippy --workspace --all-targets --all-features -- -D warnings

echo "== tests =="
if [[ "$test_runner" == "nextest" ]]; then
  cargo nextest run --workspace --all-features
else
  cargo test --workspace --all-features
fi
scripts/verify-terminal-bench-python-adapter.sh

echo "== terminal-bench-real-registered-setup =="
scripts/verify-terminal-bench-registered-setup.sh

echo "== terminal-bench-real-import-timeout-cleanup =="
scripts/verify-terminal-bench-import-timeout-cleanup.sh

echo "== terminal-bench-real-import-success-cleanup =="
scripts/verify-terminal-bench-import-success-cleanup.sh

echo "== terminal-bench-real-docker-activity-watchdog =="
scripts/verify-terminal-bench-docker-activity-watchdog.sh

echo "== terminal-bench-real-docker-activity-grace-expiry =="
scripts/verify-terminal-bench-docker-activity-grace-expiry.sh

echo "== registry-check =="
scripts/verify-test-registry.sh

echo "== test-runner-meta =="
scripts/verify-test-after-change-select-output.sh

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
