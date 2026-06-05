# Subagent VS Review: Benchmark Adapter Phase 6 Runtime Snapshot And Cleanup

- Created: 2026-06-06T00:00:00+08:00
- Updated: 2026-06-06T07:30:29+08:00
- Report schema: adversarial-v1
- Task: Complete remaining benchmark adapter phases 4 through 8; this review covers Phase 6 runtime snapshot and cleanup-report implementation.
- Report path: `vs_review/2026-06-06-benchmark-adapter-phase-6-runtime-snapshot-cleanup-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: closed; Phase 6 closure review passed

## Round 1: Phase 6 Implementation Review

### Review Input

#### Objective

Review the current Phase 6 implementation for Terminal-Bench external runtime
snapshots, public/private redaction boundaries, structured cleanup reports, and
test/registry evidence.

#### Review Target

Code implementation, integration tests, test registry routing, and Phase 6
documentation.

#### Target Locations

- `crates/harnesslab-cli/src/runner/external/terminal_bench_runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_cleanup.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_adapter.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/replay.rs`
- `crates/harnesslab-cli/tests/external_runtime_snapshot_contract.rs`
- `scripts/test-after-change.sh`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `xtask/src/adapter_claims.rs`
- `docs/plans/2026-06-06-benchmark-adapter-phase-6-runtime-snapshot-cleanup.md`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`

#### Change Introduction

Terminal-Bench now writes `external-runtime.private.json` and
`external-runtime.public.json` through a dedicated runtime snapshot writer.
The adapter writes a pre-execution snapshot and the executor rewrites it after
runner execution and cleanup evidence are available. Cleanup now returns typed
`TaskCleanupOutcome` records and writes `cleanup-report.json` with phase
outcomes, official-vs-final failure provenance, and `final_verdict_effect`.
`ADAPT-RUNTIME-003` and `ADAPT-RUNTIME-004` have been switched from planned
routes to active integration tests.

#### Risk Focus

- Public runtime snapshots, events, reports, or cleanup artifacts leaking
  private paths, setup secrets, or command material.
- Snapshot/replay material semantics that pass the new tests but cannot support
  real replay validation or diagnostics.
- Cleanup-report logic incorrectly representing official-vs-final verdict
  provenance, especially when cleanup failures overlap with runner failures.
- Test registry routes that appear active but do not cover the implemented
  artifact or failure mode.
- Maintenance risk from adding cleanup/report responsibilities in the wrong
  layer.

#### Assumptions To Attack

- Writing a pre-execution snapshot and then rewriting it after execution is
  safe for replay anchors and partial-failure diagnostics.
- `cleanup-report.json` contains enough evidence to explain final verdict
  changes.
- The public snapshot redaction refs cover all path and fake-secret leakage
  points introduced by Terminal-Bench command construction.
- `ADAPT-RUNTIME-003` and `ADAPT-RUNTIME-004` prove behavior rather than only
  asserting implementation details.
- Existing `ADAPT-RUNTIME-005` and `SWEPRO-005` coverage still protects event
  taxonomy and SWE replay behavior after the Phase 6 changes.

#### Adversarial Lenses

- implementation
- security
- testing
- observability
- replay/data consistency
- maintainability

#### Verification Status

- `scripts/test-after-change.sh --select ADAPT-RUNTIME-003` passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-004` passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-005` passed.
- `scripts/test-after-change.sh --select SWEPRO-005` passed.
- `scripts/verify-test-registry.sh` passed.
- `scripts/verify-planned-adapter-selectors.sh` passed with `active=15 planned=1`.
- `cargo check -p harnesslab-cli` passed.
- Code file line counts remain below 500 lines.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Try to falsify the implementation and tests; do not merely summarize them.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| complex | 10 minutes | one bounded extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Multi-module execution path changes touch snapshot timing, cleanup state, and final result construction. | correctness, state, replay consistency |
| security-adversary | Phase 6 explicitly guards public/private artifact boundaries and fake-secret scans. | secrets, private paths, trust boundary |
| test-validity-adversary | New selectors were activated and must prove behavior rather than only implementation shape. | coverage, false confidence, registry proof |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | multi_agent_v1.spawn_agent / code-reviewer | 019e9980-57d9-7be2-87dc-83248ed0abac | spawn_agent result nickname=Boole | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| security-adversary | multi_agent_v1.spawn_agent / security-reviewer | 019e9980-8f58-70e0-b842-3ffaecd070f1 | spawn_agent result nickname=Cicero | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| test-validity-adversary | multi_agent_v1.spawn_agent / test-engineer | 019e9980-c405-7e71-bb5a-5ef3abb2654d | spawn_agent result nickname=Turing | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| R1-implementation | implementation-adversary | 1 | 019e9980-57d9-7be2-87dc-83248ed0abac | <10m | completed | reviewer returned blocking findings | completed |
| R1-security | security-adversary | 1 | 019e9980-8f58-70e0-b842-3ffaecd070f1 | <10m | completed | reviewer returned blocking findings | completed |
| R1-test-validity | test-validity-adversary | 1 | 019e9980-c405-7e71-bb5a-5ef3abb2654d | <10m | completed | reviewer returned blocking findings | completed |

