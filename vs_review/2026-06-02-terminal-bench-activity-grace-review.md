# Subagent VS Review: Terminal-Bench Activity Grace

- Created: 2026-06-02T08:11:39+0800
- Updated: 2026-06-02T08:37:30+0800
- Report schema: adversarial-v1
- Task: Fix HarnessLab no-output monitoring so stale Docker setup/build activity cannot keep a Terminal-Bench task alive indefinitely.
- Report path: `vs_review/2026-06-02-terminal-bench-activity-grace-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: open

## Round 1: Bounded Activity Grace Review

### Review Input

#### Objective
Ensure a real Terminal-Bench run cannot waste unbounded time when Docker setup/build subprocesses remain alive but produce no durable benchmark progress.

#### Review Target
Implementation, tests, logging, and documentation for bounded no-output activity deferral.

#### Target Locations
- `crates/harnesslab-infra/src/process.rs`
- `crates/harnesslab-infra/src/process_tests.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/tests/terminal_bench_activity_grace_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/mvp-development-spec.md`
- `docs/development-operations.md`
- `docs/architecture.md`

#### Change Introduction
The executor now treats progress-file growth as durable progress and resets the no-output window. A matching activity process, such as `docker compose` or `docker-buildx`, can defer no-progress for at most one additional no-output watchdog window. Terminal-Bench runner configuration events now include `activity_grace_sec`.

#### Risk Focus
- A legitimate quiet Docker build might be killed too early.
- A stale but still-alive Docker/buildx process might still defer indefinitely through timing or reset mistakes.
- Progress-file growth and activity signals could interact incorrectly.
- Tests might only verify synthetic sleep behavior and miss the real failure mode.
- Logs might not be sufficient to diagnose why a task was killed.

#### Assumptions To Attack
- One extra watchdog window is the correct boundary for activity-only progress.
- `activity_deferral_started_at` resets only when real output or progress-file growth occurs.
- `C-SBOX-018`, `INT-029`, `INT-035`, and `INT-036` together prove bounded activity grace, real Docker positive behavior, synthetic stale activity, and real Docker stale activity.
- `external_runner_configured` plus `external_runner_activity` plus `external_runner_no_progress` is enough runtime evidence.
- The docs match executable behavior.

#### Adversarial Lenses
- concurrency
- failure
- testing
- observability
- maintenance

#### Verification Status
- RED confirmed before implementation:
  - `cargo test -p harnesslab-infra c_sbox_018_no_output_activity_has_bounded_grace -- --nocapture`
  - `cargo test -p harnesslab-cli --test terminal_bench_failure_contract int_035_terminal_bench_stale_docker_activity_becomes_no_progress -- --nocapture`
- GREEN after implementation:
  - `cargo fmt --all --check`
  - `scripts/test-after-change.sh --select C-SBOX-018`
  - `scripts/test-after-change.sh --select INT-035`
  - `scripts/verify-test-registry.sh`
  - `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Full regression and real full benchmark rerun are still pending.

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
| code-reviewer | Challenge executor state-machine logic and logging semantics. | failure, observability, maintenance |
| test-engineer | Challenge whether tests and registry prevent the regression without self-deception. | testing, real-run confidence |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | multi_agent_v1.spawn_agent | 019e85ac-af5d-76c0-a4f6-b4e963efefad / Feynman | spawn_agent result | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |
| test-engineer | multi_agent_v1.spawn_agent | 019e85ac-e2b7-7d00-a4bd-5e3aa4f5a82a / Franklin | spawn_agent result | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| code-reviewer-output | code-reviewer | 1 | 019e85ac-af5d-76c0-a4f6-b4e963efefad / Feynman | 5 minutes | completed | implementation review completed | completed |
| test-engineer-output | test-engineer | 1 | 019e85ac-e2b7-7d00-a4bd-5e3aa4f5a82a / Franklin | 5 minutes | completed | test-validity review completed | completed |

### Reviewer Outputs

#### code-reviewer-output

##### Summary
REQUEST CHANGES. The bounded-grace control path aligns with the core objective, but final no-progress diagnostics were misleading for the updated contract.

