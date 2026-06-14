# Benchmark Adapter Phase 1 Contract Inventory

- Date: 2026-06-05
- Related plan: `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- Phase: Phase 1: Data Adapter Completion
- Status: Implemented, verified, and adversarially reviewed. Round 7 closure
  review and Round 8 delta review passed with no remaining blockers.

## Landed Contract

- `BenchmarkAdapter` now exposes independent data lifecycle methods:
  `inspect_data`, `prepare`, `list_tasks`, `create_task_plan`, and
  `snapshot_task`.
- `plan(split)` remains available as a compatibility wrapper and now delegates
  through `prepare -> list_tasks -> create_task_plan`.
- `BenchmarkDataState` records split-level descriptor readiness and optional
  cache manifest path without mutating local data.
- `RuntimeTaskSnapshot` records benchmark identity, split, task id, source ref,
  upstream metadata hash, instruction hash, task-plan hash, and optional
  external runner hint.
- `PreparedBenchmark` now records selected task ids, optional source manifest
  path, and optional prepared data snapshot hash, so post-prepare lifecycle
  steps have a stable authority boundary.
- fake-terminal, fake-patch, Terminal-Bench, and SWE-bench Pro implement the
  lifecycle.

## Selector Status

- `ADAPT-DATA-001`: active, validates descriptor/inspect-data immutability and
  the structured data/runtime boundary contract.
- `ADAPT-DATA-002`: active, validates idempotent prepare and rejection of
  corrupted/unready data.
- `ADAPT-DATA-003`: active, validates deterministic task ids and source refs.
- `ADAPT-DATA-004`: active, validates replay-sufficient task snapshot identity.
- `ADAPT-DATA-005`: active, validates stable `TaskPlan` creation and
  `plan(split)` wrapper compatibility.
- `ADAPT-DATA-000`: planned retired sentinel. It remains registered only so
  Phase 0 claims do not become invisible; it must not count as active proof.

## Adapter-Specific Notes

- Terminal-Bench task descriptors are derived from sorted task directories with
  `task.yaml` files. Source refs use deterministic checksums of task metadata.
- SWE-bench Pro task descriptors are derived from sorted
  `_src/SWE-bench_Pro-os/run_scripts/<instance_id>` directories. The data
  adapter no longer shells out to `uv` or Python while planning.
- SWE-bench Pro rejects source/data skew before reporting `ready`, and later
  lifecycle methods fail with an explicit prepared-data drift error when
  parquet, evaluator, or selected run script identity changes after prepare.
- SWE-bench Pro runtime execution still extracts parquet row metadata in the
  runtime adapter path. That remains Phase 5/6 scope, not Phase 1 data adapter
  scope.
- Existing adapter tests were split into dedicated test files to keep
  production adapter files under the repository line-count constraint.
- `ADAPT-DATA-001` now records its boundary proof in
  `docs/plans/2026-06-04-benchmark-adapter-phase-1-boundary.md` and enforces
  the allowed production dependency set, forbidden runtime import families,
  forbidden runtime symbols, forbidden runtime calls, qualified runtime paths,
  allowed `std::fs` read calls, and forbidden module path attributes.
- The `ADAPT-DATA-001` boundary proof is now rooted in the production module
  graph instead of a hand-maintained source list. It also rejects renamed
  runtime packages, ambient environment inspection, write-oriented filesystem
  calls, production `#[path]`/`include!` bypasses, and runtime path literals
  such as attempt/event/runtime snapshot paths.
- `ADAPT-DATA-004` now proves FakePatch snapshot identity is sensitive to
  patch-style task content by comparing hash changes across different fake
  patch tasks, and proves SWE-bench Pro upstream metadata hash changes when
  evaluator content changes.

## Verification Evidence

- `cargo test -p harnesslab-adapters -- --nocapture`: 28 passed.
- `cargo test -p harnesslab-core -- --nocapture`: 48 passed.
- `cargo test -p xtask -- --nocapture`: 19 passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-001`: 1 passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-002`: 1 passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-003`: 1 passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-004`: 1 passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-005`: 1 passed.
- `scripts/test-after-change.sh --select META-002`: passed.
- `scripts/test-after-change.sh --select META-008`: passed with active=5 and
  planned=11.
- `cargo test -p harnesslab-cli --test benchmark_contract -- --nocapture`: 6
  passed.
- `cargo test -p harnesslab-cli --test external_smoke_contract
  int_011_swe_bench_pro_smoke_runs_external_evaluator_contract --
  --nocapture`: 1 passed.
- `docs/plans/2026-06-04-benchmark-adapter-phase-1-coverage.md` records the
  adapter/method/failure-path coverage matrix for `ADAPT-DATA-001..005`.
- `docs/plans/2026-06-04-benchmark-adapter-phase-1-boundary.md` records the
  reviewable boundary contract for `ADAPT-DATA-001`.
- `git diff --check`: passed.
- Code file line counts checked; all touched code files are below 500 lines.

## Open Before Phase Closure

- None. Phase 1 is ready for commit and push after final verification remains
  green.
