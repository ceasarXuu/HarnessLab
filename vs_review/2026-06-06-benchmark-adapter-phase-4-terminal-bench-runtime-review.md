# Subagent VS Review: Benchmark Adapter Phase 4 Terminal-Bench Runtime Extraction

- Created: 2026-06-06T00:00:00+08:00
- Updated: 2026-06-06T00:00:00+08:00
- Report schema: adversarial-v1
- Task: Complete benchmark adapter Phase 4 by extracting Terminal-Bench runtime ownership into `TerminalBenchRuntimeAdapter` while preserving official runner behavior.
- Report path: `vs_review/2026-06-06-benchmark-adapter-phase-4-terminal-bench-runtime-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: Phase 4 Implementation Review

### Review Input

#### Objective

Complete Phase 4 of the benchmark adapter architecture plan: Terminal-Bench runtime behavior should run through a real `TerminalBenchRuntimeAdapter` boundary while preserving the existing official `tb run` path, event taxonomy, timeout/failure mapping, QEMU policy, cleanup semantics, and registered selector evidence.

#### Review Target

Code implementation, test strategy, event/logging observability, and documentation for Phase 4 Terminal-Bench runtime extraction.

#### Target Locations

- `crates/harnesslab-cli/src/runner/external.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_adapter.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_adapter.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_adapter_tests.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `scripts/test-after-change.sh`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-06-benchmark-adapter-phase-4-terminal-bench-runtime-extraction.md`
- Commands: `scripts/test-after-change.sh --select ADAPT-RUNTIME-001`, `ADAPT-RUNTIME-002`, `ADAPT-RUNTIME-005`, `TB-001..011`, `PY-TB-001`, and `cargo test -p harnesslab-cli --all-features --lib runner::external:: -- --nocapture`

#### Change Introduction

The generic runtime registry now delegates Terminal-Bench to a dedicated `terminal_bench_adapter.rs` module. That module owns Terminal-Bench preflight, agent selection, official `tb run` command construction, command snapshot writing, Docker platform selection, official import-agent timeout grace, and run-level cleanup target ownership. The generic `runtime_adapter.rs` retains shared trait/registry/preflight report plumbing. `terminal_bench.rs` keeps the concrete execution/result flow and consumes adapter-provided runtime policy. `ADAPT-RUNTIME-005` has been activated as a real event-taxonomy selector, and `TB-001..004` selector routing now explicitly uses `test_target=lib`.

#### Risk Focus

- The extraction may only move names while Terminal-Bench still bypasses the adapter boundary for important runtime policy.
- The official `tb run` command could change subtly, including timeout, agent/model, import path, Docker platform, output path, or command snapshot redaction behavior.
- Event taxonomy tests could be source-string checks that do not prove runtime event emission.
- Cleanup target ownership could regress pre-run stale-run cleanup or post-run cleanup evidence.
- Test registry metadata could overstate active runtime proof.
- Documentation could claim Phase 4 completion while leaving Phase 4 deliverables unimplemented or unreviewed.

#### Assumptions To Attack

- `TerminalBenchRuntimeAdapter` is now the meaningful owner of Terminal-Bench runtime policy.
- Moving command construction into `terminal_bench_adapter.rs` does not change official runner behavior.
- `ADAPT-RUNTIME-005` proves operator-critical events remain queryable enough for Phase 4.
- `TB-001..011` and `PY-TB-001` are sufficient preservation evidence for this extraction slice.
- The new module boundaries stay maintainable and do not create circular ownership or hidden coupling.
- The docs accurately separate completed Phase 4 work from Phase 5 and Phase 6 remaining work.

#### Adversarial Lenses

- architecture
- implementation
- failure
- testing
- observability
- maintenance

#### Verification Status

- `cargo fmt --all -- --check`: passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-001`: passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-002`: passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-005`: passed.
- `scripts/test-after-change.sh --select TB-001..TB-011`: all passed after fixing `TB-001..004` to target `lib`.
- `scripts/test-after-change.sh --select PY-TB-001`: 32 tests and 7 subtests passed.
- `cargo test -p harnesslab-cli --all-features --lib runner::external:: -- --nocapture`: 45 passed.
- Full workspace gate has not yet been run in this round.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Try to falsify the completion claim; do not confirm it by default.
- Report blocking findings, non-blocking risks, required fixes, missing tests, and missing logs/observability.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 12 minutes | one bounded 6 minute extension only if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-adversary | Phase 4 changes module boundaries and adapter ownership. | abstraction, ownership, maintainability |
| implementation-adversary | Runtime command construction, cleanup, and failure paths can regress behavior. | correctness, failure mapping, compatibility |
| test-validity-adversary | Selector activation and event taxonomy proof can be self-deceptive. | registry coverage, test strength, evidence quality |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-adversary | multi_agent_v1.spawn_agent | `019e9912-c972-71b3-b0e4-e79a056ad74b` / Galileo | spawn_agent result | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |
| implementation-adversary | multi_agent_v1.spawn_agent | `019e9913-033a-7902-bc18-c8d1c53f8fd0` / Peirce | spawn_agent result | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |
| test-validity-adversary | multi_agent_v1.spawn_agent | `019e9913-346a-7ab2-9699-7dd30b01f93c` / Dirac | spawn_agent result | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| architecture-output | architecture-adversary | 1 | `019e9912-c972-71b3-b0e4-e79a056ad74b` | < 12 minutes | completed | returned architecture blockers | completed |
| implementation-output | implementation-adversary | 1 | `019e9913-033a-7902-bc18-c8d1c53f8fd0` | < 12 minutes | completed | returned proof and documentation blockers | completed |
| test-validity-output | test-validity-adversary | 1 | `019e9913-346a-7ab2-9699-7dd30b01f93c` | < 12 minutes | completed | returned test-proof blockers | completed |

