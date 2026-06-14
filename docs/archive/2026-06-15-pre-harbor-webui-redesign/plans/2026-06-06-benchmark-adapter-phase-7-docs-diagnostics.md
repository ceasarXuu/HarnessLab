# Benchmark Adapter Phase 7 Docs And Diagnostics Alignment

## Goal

Align user-facing docs, operator docs, and runtime readiness diagnostics with
the implemented adapter runtime behavior from Phases 4 through 6.

## Implemented Surface

- Runtime preflight events now include `adapter_phase=preflight` alongside
  `adapter_id`, `runner_kind`, `agent_bridge_mode`, `readiness_status`,
  `blocking_reason`, `host_execution_reason`, and compatibility details.
- Runtime preflight blockers now include adapter id, `adapter_phase=preflight`,
  task id, readiness status, blocking reason, and remediation text.
- Development operations docs now distinguish `doctor --json` checks from run
  preflight readiness diagnostics.
- Development operations docs now describe `external-runtime.private.json`,
  `external-runtime.public.json`, and `cleanup-report.json` boundaries.
- Terminal-Bench playbook now states that report HTML does not link raw
  attempt command/log artifacts and that external replay requires attempt-level
  external runtime snapshots.

## Validation Evidence

| Command | Result |
| --- | --- |
| `scripts/test-after-change.sh --select ADAPT-RUNTIME-002` | passed |
| `scripts/test-after-change.sh --select INT-011` | passed, 10 tests |
| `rg "adapter_phase=preflight" crates/harnesslab-cli/src crates/harnesslab-cli/tests docs` | passed |
| Phase 6 artifact-boundary evidence | `ADAPT-RUNTIME-003`, `ADAPT-RUNTIME-004`, and `vs_review/2026-06-06-benchmark-adapter-phase-6-runtime-snapshot-cleanup-review.md` passed/closed before this docs alignment |

## Remaining Scope

- Phase 8 must run the full gate and fresh adversarial closure.
- The 500-line code-file rule is treated as scoped to code and local test
  implementation files for this phase. `tests/TEST_REGISTRY.toml` remains a
  global 3279-line registry/config artifact and is explicitly outside this
  Phase 7 completion boundary. Strict uniform enforcement for registry/config
  artifacts requires a later registry-sharding migration with the existing
  validator retained as the composition gate.
