# Benchmark Adapter Phase 8 Final Review

- Date: 2026-06-06
- Target: Phase 8 full-gate closure for benchmark adapter architecture
- Repository: `/Volumes/XU-1TB-NPM/projects/HarnessLab`
- Review type: final fresh-session adversarial review
- Status: closed after Round 2 documentation fix

## Review Input

Objective:

Verify whether the benchmark adapter architecture track can close after
Phases 4-8. Try to disprove closure using current files, current tests, and the
recorded evidence.

Primary files:

- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-06-benchmark-adapter-phase-8-full-gate-closure.md`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_adapter.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_cleanup.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/report_context.rs`
- `crates/harnesslab-cli/src/runner/store.rs`
- `scripts/verify-terminal-bench-import-timeout-cleanup.sh`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- prior phase reports under `vs_review/2026-06-0*-benchmark-adapter-phase-*.md`

Final-gate evidence:

- `CARGO_INCREMENTAL=0 scripts/test-after-change.sh`: passed with final output
  `PASS scripts/test-after-change.sh`.
- Coverage from full gate: line 95.12% (`12284/12914`), branch 78.95%
  (`1238/1568`).
- Registry from full gate: `registry ok: 43 requirements, 171 tests`;
  `adapter proof claims ok: 16 ids from 3 sources`.
- Python bridge from full gate: `32 passed, 7 subtests passed`.
- Real Terminal-Bench verifier scripts from full gate:
  `PASS terminal-bench registered setup`,
  `PASS terminal-bench import timeout cleanup`,
  `PASS terminal-bench import success cleanup`,
  `PASS terminal-bench docker activity watchdog`, and
  `PASS terminal-bench docker activity grace expiry`.
- `CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh`: passed
  with `adapter selectors ok: active=15 planned=1`.

Known intentional behavior to verify:

- Normal reports display `[PRIVATE_COMMAND]`.
- Reports display `[ORIGINAL_COMMAND_UNAVAILABLE]` when `command.txt` is
  missing; report rendering keeps present originals private with
  `[PRIVATE_COMMAND]`.
- Terminal-Bench command snapshots keep raw import paths redacted.
- Private runtime snapshots keep raw import paths for replay evidence.
- `ADAPT-DATA-000` remains planned-only as the retired Phase 0 gap sentinel.

## Reviewer Selection

| Round | Reviewer | Role | Agent id | Fresh context | Read-only |
| --- | --- | --- | --- | --- | --- |
| 1 | Godel | architecture closure adversary | `019e9a7a-bddd-7213-b4f3-44f406565578` | yes, `fork_context=false` | yes |
| 1 | Jason | test/evidence adversary | `019e9a7a-c094-7163-a4d4-d94c9ba54408` | yes, `fork_context=false` | yes |
| 1 | Arendt | security/redaction adversary | `019e9a7a-c22a-7401-b0a6-ffa176d840ee` | yes, `fork_context=false` | yes |
| 2 | Socrates | architecture/security closure adversary | `019e9aa8-ed80-77f0-94c0-0b1f1d49fe6a` | yes, `fork_context=false` | yes |
| 2 | Maxwell | security closure adversary | `019e9aa8-ef89-7061-817e-532e5930c87c` | yes, `fork_context=false` | yes; timed out before final response |

## Round 1 Reviewer Outputs

### Godel: Architecture Closure

Summary: runtime dispatch and cleanup ownership are behind the adapter
registry, but Phase 8 could not close because Terminal-Bench replay authority
was structurally broken for directory-backed runtime materials, and this final
review report was not yet closed.

Blocking findings:

1. Terminal-Bench replay authority was not closed. Terminal-Bench runtime
   snapshots recorded dataset roots as live replay materials, while replay
   validation only accepted file-backed live materials.
2. The final `/vs_review/` closure report was still open.

Medium findings:

- The closure note overstated `[ORIGINAL_COMMAND_UNAVAILABLE]` as resume-only;
  implementation shows the sentinel for any report whose `command.txt` is
  missing.
- Add targeted tests for the new runtime redaction refs.

### Jason: Test And Evidence

Summary: requirement and registry wiring were present, but Phase 8 could not
close because the final closure artifact was still open and the expanded
runtime redaction refs were not yet proven by tests.

Blocking findings:

1. The expanded `runtime_redaction_refs` contract was not yet proven for
   `terminal_bench_agent_pythonpath`, profile/report command strings, or
   materialized setup script text.
2. The final adversarial closure record was not completed.

Medium findings:

- The selector guard proved liveness but not exact `ADAPT-DATA-*`,
  `ADAPT-RUNTIME-*`, and `SWEPRO-*` inventory.
- Registry `required_artifacts` are declarative metadata and are not directly
  interpreted by the selector runner.
- `META-008` is infrastructure proof, not direct adapter-runtime behavior
  proof.

### Arendt: Security And Redaction

Summary: `external-runtime.public.json` was hardened, but public
`agent/command.txt` still rendered Terminal-Bench command env using runtime
materialized setup data.

Blocking findings:

