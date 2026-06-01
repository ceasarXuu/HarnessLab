# Subagent VS Review: Terminal-Bench Run Monitor Stall Control

- Created: 2026-06-02T00:00:00+08:00
- Updated: 2026-06-02T00:00:00+08:00
- Report schema: adversarial-v1
- Task: prevent real Terminal-Bench runs from wasting time on external runner stalls and prevent interrupted HarnessLab runs from leaving orphan child process groups.
- Report path: `vs_review/2026-06-02-terminal-bench-run-monitor-stall-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: Post-Fix Adversarial Review

### Review Input

#### Objective
HarnessLab should not waste a long real Terminal-Bench run when Docker build/setup stalls, and interrupting HarnessLab should not leave orphan `tb`/Docker child process groups running.

#### Review Target
Implementation, tests, logs, and operations docs for Terminal-Bench no-output watchdog and `HostProcessExecutor` child process group cleanup.

#### Target Locations
- `crates/harnesslab-infra/src/process.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_timeout.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/development-operations.md`

#### Change Introduction
Terminal-Bench no-output watchdog defaults to a concrete setup watchdog instead of disabled. The external process runner tracks Unix child process groups and kills active groups on process timeout, no-progress timeout, and SIGINT/SIGTERM shutdown.

#### Risk Focus
- Signal-safety and registration races.
- Process-group lifecycle and orphan cleanup.
- Watchdog default false positives/false negatives.
- Tests that prove only event text, not behavior.
- Registry and traceability drift.

#### Assumptions To Attack
- A detached child process group cannot start running before the parent tracks it.
- Registry capacity cannot silently drop process groups.
- The CLI-visible watchdog default matches the documented formula.
- Explicit watchdog disable is honored by the CLI path.
- The added tests fail against the previous broken behavior.

#### Adversarial Lenses
- implementation
- concurrency
- failure
- testing
- observability
- maintenance

#### Verification Status
At review launch, targeted tests for watchdog defaults, no-output classification, process timeout cleanup, and INT-025 were passing; full gate had not yet passed after the new changes.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 30 minutes | one bounded extension | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Process-group, signal, and runner timeout behavior can strand real work. | implementation / concurrency / failure |
| test-validity-adversary | Product risk is self-deceptive tests around long real benchmark runs. | testing / observability |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | `multi_agent_v1.spawn_agent` code-reviewer | `019e84ca-da1f-7280-a88a-3d07b4daf0e9` | subagent completion notification | `fork_context=false` | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions | yes |
| test-validity-adversary | `multi_agent_v1.spawn_agent` test-engineer | `019e84cb-0505-7b73-b803-15ce67a1b3e8` | subagent completion notification | `fork_context=false` | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| implementation-adversary-output | implementation-adversary | 1 | `019e84ca-da1f-7280-a88a-3d07b4daf0e9` | under 30m | completed | returned blocking findings | fixes accepted |
| test-validity-adversary-output | test-validity-adversary | 1 | `019e84cb-0505-7b73-b803-15ce67a1b3e8` | under 30m | completed | returned blocking findings | fixes accepted |

### Reviewer Outputs

#### implementation-adversary-output

##### Summary
Spec compliance was partial. The no-output watchdog was wired and observable, but Unix interrupt cleanup still had a spawn/register race and a silent fixed-capacity failure mode.

##### Blocking Findings
- SIGINT/SIGTERM cleanup path had a registration race.
  - Broken assumption: the child could not run detached before registration.
  - Failure scenario: child runs `setsid()` in `pre_exec`; parent receives SIGINT before registering PGID; detached child group survives.
  - Trigger condition: interrupt in the spawn-to-register window.
  - Impact: orphan `tb`/Docker descendants during real runs.
  - Proof needed: close the race and test interrupt cleanup.
- Active process group registry silently stopped tracking children after 128 groups.
  - Broken assumption: registry capacity could not affect correctness.
  - Failure scenario: capacity exhausted, `slot=None`, SIGINT handler cannot see the child.
  - Trigger condition: higher concurrency than the static registry.
  - Impact: partial cleanup on interrupt.
  - Proof needed: fail closed or remove capacity limit, with tests.

##### Non-blocking Risks
- Process-wide SIGINT/SIGTERM handling in a low-level executor bypasses future graceful shutdown.
- Non-Unix orphan cleanup remains less complete than Unix process-group cleanup.

##### Required Fixes
- Close the spawn/register race.
- Remove silent registry overflow.

##### Missing Tests
- Interrupt-path test proving detached child group and descendant exit.
- Capacity test for registry overflow or a validated concurrency cap.
- Exact watchdog value assertion.

##### Missing Logs / Observability
- No durable event from signal shutdown before `_exit`.
- No warning/counter for registry overflow.

##### Evidence
- `crates/harnesslab-infra/src/process.rs` - spawn/register race and registry behavior.
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs` - weak INT-025 assertion before fix.

#### test-validity-adversary-output

##### Summary
Verification was not strong enough before full real Terminal-Bench. Tests proved only that watchdog was not disabled, not the documented value, and did not directly test signal cleanup.

