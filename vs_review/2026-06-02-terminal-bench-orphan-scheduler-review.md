# Subagent VS Review: Terminal-Bench Orphan Cleanup And Scheduler Refill

- Created: 2026-06-02T09:57:54+0800
- Updated: 2026-06-02T09:57:54+0800
- Report schema: adversarial-v1
- Task: Fix real Terminal-Bench run engineering failures where host agent processes can survive a completed task and where slow active tasks block scheduler throughput.
- Report path: `vs_review/2026-06-02-terminal-bench-orphan-scheduler-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: open

## Round 1: Cleanup And Scheduler Review

### Review Input

#### Objective
Ensure HarnessLab can run a real Terminal-Bench full benchmark without leaking host CLI agent processes after a task exits and without leaving free concurrency slots idle while slow tasks continue.

#### Review Target
Implementation, tests, registry traceability, and operations documentation for Terminal-Bench host agent cleanup and attempt scheduling.

#### Target Locations
- `integrations/terminal_bench/harnesslab_tb_process.py`
- `integrations/terminal_bench/harnesslab_tb_process_test.py`
- `crates/harnesslab-cli/src/runner/attempts.rs`
- `crates/harnesslab-cli/src/runner/attempts_tests.rs`
- `scripts/verify-terminal-bench-python-adapter.sh`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/architecture.md`
- `docs/development-operations.md`
- `docs/mvp-development-spec.md`

#### Change Introduction
The Terminal-Bench Python adapter now scans for inherited `HARNESSLAB_AGENT_RUN_TOKEN` processes after a successful agent command exits and cleans any lingering descendants before returning. The run attempt scheduler now uses a dynamic worker-pool refill loop so a completed attempt immediately frees a slot for the next pending attempt instead of waiting for the original fixed-size chunk to drain.

#### Risk Focus
- A successful CLI command could still leave detached, token-inheriting host agent processes running and consuming API budget.
- Cleanup might kill unrelated processes or miss descendants reparented to PID 1.
- Cleanup diagnostics might be visible but not actionable enough to stop bad benchmark results.
- Dynamic scheduling might mishandle run-health abort, worker errors, panics, pending interrupted results, or result ordering.
- Tests might prove only synthetic helpers and miss the real observed failure modes.

#### Assumptions To Attack
- `HARNESSLAB_AGENT_RUN_TOKEN` is sufficient to identify lingering host agent descendants without killing unrelated processes.
- Normal-exit cleanup should fail the task when token-inheriting survivors remain after SIGTERM/SIGKILL.
- Worker-pool refill preserves the existing run-health abort semantics while improving throughput.
- `ORCH-015` and `PY-TB-001` are enough regression coverage for these two failures.
- The docs and operations notes make it clear that orphan host agent processes are engineering failures, not benchmark failures.

#### Adversarial Lenses
- implementation
- concurrency
- failure
- testing
- observability
- maintenance

#### Verification Status
- Local focused validations already passed before review:
  - `cargo test -p harnesslab-cli --all-features runner::attempts::tests::run_004_attempt_scheduler_refills_slot_before_slow_task_finishes -- --exact`
  - `scripts/test-after-change.sh --select ORCH-015`
  - `scripts/test-after-change.sh --select PY-TB-001`
  - `cargo fmt --all --check`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
  - `scripts/verify-test-registry.sh`
- Full `scripts/test-after-change.sh`, commit, and real Terminal-Bench rerun are still pending.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 12 minutes | one 6 minute extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| code-reviewer | Challenge process cleanup correctness, worker-pool scheduling, abort handling, and error paths. | implementation, concurrency, failure |
| test-engineer | Challenge whether the new tests and registry entries lock down the real failures without self-deception. | testing, observability, regression confidence |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | multi_agent_v1.spawn_agent | 019e860e-b973-7612-a297-2582588c5fe6 / Darwin | spawn_agent result | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |
| test-engineer | multi_agent_v1.spawn_agent | 019e860f-0412-78b2-b80a-4b6ec96f5d28 / Boyle | spawn_agent result | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| code-reviewer-output | code-reviewer | 1 | 019e860e-b973-7612-a297-2582588c5fe6 / Darwin | 7 minutes | completed | implementation review completed | completed |
| test-engineer-output | test-engineer | 1 | 019e860f-0412-78b2-b80a-4b6ec96f5d28 / Boyle | 5 minutes | completed | test-validity review completed | completed |

### Reviewer Outputs

#### code-reviewer-output

##### Summary
REQUEST CHANGES. The dynamic refill loop matches the scheduling intent, but Terminal-Bench cleanup still had execution-layer holes.

##### Blocking Findings
- Normal-exit or timeout cleanup failure still degraded to benchmark `test_failed` instead of an execution failure.
  - Broken assumption: cleanup failure merely failing the task was sufficient.
  - Failure scenario: agent exits or times out, cleanup fails, adapter maps to `UNKNOWN_AGENT_ERROR`, and HarnessLab parser falls back to benchmark `test_failed`.
  - Trigger condition: any lingering host descendant that survives cleanup.
  - Impact: a run with leaked host agent processes can produce benchmark-looking evidence instead of execution invalidation.
  - Proof needed: cleanup failure maps to execution failure and has a targeted test.