### Reviewer Outputs

#### R1-implementation

##### Summary

Spec compliance did not pass. The reviewer found two blocking gaps: runtime
snapshots lacked cleanup provenance required by the Phase 6 plan, and replay did
not detect external runtime adapter version drift.

##### Blocking Findings

- Phase 6 runtime snapshots were missing cleanup provenance fields.
  - Broken assumption: activating `ADAPT-RUNTIME-003` meant `external-runtime.private.json` and `external-runtime.public.json` satisfied the Phase 6 snapshot schema.
  - Failure scenario: cleanup override or warning exists only in `cleanup-report.json`; snapshot-only consumers lose official-vs-final provenance.
  - Trigger condition: any Terminal-Bench attempt with cleanup warning or override.
  - Impact: replay/diagnostic consumers relying on runtime snapshots alone lose required authority.
  - Proof needed: add cleanup diagnostics to snapshots and assert them.
- Replay did not detect runtime adapter version drift.
  - Broken assumption: Phase 6 replay hardening warned or blocked older adapter/runtime schema snapshots.
  - Failure scenario: replay consumes self-consistent snapshots with an old `adapter_version`.
  - Trigger condition: source run created by older incompatible adapter version.
  - Impact: replay silently uses stale runtime semantics.
  - Proof needed: compare stored adapter version with current policy and test fail-closed behavior.

##### Non-blocking Risks

- Cleanup failure outcomes dropped known cleanup targets and partial-removal evidence.
  - Broken assumption: failure outcomes preserved enough audit context.
  - Failure scenario: project discovery succeeds, cleanup fails, report stores empty projects.
  - Trigger condition: cleanup failure after target discovery.
  - Impact: weaker remediation and postmortem detail.
  - Proof needed: preserve matched projects on failure.

##### Required Fixes

- Add snapshot cleanup provenance fields or narrow the claim.
- Add replay adapter-version drift handling and a contract test.
- Preserve matched cleanup projects on error.

##### Missing Tests

- Snapshot tests did not assert cleanup-token/verdict provenance.
- No replay contract covered adapter-version drift.
- No failure-path test asserted cleanup-report target preservation.

##### Missing Logs / Observability

- No explicit event distinguishes pre-execution snapshot writes from post-execution rewrites.

##### Evidence

- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/replay.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_cleanup.rs`
- `crates/harnesslab-cli/tests/external_runtime_snapshot_contract.rs`

#### R1-security

##### Summary

Security review found two blocking public-boundary leaks: `agent/command.txt`
was declared public while still pathful, and Terminal-Bench events/report output
still exposed absolute local paths.

##### Blocking Findings

- `agent/command.txt` was declared public but leaked local paths and import-path material.
  - Broken assumption: secret-only redaction made command snapshots public-safe.
  - Failure scenario: public artifact export includes `--dataset-path`, `--output-path`, or `--agent-import-path`.
  - Trigger condition: any Terminal-Bench run, especially import-path agents.
  - Impact: host filesystem and trust-boundary details leak.
  - Proof needed: command snapshot path/import redaction test.
- `events.jsonl` recorded absolute dataset/runtime/output/result paths.
  - Broken assumption: operational path details in events were safe public diagnostics.
  - Failure scenario: public event logs or rendered reports expose local paths.
  - Trigger condition: normal success, qemu dataset prep, runner config, run finish.
  - Impact: public boundary is defeated outside `external-runtime.public.json`.
  - Proof needed: scan events/report for dataset, run, attempt, output paths.

##### Non-blocking Risks

- Cleanup report/events serialized raw Docker project/resource/error strings.
- Replay blocker messages could expose private live-material paths.

##### Required Fixes

- Stop declaring raw command/logs as Terminal-Bench public artifacts or provide public-safe variants.
- Remove absolute paths from event/report messages.
- Redact cleanup report/event fields.
- Redact replay blocker path material or avoid exposing private path values.

##### Missing Tests

- No command path/import redaction test.
- No event/report local-path scan.
- No cleanup report/event Docker string redaction fixture.
- No replay blocker path redaction fixture.

##### Missing Logs / Observability

- No redaction audit signal for public artifacts.

##### Evidence

- `crates/harnesslab-cli/src/runner/external/terminal_bench_runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_runtime.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/report_context.rs`

#### R1-test-validity

##### Summary

Test-validity review found that the selectors were active but did not prove the
full claimed behavior: snapshot lifecycle/coverage, public artifact redaction,
and cleanup-report semantic variants were under-tested.

##### Blocking Findings

- `ADAPT-RUNTIME-003` did not prove the documented two-write lifecycle and missed `terminal_bench.rs` in file patterns.
  - Broken assumption: final snapshot assertions proved pre-execution and post-execution writes.
  - Failure scenario: post-run rewrite regresses but final presence assertions stay too weak.
  - Trigger condition: removing post-run write or stale material state.
  - Impact: stale runtime evidence can pass the selector.
  - Proof needed: either prove lifecycle or narrow the claim, and register real write sites.
- Redaction proof did not cover actual public artifact surface.
  - Broken assumption: fake secret scans covered declared public artifacts.
  - Failure scenario: runner/verifier logs leak secrets while still listed as public artifacts.
  - Trigger condition: inherited env or path output in raw logs.
  - Impact: public artifact leak with green Phase 6.
  - Proof needed: scan every declared public artifact or remove raw logs from public set.
- `ADAPT-RUNTIME-004` proved only one cleanup-report outcome.
  - Broken assumption: one success-to-cleanup-failure case proved report semantics.
  - Failure scenario: `cleanup_warning_only` or `none` effects drift from task results.
  - Trigger condition: benchmark/parse/timeout overlap cases.
  - Impact: audit artifact becomes unreliable.
  - Proof needed: assert multiple `final_verdict_effect` cases.

##### Non-blocking Risks

- Registry layer expectations are not enforced by the meta gate.
- Terminal-Bench replay/drift behavior is still less direct than SWE replay coverage.

##### Required Fixes

- Strengthen or narrow `ADAPT-RUNTIME-003`; add `terminal_bench.rs` file pattern.
- Scan declared Terminal-Bench public artifacts and exclude raw logs if private.
- Expand `ADAPT-RUNTIME-004` cleanup-report effect coverage.

##### Missing Tests

- Snapshot rewrite/lifecycle proof or narrowed doc claim.
- Public artifact scan over command/log/report/event surfaces.
- Cleanup `cleanup_warning_only` and `none` assertions.
- Terminal-Bench replay tamper suite remains a future strengthening item.

##### Missing Logs / Observability

- No explicit snapshot lifecycle event.
- No event records cleanup-report publication/effect.

##### Evidence

- `crates/harnesslab-cli/tests/external_runtime_snapshot_contract.rs`
- `tests/TEST_REGISTRY.toml`
- `xtask/src/adapter_claims.rs`
- `docs/plans/2026-06-06-benchmark-adapter-phase-6-runtime-snapshot-cleanup.md`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation | Missing cleanup provenance in runtime snapshots | Snapshot-only consumers lose cleanup official-vs-final provenance | blocking | accept | `runtime_diagnostics` added to public/private snapshots | Added Terminal-Bench post-execution diagnostics with cleanup phases and official/final failure snapshots; `ADAPT-RUNTIME-003/004` assert them | closure review |
| implementation | Replay adapter version drift not detected | Self-consistent old snapshots replay silently | blocking | accept | `crates/harnesslab-cli/src/runner/replay.rs` now compares stored vs current adapter version | Added fail-closed blocker and `SWEPRO-005` adapter-version drift case | closure review |
| implementation | Cleanup failure drops matched projects | Error report loses discovered cleanup targets | medium | accept | `cleanup_task_resources` now discovers projects before cleanup and redacts/preserves them in error outcome | `ADAPT-RUNTIME-004` asserts failed post-task projects are preserved | none |
| security | Raw `agent/command.txt` declared public leaks paths/imports | Public artifact export exposes host paths | blocking | accept | `terminal_bench_runtime_snapshot.rs` public list no longer includes raw command/logs | Removed raw command/logs from Terminal-Bench `public_artifacts`; command snapshot gets extra path/import redaction; tests assert no root/run path/import leak | closure review |
| security | Events/report leak absolute local paths | Public event/report surface defeats clean public snapshot | blocking | accept | Terminal-Bench events use placeholders/relative artifact paths; `run_finished` event uses `report_path=report.html`; report context uses `report.html`, `harnesslab run replay [RUN_DIR]`, and `[PRIVATE_COMMAND]` | `ADAPT-RUNTIME-003` scans events/report for fake secret and local root/run paths | closure review |
| security | Cleanup event/report raw Docker strings | Docker/project/error fields may contain paths/secrets | medium | accept | Cleanup outcomes/events use redaction refs from profile/run/attempt/dataset/output paths | Error text and project/resource strings redacted before public event/report serialization | closure review checks boundary |
| security | Replay blocker path material | Some replay blockers still include private paths | medium | defer | Existing accepted blocker closed version drift; full replay path-redaction matrix is broader than Phase 6 Terminal-Bench snapshot activation | Keep as Phase 8 hardening candidate unless Phase 7 docs claim public replay errors are path-free | Phase 7/8 audit |
| test-validity | `ADAPT-RUNTIME-003` lifecycle overclaim and missing file pattern | Tests proved final snapshot, not temporal two-write lifecycle | blocking | accept | Phase doc narrowed to final post-execution material state; `terminal_bench.rs` added to file patterns | Avoided overclaim; registry now includes both write sites; final snapshots assert material/provenance/redaction | closure review |
| test-validity | Public artifact redaction surface incomplete | Raw logs listed public but unscanned | blocking | accept | Public artifact list now excludes raw command/log files | `ADAPT-RUNTIME-003` checks declared public artifacts and public event/report boundary | closure review |
| test-validity | Cleanup report only covered override case | `cleanup_warning_only` and `none` could drift | blocking | accept | `ADAPT-RUNTIME-003` asserts `none`; `ADAPT-RUNTIME-004` asserts `cleanup_overrode_result` and `cleanup_warning_only` | Expanded integration test coverage | closure review |
| test-validity | Requirement expected_layers not enforced | Registry green may overstate layer mix | medium | defer | Existing meta gate checks active routes and file patterns; enforcing layer coverage is repo-wide policy work | Record as Phase 8 meta-gate improvement candidate | Phase 8 |
| test-validity | Terminal-Bench replay tamper suite missing | SWE replay is stronger than Terminal-Bench-specific replay | medium | defer | Generic replay path validates all external snapshots; direct Terminal-Bench tamper matrix is useful but outside this Phase 6 fix batch | Keep Phase 8 candidate after docs/diagnostics alignment | Phase 8 |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes, locally
- Blocking re-review completed: pending
- Blocking re-review passed: pending
- Blocking re-review round links:
  - Round 2 pending
- Blocking re-review launch records:
  - pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: accepted blocking findings require fresh closure review
- Allowed to proceed: no, pending Round 2 closure review

## Final Conclusion

Round 1 found accepted blocking findings. Local fixes and validation are
complete, but the review is not passed until a fresh Round 2 closure review
confirms the blocking findings are closed.

## Round 2: Accepted Blocking Closure Review

### Review Input

#### Objective

Verify whether the accepted Round 1 blocking findings are closed by the current
worktree.

#### Review Target

Closure fixes for public/private runtime snapshots, Terminal-Bench public
artifact boundaries, cleanup report semantics, replay adapter-version drift
blocking, tests, registry, and docs.

#### Target Locations

- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_runtime.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_cleanup.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_adapter.rs`
- `crates/harnesslab-cli/src/runner/replay.rs`
- `crates/harnesslab-cli/src/runner/report_context.rs`
- `crates/harnesslab-cli/tests/external_runtime_snapshot_contract.rs`
- `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_runtime_event_contract.rs`
- `tests/TEST_REGISTRY.toml`
- `xtask/src/adapter_claims.rs`
- `docs/plans/2026-06-06-benchmark-adapter-phase-6-runtime-snapshot-cleanup.md`

