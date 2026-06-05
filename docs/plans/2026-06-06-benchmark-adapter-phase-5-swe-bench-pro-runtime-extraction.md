# Benchmark Adapter Phase 5 SWE-bench Pro Runtime Extraction

Date: 2026-06-06

## Goal

Complete the fake-tool Phase 5 gate by making SWE-bench Pro runtime behavior
run through a dedicated `SweBenchProRuntimeAdapter` boundary with separately
observable metadata, workspace, patch, and evaluator failure classes.

## Completed Changes

- Added `crates/harnesslab-cli/src/runner/external/swe_bench_pro_adapter.rs`
  as the SWE-bench Pro runtime adapter ownership module.
- Removed SWE-bench Pro execution policy from the generic runtime registry.
- Changed SWE execution to consume a typed `SweBenchProRuntimeAttempt`.
- Added `metadata_extraction_failed` as a distinct failure code so metadata
  extraction errors are not collapsed into workspace preparation failures.
- Added stable `swe_bench_pro_*` phase events while preserving existing
  `external_runner_*` compatibility events.
- Added degraded-but-authoritative external runtime snapshots for setup
  failures so metadata/workspace/source-path failures still enter the runtime
  authority chain.
- Added an `agent_execution` phase to SWE runtime snapshots so gold-agent and
  sandbox-agent execution are represented in attempt authority.
- Activated `SWEPRO-001..004` as integration selectors with CLI runs against
  fake SWE-bench Pro tools.

## Evidence

| Evidence | Result |
| --- | --- |
| `scripts/test-after-change.sh --select SWEPRO-001` | passed; metadata extraction failure maps to `metadata_extraction_failed`, emits setup diagnostics, and writes external-runtime snapshots |
| `scripts/test-after-change.sh --select SWEPRO-002` | passed; workspace preparation failure maps to `workspace_prep_failed`, emits setup diagnostics, and writes external-runtime snapshots |
| `scripts/test-after-change.sh --select SWEPRO-003` | passed; empty patch maps to `benchmark/no_valid_diff`, diff capture failure maps to `execution/patch_apply_failed` |
| `scripts/test-after-change.sh --select SWEPRO-004` | passed; evaluator JSON parse corruption maps to `execution/evaluator_error` and emits `external_result_parse_failed` |
| `cargo test -p harnesslab-cli --test external_smoke_contract int_011_swe_bench_pro_workspace_failure_stays_task_failure -- --nocapture` | passed |
| `scripts/test-after-change.sh --select ADAPT-RUNTIME-001` | passed |
| `scripts/test-after-change.sh --select ADAPT-RUNTIME-002` | passed |
| `scripts/test-after-change.sh --select SWEPRO-005` | passed; replay authority accepts the new `agent_execution` snapshot phase |
| `scripts/verify-test-registry.sh` | passed; registry and adapter proof claims validate |
| `scripts/verify-planned-adapter-selectors.sh` | passed; active=13 planned=3 |
| `cargo fmt --all -- --check` | passed |

## Phase Diagnostics Covered By `SWEPRO-001..004`

- `external_runner_started`: metadata extraction started.
- `swe_bench_pro_metadata_extraction_started`: metadata extraction started with a stable phase key.
- `external_runner_workspace_started`: workspace preparation started.
- `swe_bench_pro_workspace_prep_started`: workspace preparation started with a stable phase key.
- `external_runner_agent_started`: agent execution started.
- `swe_bench_pro_agent_started`: agent execution started with a stable phase key.
- `external_runner_patch_started`: patch capture started.
- `swe_bench_pro_patch_capture_started`: patch capture started with a stable phase key.
- `external_runner_patch_captured`: patch capture completed with patch status.
- `swe_bench_pro_patch_captured`: patch capture completed with a snake_case patch status.
- `external_runner_evaluator_started`: evaluator execution started.
- `swe_bench_pro_evaluator_started`: evaluator execution started with a stable phase key.
- `external_runner_setup_failed`: metadata or workspace setup failure.
- `swe_bench_pro_setup_failed`: setup failure with phase and mapped failure code.
- `external_result_parse_failed`: evaluator result parse failure.
- `swe_bench_pro_result_parse_failed`: evaluator parse failure with parser and mapped failure code.

## Architecture Notes

The generic runtime registry now owns only dispatch and shared preflight report
shaping. SWE-bench Pro-specific dataset path binding, source path validation,
compatibility calculation, and runtime attempt creation live in
`SweBenchProRuntimeAdapter`.

The concrete SWE runner still owns official multi-phase execution: metadata
extraction, workspace preparation, agent execution, patch capture, evaluator
invocation, result projection, and runtime snapshot writing. Setup failures now
write degraded external-runtime snapshots before returning structured task
results, and successful/partial attempt snapshots include an explicit
`agent_execution` command phase. That keeps the adapter boundary explicit
without forcing the multi-step benchmark into a single-command abstraction. This
phase proves the evaluator command path with fake SWE-bench Pro tools; real
official evaluator preservation remains final gate evidence.

## Remaining Work

- `ADAPT-DATA-000` remains a planned retired sentinel.
- Complete Phase 6 public/private runtime snapshot assertions and structured
  cleanup-report proof under `ADAPT-RUNTIME-003..004`.
- Complete Phase 7 docs/doctor/readiness alignment.
- Complete Phase 8 full gate and final adversarial closure.