- The orphan scan was only token-based after process exit, so it could miss descendants that clear `HARNESSLAB_AGENT_RUN_TOKEN` before daemonizing.
  - Broken assumption: token matching is sufficient after parent exit.
  - Failure scenario: wrapper clears the token, spawns detached helper, exits, and the helper is no longer discoverable by root ancestry or token scan.
  - Trigger condition: env-sanitizing CLI helper or daemonized child.
  - Impact: host agent/helper processes can keep running and consuming API budget while the benchmark appears clean.
  - Proof needed: regression where a token-cleared descendant is captured and cleaned.

##### Non-blocking Risks
- Registry traceability for `ORCH-015` and `PY-TB-001` pointed at the wrong spec section.
  - Broken assumption: current registry source links were audit-useful.
  - Failure scenario: future traceability review cannot connect tests to the actual concurrency or cleanup contracts.
  - Trigger condition: test registry review or change-based test selection audit.
  - Impact: test coverage can be misunderstood or skipped.
  - Proof needed: corrected registry source sections.

##### Required Fixes
- Reclassify adapter cleanup failures as `FailureClass::Execution`.
- Replace or augment post-exit token scanning with ancestry capture.
- Correct registry source sections.

##### Missing Tests
- Real imported Terminal-Bench run where normal-exit cleanup path is exercised.
- Regression for token-cleared detached child.
- Scheduler tests for abort, worker error, and worker panic semantics.

##### Missing Logs / Observability
- Successful normal-exit cleanup was not durably recorded; it only printed to stderr.