#### Change Introduction

Round 1 accepted blockers were addressed by adding snapshot
`runtime_diagnostics`, blocking replay adapter-version drift, removing raw
Terminal-Bench command/log files from `public_artifacts`, redacting
command/event/report path surfaces, preserving/redacting cleanup project/error
data, expanding `ADAPT-RUNTIME-003/004` assertions, adding adapter-version drift
coverage to `SWEPRO-005`, and updating Phase 6 docs.

#### Risk Focus

- Any Round 1 blocking finding still open.
- New regressions caused by redacting report/event/command surfaces.
- Tests that were updated to match implementation without proving the boundary.
- Docs still overstating the implemented proof.

#### Assumptions To Attack

- Public Terminal-Bench artifacts now exclude raw command/log files and public
  surfaces no longer expose local paths/import paths.
- Runtime snapshots now carry enough cleanup diagnostics for Phase 6.
- Replay version drift fails closed.
- Cleanup report variants `none`, `cleanup_overrode_result`, and
  `cleanup_warning_only` are tested.
- Selector registry file patterns include the relevant implementation files.

#### Adversarial Lenses

- security
- implementation
- test validity
- documentation consistency

#### Verification Status

- `cargo check -p harnesslab-cli` passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-003` passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-004` passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-005` passed.
- `scripts/test-after-change.sh --select SWEPRO-005` passed.
- `scripts/verify-test-registry.sh` passed.
- `scripts/verify-planned-adapter-selectors.sh` passed with `active=15 planned=1`.
- `cargo test -p harnesslab-cli runner::external::terminal_bench_cleanup::tests -- --nocapture` passed.
- Code file line counts are below 500 lines.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Focus on closure of Round 1 accepted blockers.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| complex | 10 minutes | one bounded extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| closure-security-adversary | Round 1 had public/private path and secret-boundary blockers. | public artifact boundary |
| closure-test-validity-adversary | Round 1 had proof-strength blockers. | selector/test proof |
| closure-implementation-adversary | Round 1 had snapshot/replay correctness blockers. | implementation/replay semantics |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| closure-security-adversary | multi_agent_v1.spawn_agent / security-reviewer | 019e99a4-55f5-7152-a646-77e2c78ce68f | spawn_agent result nickname=Rawls | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| closure-test-validity-adversary | multi_agent_v1.spawn_agent / test-engineer | 019e99a4-d49e-7002-b993-7f19b638a5e7 | spawn_agent result nickname=Hypatia | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| closure-implementation-adversary | multi_agent_v1.spawn_agent / code-reviewer | 019e99a5-01a3-77e0-a476-522f434588cf | spawn_agent result nickname=Kierkegaard | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|

### Reviewer Outputs

#### R2-security

##### Summary

Round 1 was not fully closed. The reviewer found that command/path and
event/report blockers were closed, but cleanup report and cleanup events still
serialized raw Docker project/resource/error strings.

##### Blocking Findings

- Cleanup report and cleanup events still serialized raw Docker
  project/resource identifiers and Docker stderr.
  - Broken assumption: path/secret redaction refs were enough to make cleanup
    public-safe.
  - Failure scenario: Docker project/resource names or stderr include sensitive
    host/runtime detail and are written into public cleanup artifacts/events.
  - Impact: public boundary remains open outside the runtime snapshot.
  - Required proof: public cleanup report/events expose counts and booleans
    only; detailed cleanup data remains private-only.

##### Non-blocking Risks

- Cleanup public artifacts still carried correlatable run identity through
  tokens and official run ids.
- The reviewer did not independently rerun the full selector set.

##### Missing Tests

- No test asserted `cleanup-report.json` omits raw Docker project/resource/error
  strings.
- No test asserted `events.jsonl` omits raw Docker project/resource/error
  strings.

#### R2-test-validity

##### Summary

Round 1 closure was incomplete. Snapshot lifecycle and cleanup verdict effect
coverage were sufficient, but public-surface redaction proof still did not
cover the full declared public artifact/event/report surface.

##### Blocking Findings

- Public artifact and public-surface redaction proof remained incomplete.
  - Broken assumption: excluding `agent/command.txt` alone proved the full
    Terminal-Bench public artifact boundary.
  - Failure scenario: runner logs or other raw runtime files become public
    artifacts without a failing test.
  - Failure scenario: import-path/output-path values leak through events or
    report HTML even though command snapshot redaction passes.
  - Required proof: whitelist-style `public_artifacts` assertion; raw
    command/log artifact absence; import-path run that scans events/report.

##### Non-blocking Risks

- Cleanup effect proof is split across `ADAPT-RUNTIME-003` and
  `ADAPT-RUNTIME-004`; acceptable for closure but easy to miss in summaries.
- Phase 6 evidence doc should mention `none` and `cleanup_warning_only`
  coverage, not only `cleanup_overrode_result`.

##### Missing Tests

- Test fails if raw runner log artifacts are listed in `public_artifacts`.
- Test drives import-path case and checks public `events.jsonl` and
  `report.html`.
- Test proves declared public artifact set is complete.

#### R2-implementation

##### Summary

No blocking implementation findings. Snapshot diagnostics, replay
adapter-version drift blocking, and structured cleanup provenance were judged
closed from the implementation perspective.

##### Non-blocking Risks

- Terminal-Bench does not yet have a benchmark-local replay adapter-version
  mutation test; closure relies on the shared replay guard plus SWE replay
  contract.

### Main Agent Response

| Reviewer | Finding | Severity | Decision | Action Taken | Follow-up |
|---|---|---|---|---|---|
| R2-security | Public cleanup report/events still exposed raw Docker identifiers/error strings. | blocking | accept | Changed task-level `cleanup-report.json`, task cleanup events, and run-level runtime cleanup events to counts-only public shapes. Detailed token/projects/removed/error remain only in private runtime diagnostics. Added/updated tests to assert raw `actual-prefix-`, `metadata write failed`, `container:c1`, and `network:n1` are absent from public cleanup report/events. | Round 3 security closure |
| R2-test-validity | Public artifact surface proof was selective, not whitelist-based. | blocking | accept | `ADAPT-RUNTIME-003` now asserts exact public artifact whitelist (`cleanup-report.json` plus official result), asserts raw command/stdout/stderr/verifier logs are absent, and verifies the official result material has a public path. | Round 3 test closure |
| R2-test-validity | Import-path redaction was not proven for public events/report. | blocking | accept | Added import-path run in `ADAPT-RUNTIME-003` that scans `events.jsonl` and `report.html` for import path, root path, and run dir. `ADAPT-RUNTIME-005` keeps command/argv split coverage. | Round 3 test closure |
| R2-test-validity | Cleanup effect summary under-described `none` / `cleanup_warning_only`. | non-blocking | accept | Phase 6 doc now records counts-only public cleanup surfaces; tests cover `none`, `cleanup_overrode_result`, and `cleanup_warning_only`. | Phase 7 docs alignment |
| R2-implementation | Terminal-Bench replay mutation test is asymmetric with SWE. | non-blocking | defer | Shared replay guard plus `SWEPRO-005` cover adapter-version drift; Terminal-Bench-specific replay tamper remains a Phase 8 hardening candidate. | Phase 8 |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes, locally
- Blocking re-review completed: pending Round 3
- Blocking re-review passed: pending
- Blocking re-review round links:
  - Round 2
  - Round 3 pending
- Blocking re-review launch records:
  - Round 3 pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: Round 2 accepted blocking findings require fresh closure review
- Allowed to proceed: no, pending Round 3 closure review

## Round 3: Public Surface Closure Review

### Review Input

#### Objective

Verify whether Round 2 accepted blocking findings are closed by the current
worktree after the counts-only public cleanup and public artifact proof fixes.

#### Review Target

Closure fixes for Terminal-Bench public cleanup surfaces, public artifact
whitelist/redaction, import-path public event/report redaction, and selector
proof strength.

#### Target Locations

- `crates/harnesslab-cli/src/runner/cleanup.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_cleanup.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/report_context.rs`
- `crates/harnesslab-cli/tests/external_runtime_snapshot_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_cleanup_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_runtime_event_contract.rs`
- `tests/TEST_REGISTRY.toml`
- `xtask/src/adapter_claims.rs`
- `docs/plans/2026-06-06-benchmark-adapter-phase-6-runtime-snapshot-cleanup.md`

#### Verification Status

- `scripts/test-after-change.sh --select ADAPT-RUNTIME-003` passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-004` passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-005` passed.
- `scripts/test-after-change.sh --select SWEPRO-005` passed.
- `cargo test -p harnesslab-cli --all-features --test terminal_bench_cleanup_contract -- --nocapture` passed.
- `cargo check -p harnesslab-cli` passed.
- `scripts/verify-test-registry.sh` passed.
- `scripts/verify-planned-adapter-selectors.sh` passed with `active=15 planned=1`.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| round3-security-adversary | multi_agent_v1.spawn_agent / security-reviewer | 019e99de-0941-7fb0-ad6a-9bd9ebfc3fcd | spawn_agent result nickname=Gauss | fork_context=false | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| round3-test-validity-adversary | multi_agent_v1.spawn_agent / test-engineer | 019e99de-c821-7f72-b779-d8c2e95135a9 | spawn_agent result nickname=Russell | fork_context=false | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Outputs

#### R3-security

##### Summary

Phase 6 was not closed from a public-boundary standpoint. Cleanup report was
counts-only, but public runtime snapshot metadata, events, and report HTML still
exposed run-derived identifiers or raw command/log artifact indexes.

##### Blocking Findings

- `external-runtime.public.json` still exposed run-derived identifiers and
  public metadata for raw command/log artifacts.
- `events.jsonl` still exposed raw run ids in the event envelope and a
  Terminal-Bench `official_run_id` message field.
- `report.html` still exposed the raw run id and linked raw command/agent log
  artifacts.

##### Missing Tests

- No test asserted that `report.html` omits raw run id and raw command/log
  links.
- No test asserted that event envelope `run_id` is public-redacted.
- No test asserted that public runtime materials exclude raw command/log
  materials.

#### R3-test-validity

##### Summary

Most previously open proof gaps were closed, but two blocker classes remained:
report cleanup redaction was not tested, and `SWEPRO-005` selector traceability
did not include relevant production implementation files.

##### Blocking Findings

- `report.html` cleanup redaction was still not proven for raw Docker
  project/resource/error strings.
- `SWEPRO-005` registry traceability pointed only to the test file, not replay
  or SWE runtime snapshot implementation files.

##### Non-blocking Risks

- Event taxonomy proof remains message-fragment-coupled rather than
  schema-field-coupled.

### Main Agent Response

| Reviewer | Finding | Severity | Decision | Action Taken | Follow-up |
|---|---|---|---|---|---|
| R3-security | Public runtime snapshot metadata still indexed raw command/log artifacts and run-derived official result paths. | blocking | accept | Added `RuntimeMaterial.include_in_public`; Terminal-Bench keeps raw command/stdout/stderr in private replay materials but excludes them from `external-runtime.public.json.runtime_materials`; public command stdout/stderr paths are `[PRIVATE_ARTIFACT]`; official result public path is stable `official/terminal-bench/results.json`. | Round 4 security closure |
| R3-security | `events.jsonl` still exposed raw run ids and Terminal-Bench official run ids. | blocking | accept | `append_event` now writes `[PRIVATE_RUN_ID]` in the event envelope; Terminal-Bench started event uses `official_run_id=<run-id>`. Tests assert `[PRIVATE_RUN_ID]` and raw run id absence. | Round 4 security closure |
| R3-security | `report.html` still exposed raw run id and raw command/log links. | blocking | accept | CLI report context now passes `[PRIVATE_RUN_ID]`; report HTML no longer links `command.txt` or agent/verifier stdout/stderr logs. CLI/report tests and `ADAPT-RUNTIME-003` assert absence. | Round 4 security closure |
| R3-test-validity | Report cleanup redaction was not proven. | blocking | accept | `ADAPT-RUNTIME-004` now scans `report.html` along with `cleanup-report.json` and `events.jsonl` for raw cleanup project/resource/error strings. | Round 4 test closure |
| R3-test-validity | `SWEPRO-005` selector traceability omitted production files. | blocking | accept | Added `crates/harnesslab-cli/src/runner/replay.rs` and `crates/harnesslab-cli/src/runner/external/swe_bench_pro/runtime_snapshot.rs` to `SWEPRO-005` file patterns in `tests/TEST_REGISTRY.toml` and `xtask/src/adapter_claims.rs`. | Round 4 test closure |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes, locally
- Blocking re-review completed: pending Round 4
- Blocking re-review passed: pending
- Allowed to proceed: no, pending Round 4 closure review

## Round 4: Accepted Blocker Closure Review

### Review Input

#### Objective

Verify whether Round 3 accepted blocking findings are closed by the current
worktree after public runtime material filtering, event run-id redaction,
report link removal, cleanup report/report redaction proof, and `SWEPRO-005`
selector traceability updates.

#### Review Target

Closure fixes for public/private runtime material boundaries, public event and
report redaction, cleanup raw-string redaction proof, and selector registry
traceability.

#### Target Locations

- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_cleanup.rs`
- `crates/harnesslab-cli/src/runner/cleanup.rs`
- `crates/harnesslab-cli/src/runner/report_context.rs`
- `crates/harnesslab-infra/src/event.rs`
- `crates/harnesslab-report/src/lib.rs`
- `crates/harnesslab-cli/tests/external_runtime_snapshot_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_cleanup_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_runtime_event_contract.rs`
- `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs`
- `tests/TEST_REGISTRY.toml`
- `xtask/src/adapter_claims.rs`
- `docs/plans/2026-06-06-benchmark-adapter-phase-6-runtime-snapshot-cleanup.md`