### Reviewer Outputs

#### architecture-output

##### Summary

Phase 4 was not cleanly closed. The reviewer found that the Terminal-Bench boundary still had reverse helper calls from execution back into the adapter, and `ADAPT-RUNTIME-005` overclaimed runtime and cross-adapter proof.

##### Blocking Findings

- `TerminalBenchRuntimeAdapter` boundary was not yet a real ownership boundary; `terminal_bench.rs` still imported adapter helpers and coordinated runtime-policy side effects.
  - Broken assumption: Terminal-Bench runtime policy had moved into the adapter.
  - Failure scenario: future Phase 6 snapshot or command/platform changes would need coordinated edits across both modules.
  - Trigger condition: runtime policy changes after Phase 4.
  - Impact: adapter ownership drifts and Phase 5/6 extension costs rise.
  - Proof needed: adapter-owned typed runtime attempt or equivalent boundary; no reverse helper calls from `terminal_bench.rs` into `terminal_bench_adapter.rs`.
- `ADAPT-RUNTIME-005` was source-string proof while registered as runtime event proof and also overclaimed stable SWE-bench Pro phase events.
  - Broken assumption: active selector proved emitted event taxonomy.
  - Failure scenario: event records or SWE phase events could regress while the source-string test still passed.
  - Trigger condition: event payload/name changes or missing SWE phase events.
  - Impact: operator diagnostics and Phase 5 closure could be falsely green.
  - Proof needed: emitted `events.jsonl` assertions and scope split for SWE-bench Pro phase proof.

##### Non-blocking Risks

- Phase 4 docs mixed runtime snapshot and cleanup-report artifact scope with Phase 6 scope.

##### Required Fixes

- Make the adapter boundary real with adapter-owned attempt runtime policy.
- Replace or re-scope `ADAPT-RUNTIME-005` with real runtime assertions.
- Align docs and proof ledger before calling Phase 4 complete.

##### Missing Tests

- Boundary regression test preventing `terminal_bench.rs` from calling adapter helpers.
- Active SWE-bench Pro phase-event proof remains missing and belongs to Phase 5.

