# Subagent VS Review: Terminal-Bench Cleanup Failure Mapping

- Created: 2026-06-02T18:35:53+0800
- Updated: 2026-06-02T18:58:00+0800
- Report schema: adversarial-v1
- Task: Make Terminal-Bench post-task compose cleanup failures visible as execution failures after a real 80-task run exposed hidden Docker cleanup residue.
- Report path: `vs_review/2026-06-02-terminal-bench-cleanup-failure-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: Cleanup Failure Mapping

### Review Input

#### Objective
Terminal-Bench post-task compose cleanup failure must invalidate the task as `execution/agent_cleanup_failed`, so reports and results do not treat the score as valid benchmark performance.

#### Review Target
Implementation, failure mapping, test registration, and operations documentation for Terminal-Bench cleanup failure handling.

#### Target Locations
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_cleanup.rs`
- `crates/harnesslab-cli/tests/terminal_bench_cleanup_contract.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/development-operations.md`

#### Change Introduction
`cleanup_task_resources` returns an optional non-required cleanup failure signal. Terminal-Bench execution captures post-task cleanup failures and maps official success or benchmark failure to `execution/agent_cleanup_failed` with score `0`. Existing execution failures remain primary, with `agent_cleanup_failed` added as a warning.

#### Risk Focus
- Cleanup failure must not be hidden behind official success or benchmark failure.
- Cleanup failure must not mask existing execution-stall health signals.
- Post-task cleanup must be called exactly once.
- Pre-task required cleanup must still block runner launch.
- Report, results, selected test mapping, and registry must all expose the failure.

#### Assumptions To Attack
- A post-task Docker cleanup warning is enough evidence to invalidate a task.
- Fake Docker setup exercises post-task cleanup, not pre-task cleanup.
- Existing warning/report paths automatically surface cleanup failures.
- Keeping `agent_cleanup_failed` as non-stall health impact is acceptable when it is the primary failure.

#### Adversarial Lenses
- failure
- state
- testing
- observability
- maintenance

#### Verification Status
- `cargo fmt --all --check`: passed
- `scripts/test-after-change.sh --select INT-040`: passed
- `scripts/test-after-change.sh --select INT-041`: passed
- `scripts/test-after-change.sh --select INT-042`: passed
- `scripts/test-after-change.sh --select INT-043`: passed
- `cargo test -p harnesslab-cli --test terminal_bench_cleanup_contract`: passed
- `cargo check -p harnesslab-cli --tests`: passed
- `scripts/verify-test-registry.sh`: passed
- `git diff --check`: passed
- Real run environment was restored after Colima/Docker storage full, and residual Docker resources were manually cleaned.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Focus on high-impact concrete bugs, false positives, and evidence gaps.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 10 minutes | none | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| code-reviewer | Challenge implementation and failure precedence. | Cleanup failure mapping, result correctness |
| test-engineer | Challenge whether INT-040 and registry prevent self-deception. | Test validity, selected gate, traceability |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | `multi_agent_v1.spawn_agent` | `019e87e7-1f10-7791-b05f-5948c8480b40` | spawn_agent tool result | false | Round 1 Review Input plus target-specific packet | main-agent history, reasoning, drafts, conclusions | yes |
| test-engineer | `multi_agent_v1.spawn_agent` | `019e87e7-537a-72e2-b2e1-b97857a96f8a` | spawn_agent tool result | false | Round 1 Review Input plus target-specific packet | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| code-reviewer-output | code-reviewer | 1 | `019e87e7-1f10-7791-b05f-5948c8480b40` | < 10 minutes | completed | request-changes review returned | blocking finding accepted |
| test-engineer-output | test-engineer | 1 | `019e87e7-537a-72e2-b2e1-b97857a96f8a` | < 10 minutes | completed | test adequacy review returned | non-blocking gaps accepted and fixed |

### Reviewer Outputs

#### code-reviewer-output

##### Summary
The narrow success-plus-cleanup-failure path was fixed, but cleanup failure originally overwrote existing execution-stall failures and could erase run-health counters.

##### Blocking Findings
- Cleanup override masks existing execution-stall failures and changes run-health behavior.
  - Broken assumption: Cleanup failure can always become the primary final failure code.
  - Failure scenario: A task hits `external_runner_no_progress` or `external_runner_timeout`, then post-task cleanup also fails; the unconditional cleanup override changes the final code to `agent_cleanup_failed`.
  - Trigger condition: Existing execution failure plus post-task compose cleanup error.
  - Impact: `run-health.json` undercounts `execution_stalls` and `external_runner_*`, and may skip abort behavior.
  - Proof needed: Precedence tests preserving no-progress and timeout health counters.