#### Verification Status

- `cargo check -p harnesslab-cli` passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-003` passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-004` passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-005` passed.
- `scripts/test-after-change.sh --select SWEPRO-005` passed.
- `cargo test -p harnesslab-cli --all-features --test terminal_bench_cleanup_contract -- --nocapture` passed, 6 tests.
- `cargo test -p harnesslab-report -- --nocapture` passed.
- `cargo test -p harnesslab-infra event::tests -- --nocapture` passed.
- `cargo test -p harnesslab-cli --all-features --test cli_contract -- --nocapture` passed.
- `scripts/verify-test-registry.sh` passed.
- `scripts/verify-planned-adapter-selectors.sh` passed with `active=15 planned=1`.
- Code file line counts checked; modified files are at or below 500 lines.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| round4-security-adversary | multi_agent_v1.spawn_agent / security-reviewer | 019e99fa-1f4e-72b1-b748-857bdec29c46 | spawn_agent result nickname=Fermat | fork_context=false | Round 4 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| round4-test-validity-adversary | multi_agent_v1.spawn_agent / test-engineer | 019e99fa-596f-78a0-9544-6680de411fd5 | spawn_agent result nickname=Averroes | fork_context=false | Round 4 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Outputs

#### R4-security

##### Summary

Round 4 security review remained blocked. The reviewer accepted earlier
public-material and report-link fixes, but found that public command text could
still expose run-derived official ids and import-path agent values, and that
run-level Docker cleanup warnings could still serialize raw error strings.

##### Blocking Findings

- `external-runtime.public.json` could still leak run-derived official ids and
  import-path agents through `commands[].command`.
- Run-level Docker cleanup warning events were not counts-only on all public
  surfaces because `cleanup.rs` formatted the raw cleanup error into
  `events.jsonl`.

##### Non-blocking Risks

- Dependency audit was not executed because `cargo-audit` is not installed in
  this environment.

#### R4-test-validity

##### Summary

Round 4 test review found the requested Phase 6 proof gaps closed, but raised
one blocker against `tests/TEST_REGISTRY.toml` being over 500 lines.

##### Blocking Findings

- `tests/TEST_REGISTRY.toml` is 3279 lines, above the AGENTS.md single-code-file
  line guideline.

##### Closed Proofs

- `report.html` cleanup redaction for raw Docker project/resource/error strings
  is asserted.
- Public runtime material whitelist and private replay-material retention are
  asserted.
- Stable public official-result aliasing is asserted.
- `[PRIVATE_RUN_ID]` plus no real run id/import-path public event/report leakage
  is asserted.
- `SWEPRO-005` traceability includes `replay.rs`,
  `swe_bench_pro/runtime_snapshot.rs`, and the contract test in both registry
  surfaces.

### Main Agent Response

| Reviewer | Finding | Severity | Decision | Action Taken | Follow-up |
|---|---|---|---|---|---|
| R4-security | Public snapshot `commands[].command` could leak real `--run-id` and `--agent-import-path` values. | blocking | accept | Added public command flag-value redaction for `--run-id` -> `[PRIVATE_RUN_ID]` and `--agent-import-path` -> `[PRIVATE_AGENT_IMPORT]`. `ADAPT-RUNTIME-003` now asserts those placeholders and import-path absence. | Round 5 security closure |
| R4-security | Run-level Docker cleanup warning event could serialize raw cleanup error text. | blocking | accept | Changed run-level Docker cleanup events to counts-only `removed_count` / `has_error` messages; unit test now asserts raw warning text is absent. | Round 5 security closure |
| R4-test-validity | `tests/TEST_REGISTRY.toml` exceeds 500 lines. | blocking | reject for Phase 6 closure | This file is a registry/config artifact, not a single code file, and splitting the global registry is a separate architecture migration outside Phase 6. Modified Rust code/test files are at or below 500 lines; the registry validator passes. | Track as Phase 7/registry-architecture debt if the project wants a sharded registry |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes, locally
- Blocking re-review completed: pending Round 5
- Blocking re-review passed: pending
- Allowed to proceed: no, pending Round 5 closure review

## Round 5: Security Fix Closure Review

### Review Input

#### Objective

Verify whether the two Round 4 accepted security blockers are closed by the
current worktree, and whether the `TEST_REGISTRY.toml` line-count finding
should block Phase 6 closure.

#### Review Target

Latest public command placeholder redaction, counts-only run-level Docker
cleanup warning events, corresponding tests, and Phase 6 closure rationale.

#### Target Locations

- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs`
- `crates/harnesslab-cli/tests/external_runtime_snapshot_contract.rs`
- `crates/harnesslab-cli/src/runner/cleanup.rs`
- `crates/harnesslab-cli/src/runner/cleanup_tests.rs`
- `crates/harnesslab-cli/tests/terminal_bench_cleanup_contract.rs`
- `tests/TEST_REGISTRY.toml`
- `xtask/src/adapter_claims.rs`
- `docs/plans/2026-06-06-benchmark-adapter-phase-6-runtime-snapshot-cleanup.md`
- `vs_review/2026-06-06-benchmark-adapter-phase-6-runtime-snapshot-cleanup-review.md`

