# Universal Benchmark Adapter Protocol Phase 0 Branch Inventory

- Created: 2026-06-08
- Updated: 2026-06-08
- Source plan: `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`
- Phase: 0, Discovery And Branch Inventory
- Status: Implemented, pending closure review

## 1. Purpose

This inventory freezes the current benchmark-specific coupling before protocol
migration starts. The target architecture requires benchmark-specific behavior
to stop at adapter-owned modules; generic run, replay, doctor, report, registry,
and selector flows must not branch on concrete benchmark identity after the
protocol migration.

## 2. Entry Evidence

| Check | Command | Result |
|---|---|---|
| Clean worktree before Phase 0 edits | `git status --short --branch` | `## main...origin/main` |
| Latest plan commits present | `git log --oneline -3` | `de0e1b1 Plan universal benchmark adapter protocol`; `16b68e0 Define universal benchmark adapter protocol PRD`; `ad4a93a Fix benchmark adapter blocker closure` |
| Current adapter selector baseline | `CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh` | passed; `adapter selectors ok: active=16 planned=1` |

## 3. Scan Commands

The current benchmark-aware inventory was built from these scans:

```bash
rg -l "ExternalRunnerKind|TerminalBench|SweBenchPro|terminal-bench|swe-bench-pro|TERMINAL_BENCH|SWE_BENCH|terminal_bench|swe_bench" crates tests scripts xtask integrations | sort
rg -n "ExternalRunnerKind|TerminalBench|SweBenchPro|terminal-bench|swe-bench-pro|TERMINAL_BENCH|SWE_BENCH|terminal_bench|swe_bench" crates tests scripts xtask integrations
rg -n 'id = "(ADAPT-DATA|ADAPT-RUNTIME|SWEPRO|TB|INT|SEC)-' tests/TEST_REGISTRY.toml
```

The file-level scan currently returns 80 files. Every matched file is assigned
to one of the dispositions below.

## 4. Disposition Legend

| Disposition | Meaning | Migration Rule |
|---|---|---|
| Adapter-owned acceptable | Benchmark-specific behavior is already in an adapter-owned implementation surface. | Keep behavior local, then express it through protocol contracts. |
| Registry metadata acceptable | Central registry/test metadata names benchmark ids but should not contain behavior switches. | Keep metadata; forbid behavior branching. |
| Generic-layer leak | Generic orchestration, replay, doctor, report, or compatibility logic knows concrete benchmark identity. | Remove or replace with protocol/capability dispatch. |
| Legacy compatibility shim | Serialized legacy enum or kind field must remain readable during migration. | Isolate in a named shim; fail closed on mixed authority. |
| Official preservation evidence | Benchmark-specific verifier or integration script proves existing behavior. | Keep as frozen regression evidence, not generic dispatch logic. |
| Test-only assertion | Tests intentionally assert current behavior. | Preserve until equivalent protocol tests replace them. |

## 5. Core Serialization And Authority

| File | Current Coupling | Disposition | Required Migration |
|---|---|---|---|
| `crates/harnesslab-core/src/benchmark.rs` | `ExternalRunnerSpec.kind` serializes `ExternalRunnerKind::TerminalBench` and `ExternalRunnerKind::SweBenchPro`. | Legacy compatibility shim | Add `AdapterProtocolAuthority` / `TaskRuntimeBinding`; dual-write only during migration; old-only runs route through named legacy shim. |
| `crates/harnesslab-core/src/runtime.rs` | `RuntimePreflightReport.runner_kind` serializes runner kind. | Legacy compatibility shim | Add protocol identity fields and make `runner_kind` compatibility-only. |
| `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs` | Persists `runner_kind` in runtime snapshots. | Legacy compatibility shim | Persist canonical protocol authority; reject mixed old/new authority sets. |
| `crates/harnesslab-cli/src/runner/external/runtime_anchor.rs` | Anchors runtime snapshots that currently include external runner fields. | Legacy compatibility shim | Anchor protocol authority and declare drift policy by adapter id/version. |
| `crates/harnesslab-cli/src/runner/store.rs` | Stores/reloads run artifacts that include external runner snapshots. | Legacy compatibility shim | Ensure protocol authority survives store/load and report snapshot paths. |