##### Blocking Findings
- `external_runner_no_progress` still reported only "no log output for Ns"; after bounded grace, a kill can occur after progress-file growth and one extra activity window, so the event no longer explained whether activity grace was exhausted.
  - Broken assumption: existing no-progress event text was still accurate.
  - Failure scenario: a real stale build investigation sees earlier activity and later no-progress without the grace-expiration context.
  - Trigger condition: Terminal-Bench Docker setup/build remains alive but `run.log` stops growing.
  - Impact: operators cannot distinguish agent failure, runner stall, stale Docker activity, or progress-file reset behavior.
  - Proof needed: final no-progress event includes bounded-grace and last activity/progress context.

##### Non-blocking Risks
- Progress and activity logic were covered separately but not as `activity -> progress-file growth -> later stale activity`.

##### Required Fixes
- Update final no-progress emission so it reflects durable progress and bounded activity-grace exhaustion.

##### Missing Tests
- Add a unit or integration test proving progress growth resets activity grace and later stale activity gets only one extra window.
- Add an integration assertion for the final no-progress event payload.

##### Missing Logs / Observability
- Add explicit final no-progress diagnostics with `activity_grace_exhausted`, current activity, last activity, and last progress.

##### Evidence
- `crates/harnesslab-infra/src/process.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `docs/architecture.md`
- `docs/development-operations.md`
- `docs/mvp-development-spec.md`

#### test-engineer-output

##### Summary
The first suite was not sufficient to prove the regression was locked down for real Terminal-Bench runs. It needed tighter timing, real-Docker negative coverage, and stronger registry traceability.

##### Blocking Findings
- `C-SBOX-018` only asserted elapsed `< 4s` for a 1s watchdog, so multi-window regressions could still pass.
  - Broken assumption: "not hard timeout" was enough to prove one-window grace.
  - Failure scenario: implementation grants two or three watchdog windows and still passes.
  - Trigger condition: activity-only no-output path.
  - Impact: long stuck Docker activity may still waste time.
  - Proof needed: tighter elapsed bound.
- `INT-035` did not verify elapsed time, so an eventual no-progress before hard timeout could still pass after excessive waiting.
  - Broken assumption: failure classification alone proved monitor timing.
  - Failure scenario: task waits far longer than one grace window and still eventually returns no-progress.
  - Trigger condition: stale synthetic Docker activity.
  - Impact: CLI contract misses wasted runtime.
  - Proof needed: elapsed assertion and run-health checks.
- No real-Docker negative path covered stale Docker setup/build activity expiring into no-progress.
  - Broken assumption: synthetic `docker-buildx` sleep was enough.
  - Failure scenario: real `docker compose build` process tree behaves differently from the synthetic helper.
  - Trigger condition: real Terminal-Bench Docker build stalls without durable `run.log` progress.
  - Impact: production full bench can still stall in Docker despite green tests.
  - Proof needed: real Docker script that exits `execution/external_runner_no_progress` before hard timeout.

##### Non-blocking Risks
- `INT-030` and `INT-035` registry entries missed implementation files.
- Two-token Docker activity patterns had only parser coverage, not full CLI process-group coverage.

##### Required Fixes
- Tighten elapsed assertions.
- Add real Docker stale-activity expiry verifier.
- Expand registry `file_patterns`.

##### Missing Tests
- Real Docker stale-activity expiry.
- Progress growth resets activity grace.
- Run-health assertions for the bounded-grace stale-activity path.

##### Missing Logs / Observability
- Assert final event order/context and avoid relying only on rate-limited deferral events.

##### Evidence
- `crates/harnesslab-infra/src/process_tests.rs`
- `crates/harnesslab-cli/tests/terminal_bench_activity_grace_contract.rs`
- `scripts/verify-terminal-bench-docker-activity-watchdog.sh`
- `tests/TEST_REGISTRY.toml`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| code-reviewer | Final no-progress event was misleading | Event did not mention durable progress or bounded activity grace | blocking | accept | `external_runner_no_progress` had old "no log output" wording | Added `process_no_output.rs` state, final no-progress event with `activity_grace_exhausted`, `current_activity`, `last_activity`, `last_progress`; Terminal-Bench now configures `no_progress_event_name` | Round 2 closure review |
| code-reviewer | Missing combined progress/activity reset test | Progress growth could fail to reset grace without detection | non-blocking | accept | Important state-machine interaction | Added `c_sbox_018_progress_growth_resets_activity_grace` | Round 2 closure review |
| test-engineer | `C-SBOX-018` elapsed bound too loose | Multi-window grace could pass | blocking | accept | `< 4s` did not prove one extra window | Tightened to `< 2700ms` for 1s watchdog | Round 2 closure review |
| test-engineer | `INT-035` lacked elapsed/run-health/final-event assertions | Eventual no-progress could hide long wait | blocking | accept | Failure code alone was insufficient | Added elapsed `< 9s`, run-health counters, `activity_grace_exhausted`, `last_activity`, and pattern assertions | Round 2 closure review |
| test-engineer | No real Docker negative stale-activity path | Synthetic sleep was not equivalent to real Docker build | blocking | accept | Production issue came from real Docker build | Added `scripts/verify-terminal-bench-docker-activity-grace-expiry.sh` and `INT-036` | Round 2 closure review |
| test-engineer | Registry traceability incomplete | Future change-based selection could skip relevant tests | non-blocking | accept | Implementation files missing from `file_patterns` | Expanded `INT-029`, `INT-035`, `INT-036`, and C-SBOX file patterns | Round 2 closure review |

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
Verify that Round 1 accepted blocking findings are actually fixed, not only explained.

#### Review Target
Closure state for bounded no-output activity grace implementation, diagnostics, and tests.

#### Target Locations
- `crates/harnesslab-infra/src/process.rs`
- `crates/harnesslab-infra/src/process_no_output.rs`
- `crates/harnesslab-infra/src/process_activity.rs`
- `crates/harnesslab-infra/src/process_tests.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/tests/terminal_bench_activity_grace_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `scripts/verify-terminal-bench-docker-activity-watchdog.sh`
- `scripts/verify-terminal-bench-docker-activity-grace-expiry.sh`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `vs_review/2026-06-02-terminal-bench-activity-grace-review.md`