##### Non-blocking Risks
- Cleanup error detail is present in `events.jsonl`, but not embedded in task-level result artifacts.
  - Broken assumption: `execution/agent_cleanup_failed` alone is enough for triage.
  - Failure scenario: User sees report failure but must open events to find Docker metadata error.
  - Trigger condition: Any post-task cleanup failure.
  - Impact: Slower incident diagnosis.
  - Proof needed: Product decision on whether result schema should carry failure messages.

##### Required Fixes
- Preserve existing execution-stall semantics when post-task cleanup also fails.
- Add precedence regression tests so cleanup failure cannot erase run-health stall accounting.

##### Missing Tests
- `external_runner_no_progress` plus post-task cleanup failure.
- `external_runner_timeout` plus post-task cleanup failure.
- `benchmark failure` plus post-task cleanup failure.

##### Missing Logs / Observability
- No dedicated result field for cleanup failure reason.

##### Evidence
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs:120` - post-task cleanup location.
- `crates/harnesslab-core/src/model.rs:141` - health impact derives from final failure code.
- `crates/harnesslab-cli/src/runner/monitor.rs:111` - run-health counters depend on health impact and failure code.

#### test-engineer-output

##### Summary
`INT-040` covers the reported bug path and would fail the old warning-only behavior. The main gap was matrix coverage for official benchmark failure plus cleanup failure.

##### Blocking Findings
- none

##### Non-blocking Risks
- `INT-040` proves final outcome but does not explicitly assert Docker transcript ordering.
  - Broken assumption: Marker-file protocol is enough to prove post-task-only cleanup failure.
  - Failure scenario: Future test rewrite accidentally makes pre-task cleanup fail instead.
  - Trigger condition: Fake Docker script changes without transcript assertions.
  - Impact: Weaker diagnostic value.
  - Proof needed: Optional transcript assertion or stronger marker checks.
- Registry traceability originally pointed to broader architecture docs rather than the explicit operations contract.
  - Broken assumption: Broad doc reference is specific enough.
  - Failure scenario: Future maintainers miss the concrete real-run cleanup contract.
  - Trigger condition: Registry traceability review.
  - Impact: Test intent becomes less discoverable.
  - Proof needed: Registry source should cite the specific operations section.

##### Required Fixes
- Add black-box regression for official benchmark failure plus post-task cleanup failure.

##### Missing Tests
- Benchmark failure override case.
- Optional: assert compose project snapshot exists on cleanup failure path.

##### Missing Logs / Observability
- `INT-040` does not assert Docker transcript ordering.
- No failure-path assertion for `terminal-bench-compose-projects.json` before the later fix.

##### Evidence
- `crates/harnesslab-cli/tests/terminal_bench_cleanup_contract.rs:140` - INT-040 success plus cleanup failure path.
- `tests/TEST_REGISTRY.toml:1937` - INT-040 registry entry.
- `scripts/test-after-change.sh:129` - INT-040 selector.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| code-reviewer | Cleanup override masks existing execution-stall failures. | Cleanup failure always primary can erase `external_runner_no_progress` health counters. | blocking | accept | `health_impact_for_failure` is based on final code, so unconditional cleanup override was wrong. | Changed cleanup override to apply only when final class is not already `Execution`; otherwise add `agent_cleanup_failed` warning. Added INT-041 and INT-043. | Round 2 closure review |
| code-reviewer | Cleanup error detail only in events. | Report/result show code but not raw Docker error. | non-blocking | defer | Current schema has no result message field; events carry exact error. | Kept error in `events.jsonl`; no schema expansion in this patch. | Future result-message design |
| test-engineer | Benchmark failure plus cleanup failure missing. | Official benchmark failure path could avoid cleanup override. | non-blocking | accept | Contract says cleanup failure invalidates success or benchmark failure. | Added INT-042 and registry selector; asserts original `test_failed` appears in warnings and snapshot exists. | Covered |
| test-engineer | Registry source too broad. | Test traceability pointed to architecture instead of concrete ops contract. | non-blocking | accept | Operations doc now has explicit cleanup contract. | Updated INT-040 source to `docs/development-operations.md`, same section as INT-041/042/043. | Covered |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 2
- Blocking re-review launch records:
  - `019e87f1-2f88-7e70-95ec-f2b67da0d976`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: n/a
- Allowed to proceed: yes

## Round 2: Closure Review

### Review Input

#### Objective
Verify that the accepted blocking issue from Round 1 is closed without introducing high-impact regressions.

#### Review Target
Precedence fix and added test matrix for Terminal-Bench cleanup failure handling.

#### Target Locations
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/tests/terminal_bench_cleanup_contract.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`

