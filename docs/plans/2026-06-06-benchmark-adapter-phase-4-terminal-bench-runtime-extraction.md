# Benchmark Adapter Phase 4 Terminal-Bench Runtime Extraction

Date: 2026-06-06

## Goal

Complete Phase 4 by making Terminal-Bench runtime behavior run through a real
`TerminalBenchRuntimeAdapter` boundary while preserving the existing official
`tb run` behavior, emitted event records, timeout mapping, QEMU policy, and
cleanup semantics.

## Completed Changes

- Added `crates/harnesslab-cli/src/runner/external/terminal_bench_adapter.rs`
  as the Terminal-Bench runtime adapter ownership module.
- Moved Terminal-Bench preflight, agent selection, official `tb run` command
  construction, command snapshot writing, Docker platform selection, official
  agent timeout grace, runtime attempt policy, and run-level cleanup target
  ownership out of the generic registry and task execution file.
- Kept `crates/harnesslab-cli/src/runner/external/runtime_adapter.rs` focused
  on the shared adapter trait, registry dispatch, preflight report shaping, and
  generic cleanup dispatch.
- Reduced `terminal_bench.rs` to the concrete execution/result flow that
  consumes adapter-provided runtime policy instead of owning command policy.
- Activated `ADAPT-RUNTIME-005` with a runtime event-taxonomy contract test
  that parses emitted `events.jsonl` records and checks command preservation.
- Fixed `TB-001..004` selector routing to run as `lib` tests, making the
  Terminal-Bench selector gate stable and fast.

## Evidence

| Evidence | Result |
| --- | --- |
| `scripts/test-after-change.sh --select ADAPT-RUNTIME-001` | passed |
| `scripts/test-after-change.sh --select ADAPT-RUNTIME-002` | passed |
| `scripts/test-after-change.sh --select ADAPT-RUNTIME-005` | passed; executes Terminal-Bench fake-run scenarios, parses `events.jsonl`, and asserts command/argv preservation |
| `for id in TB-001 TB-002 TB-003 TB-004 TB-005 TB-006 TB-007 TB-008 TB-009 TB-010 TB-011; do scripts/test-after-change.sh --select "$id"; done` | all passed |
| `scripts/test-after-change.sh --select PY-TB-001` | 32 tests and 7 subtests passed |
| `cargo test -p harnesslab-cli --all-features --lib runner::external:: -- --nocapture` | 45 passed |
| `cargo fmt --all -- --check` | passed |
| `cargo check --workspace --all-targets --all-features` | passed |
| `scripts/verify-test-registry.sh` | passed |
| `scripts/verify-planned-adapter-selectors.sh` | passed at Phase 4 closure; active=9 planned=7 |

## Event Taxonomy Covered By `ADAPT-RUNTIME-005`

- `external_runner_configured`
- `terminal_bench_dataset_prepared`
- `external_runner_activity`
- `external_runner_no_progress`
- `external_runner_timeout`
- `external_runner_setup_failed`
- `terminal_bench_cleanup`
- `task_warning`
- `external_result_parse_failed`

`external_runner_configured` now includes timeout policy, Docker platform,
progress path template, activity patterns, official result path, and command
snapshot path.

## Architecture Notes

The generic runtime registry no longer contains Terminal-Bench command or
cleanup target policy. That prevents the registry from becoming a benchmark
specific switchboard while still keeping dispatch centralized. Terminal-Bench
execution keeps using the hardened official runner path; this phase changes
ownership boundaries and proof coverage, not the user CLI contract.

`ADAPT-RUNTIME-005` is intentionally limited to the Phase 4 Terminal-Bench event
taxonomy and command preservation proof. Phase 5 now records stable SWE-bench
Pro phase diagnostics under `SWEPRO-001..004`, and Phase 6 must add
public/private runtime snapshot and structured cleanup-report artifact
assertions under `ADAPT-RUNTIME-003..004`.

## Remaining Work

- Phase 4 adversarial review is passed in
  `vs_review/2026-06-06-benchmark-adapter-phase-4-terminal-bench-runtime-review.md`.
- Phase 5 is completed in
  `docs/plans/2026-06-06-benchmark-adapter-phase-5-swe-bench-pro-runtime-extraction.md`.
- Complete Phase 6 public/private runtime snapshots, cleanup-report artifacts,
  redaction scans, and replay hardening.
- Complete Phase 7 docs/doctor/readiness alignment.
- Complete Phase 8 full gate and final adversarial closure.
