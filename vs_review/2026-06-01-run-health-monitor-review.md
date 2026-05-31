# Subagent VS Review: Run Health Monitor

- Created: 2026-06-01T00:00:00+08:00
- Updated: 2026-06-01T00:00:00+08:00
- Report schema: adversarial-v1
- Task: Prevent long benchmark runs from continuing after global infrastructure or timeout pathologies make the final score invalid.
- Report path: `vs_review/2026-06-01-run-health-monitor-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: Implementation Review

### Review Input

#### Objective
Detect unhealthy benchmark runs while they are still running, stop scheduling later tasks when the run is invalid, and record enough evidence that reports do not present infrastructure failures as normal agent score.

#### Review Target
Implementation, tests, and observability for run health monitoring and Terminal-Bench infrastructure failure classification.

#### Target Locations
- `crates/harnesslab-cli/src/runner/attempts.rs`
- `crates/harnesslab-cli/src/runner/monitor.rs`
- `crates/harnesslab-cli/src/runner/external.rs`
- `crates/harnesslab-cli/src/runner/external/log_scan.rs`
- `crates/harnesslab-core/src/model.rs`
- `crates/harnesslab-cli/tests/terminal_bench_contract.rs`
- `crates/harnesslab-cli/tests/support/terminal_bench.rs`

#### Change Introduction
The scheduler now owns a run-level monitor. It records per-result health state, writes `run-health.json`, stops scheduling pending work after fatal infrastructure signals, and writes interrupted results for unscheduled tasks. Terminal-Bench logs are scanned for Docker network pool exhaustion and reclassified from benchmark failure to execution failure. Terminal-Bench outer process timeout now includes setup overhead instead of matching the agent/test timeout exactly.

#### Risk Focus
- A fatal global failure might still be treated as a normal low score.
- The monitor might stop valid runs too aggressively.
- Pending tasks might disappear rather than become auditable interrupted results.
- Resume behavior might be confused by monitor-created interrupted attempts.
- Log scanning might miss real infrastructure failures or misclassify agent failures.
- Tests might only cover a narrow fake path.

#### Assumptions To Attack
- Docker network pool exhaustion is a global infrastructure failure that should abort later scheduling.
- Five agent timeouts before any success is a reasonable default abort threshold.
- Writing interrupted attempts for unscheduled work is compatible with summary, report, and resume semantics.
- `run-health.json` plus events are sufficient observability for MVP.
- Adding 600 seconds of Terminal-Bench process overhead prevents setup from being killed without masking actual hangs.

#### Adversarial Lenses
- requirements
- concurrency
- failure
- state
- observability
- testing
- maintenance

#### Verification Status
- Targeted unit tests passed for run monitor and Terminal-Bench log scanning.
- Terminal-Bench contract test passed for Docker network exhaustion aborting remaining work.
- Full regression and coverage gate not yet rerun after this change.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Focus on high-impact product and engineering failure modes, not style.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| complex | 15 minutes | none | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| code-reviewer | Reviews scheduler, classification, model, and observability correctness. | architecture and failure semantics |
| test-engineer | Reviews whether tests prevent another long invalid benchmark run. | regression coverage and test realism |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | multi_agent_v1.spawn_agent | 019e7ef7-e76b-7eb1-821e-0bb83ee82ac7 | tool call | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| test-engineer | multi_agent_v1.spawn_agent | 019e7ef8-16a7-7f20-b56e-db10b24a6df7 | tool call | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| code-reviewer-output | code-reviewer | 1 | 019e7ef7-e76b-7eb1-821e-0bb83ee82ac7 | under 15 minutes | completed | blocking findings returned | accepted and fixed |
| test-engineer-output | test-engineer | 1 | 019e7ef8-16a7-7f20-b56e-db10b24a6df7 | under 15 minutes | completed | blocking findings returned | accepted and fixed |

### Reviewer Outputs

#### code-reviewer-output

Blocking findings:
- The concurrent scheduler returned immediately after the first abort decision instead of joining all active tasks in the same chunk. This could drop or rewrite valid active task evidence.
- Monitor-generated `run_health_aborted` interrupted placeholders were counted by resume planning as consumed attempts. This made the fail-fast path incompatible with replaying the tasks that were never actually run.

Non-blocking risks:
- Very large diagnostic logs could hide an infrastructure signature if the scanner skipped them entirely.
- The timeout-abort heuristic could be too aggressive if it did not distinguish runs that had already made non-timeout progress.
- The HTML report did not surface the run-health reason clearly enough for users to see that the score was invalid.

#### test-engineer-output

Blocking findings:
- Missing direct coverage for the timeout threshold behavior.
- Missing concurrency coverage proving an abort drains the active batch and only interrupts future unscheduled work.
- Missing coverage for the important false-success case where `results.json` looks valid but infrastructure logs prove Docker failure.

### Main Agent Response

| Finding | Decision | Fix / Evidence |
|---|---|---|
| Active concurrent chunk was not drained before abort | accepted | `crates/harnesslab-cli/src/runner/attempts.rs` now joins all handles in the active chunk before writing placeholders for `attempts[end..]`. Covered by `int_011_terminal_bench_concurrent_abort_drains_active_chunk_only`. |
| `run_health_aborted` placeholders consumed resume attempts | accepted | `crates/harnesslab-cli/src/runner/schedule.rs` filters those placeholders out before partitioning completed/pending attempts. Covered by `replay_002_resume_ignores_run_health_aborted_placeholders` and `int_016_resume_ignores_run_health_aborted_placeholders`. |
| Timeout threshold lacked focused tests | accepted | `crates/harnesslab-cli/src/runner/monitor.rs` includes unit coverage for timeout threshold, non-timeout progress, and Docker network abort. |
| Concurrency abort path lacked contract coverage | accepted | `crates/harnesslab-cli/tests/terminal_bench_contract.rs` proves only active tasks launch and future tasks become interrupted. |
| Valid-looking Terminal-Bench output could hide infra failure | accepted | `crates/harnesslab-cli/src/runner/external.rs` lets infra-log classification override parsed results. Covered by `int_011_terminal_bench_infra_log_overrides_normal_looking_results`. |
| Large logs skipped by scanner | accepted | `crates/harnesslab-cli/src/runner/external/log_scan.rs` now scans the tail of large logs. Covered by tail-scan regression coverage. |
| Report did not expose health reason | accepted | `crates/harnesslab-report/src/lib.rs` renders run-health status/reason and interrupted count. |

No findings were rejected or deferred.

## Round 2: Blocking Closure Review

### Review Input

Fresh internal re-review of the accepted blocking fixes from Round 1. Scope was limited to scheduler draining, resume semantics, timeout heuristics, infra-log overrides, report observability, and test closure.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | multi_agent_v1.spawn_agent | 019e7f02-026b-7fb3-91b9-373b9c48eb7b | tool call | fork_context=false | Round 2 Closure Review Input | main-agent history, reasoning, drafts, conclusions beyond target scope | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| closure-code-reviewer-output | code-reviewer | 1 | 019e7f02-026b-7fb3-91b9-373b9c48eb7b | under 15 minutes | completed | no blocking findings | proceed |

### Reviewer Outputs

#### closure-code-reviewer-output

Summary:
- APPROVE. No blocking findings in the requested run-health monitor closure scope.
- Verified closure by diff inspection, targeted code-path review, and fresh execution of `cargo check -p harnesslab-cli -p harnesslab-report`, `runner::monitor` unit tests, four Terminal-Bench contract closure tests, `resume_contract::int_016_resume_ignores_run_health_aborted_placeholders`, and `cargo test -p harnesslab-report --lib`.

Blocking findings:
- none

Non-blocking risks:
- none

Missing tests:
- none

Missing logs / observability:
- none

### Closure Status

- Blocking findings found: yes, in Round 1
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 2: `019e7f02-026b-7fb3-91b9-373b9c48eb7b`
- Blocking re-review launch records:
  - `019e7f02-026b-7fb3-91b9-373b9c48eb7b`
- Rejected findings backed by evidence: not applicable
- Deferred findings documented: none
- Blocked reason: none
- Allowed to proceed: yes

## Verification

- `scripts/test-after-change.sh`: PASS
- Rust tests: 214/214 passed in coverage run
- Python adapter tests: 7/7 passed
- Coverage gate: lines 95.31% (5775/6059), branches 82.75% (470/568)
- New production Rust file coverage gate: PASS

## Final Conclusion

Passed. The run-health monitor change is allowed to proceed.