##### Missing Logs / Observability

- Structured cleanup-report and public/private runtime snapshot artifacts remain Phase 6 planned work.
- Event field queryability needed runtime assertions.

##### Evidence

- `crates/harnesslab-cli/src/runner/external/terminal_bench_adapter.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_adapter_tests.rs`
- `tests/TEST_REGISTRY.toml`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`

#### implementation-output

##### Summary

The reviewer found no concrete command-construction, import-path, timeout-grace, QEMU, or cleanup regression in the extracted code path. The blockers were proof and closure defects: runtime event proof was not executable, file patterns did not cover the moved modules, and Phase 4 was marked complete while Phase 4/Phase 6 scope was ambiguous.

##### Blocking Findings

- `ADAPT-RUNTIME-005` claimed active integration/runtime proof but only scanned source strings.
  - Broken assumption: selector produced `events.jsonl` runtime evidence.
  - Failure scenario: emitted payload fields such as timeout/progress/setup details could regress undetected.
  - Trigger condition: event message changes without removing source strings.
  - Impact: operator diagnostics proof is false.
  - Proof needed: executable Terminal-Bench contract parsing `events.jsonl`.
- `ADAPT-RUNTIME-005` file patterns did not cover event-emitting modules.
  - Broken assumption: registry metadata guards the moved event code.
  - Failure scenario: changes in `terminal_bench_adapter.rs`, `terminal_bench_runtime.rs`, `terminal_bench_result.rs`, or `terminal_bench_cleanup.rs` would not map to the selector.
  - Trigger condition: file-pattern driven selection or audit.
  - Impact: coverage and traceability gaps.
  - Proof needed: registry file patterns covering the selector test and event-emitting modules.
- Phase 4 was marked complete while snapshot and cleanup-report artifact requirements remained planned.
  - Broken assumption: Phase 4 completion matched plan deliverables.
  - Failure scenario: docs claim closure while `ADAPT-RUNTIME-003/004` remain planned.
  - Trigger condition: future closure audit.
  - Impact: false readiness claim.
  - Proof needed: either implement those proofs or explicitly narrow Phase 4 and defer artifacts to Phase 6.

##### Non-blocking Risks

- Timeout/watchdog/result precedence ownership remained split outside the adapter module.

##### Required Fixes

- Replace `ADAPT-RUNTIME-005` with runtime-emission contract.
- Expand `ADAPT-RUNTIME-005` file patterns.
- Reopen or narrow Phase 4 documentation relative to `ADAPT-RUNTIME-003/004`.

##### Missing Tests

- End-to-end proof for Docker platform override, command snapshot runtime/report split, and event payload fields.

##### Missing Logs / Observability

- No active executable proof checked configured timeouts, progress paths, Docker platform, setup-failure reason text, or cleanup tokens/projects.

##### Evidence

- `crates/harnesslab-cli/src/runner/external/runtime_adapter_tests.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/plans/2026-06-06-benchmark-adapter-phase-4-terminal-bench-runtime-extraction.md`

#### test-validity-output

##### Summary

Two blocking findings: the Phase 4 proof set did not prove official command preservation, and `ADAPT-RUNTIME-005` was materially weaker than its requirement and registry claim.

##### Blocking Findings

- Official `tb run` preservation was not actually covered by the claimed proof set.
  - Broken assumption: existing Terminal-Bench tests assert the official command shape.
  - Failure scenario: removing `--no-upload-results`, changing the launcher, or dropping timeout/agent flags could still leave tests green.
  - Trigger condition: command construction regression.
  - Impact: official-runner behavior changes silently.
  - Proof needed: runtime command or `agent/command.txt` assertions for launcher and critical flags.
- `ADAPT-RUNTIME-005` was a static string-presence test despite being presented as integration/runtime proof.
  - Broken assumption: event names are emitted as queryable records.
  - Failure scenario: dead constants/comments or malformed event JSON pass the test.
  - Trigger condition: event emission regression.
  - Impact: event taxonomy proof is self-deceptive.
  - Proof needed: run Terminal-Bench, parse `events.jsonl`, and assert event records and fields.

##### Non-blocking Risks

- The documented `scripts/test-after-change.sh --select TB-001..TB-011` command is not executable as written.
- `PY-TB-001` does not pin test count in the selector.
- `ADAPT-RUNTIME-005` registry metadata had stale file patterns.

##### Required Fixes

- Add real Terminal-Bench command-preservation test.
- Replace or supplement `ADAPT-RUNTIME-005` with runtime-emission contract.
- Correct Phase 4 evidence commands.
- Fix `ADAPT-RUNTIME-005` registry metadata.

##### Missing Tests

- Built-in and import-path command construction proof.
- Runtime event contract parsing `events.jsonl`.
- Selector/gate count guard for `PY-TB-001`.

##### Missing Logs / Observability

- No test proved structured event queryability by event identity.
- No emitted-runtime assertion tied command-preservation proof to durable artifacts or argv.

##### Evidence

- `crates/harnesslab-cli/src/runner/external/terminal_bench_adapter.rs`
- `crates/harnesslab-cli/tests/terminal_bench_runtime_contract.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_adapter_tests.rs`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `docs/plans/2026-06-06-benchmark-adapter-phase-4-terminal-bench-runtime-extraction.md`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| architecture-adversary | Adapter boundary still used reverse helper calls. | `terminal_bench.rs` imported `terminal_bench_adapter` helpers, so runtime policy was not fully adapter-owned. | blocking | accept | Reviewer evidence matched code before fix. | Added `TerminalBenchRuntimeAttempt`; `TerminalBenchRuntimeAdapter::execute` now prepares dataset path, command snapshot, platform, timeout/watchdog policy, progress paths, and activity patterns before calling `terminal_bench::execute_prepared`. `terminal_bench.rs` no longer imports `terminal_bench_adapter`. | Round 2 closure review |
| architecture-adversary | `ADAPT-RUNTIME-005` overclaimed runtime and SWE-bench Pro coverage. | Source-string scan passed without emitted `events.jsonl`; stable SWE phase events are not Phase 4. | blocking | accept | Reviewer evidence matched selector and docs before fix. | Replaced selector with `crates/harnesslab-cli/tests/terminal_bench_runtime_event_contract.rs`, which runs Terminal-Bench fake-run scenarios, parses `events.jsonl`, and asserts event records/fields. Re-scoped requirement/docs to Terminal-Bench Phase 4; SWE phase proof stays Phase 5 under `SWEPRO-001..004`. | Round 2 closure review |
| architecture-adversary | Phase 4 docs mixed snapshot and cleanup-report artifact scope. | Phase 4 appeared complete while `ADAPT-RUNTIME-003/004` remain planned. | medium | accept | The plan had Phase 4 wording that overlapped Phase 6. | Updated architecture and Phase 4 report to state public/private snapshots and structured cleanup-report artifacts remain Phase 6 scope. | Verify docs in Round 2 |
| implementation-adversary | `ADAPT-RUNTIME-005` was source-scan-only. | Payload fields could regress undetected. | blocking | accept | Same as test-validity blocker. | New runtime integration selector parses real event records and checks timeout, progress path, Docker platform, setup failure, activity/no-progress, timeout, cleanup, warning, dataset-prepared, and parse-failure messages. | Round 2 closure review |
| implementation-adversary | `ADAPT-RUNTIME-005` file patterns were stale. | Event-emitting module edits could bypass selector traceability. | blocking | accept | Registry listed only `external.rs`. | Expanded `tests/TEST_REGISTRY.toml` and `xtask/src/adapter_claims.rs` expected route to include the integration test and Terminal-Bench event-emitting modules. `scripts/verify-test-registry.sh` passes. | None |
| implementation-adversary | Phase 4 completion overstated while `ADAPT-RUNTIME-003/004` planned. | Closure state did not match plan contract. | blocking | accept | Requirements remain planned by design for Phase 6. | Reworded Phase 4 status to closure-in-progress and narrowed deliverables to Terminal-Bench runtime command/event behavior; Phase 6 owns snapshot and structured cleanup-report artifacts. | Round 2 closure review |
| test-validity-adversary | Official command preservation was unproven. | Fake scripts ignored most official flags. | blocking | accept | Existing tests did not assert launcher/flags. | New `ADAPT-RUNTIME-005` records actual fake-runner argv and asserts `uvx --from terminal-bench tb run`, core flags, built-in `--agent`, import-path `--agent-import-path`, and import timeout grace. It also checks report-facing `agent/command.txt` for visible command structure. | Round 2 closure review |
| test-validity-adversary | `TB-001..TB-011` documentation was not executable as a literal selector. | Script accepts one ID, not a range. | non-blocking | accept | Reviewer evidence matched script behavior. | Updated Phase 4 evidence to show the actual shell loop over individual IDs. | None |
| test-validity-adversary | `PY-TB-001` does not pin exact count. | Broad pytest delegation can drift. | non-blocking | defer | Useful but outside Phase 4 extraction blocker; current run output records 32 tests and 7 subtests. | No code change in this slice. | Track for Phase 8 full gate hardening if needed |

### Round 1 Response Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review required: yes
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Next step: run Round 2 closure review before allowing Phase 5.

## Interim Conclusion

Round 1 accepted blockers were fixed by the main agent and sent to Round 2
closure review. Final closure status is recorded after Round 2 below.

## Round 2: Accepted Blocker Closure Review

### Review Input

#### Objective

Verify closure of accepted Round 1 blockers for Phase 4 Terminal-Bench runtime extraction.

#### Review Target

Closure fixes for adapter boundary, `ADAPT-RUNTIME-005` runtime proof, command preservation proof, selector metadata, and Phase 4 documentation scope.

#### Target Locations

- `crates/harnesslab-cli/src/runner/external/terminal_bench_adapter.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_runtime.rs`
- `crates/harnesslab-cli/tests/terminal_bench_runtime_event_contract.rs`
- `scripts/test-after-change.sh`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `xtask/src/adapter_claims.rs`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-06-benchmark-adapter-phase-4-terminal-bench-runtime-extraction.md`
- `vs_review/2026-06-06-benchmark-adapter-phase-4-terminal-bench-runtime-review.md`