#### Verification Status

- `scripts/test-after-change.sh --select ADAPT-RUNTIME-003` passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-004` passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-005` passed.
- `scripts/test-after-change.sh --select SWEPRO-005` passed.
- `cargo test -p harnesslab-cli cleanup_004_cleanup_warning_is_recorded --lib -- --nocapture` passed.
- `cargo test -p harnesslab-cli --all-features --test terminal_bench_cleanup_contract -- --nocapture` passed, 6 tests.
- `cargo test -p harnesslab-report -- --nocapture` passed.
- `cargo test -p harnesslab-infra event::tests -- --nocapture` passed.
- `cargo test -p harnesslab-cli --all-features --test cli_contract -- --nocapture` passed.
- `cargo check -p harnesslab-cli` passed.
- `scripts/verify-test-registry.sh && scripts/verify-planned-adapter-selectors.sh` passed with `active=15 planned=1`.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| round5-security-adversary | multi_agent_v1.spawn_agent / security-reviewer | 019e9a1c-3f13-7623-8f86-0ff82bcf0738 | spawn_agent result nickname=Popper | fork_context=false | Round 5 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| round5-test-validity-adversary | multi_agent_v1.spawn_agent / test-engineer | 019e9a1c-80b6-7210-ac36-8669ac147b0f | spawn_agent result nickname=Nietzsche | fork_context=false | Round 5 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Outputs

