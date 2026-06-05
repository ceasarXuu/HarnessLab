# Benchmark Adapter Phase 2 Snapshot Authority Inventory

- Date: 2026-06-05
- Related plan: `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- Phase: Phase 2: Snapshot Authority And Replay Contract
- Status: Started. Missing authoritative benchmark snapshot now blocks replay
  by default; drift checks, task-runtime schema, external-runtime schema,
  legacy degraded mode decision, and `SWEPRO-005` remain open.

## Landed Contract

- Replay reads `benchmark.snapshot.json` as the authoritative benchmark plan.
- If `benchmark.snapshot.json` is missing, replay now fails before writing a
  replacement run or executing any task.
- The previous fallback path that silently called the current benchmark adapter
  to re-plan from live data has been removed.
- `INT-013` now proves the missing-snapshot blocker instead of proving fallback
  success.

## Selector Status

- `INT-013`: active, validates missing authoritative benchmark snapshot blocks
  replay before task execution.
- `SWEPRO-005`: still planned, must prove SWE-bench Pro replay/readiness uses
  stored runtime materials instead of silent live replanning.

## Open Before Phase Closure

- Define what moves from `BenchmarkPlan`/`TaskPlan` into
  `task-runtime.snapshot.json`.
- Define public/private `external-runtime.*.json` schema handoff for Phase 6.
- Add mutable data drift checks for dataset, evaluator, source, and official
  runner identity.
- Decide whether legacy degraded replay exists; if retained, it must require an
  explicit CLI option and emit a warning.
- Add `SWEPRO-005` coverage for stored SWE-bench Pro runtime materials.
- Run focused adversarial review for the full Phase 2 slice after the remaining
  snapshot authority work lands.

## Verification Evidence

- `scripts/test-after-change.sh --select INT-013`: 1 passed.
- `cargo test -p harnesslab-cli --test replay_contract -- --nocapture`: 12
  passed.
- `scripts/test-after-change.sh --select META-002`: passed with 42
  requirements, 168 tests, and 16 adapter claims from 3 sources.
- `git diff --check`: passed.
- `crates/harnesslab-cli/tests/replay_contract.rs` line count checked: 491
  lines, below the 500-line repository constraint.