### Serialized Authority Field Matrix

| Surface | Current Serialized Authority Fields | Current Validator / Fingerprint Use | Protocol Migration Rule |
|---|---|---|---|
| Benchmark plan / task plan | `benchmark.name`, `benchmark.version`, `split`, `external_runner.kind`, task `external_runner` spec | `validate_benchmark_plan` checks benchmark/split/task snapshot alignment. | Add canonical authority to task runtime binding; keep legacy `external_runner.kind` only behind old-run shim. |
| Runtime preflight | `runner_kind` | Doctor/readiness and compatibility branches consume kind-specific expectations. | Replace with adapter-declared readiness probe authority. |
| Task runtime snapshot | `benchmark`, `split`, `task_id`, `source_ref`, `external_runner`, `external_runtime_attempts` | Replay blocks missing/mismatched task runtime snapshots. | Persist protocol authority per task and fail closed on mixed task/attempt authority. |
| External runtime private snapshot | `benchmark`, `task_id`, `attempt`, `runner_kind`, `adapter_version`, `runtime_policy`, `dataset_path`, `source_path`, `commands`, `replay_materials`, `public_artifacts`, `runtime_diagnostics` | Replay validates benchmark/task/attempt/kind/version/policy/materials and fingerprints the private authority. | Replace kind/version lookup with `AdapterProtocolAuthority`; keep dataset/source private-only and adapter-declared. |
| External runtime public snapshot | `benchmark`, `task_id`, `attempt`, `runner_kind`, `adapter_version`, `runtime_policy`, `commands`, `runtime_materials`, `public_artifacts`, `runtime_diagnostics`, fingerprints | Replay validates public/private agreement and rejects private-only dataset/source fields in public output. | Dual-write only during migration; public authority must match private authority and omit private paths. |
| External runtime anchor | `private_path`, `public_path`, checksums, `runtime_fingerprint`, `public_fingerprint` | Replay compares anchor paths, checksums, and fingerprints before reuse. | Anchor protocol authority fingerprints and reject mismatched old/new anchor sets. |
| Runtime fingerprints | `schema_version`, `benchmark`, `task_id`, `attempt`, `runner_kind`, `adapter_version`, `runtime_policy`, commands/materials/artifacts/diagnostics | Replay recomputes runtime/public fingerprints from stored JSON. | Include canonical protocol authority in fingerprints before removing legacy fields. |

## 6. Generic-layer Leaks To Remove

| File | Current Coupling | Required Replacement |
|---|---|---|
| `crates/harnesslab-cli/src/runtime_compatibility.rs` | Branches on `ExternalRunnerKind` for host execution, bridge mode, and consumed labels. | Adapter-declared readiness/capability schema. |
| `crates/harnesslab-cli/src/doctor_run_as.rs` | Doctor checks know Terminal-Bench and SWE-bench Pro run-as behavior. | Generic doctor consumes adapter readiness probes. |
| `crates/harnesslab-cli/src/runner/replay.rs` | Validates runtime adapter version through `runner.kind`. | Validate `AdapterProtocolAuthority` and adapter-declared replay materials. |
| `crates/harnesslab-cli/src/runner/external/runtime_adapter.rs` | Closed enum match dispatches runtime adapters. | `TaskRuntimeBinding.adapter_id` registry dispatch. |
| `crates/harnesslab-cli/src/runner/external.rs` | Generic external module declares concrete benchmark modules and exposes `runtime_adapter_version(kind)`. | Move concrete modules behind adapter registry; expose protocol authority lookup. |
| `scripts/test-after-change.sh` | Selector routing hard-codes current benchmark proof ids and behavior selectors. | Keep as test registry executor but add protocol selector routing and prevent selector weakening. |
| `scripts/verify-planned-adapter-selectors.sh` | Hard-codes active `ADAPT-*` / `SWEPRO-*` inventory. | Extend or replace with protocol selector inventory guard. |
| `tests/FROZEN_SELECTOR_MANIFEST.toml` | Locks current benchmark selector commands, artifacts, contracts, file patterns, and expected counts. | Keep as Phase 0 weakening guard until protocol selectors supersede each row. |
| `xtask/src/frozen_selector_ids.rs` | Locks the independent 86-id baseline for frozen selector guard coverage, including the first active protocol selectors. | Keep as guard-owned baseline; changes require review because registry/manifest co-deletion must not pass. |
| `xtask/src/adapter_claims.rs` | Hard-codes adapter proof prefixes, routes, and file patterns. | Add `ADAPT-PROTOCOL-*` claim extraction and protocol route validation. |
| `xtask/src/frozen_selectors.rs` | Machine-enforces the Phase 0 frozen selector manifest. | Extend manifest families when protocol rows are introduced; do not weaken without equivalent replacement. |
| `xtask/src/adapter_claims_tests.rs` | Tests current hard-coded claim behavior. | Add protocol claim positive/negative tests. |
| `xtask/src/runtime_artifacts.rs` | Validates current SWE runtime artifact contract. | Add versioned artifact declaration validation. |