##### Evidence
- `integrations/terminal_bench/harnesslab_tb_process.py`
- `integrations/terminal_bench/harnesslab_tb_agent.py`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_result.rs`
- `crates/harnesslab-cli/src/runner/attempts.rs`
- `crates/harnesslab-cli/src/runner/attempts_tests.rs`
- `tests/TEST_REGISTRY.toml`

#### test-engineer-output

##### Summary
The main functional holes were addressed, but one blocking observability gap remained and scheduler edge cases needed better tests.

##### Blocking Findings
- Successful orphan cleanup was not persisted anywhere actionable.
  - Broken assumption: stderr diagnostics on the normal-exit path are visible enough.
  - Failure scenario: agent exits `0`, leaves a detached token-inheriting child, adapter kills it successfully, and the task still looks like a clean success with no artifact proving forced cleanup occurred.
  - Trigger condition: any success-path cleanup where `terminate_lingering_processes()` returns successful cleanup.
  - Impact: operators cannot distinguish clean success from success after forced host cleanup.
  - Proof needed: durable warning/event/log artifact visible after the run.

##### Non-blocking Risks
- Dynamic refill can start extra attempts before a latent slow worker error or panic becomes observable.
  - Broken assumption: refill preserves old fail-fast timing for not-yet-observed worker failures.
  - Failure scenario: one slow worker will later error while fast tasks keep completing and refilling.
  - Trigger condition: `concurrency > 1` with a latent worker failure.
  - Impact: a doomed run can consume extra API/task budget before the error is observed.
  - Proof needed: explicit tests or design decision for worker error/panic behavior.

##### Required Fixes
- Persist success-path orphan-cleanup diagnostics in a durable HarnessLab artifact or structured event.

##### Missing Tests
- No test proved successful post-exit cleanup is durably observable.
- No test covered success-path cleanup failure mapping.
- Scheduler tests covered refill happy path only, not abort/error/panic semantics.

##### Missing Logs / Observability
- No persisted success-path cleanup signal existed.

##### Evidence
- `integrations/terminal_bench/harnesslab_tb_process.py`
- `integrations/terminal_bench/harnesslab_tb_agent.py`
- `integrations/terminal_bench/harnesslab_tb_agent_test.py`
- `crates/harnesslab-cli/src/runner/attempts.rs`
- `crates/harnesslab-cli/src/runner/attempts_tests.rs`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| code-reviewer | Cleanup failure degraded to benchmark `test_failed` | Leaked host process cleanup failure invalidates execution, not benchmark scoring | blocking | accept | `terminal_bench_result_failed_adapter_cleanup_is_execution_failure` now passes | Added `FailureCode::AgentCleanupFailed`; parser maps cleanup failure logs to `FailureClass::Execution`; updated docs | Round 2 closure review |
| code-reviewer | Token-only post-exit scan can miss env-sanitized descendants | Token-cleared detached child can outlive parent and disappear from root ancestry | blocking | accept | `test_success_kills_reparented_descendant_that_clears_run_token` now passes | Added `ProcessTracker` running during agent execution to capture ancestry snapshots and merge them with token scanning during cleanup | Round 2 closure review |
| code-reviewer | Registry source sections were wrong | Audit could point tests to unrelated replay section | major | accept | `scripts/verify-test-registry.sh` reports 109 tests | Corrected `ORCH-015` to `9.3`, moved `PY-TB-001` to architecture `7.1`, added `ORCH-016/017/018` and `INT-037` | Round 2 closure review |
| code-reviewer | Missing real normal-exit cleanup run | Normal-exit cleanup was only covered by unit tests | blocking | accept | `scripts/test-after-change.sh --select INT-037` passed with benchmark success and cleanup artifact | Added `scripts/verify-terminal-bench-import-success-cleanup.sh` and registry `INT-037` | Round 2 closure review |
| code-reviewer | Missing scheduler abort/error/panic tests | Refill could continue after abort/error/panic without regression coverage | major | accept | `ORCH-016`, `ORCH-017`, and `ORCH-018` passed | Added deterministic scheduler tests for run-health abort, worker error, and worker panic | Round 2 closure review |
| test-engineer | Successful cleanup was not durably observable | `stderr` on successful adapter subprocess was discarded by agent wrapper | blocking | accept | `test_perform_task_persists_successful_cleanup_diagnostics` and `INT-037` passed | Added `agent_cleanup.log` via `cleanup_log_path`; timeout and success cleanup write durable diagnostics | Round 2 closure review |
| test-engineer | Latent worker failures may allow speculative work before observation | Dynamic refill cannot stop work before the failure signal is received | major | accept | Docs now state already-started workers are allowed to finish; tests cover no new dispatch after observed error/panic | Added `ORCH-017/018` and explicit scheduler contract text | Round 2 closure review |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: pending
- Blocking re-review passed: pending
- Blocking re-review round links:
  - Round 2 pending
- Blocking re-review launch records:
  - pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: pending closure review
- Allowed to proceed: no

## Round 2: Blocking Fix Closure Review

### Review Input

#### Objective
Verify that Round 1 accepted blocking findings are fixed in code, tests, registry, and operations docs.

#### Review Target
Closure review for Terminal-Bench adapter cleanup and dynamic scheduler edge-case fixes.

#### Target Locations
- `integrations/terminal_bench/harnesslab_tb_process.py`
- `integrations/terminal_bench/harnesslab_tb_process_test.py`
- `integrations/terminal_bench/harnesslab_tb_agent.py`
- `integrations/terminal_bench/harnesslab_tb_agent_test.py`
- `crates/harnesslab-core/src/model.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_result.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `crates/harnesslab-cli/src/runner/attempts.rs`
- `crates/harnesslab-cli/src/runner/attempts_tests.rs`
- `scripts/verify-terminal-bench-import-timeout-cleanup.sh`
- `scripts/verify-terminal-bench-import-success-cleanup.sh`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/architecture.md`
- `docs/development-operations.md`
- `docs/mvp-development-spec.md`

#### Change Introduction
The implementation now records runtime ancestry snapshots for host agent processes, persists cleanup diagnostics to `agent_cleanup.log`, maps cleanup failures to execution `agent_cleanup_failed`, adds real normal-exit cleanup verification, and expands scheduler tests for run-health abort, worker error, and worker panic.

#### Risk Focus
- The ancestry tracker might still miss token-cleared descendants or introduce thread leaks.
- Cleanup failure might still be scored as benchmark failure in some parser path.
- Durable cleanup diagnostics might not be present in real Terminal-Bench artifacts.
- Scheduler edge-case tests might be timing-sensitive or not prove the contract.
- Test registry/docs might still not match executable behavior.

#### Assumptions To Attack
- `ProcessTracker` plus token scanning is a meaningful closure for token-cleared descendant cleanup.
- `agent_cleanup.log` is persisted under official task logs for timeout and normal-exit cleanup.
- `agent_cleanup_failed` invalidates benchmark results as an execution failure.
- `ORCH-016`, `ORCH-017`, `ORCH-018`, `PY-TB-001`, and `INT-037` close the test gaps.

#### Adversarial Lenses
- implementation
- concurrency
- failure
- testing
- observability

#### Verification Status
- `PYTHONPATH="$PWD/integrations/terminal_bench" uvx --from terminal-bench python -m unittest integrations/terminal_bench/harnesslab_tb_agent_test.py integrations/terminal_bench/harnesslab_tb_process_test.py`
- `cargo test -p harnesslab-cli --all-features runner::external::tests::terminal_bench_result_failed_adapter_cleanup_is_execution_failure -- --exact`
- `scripts/test-after-change.sh --select ORCH-015`
- `scripts/test-after-change.sh --select ORCH-016`
- `scripts/test-after-change.sh --select ORCH-017`
- `scripts/test-after-change.sh --select ORCH-018`
- `scripts/test-after-change.sh --select PY-TB-001`
- `scripts/test-after-change.sh --select INT-037`
- `scripts/verify-terminal-bench-import-timeout-cleanup.sh`
- `cargo fmt --all --check`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `scripts/verify-test-registry.sh`
- Full `scripts/test-after-change.sh` and real full benchmark rerun are still pending.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 12 minutes | one 6 minute extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| code-reviewer | Verify accepted implementation blockers are actually closed. | implementation, failure, concurrency |
| test-engineer | Verify new tests and real scripts prove the intended behavior. | testing, observability, registry |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | multi_agent_v1.spawn_agent | 019e8621-8d7c-7262-b5de-cad84f6239f8 / Kepler | spawn_agent result | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |
| test-engineer | multi_agent_v1.spawn_agent | 019e8621-dd5b-7193-9807-9eaccfe29485 / Kant | spawn_agent result | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| closure-code-reviewer-output | code-reviewer | 1 | 019e8621-8d7c-7262-b5de-cad84f6239f8 / Kepler | 6 minutes | completed | closure implementation review completed | completed |
| closure-test-engineer-output | test-engineer | 1 | 019e8621-dd5b-7193-9807-9eaccfe29485 / Kant | 5 minutes | completed | closure test review completed | completed |

### Reviewer Outputs

#### closure-code-reviewer-output

##### Summary
REQUEST CHANGES. Core code changes were mostly in place, but real normal-exit cleanup proof was not reproducible and architecture docs still omitted the new failure code.

##### Blocking Findings
- `INT-037` was not stable on rerun because it required full benchmark success rather than directly proving cleanup artifact persistence.
  - Broken assumption: the real normal-exit cleanup proof was stable.
  - Failure scenario: `agent_cleanup.log` existed, but the script failed due benchmark `test_failed` or `verifier_timeout`.
  - Trigger condition: current machine reruns of `scripts/verify-terminal-bench-import-success-cleanup.sh`.
  - Impact: full regression gate can fail without invalidating the cleanup contract.
  - Proof needed: make `INT-037` pass reproducibly or narrow it to cleanup proof.

##### Non-blocking Risks
- none

##### Required Fixes
- Add `agent_cleanup_failed` to the architecture failure-code table.

##### Missing Tests
- Add direct parser regression for normal-exit `agent_cleanup.log` failure, including a nominal success score overridden by cleanup failure.

##### Missing Logs / Observability
- none

##### Evidence
- `scripts/verify-terminal-bench-import-success-cleanup.sh`
- `docs/architecture.md`
- `crates/harnesslab-cli/src/runner/external_tests.rs`

#### closure-test-engineer-output

##### Summary
REQUEST CHANGES. Most Round 1 blockers were substantively closed, but sampled ancestry still allowed token-cleared descendants to escape and timeout cleanup lacked a dedicated registry entry.

##### Blocking Findings
- `ProcessTracker` was sampling-based, so token-cleared descendants could escape if the launcher was shorter-lived than the polling interval.
  - Broken assumption: sampled ancestry plus token scan fully closed token-cleared descendant cleanup.
  - Failure scenario: short-lived helper clears `HARNESSLAB_AGENT_RUN_TOKEN`, spawns detached child, exits before the next sample.
  - Trigger condition: env-sanitizing daemonizing helper.
  - Impact: original orphan-host-process failure can recur while the run appears clean.
  - Proof needed: stronger capture mechanism or deterministic regression for the race.

##### Non-blocking Risks
- Real timeout-path cleanup artifact check was in the full gate but lacked a dedicated registry entry.
  - Broken assumption: registry and executable coverage fully aligned for cleanup.
  - Failure scenario: targeted selector validation misses timeout cleanup artifact regression.
  - Trigger condition: selector-driven validation instead of full regression.
  - Impact: timeout cleanup artifact persistence can regress unnoticed.
  - Proof needed: registry-backed selector.

##### Required Fixes
- Close or explicitly bound the sampled ancestry race.
- Add registry-backed timeout cleanup verification.

##### Missing Tests
- Deterministic regression for fast token-cleared detach.
- Parser test for normal-exit cleanup-failure string path.

##### Missing Logs / Observability
- none

##### Evidence
- `integrations/terminal_bench/harnesslab_tb_process.py`
- `integrations/terminal_bench/harnesslab_tb_process_test.py`
- `scripts/verify-terminal-bench-import-timeout-cleanup.sh`
- `tests/TEST_REGISTRY.toml`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| code-reviewer | `INT-037` failed by depending on full benchmark success | Cleanup proof should not fail solely because official verifier times out after cleanup artifact exists | blocking | accept | `scripts/test-after-change.sh --select INT-037` now passes and printed `result ok: none None` on rerun | Increased run timeout and allowed benchmark-level failure while still requiring `agent_cleanup.log`, no live marker process, and no `agent_cleanup_failed` | Round 3 closure review |
| code-reviewer | Architecture failure-code table omitted `agent_cleanup_failed` | Docs were out of sync with executable failure taxonomy | major | accept | `docs/architecture.md` table updated | Added `agent_cleanup_failed` to execution code list | Round 3 closure review |
| code-reviewer | Missing normal-exit cleanup-failure parser test | Parser only covered timeout-style `succeeded=False` | major | accept | `terminal_bench_result_live_child_cleanup_error_is_execution_failure` now passes | Added parser test for `left live child processes` overriding official result as `execution/agent_cleanup_failed` | Round 3 closure review |
| test-engineer | Sampled ancestry still missed fast token-cleared detach | A helper could clear token and exit before polling saw it | blocking | accept | Fast-detach test initially failed, then strict diagnostic mode test passed | Added supervisor attach delay, kqueue fork-event capture where available, high-frequency startup capture, and explicit strict global process scan mode for unowned live pid diagnostics; documented default safety boundary | Round 3 closure review |
| test-engineer | Timeout cleanup script lacked dedicated registry entry | Selector-driven validation could skip timeout artifact persistence | major | accept | `scripts/test-after-change.sh --select INT-038` passed | Added `INT-038` selector and registry entry for timeout cleanup artifact proof | Round 3 closure review |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: pending
- Blocking re-review passed: pending
- Blocking re-review round links:
  - Round 3 pending
- Blocking re-review launch records:
  - pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: pending Round 3 closure review
- Allowed to proceed: no

## Round 3: Final Closure Review

### Review Input

#### Objective
Verify that Round 2 closure findings are fixed and no accepted blocking issue remains open before full regression and real benchmark rerun.

#### Review Target
Final closure review for Terminal-Bench process cleanup diagnostics, result classification, registry coverage, and scheduler edge-case coverage.

#### Target Locations
- `integrations/terminal_bench/harnesslab_tb_process.py`
- `integrations/terminal_bench/harnesslab_tb_process_test.py`
- `integrations/terminal_bench/harnesslab_tb_agent.py`
- `integrations/terminal_bench/harnesslab_tb_agent_test.py`
- `crates/harnesslab-core/src/model.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_result.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `crates/harnesslab-cli/src/runner/attempts.rs`
- `crates/harnesslab-cli/src/runner/attempts_tests.rs`
- `scripts/verify-terminal-bench-import-success-cleanup.sh`
- `scripts/verify-terminal-bench-import-timeout-cleanup.sh`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/architecture.md`
- `docs/development-operations.md`
- `docs/mvp-development-spec.md`

#### Change Introduction
Round 2 follow-up made `INT-037` prove cleanup artifact persistence instead of depending on full benchmark success, added `INT-038`, added a normal-exit cleanup-failure parser test, updated architecture failure-code docs, and made strict global process scan an explicit diagnostic mode rather than a default that can misclassify concurrent processes.

#### Risk Focus
- The strict diagnostic mode may be overclaimed as default cleanup capability.
- Normal-exit and timeout cleanup artifact tests may still be weak or flaky.
- Parser cleanup failure priority may still miss success-score override.
- Registry/docs may still be incomplete.
- Full regression remains pending.

#### Assumptions To Attack
- `INT-037` and `INT-038` now stably prove normal-exit and timeout cleanup artifacts.
- `PY-TB-001` covers process cleanup boundaries without leaving test orphans.
- `agent_cleanup_failed` docs and parser behavior are aligned.
- Dynamic scheduler tests cover refill, abort, worker error, and worker panic.

#### Adversarial Lenses
- implementation
- testing
- observability
- failure

#### Verification Status
- `PYTHONPATH="$PWD/integrations/terminal_bench" uvx --from terminal-bench python -m unittest integrations/terminal_bench/harnesslab_tb_process_test.py`
- `scripts/test-after-change.sh --select PY-TB-001`
- `cargo test -p harnesslab-cli --all-features runner::external::tests::terminal_bench_result_live_child_cleanup_error_is_execution_failure -- --exact`
- `scripts/test-after-change.sh --select INT-037`
- `scripts/test-after-change.sh --select INT-038`
- `scripts/verify-test-registry.sh`
- `cargo fmt --all --check`
- Full `scripts/test-after-change.sh` and real full benchmark rerun are still pending.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 12 minutes | one 6 minute extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| code-reviewer | Verify final closure on implementation and docs. | implementation, failure |
| test-engineer | Verify tests, registry, and real scripts are adequate. | testing, observability |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | multi_agent_v1.spawn_agent | 019e8635-2961-78f1-92cf-f3ce0be8a560 / Dewey | spawn_agent result | fork_context=false | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |
| test-engineer | multi_agent_v1.spawn_agent | 019e8635-6d53-7dd3-9a1c-6d9d90f8f226 / Archimedes | spawn_agent result | fork_context=false | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| final-code-reviewer-output | code-reviewer | 1 | 019e8635-2961-78f1-92cf-f3ce0be8a560 / Dewey | completed notification | completed | closure implementation review completed | completed |
| final-test-engineer-output | test-engineer | 1 | 019e8635-6d53-7dd3-9a1c-6d9d90f8f226 / Archimedes | completed notification | completed | closure test review completed | completed |

### Reviewer Outputs

#### final-code-reviewer-output

##### Summary
No accepted Round 2 blocker remained open in code. Targeted diagnostics run by the reviewer passed, but full regression and real full benchmark rerun were still pending.

##### Blocking Findings
none

##### Non-blocking Risks
- Full `scripts/test-after-change.sh` and the real full benchmark rerun were still pending.
- Parser precedence was correct by inspection, but a dedicated regression for official success plus cleanup failure was still missing.

##### Required Fixes
none

##### Missing Tests
- Add a focused parser test for `accuracy=1.0` plus adapter cleanup failure.

##### Missing Logs / Observability
none

##### Evidence
- `crates/harnesslab-cli/src/runner/external/terminal_bench_result.rs`
- `crates/harnesslab-core/src/model.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `integrations/terminal_bench/harnesslab_tb_process.py`
- `integrations/terminal_bench/harnesslab_tb_agent.py`
- `scripts/verify-terminal-bench-import-success-cleanup.sh`
- `scripts/verify-terminal-bench-import-timeout-cleanup.sh`
- `tests/TEST_REGISTRY.toml`