#### Change Introduction

Round 1 accepted blockers were addressed by adding `TerminalBenchRuntimeAttempt`, moving Terminal-Bench attempt runtime policy preparation into `TerminalBenchRuntimeAdapter::execute`, changing `terminal_bench.rs` to `execute_prepared`, replacing source-string `ADAPT-RUNTIME-005` with a runtime integration test that parses emitted `events.jsonl` and checks command/argv proof, expanding selector metadata/file patterns, and narrowing Phase 4 docs so snapshots and structured cleanup-report artifacts remain Phase 6 scope.

#### Risk Focus

- Did the adapter boundary actually close, or does `terminal_bench.rs` still depend on adapter helpers?
- Does `ADAPT-RUNTIME-005` now produce real runtime evidence and validate event fields?
- Does the command proof cover built-in and import-path official runner branches without relying only on report redaction snapshots?
- Do registry metadata and xtask expected routes match the new active selector?
- Do docs avoid claiming Phase 6 snapshot/cleanup-report work as Phase 4 completion?

#### Assumptions To Attack

- `TerminalBenchRuntimeAdapter` now owns runtime attempt policy.
- The new integration test is not a fake proof that can pass without emitted event records.
- The docs and registry no longer overclaim SWE-bench Pro phase events or Phase 6 artifacts.
- The accepted blockers are fixed, not merely renamed.

