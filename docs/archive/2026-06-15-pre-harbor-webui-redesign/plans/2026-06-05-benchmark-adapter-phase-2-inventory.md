# Benchmark Adapter Phase 2 Snapshot Authority Inventory

- Date: 2026-06-05
- Related plan: `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- Phase: Phase 2: Snapshot Authority And Replay Contract
- Status: Started. Missing authoritative benchmark snapshot now blocks replay
  by default, new runs now persist task runtime snapshots, and external-task
  replay now blocks missing or divergent task runtime authority. SWE-bench Pro
  now writes attempt-level external runtime snapshots, projects their anchors
  into `benchmark.snapshot.json` and `task-runtime.snapshot.json` under a
  run-scoped lock, emits `external_runtime_anchor_projected` after projection,
  and blocks replay on live material drift. Terminal-Bench drift checks,
  official runner identity drift, legacy degraded mode decision, and broader
  Phase 6 redaction/version hardening remain open.

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
- External attempts append `external_runtime_attempts` to both
  `benchmark.snapshot.json.task_runtime_snapshots` and the matching
  `task-runtime.snapshot.json`. Each entry binds the attempt number,
  attempt-relative public/private snapshot paths, public/private file
  checksums, and runtime/public fingerprints. Replay treats
  `benchmark.snapshot.json` as the authoritative root, requires
  `task-runtime.snapshot.json` to match it exactly, and then uses the attempt
  anchors to reject coordinated rewrites of `external-runtime.private.json`,
  `external-runtime.public.json`, and task-runtime-only anchor rewrites.
- External runtime attempt anchor projection is serialized by
  `.harnesslab-locks/external-runtime-anchor.lock` at the run directory. The
  lock covers the read/modify/write of both `task-runtime.snapshot.json` and
  `benchmark.snapshot.json`, preventing same-task multi-attempt completion or
  multi-task parallel completion from losing anchor updates.
- After a successful projection, the run event log records
  `external_runtime_anchor_projected` with the task id, attempt number, and
  attempt-relative public/private snapshot paths.
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
  public/private artifact agreement, task-runtime/benchmark attempt anchors, or
  runtime material fingerprints diverge from the authoritative snapshot chain.
- Replay readiness also checks live external SWE-bench Pro materials recorded
  in the private snapshot through `validation_scope=live_external`. Parquet,
  evaluator source, and run-script files must still exist and match their
  stored checksums before replay can create a new run.
- `SWEPRO-005` now proves a SWE-bench Pro replay can use stored runtime
  materials, and that removing or mutating attempt runtime material snapshots
  drifting parquet/evaluator/run-script live materials, child snapshot
  rewrites, or child-plus-task-runtime rewrites block a later replay before a
  new run directory is created.

## Task Runtime Snapshot Schema

`task-runtime.snapshot.json` is initialized from `RuntimeTaskSnapshot`, which is
generated from prepared immutable benchmark data plus the task plan hash. It is
then extended with external attempt anchors as attempts complete:

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
| `external_runtime_attempts` | External runtime snapshot writer | Bind attempt-level public/private runtime snapshots to benchmark replay authority and the task-runtime mirror. |

## External Runtime Snapshot Schema

The initial SWE-bench Pro schema is attempt-level and split by visibility:

| Artifact | Visibility | Current Contents |
| --- | --- | --- |
| `external-runtime.private.json` | Private | Benchmark/task/attempt identity, runner kind, adapter version, runtime policy, dataset/source paths, raw phase commands, working dirs, stdout/stderr paths, replay material paths, material validation scopes, material checksums, public artifact list, `runtime_fingerprint`, `public_fingerprint`, redaction basis. |
| `external-runtime.public.json` | Public | Benchmark/task/attempt identity, runner kind, adapter version, runtime policy, redacted phase commands, attempt-relative stdout/stderr paths, public artifact list, material names, material validation scopes, material existence, sizes, checksums, public/relative paths only, and `runtime_fingerprint`. |

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

For SWE-bench Pro, replay additionally treats private replay materials with
`validation_scope=live_external` as live external dependencies. Their files
must exist and their current checksums must match the stored checksum before
replay can execute. Materials with `validation_scope=archived_attempt` are
validated through the snapshot fingerprint and task-runtime attempt anchor, not
through live source paths.

## Selector Status

- `INT-013`: active, validates missing authoritative benchmark snapshot blocks
  replay before task execution.
- `REPLAY-007`: active, validates new runs write `BenchmarkPlan.task_runtime_snapshots`
  and matching per-task `task-runtime.snapshot.json` beside `task.snapshot.json`.
- `REPLAY-008`: active, validates external-task replay fails before creating a
  replay run when task runtime snapshot authority is empty, missing, or
  divergent.
- `REPLAY-009`: active, validates external runtime attempt anchor projection
  is serialized and preserves both benchmark and task-runtime authority during
  same-task multi-attempt overlap and multi-task parallel completion. It also
  pins the `external_runtime_anchor_projected` event payload.
- `SWEPRO-005`: active, validates SWE-bench Pro writes public/private external
  runtime material snapshots and replay fails closed if public/private runtime
  material authority is missing, incomplete, divergent, not anchored from
  `benchmark.snapshot.json` and `task-runtime.snapshot.json`, or bound to live
  external materials that drift.

## Open Before Phase Closure

- Decide whether attempt anchors should later move into a dedicated runtime
  manifest once Terminal-Bench and future runtime adapters share the same
  contract.
- Generalize the `external-runtime.*.json` writer to Terminal-Bench and future
  runtime adapters without leaking benchmark-specific fields into the generic
  contract.
- Extend mutable data drift checks beyond SWE-bench Pro live parquet/evaluator
  and run-script materials to Terminal-Bench and official runner identity.
- Decide whether legacy degraded replay exists; if retained, it must require an
  explicit CLI option and emit a warning.
- Add fake secret scans and explicit public/private forbidden-field checks for
  report data, events, and replay warnings.
- Complete the fresh adversarial closure review for the current SWE-bench Pro
  snapshot authority slice after the concurrent anchor projection fix.

## Verification Evidence

- `scripts/test-after-change.sh --select REPLAY-008`: 1 passed.
- `scripts/test-after-change.sh --select SWEPRO-005`: 1 passed.
- `scripts/test-after-change.sh --select INT-011`: 10 passed.
- `scripts/test-after-change.sh --select REPLAY-007`: 1 passed.
- `scripts/test-after-change.sh --select REPLAY-009`: 3 passed across
  `harnesslab-cli` and `harnesslab-infra`.
- `scripts/test-after-change.sh --select ADAPT-DATA-005`: 1 passed.
- `scripts/test-after-change.sh --select INT-013`: 1 passed.
- `cargo test -p harnesslab-infra --lib lock_001_serializes_file_mutation`: 1
  passed.
- `cargo test -p harnesslab-cli --lib runtime_anchor::tests`: 2 passed,
  covering same-task multi-attempt concurrent projection, multi-task parallel
  completion, and projection event emission.
- `cargo test -p harnesslab-core --all-features`: 50 passed.
- `cargo test -p harnesslab-adapters --all-features`: 28 passed.
- `cargo test -p harnesslab-cli --all-features --lib`: 121 passed.
- `cargo test -p harnesslab-infra --all-features`: 54 unit tests and 4
  integration tests passed.
- `cargo test -p harnesslab-cli --test replay_contract -- --nocapture`: 12
  passed before the `REPLAY-007` slice.
- `cargo check -p harnesslab-cli --tests`: passed.
- `cargo check --all-targets`: passed.
- `scripts/test-after-change.sh --select META-002`: passed with 43
  requirements, 171 tests, and 16 adapter claims from 3 sources.
- `scripts/test-after-change.sh --select META-008`: passed with active=6 and
  planned=10 adapter proof selectors.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.
- Touched code file line counts checked: `runtime_snapshot.rs` 395,
  `runtime_anchor.rs` 364, `file_lock.rs` 95, `artifact.rs` 161,
  `swe_runtime_snapshot_contract.rs` 488, `tests/support/runtime_snapshot.rs`
  232, `replay.rs` 411, `benchmark.rs` 301, `registry.rs` 250,
  `model_tests.rs` 456; all are below the 500-line repository constraint.