1. Terminal-Bench public `agent/command.txt` could expose runtime-only setup
   material because public command rendering used `ctx.materialized_profile`
   instead of `ctx.report_materialized_profile`, and its redaction refs did
   not cover setup/command/pythonpath material.

## Main-Agent Triage

| Reviewer | Finding | Severity | Decision | Action Taken | Verification |
| --- | --- | --- | --- | --- | --- |
| Godel | Directory-backed Terminal-Bench replay materials fail replay validation. | blocking | accept | Added `stable_path_checksum`, material `kind`, directory-aware snapshot checksums, and replay validation for `file` and `directory` live materials. | `ADAPT-RUNTIME-003` now checks directory material kinds and unchanged Terminal-Bench replay success; full gate passed. |
| Godel / Jason | Final Phase 8 review report still open. | blocking | accept | Populated Round 1 outputs, triage, validation evidence, and launched Round 2 closure review. | Round 2 found no remaining code blocker; the closure artifact itself is now closed. |
| Arendt | Public `agent/command.txt` can expose runtime-only setup material. | blocking | accept | `terminal_bench_command` now receives explicit materialized profile; public report command uses `ctx.report_materialized_profile`; command redaction refs now include runtime/report command, setup scripts, import path, and pythonpath. | `ADAPT-RUNTIME-003` checks public command/runtime snapshot redaction for import path, pythonpath, setup command, and agent command; full gate passed. |
| Jason | Expanded runtime redaction refs lacked executable proof. | blocking | accept | Added `ADAPT-RUNTIME-003` assertions that `external-runtime.public.json` and public `agent/command.txt` do not contain raw import path, pythonpath, setup command, or agent command. | `ADAPT-RUNTIME-003` and full gate passed. |
| Godel | `[ORIGINAL_COMMAND_UNAVAILABLE]` scope overstated as resume-only. | medium | accept | Updated closure wording to reports with missing `command.txt`. | Documentation update. |
| Jason | Selector guard did not enforce exact inventory. | medium | accept | Added exact active/planned inventory checks to `scripts/verify-planned-adapter-selectors.sh`. | `CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh` passed with `active=15 planned=1`. |
| Jason | `required_artifacts` are metadata, not selector-runner interpreted. | medium | accept | Documented that artifact requirements are traceability/test-owned assertions and future selector-layer executable artifact checks remain follow-up. | Documentation update. |
| Jason | `META-008` is infrastructure proof only. | low | accept | Kept Phase 8 evidence anchored on full gate, active selector guard, and concrete adapter tests rather than treating `META-008` as direct behavior proof. | Documentation update. |

## Post-Fix Verification

- `cargo test -p harnesslab-cli --test external_runtime_snapshot_contract adapt_runtime_003_external_runtime_snapshots_are_written_and_redacted -- --nocapture`: passed.
- `CARGO_INCREMENTAL=0 scripts/test-after-change.sh --select ADAPT-RUNTIME-003`: passed.
- `CARGO_INCREMENTAL=0 scripts/test-after-change.sh --select ADAPT-RUNTIME-005`: passed.
- `CARGO_INCREMENTAL=0 scripts/test-after-change.sh --select SWEPRO-005`: passed.
- `CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh`: passed with exact inventory and `adapter selectors ok: active=15 planned=1`.
- `CARGO_INCREMENTAL=0 scripts/test-after-change.sh`: passed with final output `PASS scripts/test-after-change.sh`; coverage line 95.12% (`12284/12914`), branch 78.95% (`1238/1568`).
- `cargo fmt --all && cargo test -p harnesslab-cli --test external_runtime_snapshot_contract adapt_runtime_003_external_runtime_snapshots_are_written_and_redacted -- --nocapture`: passed after the helper split that kept the contract test file below 500 lines.
- `git diff --check`: passed.

## Round 2 Closure Review

Socrates reported one remaining blocker: this final closure artifact was still
unclosed and still contained incomplete Round 2 fields. Socrates did not find a
remaining blocker on the code fixes from Round 1:

- deterministic directory checksums and directory-aware replay validation are in
  place;
- public Terminal-Bench command and runtime snapshots redact setup, command,
  import-path, pythonpath, and runtime material;
- exact selector inventory is enforced by
  `scripts/verify-planned-adapter-selectors.sh`;
- full-gate evidence is recorded.

Accepted Round 2 documentation findings:

- The closure artifact itself must be closed before Phase 8 can proceed. This
  report now carries a closed status, populated Round 2 reviewer records,
  post-fix validation evidence, and a positive closure decision.
- The `[ORIGINAL_COMMAND_UNAVAILABLE]` wording must describe the real behavior:
  the sentinel is used when `command.txt` is missing; present original commands
  remain private in reports as `[PRIVATE_COMMAND]`.

Maxwell did not return within the allotted wait window. To avoid letting a
non-returning subagent block the main work, the closure decision is based on the
completed fresh Round 2 adversarial result plus the executable validation gates
above.

## Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes; selector-layer executable artifact
  requirements remain future hardening
- Blocked reason: n/a
- Allowed to proceed: yes
