# Benchmark Adapter Phase 6 Runtime Snapshot And Cleanup Evidence

## Goal

Complete Phase 6 by making Terminal-Bench runtime attempts persist
public/private external runtime snapshots and by replacing cleanup-only event
evidence with a structured per-attempt cleanup report.

## Implemented Surface

- Terminal-Bench attempts now write `external-runtime.private.json` and
  `external-runtime.public.json`; the final snapshot carries post-execution
  material state and cleanup diagnostics.
- Terminal-Bench snapshots use adapter version
  `terminal-bench-runtime.v1`, phase `official_runner`, and replay material
  entries for source dataset, runtime dataset, command snapshot, official
  result, runner logs, and cleanup report. Raw command and runner log files are
  replay/private materials, not Terminal-Bench public artifacts or public
  runtime materials.
- Public snapshots omit private fields such as `dataset_path`, `source_path`,
  `working_dir`, and `redaction_basis`, redact fake secret material from public
  command text, replace public `--run-id` and `--agent-import-path` values with
  placeholders, hide command stdout/stderr artifact paths, and use a stable
  `official/terminal-bench/results.json` public alias instead of a run-derived
  official result path. Public events now write `[PRIVATE_RUN_ID]` in the event
  envelope and use placeholders for Terminal-Bench official run ids. Public
  report HTML now shows `[PRIVATE_RUN_ID]` and no longer links raw command or
  agent/verifier log artifacts.
- Cleanup now records typed `pre_task` and `post_task` private diagnostics with
  phase, required flag, token, success, projects, removed resources,
  container/network counts, and redacted error text.
- Public cleanup surfaces are counts-only: `cleanup-report.json`, cleanup
  events, run-level Docker cleanup warning events, and public runtime
  diagnostics expose phase, required flag, success,
  project/removal/container/network counts, and `has_error`, but not raw run
  ids, scan tokens, Docker project names, removed resource ids, or Docker stderr.
- `cleanup-report.json` records official-vs-final failure provenance and
  `final_verdict_effect`, including `cleanup_overrode_result` when a successful
  official result becomes an execution failure due to cleanup failure.
- Contract coverage includes all current `final_verdict_effect` values:
  `none`, `cleanup_overrode_result`, and `cleanup_warning_only`.
- Replay now blocks external runtime adapter version drift instead of silently
  accepting self-consistent snapshots produced by older adapter semantics.

## Activated Selectors

| Selector | Status | Evidence |
| --- | --- | --- |
| `ADAPT-RUNTIME-003` | active | `scripts/test-after-change.sh --select ADAPT-RUNTIME-003` passed |
| `ADAPT-RUNTIME-004` | active | `scripts/test-after-change.sh --select ADAPT-RUNTIME-004` passed |

## Validation Evidence

| Command | Result |
| --- | --- |
| `scripts/test-after-change.sh --select ADAPT-RUNTIME-003` | passed |
| `scripts/test-after-change.sh --select ADAPT-RUNTIME-004` | passed |
| `scripts/test-after-change.sh --select ADAPT-RUNTIME-005` | passed |
| `scripts/test-after-change.sh --select SWEPRO-005` | passed |
| `cargo test -p harnesslab-cli cleanup_004_cleanup_warning_is_recorded --lib -- --nocapture` | passed |
| `cargo test -p harnesslab-cli --all-features --test terminal_bench_cleanup_contract -- --nocapture` | passed |
| `cargo test -p harnesslab-report -- --nocapture` | passed |
| `cargo test -p harnesslab-infra event::tests -- --nocapture` | passed |
| `cargo test -p harnesslab-cli --all-features --test cli_contract -- --nocapture` | passed |
| `cargo check -p harnesslab-cli` | passed |
| `scripts/verify-test-registry.sh` | passed |
| `scripts/verify-planned-adapter-selectors.sh` | passed; `active=15 planned=1` |

## Gate Result

Phase 6 implementation evidence is green for the current local gate. The only
remaining planned adapter selector is `ADAPT-DATA-000`, which is the retired
gap sentinel and not part of Phase 6 runtime work.

## Remaining Scope

- Phase 7 must align user-facing docs, doctor/readiness diagnostics, event
  names, and artifact names with the implemented Phase 6 behavior.
- Phase 8 must run the full gate and fresh adversarial closure before the
  overall adapter architecture track can be considered complete.