#### Adversarial Lenses

- architecture
- implementation
- testing
- observability
- documentation

#### Verification Status

- `scripts/test-after-change.sh --select ADAPT-RUNTIME-005`: passed.
- `scripts/verify-test-registry.sh`: passed.
- `scripts/verify-planned-adapter-selectors.sh`: passed.
- `cargo test -p harnesslab-cli --all-features --lib runner::external:: -- --nocapture`: passed, 45 tests.
- `scripts/test-after-change.sh --select TB-010`: passed.
- `cargo check --workspace --all-targets --all-features`: passed.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Focus only on closure of accepted Round 1 blockers and any new blocker introduced by the fixes.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 12 minutes | one bounded 6 minute extension only if alive | 2 | cannot pass if closure review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| closure-architecture-adversary | Round 1 architecture blocker challenged adapter boundary and scope claims. | boundary, ownership, docs |
| closure-test-validity-adversary | Round 1 test blocker challenged runtime proof and command preservation. | runtime proof, selector metadata, evidence quality |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| closure-architecture-adversary | multi_agent_v1.spawn_agent | `019e9933-01cf-7960-be60-26a04fae59bc` / Sagan | spawn_agent result | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |
| closure-test-validity-adversary | multi_agent_v1.spawn_agent | `019e9933-43cc-7511-9558-b7aab5b6e024` / Lagrange | spawn_agent result | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| closure-architecture-output | closure-architecture-adversary | 1 | `019e9933-01cf-7960-be60-26a04fae59bc` | < 12 minutes | completed | accepted blockers closed; non-blocking follow-ups only | completed |
| closure-test-validity-output | closure-test-validity-adversary | 1 | `019e9933-43cc-7511-9558-b7aab5b6e024` | < 12 minutes | completed | found one remaining docs blocker | completed |