#### final-test-engineer-output

##### Summary
Round 2 accepted blocking issues appeared closed across code, tests, registry, and docs. No accepted blocking issue remained open in the reviewed files.

##### Blocking Findings
none

##### Non-blocking Risks
- `ORCH-017` and `ORCH-018` rely on fixed `sleep(100ms)` delays to prove no refill after error or panic.
- Full regression and real full benchmark rerun were still pending.

##### Required Fixes
none

##### Missing Tests
- A parser test should explicitly prove `agent_cleanup_failed` overrides a `score >= 1.0` success result.
- A default-mode test could document that token-cleared escaped processes are not treated as cleanup failures without strict diagnostic mode.

##### Missing Logs / Observability
none

##### Evidence
- `crates/harnesslab-cli/src/runner/external/terminal_bench_result.rs`
- `docs/architecture.md`
- `docs/mvp-development-spec.md`
- `integrations/terminal_bench/harnesslab_tb_process.py`
- `integrations/terminal_bench/harnesslab_tb_agent.py`
- `scripts/verify-terminal-bench-import-success-cleanup.sh`
- `docs/development-operations.md`
- `crates/harnesslab-cli/src/runner/attempts_tests.rs`
- `tests/TEST_REGISTRY.toml`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| code-reviewer | Missing success-score cleanup-failure parser regression | Cleanup failure could classify as execution but still preserve official success score | major | accept | New `TB-001` selector initially proved the gap by failing with returned score `1.0` | Added `terminal_bench_result_failed_adapter_cleanup_overrides_success_score`, fixed parser to return benchmark score `0.0` on adapter cleanup failure, added selector and registry entry | Round 4 delta review |
| test-engineer | Missing explicit `score >= 1.0` cleanup-failure override test | A nominal official result might mask invalid cleanup | major | accept | `scripts/test-after-change.sh --select TB-001` now passes; all `terminal_bench_result` parser tests pass | Same as above | Round 4 delta review |
| test-engineer | Fixed sleeps in scheduler tests | Possible slow-CI flake in worker error/panic no-refill tests | minor | defer | Not directly related to the current full-benchmark blocker; full regression will still exercise the suite | Keep as future hardening unless it flakes | Full regression |
| test-engineer | Default strict-mode boundary test absent | Default mode behavior is documented by code/docs but not directly tested | minor | defer | Strict-mode behavior is covered by `PY-TB-001`; default boundary is documented | Keep as future hardening unless default scan behavior changes | none |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 3 completed
- Blocking re-review launch records:
  - code-reviewer 019e8635-2961-78f1-92cf-f3ce0be8a560 / Dewey
  - test-engineer 019e8635-6d53-7dd3-9a1c-6d9d90f8f226 / Archimedes
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: n/a
- Allowed to proceed: no, because Round 4 delta review is required after the accepted missing-test fix changed code and exposed a real score bug.