## 7. Adapter-owned Current Behavior

These files may contain benchmark-specific behavior, but that behavior must stay
inside adapter-owned protocol implementation after migration.

| Files | Current Role | Migration Disposition |
|---|---|---|
| `crates/harnesslab-adapters/src/terminal_bench.rs`, `crates/harnesslab-adapters/src/swe_bench_pro.rs` | Data adapter behavior and task plan creation. | Convert to protocol data lifecycle implementation. |
| `crates/harnesslab-cli/src/runner/external/terminal_bench*.rs` | Terminal-Bench runtime, timeout, env, result, cleanup, and snapshot behavior. | Keep benchmark logic adapter-owned; expose through runtime/artifact/failure schemas. |
| `crates/harnesslab-cli/src/runner/external/swe_bench_pro*.rs` | SWE-bench Pro metadata, workspace, agent, runtime snapshot, evaluator, and adapter behavior. | Keep benchmark logic adapter-owned; expose through protocol phases and replay materials. |
| `crates/harnesslab-cli/src/runner/external/log_scan.rs` | Benchmark-specific infra log interpretation. | Move under adapter-owned failure mapping or capability-specific parser. |
| `integrations/terminal_bench/harnesslab_tb_agent.py`, `integrations/terminal_bench/harnesslab_tb_process.py`, `integrations/terminal_bench/harnesslab_tb_ps.py`, and tests under `integrations/terminal_bench/` | Terminal-Bench Python bridge, process snapshot, and process cleanup adapter support. | Keep as Terminal-Bench adapter-owned runtime bridge; freeze through `PY-TB-001`, `AGT-REG-005`, and `AGT-REG-012`. |
| `integrations/terminal_bench/conftest.py` | Terminal-Bench Python test compatibility stubs. | Test-support only; keep with Python bridge fixtures. |

## 8. Registry Metadata Surfaces

| File | Current Coupling | Migration Disposition |
|---|---|---|
| `crates/harnesslab-adapters/src/registry.rs` | Builds concrete descriptor list and string-matches benchmark names to concrete adapters. This is behavior dispatch, not metadata-only. | Treat as a generic-layer leak until replaced by descriptor/binding metadata lookup with no behavior switch outside adapter-owned modules. |
| `tests/REQUIREMENTS.toml` | Registers current requirement ids. | Add planned/active `ADAPT-PROTOCOL-*` ids. |
| `tests/TEST_REGISTRY.toml` | Registers current selector commands, files, artifacts. | Freeze current rows and add protocol rows; do not weaken existing rows without equivalent replacement. |
| `crates/harnesslab-adapters/src/lib.rs` | Exposes concrete built-in adapter modules. | Acceptable while in-repo; future protocol module should expose descriptors/bindings. |

## 9. Official Preservation Evidence