### Reviewer Outputs

#### closure-test-validity-output

##### Summary

One closure blocker remained. Runtime proof, command proof, and selector metadata were materially closed, but the architecture design still carried stale scope wording.

##### Blocking Findings

- The architecture design still contradicted the Round 1 scope fix by saying `ADAPT-RUNTIME-005` proves both Terminal-Bench and SWE phase events queryability.
  - Broken assumption: Phase 4 docs were fully aligned after narrowing `ADAPT-RUNTIME-005`.
  - Failure scenario: a future closure audit could treat SWE phase-event proof as active even though it remains Phase 5 scope.
  - Trigger condition: using `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md` as the source of truth.
  - Impact: false closure claim for SWE phase events.
  - Proof needed: update the architecture doc so SWE phase-event proof is assigned to `SWEPRO-001..004`.

##### Non-blocking Risks

- The architecture design still used stale `TB-001..011` shorthand rather than the executable shell loop.
- Optional hardening: assert contiguous actual argv prefix `uvx --from terminal-bench tb run`; no concrete regression found.

##### Required Fixes

- Update the architecture design so `ADAPT-RUNTIME-005` is scoped to Terminal-Bench emitted events only, with SWE phase-event proof left to `SWEPRO-001..004`.
- Update the architecture design's TB evidence wording to the executable loop form.

##### Missing Tests

- none for accepted blockers 1-3.

##### Missing Logs / Observability

- none within accepted blocker scope.

##### Evidence

- `crates/harnesslab-cli/tests/terminal_bench_runtime_event_contract.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `xtask/src/adapter_claims.rs`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-06-benchmark-adapter-phase-4-terminal-bench-runtime-extraction.md`

#### closure-architecture-output

##### Summary

Accepted Round 1 blockers are closed. `TerminalBenchRuntimeAdapter::execute`
now owns Terminal-Bench attempt preparation and passes typed
`TerminalBenchRuntimeAttempt` into `terminal_bench::execute_prepared`.
`ADAPT-RUNTIME-005` is a real integration selector over emitted `events.jsonl`
and argv/command artifacts, scoped to Terminal-Bench. Docs defer
`ADAPT-RUNTIME-003/004` to Phase 6.