## Round 4: Delta Closure Review

### Review Input

#### Objective
Verify the post-Round-3 delta that fixes the success-score cleanup-failure bug and hardens no-output activity event persistence after the full regression exposed an `INT-035` activity-event flake.

#### Review Target
Delta review for Terminal-Bench parser scoring, TB-001 selector/registry, and no-output activity event observability.

#### Target Locations
- `crates/harnesslab-infra/src/process_no_output.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_result.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`

#### Change Introduction
`TB-001` was added to prove adapter cleanup failure overrides an official success score. It initially failed, revealing that `agent_cleanup_failed` returned a benchmark score of `1.0`; parser now returns `0.0` for cleanup-failed runs. The no-output watchdog now only starts activity-event throttling after append succeeds and emits one activity deferral event before stale activity expires if none has been persisted, so no-progress reports retain activity evidence.

#### Risk Focus
- Cleanup failure may still leak success through score, evaluation, report, or summary.
- Activity event persistence may still be flaky under concurrent full nextest.
- TB-001 selector may be misleading or not registry-backed.
- The event append retry behavior may cause duplicate or suppressed events.

#### Verification Status
- `scripts/test-after-change.sh --select TB-001` passed.
- `cargo test -p harnesslab-cli --all-features runner::external::tests::terminal_bench_result -- --nocapture` passed 11 tests.
- `scripts/test-after-change.sh --select INT-035` passed.
- `cargo nextest run -p harnesslab-cli --all-features int_035_terminal_bench_stale_docker_activity_becomes_no_progress --test-threads 8 --retries 0` passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `scripts/verify-test-registry.sh` passed with 111 tests.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 12 minutes | one 6 minute extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| code-reviewer | Verify implementation semantics for score and event persistence. | implementation, failure |
| test-engineer | Verify selector, registry, and flake resistance. | testing, observability |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | multi_agent_v1.spawn_agent | 019e863f-769e-7f51-94f9-67e445a7bd2b / Confucius | spawn_agent result | fork_context=false | Round 4 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |
| test-engineer | multi_agent_v1.spawn_agent | 019e863f-9e9b-7673-bc44-70ad95babfbf / Pauli | spawn_agent result | fork_context=false | Round 4 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| delta-code-reviewer-output | code-reviewer | 1 | 019e863f-769e-7f51-94f9-67e445a7bd2b / Confucius | completed notification | completed | delta implementation review completed | completed |
| delta-test-engineer-output | test-engineer | 1 | 019e863f-9e9b-7673-bc44-70ad95babfbf / Pauli | completed notification | completed | delta test review completed | completed |

