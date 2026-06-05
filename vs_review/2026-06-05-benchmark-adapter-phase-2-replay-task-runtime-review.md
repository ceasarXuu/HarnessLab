# Benchmark Adapter Phase 2 Replay Task Runtime Review

- Date: 2026-06-05
- Scope: Phase 2 replay enforcement for task runtime snapshot authority.
- Worktree: `<repo-root>`
- Review mode: fresh internal subagents, no forked context, read-only review.
- Status: Closed. No unresolved BLOCKER, P1, or P2 findings remain after
  fresh closure review.

## Reviewed Change

- `replay_run` validates source task runtime snapshot authority before
  generating a replay run id or creating the replay run directory.
- `validate_replay_task_runtime_snapshots` now blocks external-task replay when
  `BenchmarkPlan.task_runtime_snapshots` is empty, when the per-task
  `task-runtime.snapshot.json` artifact is missing, when that artifact diverges
  from `benchmark.snapshot.json`, or when `TaskPlan.external_runner` diverges
  from the runtime snapshot authority.
- `REPLAY-008` covers task-plan-vs-authority mismatch, per-task artifact
  mismatch, missing per-task artifact, and empty benchmark-level runtime
  snapshot list.

## Review Inputs

- `crates/harnesslab-cli/src/runner/replay.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/tests/task_snapshot_contract.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-05-benchmark-adapter-phase-2-inventory.md`

## Round 1 Findings

### code-reviewer

- BLOCKER: `validate_replay_task_runtime_snapshots` gated on
  `TaskPlan.external_runner` but only compared per-task artifact authority
  against `BenchmarkPlan.task_runtime_snapshots`. A source run could mutate only
  `TaskPlan.external_runner`, keep both runtime snapshots internal, and replay
  would create a new run instead of failing before replay run creation.
- P2: docs claimed stronger proof than the test provided because `REPLAY-008`
  did not cover task-plan-vs-authority divergence.

### test-engineer

- BLOCKER: none.
- P1: none.
- P2: `REPLAY-008` is snapshot-driven and synthetic rather than produced by a
  native external adapter run. This is acceptable for the replay guard because
  the guard is snapshot-driven, but a future native external-benchmark contract
  should prove the same shape without JSON mutation.

## Round 1 Response

- Accepted the code-reviewer blocker.
- Updated `validate_replay_task_runtime_snapshots` so `TaskPlan.external_runner`
  must equal the matching `RuntimeTaskSnapshot.external_runner`.
- Expanded `REPLAY-008` with a branch that mutates only
  `TaskPlan.external_runner` and expects a `task external_runner mismatch`
  replay blocker.
- Strengthened `assert_replay_blocker` so every failure branch proves the source
  `runs/` count did not increase.
- Kept Phase 2 docs honest that native external-benchmark replay material
  coverage is still represented by planned `SWEPRO-005` and the remaining
  external runtime schema work.

## Post-Fix Verification

- `scripts/test-after-change.sh --select REPLAY-008`: 1 passed.
- `cargo check --all-targets`: passed.

## Closure Review

Fresh closure review found:

- BLOCKER: none.
- P1: none.
- P2: none.

Closure evidence:

- Replay guard runs before replay run creation.
- `validate_replay_task_runtime_snapshots` now fails when
  `TaskPlan.external_runner` differs from the matching
  `RuntimeTaskSnapshot.external_runner`.
- `REPLAY-008` includes the required task-plan-only mutation branch and asserts
  the source `runs/` count does not increase after every blocker branch.
- The same test still covers benchmark-level empty authority, missing per-task
  artifact, and divergent per-task artifact.
- Internal non-external replay remains compatible because the validator exits
  early when neither the task plan nor runtime snapshot carries
  `external_runner` authority.

## Final Verification

- `scripts/test-after-change.sh --select REPLAY-007`: 1 passed.
- `scripts/test-after-change.sh --select REPLAY-008`: 1 passed.
- `scripts/test-after-change.sh --select INT-013`: 1 passed.
- `cargo test -p harnesslab-cli --all-features --lib`: 119 passed.
- `scripts/test-after-change.sh --select META-002`: passed with 42
  requirements, 170 tests, and 16 adapter proof claims from 3 sources.
- `cargo check --all-targets`: passed.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.

## Closure

This slice is acceptable to commit. It closes external-task replay authority for
empty, missing, divergent, and task-plan-mismatched task runtime snapshots before
new replay run creation. Native external-benchmark production of the same
runtime materials remains future proof surface under `SWEPRO-005` and the
external runtime snapshot schema work.