#### Change Introduction
Round 1 fixes added a dedicated `process_no_output.rs` state object, final no-progress diagnostics, tighter timing assertions, a progress-reset test, a real Docker stale-activity expiry verifier, and expanded registry traceability.

#### Risk Focus
- Closure review must disprove whether accepted blockers remain.
- New diagnostics must be accurate and not duplicate misleading old events.
- Timing tests must be strong enough without being fragile.
- Real Docker negative test must use HarnessLab formal CLI and real Terminal-Bench/Docker path.

#### Assumptions To Attack
- `activity_grace_exhausted` final event proves the correct condition.
- `C-SBOX-018`, `INT-035`, and `INT-036` cover the regression without excessive flakiness.
- `INT-029` proves the real quiet Docker activity success path.
- The test registry cannot lose these checks when implementation files change.

#### Adversarial Lenses
- failure
- testing
- observability
- maintenance

#### Verification Status
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `scripts/test-after-change.sh --select C-SBOX-018` passed.
- `scripts/test-after-change.sh --select INT-035` passed.
- `scripts/test-after-change.sh --select INT-036` passed.
- `scripts/verify-terminal-bench-docker-activity-watchdog.sh` passed.
- `scripts/verify-test-registry.sh` passed with 104 tests before `INT-030` was retired; final verification uses 103 tests.
- Full regression passed after this report update:
  - `scripts/test-after-change.sh`
  - Rust nextest: 283/283 passed
  - Python Terminal-Bench adapter tests: 22/22 passed
  - real Terminal-Bench import timeout cleanup passed
  - real Terminal-Bench Docker activity watchdog passed
  - real Terminal-Bench Docker activity grace expiry passed
  - registry check passed with 103 tests
  - coverage passed: line 95.22% (6918/7265), branch 82.97% (604/728)

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 12 minutes | one 6 minute extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| code-reviewer | Closure check for implementation and observability blockers. | failure, observability, maintenance |
| test-engineer | Closure check for test-validity blockers. | testing, traceability, real-run confidence |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | multi_agent_v1.spawn_agent | 019e85be-0d48-7411-b5b2-7890a03abaf6 / Rawls | spawn_agent result | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions | yes |
| test-engineer | multi_agent_v1.spawn_agent | 019e85be-4a21-77f3-a1ba-fecbea101dd8 / McClintock | spawn_agent result | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| closure-code-reviewer-output | code-reviewer | 1 | 019e85be-0d48-7411-b5b2-7890a03abaf6 / Rawls | 5 minutes | completed | closure implementation review completed | completed |
| closure-test-engineer-output | test-engineer | 1 | 019e85be-4a21-77f3-a1ba-fecbea101dd8 / McClintock | 5 minutes | completed | closure test review completed | completed |