### Reviewer Outputs

#### delta-code-reviewer-output

##### Summary
REQUEST CHANGES. The cleanup-failure success-score path was closed, but the watchdog activity-event flake was not fully closed.

##### Blocking Findings
- Later stale-activity windows could still expire without an activity event because `activity_event_emitted` was not reset when real output/progress cleared the quiet window.

##### Non-blocking Risks
- Registry and selector traceability only covered the success-score override case, not the live-child cleanup marker path.

##### Required Fixes
- Reset `activity_event_emitted` whenever a new no-output window starts after durable stdout/stderr or progress-file growth.
- Add a regression for activity evidence after output/progress reset.
- Add selector/registry coverage for the live-child cleanup failure marker.

##### Missing Tests
- Event persistence across multiple quiet windows.
- Live-child cleanup marker traceability.

##### Missing Logs / Observability
- Artifact-level proof that activity evidence belongs to the stale window before no-progress.

##### Evidence
- `crates/harnesslab-infra/src/process_no_output.rs`
- `crates/harnesslab-infra/src/process_tests.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_result.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`

#### delta-test-engineer-output

##### Summary
REQUEST CHANGES. The delta was not review-clean because the watchdog test did not prove the reset/throttle behavior, and `evaluation.raw_score` could still retain official success during cleanup failure.

##### Blocking Findings
- `process_no_output` could suppress required pre-expiry activity evidence after a later output/progress reset.
- TB-001 did not assert `evaluation.raw_score`, allowing a cleanup-failed attempt artifact to advertise raw success.

##### Non-blocking Risks
- `INT-035` remains timing-sensitive.

