# Benchmark Adapter Phase 2 Task Runtime Snapshot Review

- Date: 2026-06-05
- Scope: Phase 2 task runtime snapshot persistence and validation.
- Worktree: `/Volumes/XU-1TB-NPM/projects/HarnessLab`
- Review mode: fresh internal subagents, no forked context, read-only review.
- Status: Closed. No unresolved BLOCKER or P1 findings remain.

## Reviewed Change

- `BenchmarkPlan.task_runtime_snapshots` is added with `serde(default)` for old
  snapshot compatibility.
- Data adapters build runtime snapshots from the same `TaskPlan` entries emitted
  by `plan(split)`.
- Scheduler carries the matching runtime snapshot into `AttemptWork`.
- Runner writes `task.snapshot.json` and `task-runtime.snapshot.json` through a
  dedicated `runner/task_snapshot.rs` boundary.
- `REPLAY-007` proves benchmark-level and per-task runtime snapshot persistence.
- `ADAPT-DATA-005` proves `plan(split)` emits task runtime snapshots matching
  `snapshot_task(...)` on a multi-task Terminal-Bench fixture.

## Initial Review Inputs

- `crates/harnesslab-core/src/benchmark.rs`
- `crates/harnesslab-core/src/model.rs`
- `crates/harnesslab-adapters/src/registry.rs`
- `crates/harnesslab-adapters/src/data_contract_tests.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/src/runner/schedule.rs`
- `crates/harnesslab-cli/src/runner/task_snapshot.rs`
- `crates/harnesslab-cli/tests/task_snapshot_contract.rs`
- `tests/TEST_REGISTRY.toml`
- `scripts/test-after-change.sh`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-05-benchmark-adapter-phase-2-inventory.md`

## Round 1 Findings

### architecture-adversary

- BLOCKER: none.
- P1: none.
- P2: snapshot authority was still best-effort because non-empty
  `task_runtime_snapshots` were not validated by `validate_benchmark_plan`.
- P2: per-task runtime snapshots are currently projected during attempt
  execution instead of run setup. This remains a Phase 2 policy decision and is
  documented as open.
- P2: legacy missing-field deserialization was implemented but not directly
  proven.

### test-validity-adversary

- BLOCKER: none.
- P1: none.
- P2: missing invariant proof for task-to-runtime-snapshot pairing.
- P2: `REPLAY-007` only proved `task.snapshot.json` existence, not semantic
  consistency with `task-runtime.snapshot.json.task_plan_hash`.
- P2: `REPLAY-007` registry traceability omitted `runner.rs`.
- P2: `ADAPT-DATA-005` did not prove `plan().task_runtime_snapshots`.

## Round 1 Response

- Added `ModelError::InvalidTaskRuntimeSnapshot` and validation for non-empty
  `BenchmarkPlan.task_runtime_snapshots`.
- Added validation tests for runtime snapshot pairing, duplicate snapshot ids,
  benchmark/split mismatch, and legacy missing-field deserialization.
- Strengthened `REPLAY-007` to parse `task.snapshot.json` and recompute the
  stable FNV64 task plan hash.
- Updated `TEST_REGISTRY.toml` so `REPLAY-007` includes `runner.rs`.
- Extended `ADAPT-DATA-005` to assert `plan().task_runtime_snapshots` equals
  `snapshot_task(...)` for each descriptor in the multi-task Terminal-Bench
  fixture.
- Updated Phase 2 inventory and the architecture plan to keep enforcement,
  drift checks, external runtime snapshots, degraded replay policy, and
  run-setup projection open.

## Delta Review Finding

- BLOCKER: none.
- P1: none.
- P2: duplicate `TaskPlan.task_id` values could be collapsed by the task-id set
  during snapshot validation.

## Delta Response

- Updated `validate_benchmark_plan` to reject duplicate `TaskPlan.task_id`
  values.
- Added a regression case in
  `core_001_benchmark_plan_validation_checks_runtime_snapshot_pairing`.

## Final Verification

- `scripts/test-after-change.sh --select REPLAY-007`: 1 passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-005`: 1 passed.
- `scripts/test-after-change.sh --select INT-013`: 1 passed.
- `cargo test -p harnesslab-core --all-features`: 50 passed.
- `cargo test -p harnesslab-adapters --all-features`: 28 passed.
- `cargo test -p harnesslab-cli --all-features --lib`: 119 passed.
- `scripts/test-after-change.sh --select META-002`: passed with 42
  requirements, 169 tests, and 16 adapter proof claims from 3 sources.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- Line-count check: modified single code/test files remain below 500 lines.

## Closure

This slice is acceptable to commit. It lands task runtime snapshot persistence
and validation, but Phase 2 remains open for replay hard-block policy around
empty or missing runtime snapshots, dataset/evaluator/source/official-runner
drift checks, external runtime public/private schemas, degraded replay policy,
`SWEPRO-005`, and the decision whether per-task snapshot projection should move
from attempt execution to run setup.