#### Change Introduction
Post-task cleanup failure now overrides only non-execution results. Existing execution failures remain primary and add `agent_cleanup_failed` to warnings. INT-041 covers no-progress plus cleanup failure preserving run-health. INT-042 covers benchmark failure plus cleanup failure becoming `agent_cleanup_failed` with the original benchmark failure retained as warning. INT-043 covers runner hard-timeout plus cleanup failure preserving timeout health.

#### Risk Focus
- Accepted blocking finding must be closed.
- Existing execution-stall counters must remain intact.
- Benchmark/success cleanup failures must still invalidate score.
- Test registration must cover the new matrix.

#### Assumptions To Attack
- `failure_class != Execution` is the correct precedence boundary.
- Warning-based secondary cleanup signal is visible enough in reports.
- INT-041, INT-042, and INT-043 prevent the previously found regression.

#### Adversarial Lenses
- failure
- state
- testing
- observability

#### Verification Status
- `scripts/test-after-change.sh --select INT-041`: passed
- `scripts/test-after-change.sh --select INT-042`: passed
- `scripts/test-after-change.sh --select INT-043`: passed
- `cargo test -p harnesslab-cli --test terminal_bench_cleanup_contract`: passed, 6 tests
- `scripts/verify-test-registry.sh`: passed, 126 tests

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Focus on whether the accepted blocking finding is actually closed.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 10 minutes | none | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| code-reviewer | Closure check for accepted blocking implementation finding. | Failure precedence, run-health preservation |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | `multi_agent_v1.spawn_agent` | `019e87f1-2f88-7e70-95ec-f2b67da0d976` | spawn_agent tool result | false | Round 2 Review Input plus target-specific packet | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| closure-code-reviewer-output | code-reviewer | 1 | `019e87f1-2f88-7e70-95ec-f2b67da0d976` | < 10 minutes | completed | approved closure | closed |

### Reviewer Outputs

#### closure-code-reviewer-output

##### Summary
Verdict: approve. The accepted blocking finding is closed. The fix preserves existing execution failure as primary and only downgrades post-task cleanup failure to a warning in that case.

##### Blocking Findings
- none

##### Non-blocking Risks
- Low: direct integration coverage for `external_runner_timeout` plus cleanup failure was initially absent.
  - Broken assumption: No-progress coverage alone proves timeout coverage.
  - Failure scenario: Timeout-specific run-health counter could be missed.
  - Trigger condition: Hard-timeout process plus cleanup failure.
  - Impact: Low, because implementation gate is generic for all execution failures.
  - Proof needed: Add a timeout-specific variant of INT-041.

##### Required Fixes
- none for closure

##### Missing Tests
- Optional timeout-specific variant was recommended and implemented as INT-043 after closure output.

##### Missing Logs / Observability
- none for the accepted issue

##### Evidence
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs:172` - result-preservation gate only overrides when final class is not execution.
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs:193` - cleanup failure becomes a warning when it does not override.
- `crates/harnesslab-cli/tests/terminal_bench_cleanup_contract.rs:199` - INT-041 preserves no-progress health.
- `crates/harnesslab-cli/tests/terminal_bench_cleanup_contract.rs:273` - INT-042 covers benchmark failure override.
- `scripts/test-after-change.sh:129` - INT-040 through INT-043 selectors are registered.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| code-reviewer | No direct timeout plus cleanup failure test. | No-progress coverage alone might miss timeout counter preservation. | non-blocking | accept | Cheap to cover and strengthens the matrix. | Added INT-043 and registry selector; test passes. | Covered |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 2
- Blocking re-review launch records:
  - `019e87f1-2f88-7e70-95ec-f2b67da0d976`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

Passed. The cleanup failure mapping now invalidates official success and benchmark failure, preserves existing execution-stall and hard-timeout health signals, records secondary cleanup failure as a warning, and is covered by registered integration tests INT-040 through INT-043.