| File | Current Role | Migration Disposition |
|---|---|---|
| `scripts/verify-terminal-bench-python-adapter.sh` | Terminal-Bench Python adapter preservation. | Keep as stable promotion evidence. |
| `scripts/verify-terminal-bench-registered-setup.sh` | Terminal-Bench official registered setup proof. | Keep as frozen regression selector evidence. |
| `scripts/verify-terminal-bench-import-success-cleanup.sh` | Import-agent cleanup success proof. | Keep as frozen regression selector evidence. |
| `scripts/verify-terminal-bench-import-timeout-cleanup.sh` | Import-agent timeout cleanup proof. | Keep as frozen regression selector evidence. |
| `scripts/verify-terminal-bench-docker-activity-watchdog.sh` | Docker activity watchdog proof. | Keep as frozen regression selector evidence. |
| `scripts/verify-terminal-bench-docker-activity-grace-expiry.sh` | Docker stale-activity grace proof. | Keep as frozen regression selector evidence. |
| `tests/artifact_contracts/int_011_swe_bench_pro_runtime_artifacts.txt` | SWE runtime artifact contract. | Supersede with versioned artifact declaration schema after protocol migration. |

## 10. Test-only Current Behavior

The following matched files are current behavior tests or test support. They
must remain green until equivalent or stronger protocol tests replace them:

- `crates/harnesslab-adapters/src/data_contract_tests.rs`
- `crates/harnesslab-adapters/src/swe_bench_pro_tests.rs`
- `crates/harnesslab-adapters/src/terminal_bench_tests.rs`
- `crates/harnesslab-cli/src/runner/cleanup_tests.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_adapter_tests.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `crates/harnesslab-cli/tests/agent_registry_contract.rs`
- `crates/harnesslab-cli/tests/benchmark_contract.rs`
- `crates/harnesslab-cli/tests/cli_contract.rs`
- `crates/harnesslab-cli/tests/doctor_contract.rs`
- `crates/harnesslab-cli/tests/doctor_run_as_contract.rs`
- `crates/harnesslab-cli/tests/external_runtime_error_contract.rs`
- `crates/harnesslab-cli/tests/external_runtime_snapshot_contract.rs`
- `crates/harnesslab-cli/tests/external_smoke_contract.rs`
- `crates/harnesslab-cli/tests/support/external_runtime_assertions.rs`
- `crates/harnesslab-cli/tests/support/swe.rs`
- `crates/harnesslab-cli/tests/support/terminal_bench.rs`
- `crates/harnesslab-cli/tests/swe_env_contract.rs`
- `crates/harnesslab-cli/tests/swe_runtime_phase_contract.rs`
- `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs`
- `crates/harnesslab-cli/tests/task_snapshot_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_activity_grace_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_cleanup_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_redaction_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_run_as_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_runtime_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_runtime_event_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_setup_failure_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_watchdog_contract.rs`

## 11. Files With No Phase 0 Migration Action

| File | Reason |
|---|---|
| `crates/harnesslab-core/src/agent_profile_reference.rs` | Match comes from benchmark-specific label strings; owned by Phase 4 readiness/schema migration because label allowlists feed compatibility and doctor output. |
| Documentation files under `docs/plans/` | Excluded from no-branch behavior guard; they describe migration state. |

## 12. Exit Criteria Status

| Exit Criterion | Status | Evidence |
|---|---|---|
| No untriaged benchmark-specific branch remains in non-adapter layers. | Met for Phase 0 inventory | All 80 matched files are assigned to a disposition category above, including `integrations/terminal_bench`. |
| Compatibility risks classified before code changes begin. | Met for Phase 0 inventory | Core serialization, runtime snapshots, replay, and registry dispatch are identified as legacy shim or generic-layer leak work. |
| Baseline selector guard is green. | Met | `scripts/verify-planned-adapter-selectors.sh` passed with `active=16 planned=1`. |
| Frozen selector weakening guard exists. | Met | `cargo run -p xtask -- verify-frozen-selector-manifest` passes over `tests/FROZEN_SELECTOR_MANIFEST.toml`. |

## 13. Next Required Work

- Add `AdapterProtocolAuthority` and `TaskRuntimeBinding` only after the legacy
  shim and mixed-authority fixture strategy are implemented.
- Implement no-branch guard before changing generic runner/replay/doctor/report
  logic.
- Use the frozen selector manifest before any selector remapping.
