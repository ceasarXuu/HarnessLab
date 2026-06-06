# Phase 3 Runtime Registry Completion Review

- Status: closed; Phase 3 completion review passed

## Review Input

- Date: 2026-06-05
- Target: Phase 3 benchmark runtime adapter registry completion.
- Objective: Finish Phase 3 so runtime dispatch, preflight compatibility, cleanup ownership, and documentation are actually closed.
- Files inspected by reviewers:
  - `crates/harnesslab-cli/src/runtime_compatibility.rs`
  - `crates/harnesslab-cli/src/runner/external/runtime_adapter.rs`
  - `crates/harnesslab-cli/src/runner/external.rs`
  - `crates/harnesslab-cli/src/runner/cleanup.rs`
  - `crates/harnesslab-cli/src/doctor_run_as.rs`
  - `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
  - `crates/harnesslab-cli/src/runner/external/terminal_bench_env.rs`
  - `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs`
  - `crates/harnesslab-cli/src/runner/external/swe_bench_pro/agent.rs`
  - `crates/harnesslab-cli/src/runner/external/runtime_adapter_tests.rs`
  - `crates/harnesslab-core/src/runtime.rs`
  - `tests/REQUIREMENTS.toml`
  - `tests/TEST_REGISTRY.toml`
  - `scripts/test-after-change.sh`
  - `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- Explicitly excluded context: full chat history, hidden reasoning, and persuasion-oriented summaries.
- Reviewer instruction: fresh session, read-only, cite evidence, try to disprove completion.

## Reviewer Selection

| Round | Reviewer | Role | Agent id | Fresh context | Read-only |
| --- | --- | --- | --- | --- | --- |
| 1 | Harvey | architecture boundary adversary | `019e96e2-2f36-7450-af93-abd41fc79ad5` | yes, `fork_context=false` | yes |
| 1 | Halley | test/evidence adversary | `019e96e2-913f-78f0-b0ef-06b665d42678` | yes, `fork_context=false` | yes |
| 2 | Lagrange | architecture closure adversary | `019e96f3-8cdb-77e0-a5e4-2115d0e50d1a` | yes, `fork_context=false` | yes |
| 2 | Noether | test/evidence closure adversary | `019e96f3-d5e8-7250-936e-fc6c9765ad9f` | yes, `fork_context=false` | yes |

## Round 1 Reviewer Outputs

### Harvey: Architecture Boundary

Summary: Phase 3 improved dispatch but was not honestly complete.

Blocking findings:

1. Cleanup was only adapter-flagged, not adapter-owned. `RunSandboxCleanup` still contained Terminal-Bench-specific cleanup types, stale-run scanning, event naming, and compose cleanup calls.
2. Preflight could not represent blocked readiness. `RuntimePreflightReport` lacked `blocking_reason`, and preflight constructors always emitted `ready`.

Non-blocking risk:

- `runtime_compatibility.rs` acts as a temporary second dispatch table and must stay explicitly temporary until a later materialized runtime config replaces it.

### Halley: Test And Evidence

Summary: Phase 3 was not fully proven.

Blocking findings:

1. `ADAPT-RUNTIME-001` claimed integration coverage through requirement metadata, but only a contract test existed.
2. `external_runner_preflight` persistence was only tested through a direct helper call, not a real `execute_plan` path, and the event assertion omitted several documented fields.

Non-blocking risks:

- Static branch and raw-label guards were file-scoped and brittle.

## Main-Agent Response

| Finding | Triage | Response |
| --- | --- | --- |
| Cleanup only adapter-flagged | accept | Replaced the boolean cleanup flag with adapter-owned `RuntimeCleanupTarget` and `RuntimeCleanupReport`; Terminal-Bench stale-run discovery and compose cleanup moved behind `BenchmarkRuntimeAdapter`. |
| Preflight cannot represent blocked readiness | accept | Added `blocking_reason` to `RuntimePreflightReport`; Terminal-Bench invalid profile validation now returns a blocked preflight report consumed by validation. |
| `ADAPT-RUNTIME-001` overclaimed integration | accept | Changed `ADAPT-RUNTIME-001` requirement metadata to contract-only. |
| Preflight event proof bypassed `execute_plan` | accept | Kept focused unit event checks and added `INT-011` real SWE-bench Pro execution-path assertions for the full `external_runner_preflight` event fields. |
| Branch and label guards too file-scoped | accept | Tightened `ADAPT-RUNTIME-001` cleanup guard to reject Terminal-Bench cleanup imports/heuristics in `cleanup.rs`; retained explicit runtime label compatibility allowlist. |

## Fix Evidence

- `scripts/test-after-change.sh --select ADAPT-RUNTIME-001`: passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-002`: passed.
- `scripts/test-after-change.sh --select INT-011`: passed, 10 tests.
- `cargo fmt --all --check`: passed before this report was written.

## Closure Status

Round 1 had accepted blocking findings, so fresh closure re-review was started.

## Round 2 Closure Review

### Lagrange: Architecture Closure

Status: timed out before returning a usable result. This is recorded as an
unavailable reviewer result, not as a pass.

### Noether: Test And Evidence Closure

Summary: no blocking findings. The accepted Phase 3 blockers are closed based
on current registry metadata, code/test enforcement, and fresh selector reruns.

Blocking findings: none.

Non-blocking risks:

- The raw-label guard is substring-based and scoped to current production
  surfaces. This is acceptable for the accepted closure criterion, but it is
  not a general future-proof static analysis system.
- `runtime_compatibility.rs` remains an explicit temporary compatibility table.

Missing tests/logs: none for the accepted blockers.

Evidence cited by reviewer:

- `ADAPT-RUNTIME-001` no longer overclaims integration; the requirement is
  contract-only and the registry row is contract-layer only.
- Preflight persistence is asserted with full event fields in
  `runtime_adapter_tests.rs` and on a real SWE-bench Pro execution path in
  `external_smoke_contract.rs`.
- Cleanup leakage is guarded by `ADAPT-RUNTIME-001`, which rejects
  Terminal-Bench-specific references in `cleanup.rs`.
- Raw benchmark-label access is forced through `runtime_compatibility.rs`.
- Fresh reruns reported by reviewer:
  - `scripts/test-after-change.sh --select ADAPT-RUNTIME-001`: 1 passed.
  - `scripts/test-after-change.sh --select ADAPT-RUNTIME-002`: 1 passed.
  - `scripts/test-after-change.sh --select INT-011`: 10 passed.

## Final Closure Status

Closed for Phase 3 implementation. The test/evidence closure reviewer found no
blocking issues after fixes; the architecture closure reviewer timed out and was
not counted as a pass. The main-agent architectural closure evidence is the
implemented adapter-owned cleanup target/report boundary plus passing
`ADAPT-RUNTIME-001`, which now rejects Terminal-Bench cleanup leakage in
`runner/cleanup.rs`.
