# Benchmark Adapter Phase 8 Full Gate Closure

## Scope

Phase 8 closes the benchmark adapter architecture track after Phases 4-7
implemented the Terminal-Bench runtime adapter, the SWE-bench Pro runtime
adapter, runtime snapshot/cleanup hardening, and diagnostics/documentation
alignment.

## Current Closure State

- Status: closed locally after final closure review and final full-gate rerun.
- Date: 2026-06-06
- Repository: `<repo-root>`
- Architecture plan: `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`

## Implemented Final-Gate Fixes

- Fixed Rust 1.95 clippy failures in adapter boundary tests, file locking,
  Terminal-Bench command construction, cleanup-report writing, runtime
  snapshot helper ordering, and SWE runtime event assertions.
- Preserved report privacy while restoring explicit missing-command reporting:
  normal reports keep `[PRIVATE_COMMAND]`; resumes with missing `command.txt`
  show `[ORIGINAL_COMMAND_UNAVAILABLE]`.
- Updated the real Terminal-Bench import-timeout verifier so public command
  snapshots must keep `--agent-import-path` redacted while private runtime
  snapshots retain the raw import path needed for replay evidence.
- Added directory-aware runtime material checksums and replay validation so
  unchanged Terminal-Bench dataset directories remain replay-valid and live
  directory drift remains fail-closed.
- Aligned Terminal-Bench public `agent/command.txt` generation with the
  public/private boundary: public command snapshots use report materialization
  and redact runtime setup, command, import-path, and pythonpath material.
- Hardened the adapter selector verifier so it checks the exact Phase 8 active
  and planned inventory, not only selector liveness.
- Closed stale review metadata for prior completed phase reports so Phase 8
  closure does not depend on ambiguous `open` or `in progress` headers.

## Validation Evidence

| Gate | Command | Result |
| --- | --- | --- |
| Full local gate | `CARGO_INCREMENTAL=0 scripts/test-after-change.sh` | passed; final output `PASS scripts/test-after-change.sh` |
| Coverage gate | included in full local gate | line 95.12% (`12284/12914`), branch 78.95% (`1238/1568`) |
| Registry check | included in full local gate | `registry ok: 43 requirements, 171 tests`; `adapter proof claims ok: 16 ids from 3 sources` |
| Python bridge | included in full local gate | `32 passed, 7 subtests passed` |
| Terminal-Bench registered setup | included in full local gate | `PASS terminal-bench registered setup` |
| Terminal-Bench import timeout cleanup | included in full local gate and rerun directly | `PASS terminal-bench import timeout cleanup`; private runtime snapshot import-path proof ok |
| Terminal-Bench import success cleanup | included in full local gate | `PASS terminal-bench import success cleanup` |
| Terminal-Bench docker activity watchdog | included in full local gate | `PASS terminal-bench docker activity watchdog` |
| Terminal-Bench docker activity grace expiry | included in full local gate | `PASS terminal-bench docker activity grace expiry` |
| Active adapter selector guard | `CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh` | exact inventory verified; `adapter selectors ok: active=15 planned=1` |
| Post-helper-split runtime snapshot contract | `cargo fmt --all && cargo test -p harnesslab-cli --test external_runtime_snapshot_contract adapt_runtime_003_external_runtime_snapshots_are_written_and_redacted -- --nocapture` | passed |
| Diff hygiene | `git diff --check` | passed |

## Selector Coverage

The active adapter selector guard reran after the final code changes and now
enforces this exact active-route inventory:

- `ADAPT-DATA-001..005`
- `ADAPT-RUNTIME-001..005`
- `SWEPRO-001..005`

The full local gate also executed the registered `TB-*`, `INT-*`, `SEC-*`,
Python bridge, and real Terminal-Bench verifier coverage through the workspace
test suite and scripts. The planned-only selector remains
`ADAPT-DATA-000`, the retired Phase 0 gap sentinel.

## Rollback And Fallback Readiness

- Rollback unit: this Phase 8 closure commit can be reverted independently of
  Phase 4-7 implementation commits because it mostly contains final-gate
  hardening, verifier updates, and closure metadata.
- Runtime fallback: existing fake benchmark paths and non-external runners
  continue to use their established paths; full gate covered CLI, replay,
  resume, doctor, fake terminal, fake patch, Terminal-Bench, and SWE-bench Pro
  contracts.
- Snapshot fallback: replay remains fail-closed when runtime snapshots are
  missing, redacted legacy snapshots are insufficient, or runtime adapter
  versions drift.
- Public/private artifact fallback: public command snapshots keep sensitive
  setup, command, import-path, pythonpath, and path material redacted; private
  runtime snapshots keep replay-critical raw command material.
- Directory material fallback: runtime snapshot replay validates both files and
  directory-backed live materials with deterministic checksums.
- Operational fallback: if real Terminal-Bench verifier scripts fail in CI due
  environment drift, keep Phase 8 closed only with equivalent manual evidence
  recorded in `/vs_review/`.

## Post-Closure Enhancements

- `tests/TEST_REGISTRY.toml` remains a global registry/config artifact. It is
  accepted outside the single-code-file line-count boundary, but future registry
  sharding can improve maintainability.
- Direct OS-level directory-entry fault injection and direct public material
  checksum mutation are optional hardening beyond the current replay snapshot
  closure.
- Registry `required_artifacts` now have registry-level executable validation
  for safe relative artifact paths, duplicate rejection, and the existing
  `INT-011` exact runtime artifact contract. Future work can still add
  post-test artifact existence checks once selected tests publish artifacts to a
  shared location.

## Final Closure Decision

- Final adversarial review: `vs_review/2026-06-06-benchmark-adapter-phase-8-final-review.md`.
- Remaining follow-up closure review:
  `vs_review/2026-06-07-benchmark-adapter-remaining-closure-review.md`.
- Blocking findings from Round 1: accepted and fixed.
- Round 2 closure review: completed with no remaining code blocker; the only
  accepted blocker was the open closure artifact itself, now fixed.
- Final full-gate rerun passed with line coverage 95.12% (`12284/12914`) and
  branch coverage 78.95% (`1238/1568`).
- Phase 8 closure criteria are satisfied.