### Reviewer Outputs

#### closure-code-reviewer-output

##### Summary
Accepted Round 1 blocking findings are fixed. No remaining closure blocker for bounded Terminal-Bench no-output activity grace.

##### Blocking Findings
- none

##### Non-blocking Risks
- none

##### Required Fixes
- none

##### Missing Tests
- Non-blocking: assertions did not explicitly check `current_activity=` or `last_progress=` in the final payload.

##### Missing Logs / Observability
- none

##### Evidence
- `crates/harnesslab-infra/src/process_no_output.rs` emits final no-progress diagnostics with durable-progress wording and `activity_grace_exhausted`, `current_activity`, `last_activity`, `last_progress`.
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs` wires `no_progress_event_name` and no longer emits the old misleading no-progress message.
- `crates/harnesslab-infra/src/process_tests.rs` covers bounded grace and progress-reset-plus-later-stale-activity.
- `crates/harnesslab-infra/src/process.rs` is below 500 lines.

#### closure-test-engineer-output

##### Summary
Accepted Round 1 test-engineering blockers appear closed. No remaining blocking gap against the five verification objectives.

##### Blocking Findings
- none

##### Non-blocking Risks
- The synthetic `INT-030` was weaker than the real-Docker checks and was retired after full regression exposed timing flakiness.
- Script-backed selectors use `exec` and do not share the cargo-test single-test guard; acceptable for script checks.

##### Required Fixes
- none

##### Missing Tests
- none

##### Missing Logs / Observability
- none

##### Evidence
- `C-SBOX-018` has a tighter elapsed bound for a 1s watchdog.
- `INT-035` verifies elapsed time, run-health counters, and final diagnostics.
- `INT-036` real Docker stale-activity expiry is wired into `scripts/test-after-change.sh` and `tests/TEST_REGISTRY.toml`.
- Registry `file_patterns` cover implementation files for `C-SBOX-018`, `INT-029`, `INT-035`, and `INT-036`.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| closure-code-reviewer | Missing explicit `current_activity` / `last_progress` assertions | Diagnostic fields could regress silently | non-blocking | accept | Low-cost assertion improves observability contract | Added `current_activity=pid=` and `last_progress=none` assertions to `INT-035`; added `current_activity=pid=` assertion to `INT-036` script | Full regression |
| closure-test-engineer | `INT-030` lacks elapsed/order assertions | Success-path diagnosis weaker than failure-path checks | non-blocking | accept | Full regression later showed the synthetic check was flaky | Retired `INT-030`; real positive coverage remains in `INT-029`, negative coverage in `INT-036` | Full regression rerun |
| closure-test-engineer | Script selectors use `exec` path | Script checks do not have cargo single-test guard | non-blocking | reject | Script-backed checks cannot use cargo-test output; registry maps them directly and script exits enforce result | None | none |
| full-regression | `INT-030` success-path activity test raced under nextest | Synthetic Docker activity relied on fragile process-name and probe timing | non-blocking | accept | Full `scripts/test-after-change.sh` failed repeatedly at `INT-030`; real `INT-029` and `INT-036` cover the production path better | Removed `INT-030` from code, selector, and registry | Full regression rerun |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 2
- Blocking re-review launch records:
  - code-reviewer 019e85be-0d48-7411-b5b2-7890a03abaf6 / Rawls
  - test-engineer 019e85be-4a21-77f3-a1ba-fecbea101dd8 / McClintock
- Rejected findings backed by evidence: yes
- Deferred findings documented: n/a
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

Blocking review is closed. Full regression passed, including the real Docker activity watchdog and stale-activity expiry paths. Pending commit and real full benchmark rerun.