#### R5-security

##### Summary

PASS. Both Round 4 accepted security blockers were verified closed.

##### Closure Evidence

- Public command snapshots redact `--run-id` and `--agent-import-path` to
  `[PRIVATE_RUN_ID]` and `[PRIVATE_AGENT_IMPORT]`, and `ADAPT-RUNTIME-003`
  proves the placeholders and raw-value absence.
- Run-level Docker cleanup warning events now emit counts-only summaries; the
  warning branch no longer serializes raw cleanup error text. Cleanup tests and
  Terminal-Bench cleanup contract prove raw warning/project/error absence.

##### Residual Risk

- The reviewer noted token-based command redaction should also cover
  `--flag=value` forms. Main agent expanded the redactor and added
  `public_command_text_redacts_sensitive_flag_values`; the unit test passed.

#### R5-test-validity

##### Summary

PASS. Phase 6 proof gaps are closed, selector evidence remains valid, and
`tests/TEST_REGISTRY.toml` line count is not a Phase 6 blocker.

##### Closure Evidence

- Public snapshot command placeholder assertions are present for run id and
  import-path values.
- Cleanup warning proof is counts-only and forbids raw warning text.
- `ADAPT-RUNTIME-003/004/005` and `SWEPRO-005` selector evidence remains valid
  in `xtask` and `TEST_REGISTRY.toml`.
- The 3279-line `TEST_REGISTRY.toml` is a global registry/config artifact; if
  the project wants strict uniform 500-line enforcement, the smallest follow-up
  is a later registry-sharding migration with the existing validator as the
  composition gate.

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes, locally
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Allowed to proceed: yes, Phase 6 can be committed and Phase 7 can start