##### Blocking Findings

- none

##### Non-blocking Risks

- Architecture plan metadata and one evidence count were stale after the fix.
- There is no explicit regression test forbidding reverse imports/calls from
  `terminal_bench.rs` back into adapter helpers.
- `external_runner_configured` did not expose official result path or command
  snapshot path before the follow-up patch.

##### Required Fixes

- none

##### Missing Tests

- Optional boundary regression test for reverse imports/calls.

##### Missing Logs / Observability

- Optional broader runtime event payload coverage.

##### Evidence

- `crates/harnesslab-cli/src/runner/external/terminal_bench_adapter.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_runtime.rs`
- `crates/harnesslab-cli/tests/terminal_bench_runtime_event_contract.rs`
- `tests/TEST_REGISTRY.toml`
- `scripts/test-after-change.sh`
- `xtask/src/adapter_claims.rs`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| closure-test-validity-adversary | Architecture doc still said `ADAPT-RUNTIME-005` proves Terminal-Bench and SWE phase events. | SWE phase-event proof could be falsely considered active before Phase 5. | blocking | accept | Reviewer cited stale Phase 6 validation table. | Updated `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md` so event taxonomy preservation is `ADAPT-RUNTIME-005` plus Phase 5 `SWEPRO-001..004`; Terminal-Bench emitted events are proven by `ADAPT-RUNTIME-005`, SWE phase events by dedicated Phase 5 selectors. | Final closure confirmation |
| closure-test-validity-adversary | Architecture doc used stale `TB-001..011` shorthand. | The phrase is not an executable selector command. | non-blocking | accept | The Phase 4 report already had the correct loop. | Updated architecture doc completion evidence to the executable shell loop over `TB-001` through `TB-011`. | None |
| closure-test-validity-adversary | Optional argv prefix hardening. | Token presence could miss ordering, though command snapshot already checks launcher substring. | non-blocking | defer | Reviewer found no concrete regression; current test asserts visible command launcher plus argv tokens and branches. | No code change in this slice. | Consider in Phase 8 hardening if desired |
| closure-architecture-adversary | Architecture metadata and Phase 4 report lib count stale. | Readers could see stale `Updated`, `Version`, and 46-test count after `ADAPT-RUNTIME-005` moved to integration. | non-blocking | accept | Reviewer cited metadata and count mismatch. | Updated architecture metadata to 2026-06-06 / 0.20 and Phase 4 report lib count to 45. | None |
| closure-architecture-adversary | No explicit reverse-import boundary regression test. | Future edits could reintroduce helper calls. | non-blocking | defer | Current `rg` confirms no reverse calls; existing registry test guards external entrypoints. | No new static test in this slice. | Consider adding in Phase 8 hardening |
| closure-architecture-adversary | `external_runner_configured` lacked official result path and command snapshot path. | Incident responder would need filesystem inference. | non-blocking | accept | Low-cost observability improvement. | Added `official_result_path=` and `command_snapshot_path=` to event message and `ADAPT-RUNTIME-005` assertions. | None |

### Closure Status

- Blocking findings found: yes in Round 2; accepted and fixed
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 2
- Blocking re-review launch records:
  - closure-architecture-adversary `019e9933-01cf-7960-be60-26a04fae59bc`
  - closure-test-validity-adversary `019e9933-43cc-7511-9558-b7aab5b6e024`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

Passed. Round 1 accepted blockers were fixed and re-reviewed by fresh internal
subagents. Round 2 found one remaining documentation blocker, which was
accepted and fixed in the architecture design. The final targeted validation
passed: `ADAPT-RUNTIME-001`, `ADAPT-RUNTIME-002`, `ADAPT-RUNTIME-005`,
`TB-001..011` via executable loop, `PY-TB-001`, `cargo fmt --all -- --check`,
`cargo check --workspace --all-targets --all-features`,
`scripts/verify-test-registry.sh`, and
`scripts/verify-planned-adapter-selectors.sh`.