##### Blocking Findings
- Interrupt cleanup was effectively untested.
  - Broken assumption: timeout cleanup tests also prove SIGINT/SIGTERM cleanup.
  - Failure scenario: timeout path works, signal path leaks children.
  - Trigger condition: external interrupt during a real benchmark task.
  - Impact: stale processes and Docker work continue after HarnessLab exits.
  - Proof needed: subprocess-based interrupt test.
- INT-025 was too weak.
  - Broken assumption: `no_output_timeout_sec=` implies correct default formula.
  - Failure scenario: wrong concrete timeout still passes.
  - Trigger condition: regression to any non-disabled value.
  - Impact: operator contract drifts.
  - Proof needed: assert exact configured value.

##### Non-blocking Risks
- Env-disable support was unit-tested but not CLI-tested.
- Registry overflow was silent.

##### Required Fixes
- Assert exact default watchdog timeout.
- Add CLI disable assertion.
- Add SIGTERM/SIGINT process group cleanup test.

##### Missing Tests
- SIGINT/SIGTERM cleanup test.
- Default watchdog value contract.
- Env-disable end-to-end contract.
- Registry-level coverage.

##### Missing Logs / Observability
- No structured evidence when shutdown cleanup runs.

##### Evidence
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `crates/harnesslab-infra/src/process.rs`
- `tests/TEST_REGISTRY.toml`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | Spawn/register interrupt race | Child group could run before being tracked. | blocking | accept | Race was real because `setsid()` happened before parent registration. | Added start-gate wrapper: child waits for parent ack after PGID registration; parent installs signal handlers before spawn. | Round 2 closure review |
| implementation-adversary | Registry overflow silently untracked groups | Capacity exhaustion made signal cleanup partial. | blocking | accept | `slot=None` was previously allowed. | Increased capacity to 4096 and made register fail closed: kill group, wait, structured spawn error. | Round 2 closure review |
| test-validity-adversary | Interrupt cleanup untested | Timeout tests did not prove signal handler path. | blocking | accept | Existing tests only covered executor-owned timeouts. | Added `C-SBOX-014` integration test that sends SIGTERM to helper running `HostProcessExecutor` and asserts process group exit. | Round 2 closure review |
| test-validity-adversary | INT-025 too weak | Any non-disabled timeout could pass. | blocking | accept | Event assertion did not check formula. | INT-025 now asserts `process_timeout_sec=7800 no_output_timeout_sec=3720`; INT-028 asserts disable with `off` and hard timeout behavior. | Round 2 closure review |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 2
- Blocking re-review launch records:
  - `019e84d9-ad64-7c82-b91a-f3d6b83f793e`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: n/a
- Allowed to proceed: yes

## Round 2: Blocking Closure Review

### Review Input

#### Objective
Re-review fixes after accepted blocking findings in the Terminal-Bench run monitor/orphan cleanup work.

#### Review Target
The revised process start gate, process group registry, signal cleanup, and strengthened Terminal-Bench failure contracts.

#### Target Locations
- `crates/harnesslab-infra/src/process.rs`
- `crates/harnesslab-infra/src/process_start_gate.rs`
- `crates/harnesslab-infra/src/process_tests.rs`
- `crates/harnesslab-infra/tests/process_signal_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`

#### Change Introduction
Children are spawned behind an ack gate and registered before the user command can run. Registry capacity now fails closed. Exact watchdog defaults and explicit disable are asserted through CLI artifacts.

#### Risk Focus
- Wrapper correctness.
- FD leak/deadlock.
- Stdin preservation.
- Signal cleanup race closure.
- Registry overflow fail-closed semantics.
- Test validity.

#### Assumptions To Attack
- Wrapper cannot deadlock `Command::spawn`.
- Wrapper cannot steal stdin.
- Parent death before release prevents user command execution.
- Registry overflow is not silent.
- INT-025 and INT-028 prove the documented timeout contract.

#### Adversarial Lenses
- implementation
- concurrency
- testing
- maintenance

#### Verification Status
Targeted infra tests, INT-025, INT-028, C-SBOX-014, and registry check passed before this closure review.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 30 minutes | one bounded extension | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Closure review for accepted implementation blockers. | signal/process cleanup |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | `multi_agent_v1.spawn_agent` code-reviewer | `019e84d9-ad64-7c82-b91a-f3d6b83f793e` | subagent completion notification | `fork_context=false` | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| closure-review-output | implementation-adversary | 1 | `019e84d9-ad64-7c82-b91a-f3d6b83f793e` | under 30m | completed | approved with one low risk | low risk accepted and fixed |

### Reviewer Outputs

#### closure-review-output

##### Summary
Stage 1 spec compliance passes for accepted blockers. Spawn/register race is materially closed, registry no longer silently stops at 128 and fails closed, Terminal-Bench timeout contract is asserted at exact CLI-visible values and disable semantics.

##### Blocking Findings
- none