##### Required Fixes
- Reset `activity_event_emitted` with the activity-deferral reset.
- Zero `evaluation.raw_score` on cleanup failure.
- Narrow TB-001 registry claims to parser-level coverage or add execute-path evidence.

##### Missing Tests
- Watchdog reset activity evidence regression.
- Cleanup-failed official success should assert both returned score and raw evaluation score.

##### Missing Logs / Observability
- No dedicated event explaining cleanup-failure override.

##### Evidence
- `crates/harnesslab-infra/src/process_no_output.rs`
- `crates/harnesslab-infra/src/process_tests.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_result.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `tests/TEST_REGISTRY.toml`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| code-reviewer | Activity event flag persisted across quiet windows | Later stale Docker/build activity could be killed with no activity evidence in `events.jsonl` | blocking | accept | Round 4 review showed `activity_event_emitted` was latched across reset | `clear_activity_deferral()` now clears `activity_event_emitted`; expiry-side activity event can force append inside throttle | Round 5 closure review |
| test-engineer | Cleanup failure left raw evaluation success score | Attempt artifact could carry `evaluation.raw_score=1.0` while final state was execution failure | blocking | accept | TB-001 was extended to assert `evaluation.raw_score` and initially exposed the gap | Parser now sets `evaluation.raw_score=0.0` and returned benchmark score `0.0` when adapter cleanup failed | Round 5 closure review |
| code-reviewer | Live-child marker lacked selector/registry coverage | Parser path existed but was not tracked as a selectable test | major | accept | `terminal_bench_result_live_child_cleanup_error_is_execution_failure` existed without registry selector | Added `TB-002` selector and registry entry | Round 5 closure review |
| test-engineer | TB-001 registry overclaimed artifact coverage | Unit parser test claimed artifact-event-store artifacts | major | accept | Registry entry listed artifacts not produced by selector | Narrowed TB-001 required artifacts and requirements to parser-level coverage | Round 5 closure review |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: no
- Blocking re-review round links:
  - Round 5 required
- Blocking re-review launch records:
  - code-reviewer 019e863f-769e-7f51-94f9-67e445a7bd2b / Confucius
  - test-engineer 019e863f-9e9b-7673-bc44-70ad95babfbf / Pauli
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: accepted blocking findings required Round 5 re-review
- Allowed to proceed: no

## Round 5: Accepted-Blocker Closure Review

### Review Input

#### Objective
Verify that Round 4 accepted blocking findings were fixed before full regression and real benchmark rerun.

#### Review Target
Closure review for watchdog reset activity evidence, cleanup-failure scoring, and parser selector/registry traceability.

#### Target Locations
- `crates/harnesslab-infra/src/process_no_output.rs`
- `crates/harnesslab-infra/src/process_tests.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_result.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `vs_review/2026-06-02-terminal-bench-orphan-scheduler-review.md`

#### Verification Status
- `scripts/test-after-change.sh --select C-SBOX-019` passed.
- `scripts/test-after-change.sh --select TB-001` passed.
- `scripts/test-after-change.sh --select TB-002` passed.
- `cargo test -p harnesslab-cli --all-features runner::external::tests::terminal_bench_result -- --nocapture` passed.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `scripts/verify-test-registry.sh` passed.
- Python adapter file was split to keep all changed code files under 500 lines.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | multi_agent_v1.spawn_agent | 019e8649-27d8-7e92-856b-31e43a824bde / Wegener | spawn_agent result | fork_context=false | Round 5 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |
| test-engineer | multi_agent_v1.spawn_agent | 019e8649-54b5-7d53-af52-4611cd4efd66 / Hubble | spawn_agent result | fork_context=false | Round 5 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round5-code-reviewer-output | code-reviewer | 1 | 019e8649-27d8-7e92-856b-31e43a824bde / Wegener | completed notification | completed | code review completed | completed |
| round5-test-engineer-output | test-engineer | 1 | 019e8649-54b5-7d53-af52-4611cd4efd66 / Hubble | completed notification | completed | test review completed | completed |

### Reviewer Outputs

#### round5-code-reviewer-output

##### Summary
REQUEST CHANGES for workflow closure only. Runtime code-path fixes looked correct.

##### Blocking Findings
- The tracked review artifact still recorded Round 4 as pending and `Allowed to proceed: no`.

##### Non-blocking Risks
- C-SBOX-019 proved stdout reset but not progress-file reset.
- Forced expiry-side append failure remains silent.

##### Required Fixes
- Update the review artifact to close Round 4 formally.

##### Missing Tests
- Optional progress-file-reset analogue to C-SBOX-019.

##### Missing Logs / Observability
- Review artifact needed closure state.

#### round5-test-engineer-output

##### Summary
REQUEST CHANGES. Selector wiring and registry were mostly honest, but C-SBOX-019 did not fully prove the throttle-sensitive contract.

##### Blocking Findings
- C-SBOX-019 only checked `>= 2` activity events, so a spammy implementation could still pass.

##### Non-blocking Risks
- The synthetic reset output sat on the grace-expiry boundary.
- `agent_cleanup.log` live-child marker path was unpinned.

##### Required Fixes
- Tighten C-SBOX-019 to assert exact per-window activity-event behavior.
- Move reset output away from the exact grace-expiry boundary.
- Add `agent_cleanup.log` live-child marker regression.

##### Missing Tests
- Event-spam regression.
- Cleanup-log live-child marker regression.

