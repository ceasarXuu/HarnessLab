# Benchmark Adapter Phase 0 Contract Inventory

- Date: 2026-06-04
- Related plan: `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- Phase: Phase 0: Contract Inventory
- Status: Implemented as proof-surface registration. Superseded for data
  adapter behavior by Phase 1 active selectors.

## Phase 0 Baseline Facts

- `crates/harnesslab-adapters/src/registry.rs` exposes `BenchmarkAdapter` with
  only `descriptor()` and `plan(split)`.
- `harnesslab-core` already defines `PreparedBenchmark` and `TaskDescriptor`,
  but the adapter trait does not yet expose independent `inspect_data`,
  `prepare`, `list_tasks`, `create_task_plan`, or `snapshot_task` calls.
- Terminal-Bench and SWE-bench Pro runtime execution still dispatch through
  direct CLI runner branches.
- `crates/harnesslab-cli/tests/external_smoke_contract.rs` contains ten
  `int_011_swe_bench_pro_*` tests, while the previous selector routed
  `INT-011` to only one function.

## Phase 0 Landed Controls

- `ADAPT-DATA-000` was active as a temporary gap sentinel during Phase 0. Phase
  1 retired it back to planned status so it no longer counts as active proof.
- `ADAPT-DATA-001..005` are active as of Phase 1. `ADAPT-RUNTIME-001..005` and
  `SWEPRO-001..005` remain registered in `tests/REQUIREMENTS.toml` with
  `status = "planned"`.
- Matching planned test entries exist in `tests/TEST_REGISTRY.toml`.
- `scripts/test-after-change.sh --select <planned-id>` now has explicit routes
  for every planned adapter ID and fails with a planned-proof message instead
  of silently passing.
- `scripts/test-after-change.sh --select META-008` derives active and planned
  adapter proof IDs from the same claim/registry source used by
  `xtask list-adapter-proof-selectors`, executes the current active adapter
  proof selector, and verifies planned selectors exit with the planned-proof
  message.
- `scripts/test-after-change.sh --select INT-011` now runs the counted
  `int_011_swe_bench_pro` group, requires ten passing tests, and asserts the
  run-level and attempt-level runtime artifacts declared in the shared
  `tests/artifact_contracts/int_011_swe_bench_pro_runtime_artifacts.txt`
  contract.
- `xtask verify-test-registry` now scans the adapter architecture plan and this
  inventory document for claimed `ADAPT-DATA-*`, `ADAPT-RUNTIME-*`, and
  `SWEPRO-*` IDs and fails when any claimed ID is absent from requirements,
  test registry, or selector routing.
- `xtask verify-test-registry` rejects planned status outside the claimed
  adapter inventory, rejects planned adapter selectors that do not route through
  the planned-proof handler, rejects active adapter selectors that do not match
  the exact expected route spec, and verifies the `INT-011` registry artifact
  contract against the shared SWE-bench Pro runtime artifact manifest used by
  the smoke test.

## Remaining Planned Work

- Phase 1 must complete focused adversarial review and close any accepted
  blocking findings.
- Phase 3 must replace the planned `ADAPT-RUNTIME-001..002` routes with real
  runtime registry and preflight tests.
- Phase 5 replaced the planned `SWEPRO-001..004` routes with real SWE-bench Pro
  runtime failure classification tests.
- Phase 6 must replace the planned `ADAPT-RUNTIME-003..004` routes with real
  runtime snapshot, cleanup, redaction, and replay-material tests.