##### Non-blocking Risks
- Child cleanup still leaked on unexpected stdin write failures.
  - Broken assumption: all early-return paths after child start clean up the child group.
  - Failure scenario: non-`BrokenPipe` stdin write error returns without killing child/process group.
  - Trigger condition: unusual pipe I/O failure after child start.
  - Impact: narrow orphan path.
  - Proof needed: mirror start-gate failure cleanup and add stdin regression.

##### Required Fixes
- none to close accepted blocking findings

##### Missing Tests
- End-to-end registry overflow through `HostProcessExecutor::exec`.
- Parent dies before release without running user command.
- Positive stdin preservation through the wrapper.

##### Missing Logs / Observability
- none blocking

##### Evidence
- `crates/harnesslab-infra/src/process.rs`
- `crates/harnesslab-infra/src/process_start_gate.rs`
- `crates/harnesslab-infra/tests/process_signal_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | Unexpected stdin write failure could leak child group | Early return after child start did not kill process group. | non-blocking | accept | The branch returned `Err` after registration without cleanup. | Added `kill_process_tree`, `wait`, and a positive stdin-through-gate test. | Round 3 narrow follow-up review |
| implementation-adversary | End-to-end overflow and parent-before-release tests missing | Helper-level tests do not cover every possible path. | non-blocking | defer | Existing fail-closed helper and SIGTERM integration cover MVP risk; deeper fault injection can be added later. | No product behavior change required now. | Track in future test-hardening backlog |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: n/a
- Blocking re-review completed: n/a
- Blocking re-review passed: n/a
- Blocking re-review round links:
  - n/a
- Blocking re-review launch records:
  - n/a
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: n/a
- Allowed to proceed: yes

## Round 3: Narrow Follow-Up Review

### Review Input

#### Objective
Review only the follow-up fix: stdin write failures in `HostProcessExecutor` should not leave started child process groups orphaned, and the start-gate wrapper should preserve stdin.

#### Review Target
Small implementation and test changes in the process executor stdin path.

#### Target Locations
- `crates/harnesslab-infra/src/process.rs`
- `crates/harnesslab-infra/src/process_tests.rs`

#### Change Introduction
Non-`BrokenPipe` stdin write failures now kill and wait the child process group before returning. A new `cat` regression verifies stdin is preserved through the start gate.

#### Risk Focus
- Deadlocks.
- Double-kill/drop behavior.
- Losing logs.
- Breaking `BrokenPipe` tolerance.
- Wrapper consuming stdin.

#### Assumptions To Attack
- Cleanup before returning error does not deadlock.
- `BrokenPipe` remains tolerated.
- The start gate does not consume user stdin.

#### Adversarial Lenses
- implementation
- failure
- testing

#### Verification Status
Targeted infra checks passed before this narrow review.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 10 minutes | one bounded extension | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Narrow code-path review after accepted non-blocking fix. | stdin/process cleanup |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | `multi_agent_v1.spawn_agent` code-reviewer | `019e84e0-afb3-7b22-a70c-0b51d8c5c36f` | subagent completion notification | `fork_context=false` | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| stdin-followup-output | implementation-adversary | 1 | `019e84e0-afb3-7b22-a70c-0b51d8c5c36f` | under 10m | completed | approved with low diagnostic risk | accepted-risk for diagnostics only |

### Reviewer Outputs

#### stdin-followup-output

##### Summary
Verdict: approve. No blocking issue found. The branch now cleans up child process groups before returning non-`BrokenPipe` stdin write errors and the `cat` regression verifies the start gate does not steal stdin.

##### Blocking Findings
- none

##### Non-blocking Risks
- Logs emitted before a stdin-write failure can still be lost because output-drain threads start after stdin write.
  - Broken assumption: rare stdin write failures need full diagnostics.
  - Failure scenario: child emits output, stdin write fails, process is killed before drain threads start.
  - Trigger condition: unusual pipe I/O failure.
  - Impact: weaker diagnostics, not orphan cleanup failure.
  - Proof needed: start drain before stdin or drain synchronously if this path becomes important.

##### Required Fixes
- none

##### Missing Tests
- Direct fault-injection for non-`BrokenPipe` cleanup branch.

##### Missing Logs / Observability
- No structured marker for stdin-write cleanup branch.

##### Evidence
- `crates/harnesslab-infra/src/process.rs`
- `crates/harnesslab-infra/src/process_tests.rs`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | Possible log loss on rare stdin write failure | Diagnostics can be weaker if output occurs before write failure. | low | defer | Does not affect orphan cleanup or benchmark run monitor product goal. | No runtime change. | Revisit if this failure mode appears in real logs. |
| implementation-adversary | Missing direct fault injection for non-BrokenPipe cleanup | Branch is hard to force deterministically. | low | defer | Positive stdin and BrokenPipe coverage plus cleanup code review are sufficient for current scope. | No additional test now. | Consider a test seam if process executor grows. |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: n/a
- Blocking re-review completed: n/a
- Blocking re-review passed: n/a
- Blocking re-review round links:
  - n/a
- Blocking re-review launch records:
  - n/a
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

The implementation may proceed. Accepted blocking findings were fixed and passed fresh closure review. Remaining items are low-risk diagnostic/test-hardening improvements outside the immediate product goal.