##### Missing Logs / Observability
- Nice-to-have: record which cleanup log source triggered classification.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| code-reviewer | Review artifact still pending | Repo workflow cannot close accepted blocker while artifact says pending/no-go | blocking | accept | Report still had Round 4 pending state | Updated report through Round 5 and Round 6 pending | Round 6 closure review |
| test-engineer | C-SBOX-019 allowed event spam | A broken implementation could emit repeated post-reset activity events and still pass | blocking | accept | Test used `>= 2` activity events | Tightened C-SBOX-019 to require exactly two activity events and exactly one post-reset activity event line for `sleep 8` | Round 6 closure review |
| test-engineer | Reset timing sat on grace boundary | Test could miss first activity window or race with no-progress | major | accept | Initial `sleep 2` then `reset` aligned with 1s no-output grace expiry; `sleep 1.5` version missed pattern | Changed test to `sleep 1.8` and matching activity pattern | Round 6 closure review |
| test-engineer | `agent_cleanup.log` live-child marker path unpinned | Parser scans both `agent_error.log` and `agent_cleanup.log`, but only one path had a selector-backed test | major | accept | TB-002 covered `agent_error.log` only | Added `terminal_bench_result_live_child_cleanup_log_is_execution_failure`, TB-003 selector, and registry entry | Round 6 closure review |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: pending
- Blocking re-review passed: pending
- Blocking re-review round links:
  - Round 6 pending
- Blocking re-review launch records:
  - pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: pending Round 6 closure review
- Allowed to proceed: no

## Round 6: Latest Closure Review

### Review Input

#### Objective
Verify that Round 5 accepted blocking findings are fixed.

#### Review Target
Latest closure review for exact watchdog activity-event cardinality, reset timing, TB-003 selector/registry, and workflow closure.

#### Target Locations
- `crates/harnesslab-infra/src/process_no_output.rs`
- `crates/harnesslab-infra/src/process_tests.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_result.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `vs_review/2026-06-02-terminal-bench-orphan-scheduler-review.md`

#### Verification Status
- `scripts/test-after-change.sh --select C-SBOX-019` passed five consecutive runs.
- `scripts/test-after-change.sh --select TB-001` passed.
- `scripts/test-after-change.sh --select TB-002` passed.
- `scripts/test-after-change.sh --select TB-003` passed.
- `cargo test -p harnesslab-cli --all-features runner::external::tests::terminal_bench_result -- --nocapture` passed 12 tests.
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `scripts/verify-test-registry.sh` passed with 114 tests.
- Changed code files are under 500 lines.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| test-engineer | multi_agent_v1.spawn_agent | 019e8656-8c6d-7cc3-8699-50ead776b9aa / Bernoulli | spawn_agent result | fork_context=false | Round 6 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round6-test-engineer-output | test-engineer | 1 | 019e8656-8c6d-7cc3-8699-50ead776b9aa / Bernoulli | completed notification | completed | closure test review completed | completed |

### Reviewer Outputs

#### round6-test-engineer-output

##### Summary
No blocking findings in the Round 6 fix set. The Round 5 blocker about C-SBOX-019 permitting event spam is closed, and TB-003 is selector-backed and registry-backed.

##### Blocking Findings
none

##### Non-blocking Risks
- `sleep 1.8` is still timer-sensitive under an unusually stalled runner, though much safer than the prior boundary case.
- Cleanup-failure detection is substring-based over log tails, which is acceptable for the current marker contract but less precise than structured parsing.

##### Required Fixes
none

##### Missing Tests
- Optional progress-file reset analogue to C-SBOX-019.
- Optional mixed-source precedence test where `agent_error.log` shows timeout cleanup success while `agent_cleanup.log` shows live children.
- Optional large-log tail test around `MAX_LOG_BYTES`.

##### Missing Logs / Observability
- Parser output does not record which file triggered `AgentCleanupFailed`.
- Watchdog events remain text-based rather than structured with a grace-window index.

##### Evidence
- `crates/harnesslab-infra/src/process_tests.rs`
- `crates/harnesslab-infra/src/process_no_output.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_result.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| test-engineer | No blocking findings | Round 5 accepted blockers appear fixed | n/a | accept | C-SBOX-019 now requires exactly two activity events and one post-reset `sleep 8` activity line; TB-003 covers `agent_cleanup.log` live-child marker | No further code change required for closure | Full regression and real benchmark |
| test-engineer | Progress-file reset analogue missing | Same reset helper is used by stdout/output and progress paths, but only stdout reset has the exact event-cardinality regression | minor | defer | Existing `c_sbox_018_progress_growth_resets_activity_grace` covers timing for progress reset; Round 6 blocker was event spam in output-reset path | Track as future hardening, not blocker | none |
| test-engineer | Mixed-source and tail-boundary parser tests absent | Parser is substring-based over log tails | minor | defer | Current TB-001/TB-002/TB-003 cover all marker kinds and both log filenames for live-child marker | Track as future hardening, not blocker | none |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 6 completed
- Blocking re-review launch records:
  - test-engineer 019e8656-8c6d-7cc3-8699-50ead776b9aa / Bernoulli
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: n/a
- Allowed to proceed: yes, after full regression passes

## Final Conclusion

Round 6 closure review passed. Full `scripts/test-after-change.sh` passed, including nextest 291/291, Python adapter tests, real Terminal-Bench import cleanup checks, Docker activity watchdog checks, registry, traceability, secret scan, and coverage at 95.29% line / 83.11% branch. Real full benchmark rerun remains pending.
