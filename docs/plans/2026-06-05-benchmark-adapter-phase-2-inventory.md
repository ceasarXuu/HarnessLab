# Benchmark Adapter Phase 2 Snapshot Authority Inventory

- Date: 2026-06-05
- Related plan: `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- Phase: Phase 2: Snapshot Authority And Replay Contract
- Status: Started. Missing authoritative benchmark snapshot now blocks replay
  by default, new runs now persist task runtime snapshots, and external-task
  replay now blocks missing or divergent task runtime authority. SWE-bench Pro
  now writes attempt-level external runtime snapshots and replay requires them.
  Drift checks, legacy degraded mode decision, and broader Phase 6
  redaction/version hardening remain open.

## Landed Contract

- Replay reads `benchmark.snapshot.json` as the authoritative benchmark plan.
- If `benchmark.snapshot.json` is missing, replay now fails before writing a
  replacement run or executing any task.
- The previous fallback path that silently called the current benchmark adapter
  to re-plan from live data has been removed.
- `INT-013` now proves the missing-snapshot blocker instead of proving fallback
  success.
- `BenchmarkPlan.task_runtime_snapshots` records per-task runtime identity from
  the adapter data lifecycle. The field is `serde(default)` so older snapshots
  remain readable while new runs gain stronger authority.
- If `BenchmarkPlan.task_runtime_snapshots` is non-empty, model validation now
  requires one snapshot per task, no duplicate snapshot task ids, and matching
  benchmark/split identity.
- `task-runtime.snapshot.json` is written beside each task's existing
  `task.snapshot.json` when the benchmark plan contains a matching runtime
  snapshot.
- `REPLAY-007` now verifies the `task.snapshot.json` hash matches
  `task-runtime.snapshot.json.task_plan_hash`, not just artifact existence.
- `REPLAY-008` now proves external-task replay blocks when
  `BenchmarkPlan.task_runtime_snapshots` is empty, when the per-task
  `task-runtime.snapshot.json` artifact is missing, or when that artifact
  diverges from `benchmark.snapshot.json`.
- SWE-bench Pro attempts now write `external-runtime.private.json` and
  `external-runtime.public.json`.
- Replay readiness now reads those attempt-level external runtime snapshots for
  external tasks and blocks before creating a replay run if either file is
  missing or if schema version, visibility, attempt number, adapter version,
  runtime policy, benchmark, task id, runner kind, dataset path, source path,
  public/private artifact agreement, or runtime material fingerprints diverge
  from the authoritative snapshot chain.
- `SWEPRO-005` now proves a SWE-bench Pro replay can use stored runtime
  materials, and that removing or mutating attempt runtime material snapshots
  blocks a later replay before a new run directory is created.

## Task Runtime Snapshot Schema

`task-runtime.snapshot.json` currently mirrors `RuntimeTaskSnapshot` and is
generated from prepared immutable benchmark data plus the task plan hash:

| Field | Source | Replay Purpose |
| --- | --- | --- |
| `benchmark.name`, `benchmark.version` | `PreparedBenchmark.descriptor` | Bind task identity to the adapter and dataset version. |
| `split` | `PreparedBenchmark.split` | Prevent cross-split rebinding. |
| `task_id` | `TaskDescriptor.task_id` | Join runtime identity to `TaskPlan` and task artifact directory. |
| `source_ref` | `TaskDescriptor.source_ref` | Preserve upstream task id and checksum. |
| `upstream_metadata_hash` | `TaskDescriptor.source_ref.checksum` today | Seed future dataset/evaluator/source drift checks. |
| `instruction_hash` | Stable hash of `TaskPlan.instruction` | Detect instruction mutation independent of full plan encoding. |
| `task_plan_hash` | Stable JSON hash of `TaskPlan` | Bind runtime identity to the exact executable task plan. |
| `external_runner` | `TaskPlan.external_runner` | Carry adapter runtime kind and source/material paths into Phase 6. |

## External Runtime Snapshot Schema

The initial SWE-bench Pro schema is attempt-level and split by visibility:

| Artifact | Visibility | Current Contents |
| --- | --- | --- |
| `external-runtime.private.json` | Private | Benchmark/task/attempt identity, runner kind, adapter version, runtime policy, dataset/source paths, raw phase commands, working dirs, stdout/stderr paths, replay material paths, material checksums, public artifact list, `runtime_fingerprint`, `public_fingerprint`, redaction basis. |
| `external-runtime.public.json` | Public | Benchmark/task/attempt identity, runner kind, adapter version, runtime policy, redacted phase commands, attempt-relative stdout/stderr paths, public artifact list, material names, material existence, sizes, checksums, public/relative paths only, and `runtime_fingerprint`. |

Public snapshots intentionally omit `dataset_path`, `source_path`, and
`working_dir`, and do not include `redaction_basis`. `SWEPRO-005` asserts those
fields and raw or shell-escaped absolute run/benchmark paths are absent from
the public snapshot while private snapshots retain them for replay/readiness
authority.

`runtime_fingerprint` is recomputed from the private snapshot's raw runtime
policy, dataset/source paths, commands, replay materials, and public artifact
list. `public_fingerprint` is stored privately and recomputed from the public
redacted projection plus the private runtime fingerprint. Replay blocks when
either fingerprint diverges.

## Selector Status

- `INT-013`: active, validates missing authoritative benchmark snapshot blocks
  replay before task execution.
- `REPLAY-007`: active, validates new runs write `BenchmarkPlan.task_runtime_snapshots`
  and matching per-task `task-runtime.snapshot.json` beside `task.snapshot.json`.
- `REPLAY-008`: active, validates external-task replay fails before creating a
  replay run when task runtime snapshot authority is empty, missing, or
  divergent.
- `SWEPRO-005`: active, validates SWE-bench Pro writes public/private external
  runtime material snapshots and replay fails closed if public/private runtime
  material authority is missing, incomplete, or divergent.

## Open Before Phase Closure

- Decide whether per-task runtime snapshots should be projected during run
  setup instead of attempt execution before they become first-class replay
  authority.
- Generalize the `external-runtime.*.json` writer to Terminal-Bench and future
  runtime adapters without leaking benchmark-specific fields into the generic
  contract.
- Add mutable data drift checks for dataset, evaluator, source, and official
  runner identity.
- Decide whether legacy degraded replay exists; if retained, it must require an
  explicit CLI option and emit a warning.
- Add fake secret scans and explicit public/private forbidden-field checks for
  report data, events, and replay warnings.
- Run focused adversarial review for the full Phase 2 slice after the remaining
  snapshot authority work lands.

## Verification Evidence

- `scripts/test-after-change.sh --select REPLAY-008`: 1 passed.
- `scripts/test-after-change.sh --select SWEPRO-005`: 1 passed.
- `scripts/test-after-change.sh --select INT-011`: 10 passed.
- `scripts/test-after-change.sh --select REPLAY-007`: 1 passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-005`: 1 passed.
- `scripts/test-after-change.sh --select INT-013`: 1 passed.
- `cargo test -p harnesslab-core --all-features`: 50 passed.
- `cargo test -p harnesslab-adapters --all-features`: 28 passed.
- `cargo test -p harnesslab-cli --all-features --lib`: 119 passed.
- `cargo test -p harnesslab-cli --test replay_contract -- --nocapture`: 12
  passed before the `REPLAY-007` slice.
- `cargo check --all-targets`: passed.
- `scripts/test-after-change.sh --select META-002`: passed with 42
  requirements, 170 tests, and 16 adapter claims from 3 sources.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- `crates/harnesslab-cli/src/runner.rs` line count checked: 492 lines, below
  the 500-line repository constraint.
- `crates/harnesslab-cli/tests/task_snapshot_contract.rs` line count checked:
  205 lines, below the 500-line repository constraint.
- `crates/harnesslab-cli/tests/replay_contract.rs` line count checked: 491
  lines, below the 500-line repository constraint.
