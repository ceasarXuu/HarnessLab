# Subagent VS Review: Terminal-Bench Failure Mode Mapping

- Created: 2026-06-01T03:45:35+0800
- Updated: 2026-06-01T04:10:00+0800
- Report schema: adversarial-v1
- Task: Correct HarnessLab classification of official Terminal-Bench agent timeouts found during a real run.
- Report path: `vs_review/2026-06-01-terminal-bench-failure-mode-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: Implementation And Test Validity

### Review Input

#### Objective
Verify a small implementation fix for Terminal-Bench official result parsing after a real HarnessLab Terminal-Bench run showed official `failure_mode: agent_timeout` being reported by HarnessLab as `benchmark/test_failed`, so run-health missed agent timeouts.

#### Review Target
Code implementation, failure-classification behavior, and regression tests.

#### Target Locations
- `crates/harnesslab-cli/src/runner/external.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `crates/harnesslab-cli/tests/support/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/monitor.rs`
- `docs/mvp-development-spec.md`

#### Change Introduction
The parser now looks at Terminal-Bench aggregate `results[].failure_mode` when score is below 1.0 and maps known official modes to HarnessLab canonical failure classes and codes. New tests cover official `agent_timeout` mapping at parser level and integration level so run-health counts agent timeouts.

#### Risk Focus
- Whether success scoring still wins over stale or non-empty `failure_mode` fields.
- Whether aggregate results with multiple records are handled safely.
- Whether mapping `agent_timeout` as execution failure causes the correct CLI exit code and run-health behavior.
- Whether ordinary unresolved benchmark failures still report `benchmark/test_failed`.
- Whether new tests are black-box enough to catch the real regression.
- Whether report artifacts and user-facing failure reasons remain accurate.

#### Assumptions To Attack
- Official Terminal-Bench aggregate result shape always has `results` array with per-task `failure_mode`.
- First mapped failure mode is enough for a one-task HarnessLab external execution.
- Mapping `unknown_agent_error` to `agent_nonzero_exit` is acceptable and not misleading.
- The fake `uvx` result is representative enough of real Terminal-Bench output.
- Counting `agent_timeouts` in run-health is enough to ensure monitor behavior.
- No file line-count or maintainability rule is violated.

#### Adversarial Lenses
- implementation
- failure
- state
- testing
- observability
- maintenance

#### Verification Status
- `cargo test -p harnesslab-cli terminal_bench_result_maps_official_agent_timeout --quiet` passed.
- `cargo test -p harnesslab-cli --test terminal_bench_failure_contract --quiet` passed.
- `scripts/test-after-change.sh` passed: 229 Rust/nextest tests, 7 Python tests, line coverage 95.31%, branch coverage 83.39%.
- Real rerun after the fix has not yet been performed.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 10 minutes | 5 minutes max if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | The change touches parser classification, canonical failure mapping, and monitor-visible state. | correctness, state, failure mapping |
| test-validity-adversary | The regression came from real-run evidence, so tests must not merely confirm the implementation shape. | test validity, black-box coverage, observability |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | `multi_agent_v1.spawn_agent` code-reviewer | `019e7f92-47a8-7430-9b68-d0a9ed85b950` (`Hubble`) | spawn_agent tool result | false | Round 1 implementation review input | main-agent history, reasoning, drafts, conclusions, full diff by default | yes |
| test-validity-adversary | `multi_agent_v1.spawn_agent` test-engineer | `019e7f92-8498-7110-b6bd-4bbf885ac7c1` (`Bohr`) | spawn_agent tool result | false | Round 1 test-validity review input | main-agent history, reasoning, drafts, conclusions, full diff by default | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| implementation-output | implementation-adversary | 1 | `019e7f92-47a8-7430-9b68-d0a9ed85b950` | <10 minutes | completed | reviewer returned `REQUEST CHANGES` | completed |
| test-validity-output | test-validity-adversary | 1 | `019e7f92-8498-7110-b6bd-4bbf885ac7c1` | <10 minutes | completed | reviewer returned blocking test gaps | completed |

### Reviewer Outputs

#### implementation-output

##### Summary
The targeted `agent_timeout` fix is directionally correct: success still wins before failure-mode parsing, ordinary unresolved benchmark failures still fall back to `benchmark/test_failed`, and the new integration contract proves `agent_timeout -> execution/agent_timeout` with run-health counting. Verdict: `REQUEST CHANGES`.

##### Blocking Findings
- Unsafe remap of `unknown_agent_error` to `execution/agent_nonzero_exit`.
  - Broken assumption: official `unknown_agent_error` is equivalent to agent exited non-zero.
  - Failure scenario: the local Terminal-Bench shim emits `UNKNOWN_AGENT_ERROR` for any exception from `run_registered_agent`, including timeout/config/runtime failures, not just non-zero exit.
  - Trigger condition: any official result with `results[].failure_mode = "unknown_agent_error"`.
  - Impact: incorrect failure taxonomy, changed CLI exit code, and misleading downstream automation/reporting.
  - Proof needed: narrow the change to the proven `agent_timeout` case, or add upstream/agent-specific evidence and black-box tests.

##### Non-blocking Risks
- Multi-record aggregates are not tied to the requested task id.
  - Broken assumption: one-task HarnessLab execution always yields exactly one relevant `results[]` record.
  - Failure scenario: if official output contains extra records, `find_map` can classify from the wrong record.
  - Trigger condition: multi-entry `results[]`.
  - Impact: wrong failure class/code and wrong run-health timeout counting.
  - Proof needed: filter by `task_id` or assert single-record shape and cover it with a test.
- Success precedence is implemented correctly but not locked by tests.
  - Broken assumption: future refactors will preserve the `score >= 1.0` short-circuit.
  - Failure scenario: stale `failure_mode` on a resolved result flips success into failure.
  - Trigger condition: successful result with non-empty `failure_mode`.
  - Impact: false failures.
  - Proof needed: add parser-level regression test.

##### Required Fixes
- Remove or justify the `unknown_agent_error -> agent_nonzero_exit` remap with explicit contract evidence and tests before merge.
- If broader mapping stays, add task-targeted parsing for `results[]` instead of taking the first recognized mode.

##### Missing Tests
- A black-box contract test for `unknown_agent_error` classification and resulting CLI exit code.
- A parser test for success with stale/non-empty `failure_mode`.
- A parser test for multi-record `results[]` proving the current task is selected, or that unexpected shape is rejected.

##### Missing Logs / Observability
- No structured signal shows whether classification came from official `failure_mode` or fallback to `benchmark/test_failed`; raw official JSON is preserved in verifier logs.

##### Evidence
- `crates/harnesslab-cli/src/runner/external.rs:356` - success short-circuit.
- `crates/harnesslab-cli/src/runner/external.rs:373` - fallback unresolved classification.
- `integrations/terminal_bench/harnesslab_tb_agent.py:33` - shim returns `UNKNOWN_AGENT_ERROR` for broad exceptions.
- `crates/harnesslab-core/src/model.rs:305` - execution failures drive exit code priority.

#### test-validity-output

##### Summary
The narrow regression is substantially covered by parser and CLI tests, but the original tests did not prove the user-visible HTML artifact or multi-task run-health self-healing path.

##### Blocking Findings
- The tests do not prove the user-visible artifact is fixed.
  - Broken assumption: full gate passing proves the new failure reason is surfaced in user artifacts.
  - Failure scenario: `results.json` and `run-health.json` are correct, but `report.html` still renders misleading failure text.
  - Trigger condition: future report formatting or wiring regression.
  - Impact: user opens HTML report and still sees misleading failure information.
  - Proof needed: integration assertion on `report.html` showing the execution timeout failure and not benchmark/test_failed wording.
- The tests do not prove real run-health self-healing for Terminal-Bench timeouts.
  - Broken assumption: counting `agent_timeouts` in run-health is enough to ensure monitor behavior.
  - Failure scenario: single-task mapping works, but a multi-task run does not abort after threshold.
  - Trigger condition: five official `agent_timeout` results before any non-timeout completion.
  - Impact: unhealthy runs can continue wasting time.
  - Proof needed: end-to-end Terminal-Bench integration test with repeated official `agent_timeout` results, aborted run, interrupted pending tasks, and invalid run-health.

##### Non-blocking Risks
- Exit code `1` is not agent-timeout-specific.
  - Broken assumption: exit code is the primary proof.
  - Failure scenario: another execution failure also exits `1`.
  - Trigger condition: execution-class misclassification.
  - Impact: low because JSON assertions are stronger evidence.
  - Proof needed: keep asserting JSON classification.
- Fake `uvx` payload is minimal and does not validate broader aggregate-shape assumptions.
  - Broken assumption: fake result is representative enough of real Terminal-Bench output.
  - Failure scenario: official output changes shape or includes mixed records.
  - Trigger condition: mixed or expanded `results[]` records.
  - Impact: future-proofing risk.
  - Proof needed: fixtures for mixed arrays and known alternate official failure modes.

##### Required Fixes
- Add an integration assertion that `report.html` surfaces the timeout classification.
- Add an end-to-end Terminal-Bench timeout-threshold contract.

##### Missing Tests
- Official `failure_mode: test_timeout` maps to `benchmark/verifier_timeout`.
- Unknown `failure_mode` falls back to `benchmark/test_failed`.
- Mixed aggregate records have explicit precedence.
- Multi-task Terminal-Bench run-health abort path for repeated official `agent_timeout`.
- HTML report assertion for the new timeout case.

##### Missing Logs / Observability
- No test asserted `verifier/stdout.log` preserves the official result path and JSON payload for the timeout case.

##### Evidence
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs:7` - original single-task test.
- `crates/harnesslab-cli/src/runner/monitor.rs:112` - timeout threshold abort logic.
- `crates/harnesslab-cli/src/runner/external.rs:407` - verifier stdout captures official JSON.
- `harnesslab-report/src/lib.rs:80` - report rendering path.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | Unsafe remap of `unknown_agent_error` | `unknown_agent_error` is broader than non-zero agent exit | blocking | accept | `harnesslab_tb_agent.py` returns it for broad exceptions | Removed `unknown_agent_error` and unproven installation mappings; added unknown-mode fallback test | Round 2 closure review |
| implementation-adversary | Multi-record aggregate not tied to task id | parser could classify from another task | major | accept | failure-mode parser previously used first recognized mode | Parser now checks `results[].task_id == task_id`; added mixed-record test | Round 2 closure review |
| implementation-adversary | Success precedence not locked | stale failure mode could flip success | non-blocking | accept | success short-circuit existed but had no direct test | Added parser regression for success with stale `agent_timeout` | none |
| test-validity-adversary | HTML report not asserted | JSON could be correct while report misleads | blocking | accept | report is the primary user artifact | Report now renders snake_case failure strings; integration asserts `execution/agent_timeout` and absence of `benchmark/test_failed` | Round 2 closure review |
| test-validity-adversary | Multi-task timeout self-healing unproven | one timeout counter does not prove abort behavior | blocking | accept | monitor aborts only after threshold | Added end-to-end Terminal-Bench fake-uvx contract: 5 official timeouts abort and 1 pending task is interrupted | Round 2 closure review |
| test-validity-adversary | Exit code `1` not timeout-specific | exit code alone is weak proof | non-blocking | accept | execution failures share exit `1` | Kept exit code assertion but primary assertions are JSON `failure_code`, run-health, and report content | none |
| test-validity-adversary | Minimal fake payload | missing broader aggregate shape checks | non-blocking | accept | fake payload previously covered only one mode | Added `test_timeout`, unknown mode fallback, stale success, and mixed-record parser tests | none |
| implementation-adversary | Missing structured classification event | raw JSON exists but no classification event | non-blocking | defer | verifier stdout now has tested official path and payload; adding event is useful but not required for this bug fix | Integration asserts verifier log includes `official_results_path` and `failure_mode` | Track if another diagnosis gap appears |

Validation after response:

- `cargo test -p harnesslab-cli terminal_bench_result_ --quiet` passed: 5 parser tests.
- `cargo test -p harnesslab-cli --test terminal_bench_failure_contract --quiet` passed: 2 integration tests.
- `cargo test -p harnesslab-report --quiet` passed.
- `cargo test -p harnesslab-cli --test terminal_bench_contract int_011_terminal_bench_nonzero_with_results_uses_benchmark_result --quiet` passed.
- `cargo test -p harnesslab-cli --test terminal_bench_contract int_011_terminal_bench_docker_network_exhaustion_aborts_remaining_tasks --quiet` passed.
- `scripts/test-after-change.sh` passed: 234 Rust/nextest tests, 7 Python tests, line coverage 95.39%, branch coverage 83.55%.

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 2
- Blocking re-review launch records:
  - `019e7f9d-c10a-7b43-ab18-06dd624470ba`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

Passed after Round 2 closure review.

## Round 2: Blocking Closure Review

### Review Input

#### Objective
Verify closure of accepted blocking findings from Round 1 for Terminal-Bench official failure-mode parsing.

#### Review Target
Implementation and tests changed after Round 1.

#### Target Locations
- `crates/harnesslab-cli/src/runner/external.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `crates/harnesslab-report/src/lib.rs`
- `vs_review/2026-06-01-terminal-bench-failure-mode-review.md`

#### Change Introduction
The implementation was narrowed and hardened after Round 1: unproven official failure modes are no longer remapped, known official modes are matched only for the current task id, report failure text is rendered as snake_case, and new parser/integration tests cover stale success, mixed results, unknown modes, HTML output, verifier logs, and repeated timeout abort behavior.

#### Risk Focus
- Whether accepted blocking findings from Round 1 are actually closed.
- Whether the fixes introduced a new blocker.
- Whether tests prove the user-facing report and run-health behavior.

#### Assumptions To Attack
- Unknown official failure modes now fall back to `benchmark/test_failed`.
- Current task id filtering prevents cross-task failure-mode contamination.
- HTML report is now user-readable and asserted by integration tests.
- Five repeated official `agent_timeout` results abort the run and interrupt pending work.

#### Adversarial Lenses
- implementation
- testing
- failure
- observability

#### Verification Status
- `cargo test -p harnesslab-cli terminal_bench_result_ --quiet` passed.
- `cargo test -p harnesslab-cli --test terminal_bench_failure_contract --quiet` passed.
- `cargo test -p harnesslab-report --quiet` passed.
- `scripts/test-after-change.sh` passed: 234 Rust/nextest tests, 7 Python tests, line coverage 95.39%, branch coverage 83.55%.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Focus on closure of accepted blocking findings and any new blocker.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 10 minutes | 5 minutes max if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| closure-implementation-adversary | Round 1 accepted blocking findings affected parser correctness, report output, and multi-task monitor behavior. | closure correctness, new blockers |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| closure-implementation-adversary | `multi_agent_v1.spawn_agent` code-reviewer | `019e7f9d-c10a-7b43-ab18-06dd624470ba` (`Huygens`) | spawn_agent tool result | false | Round 2 closure review input | main-agent history, reasoning, drafts, conclusions, full diff by default | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| closure-output | closure-implementation-adversary | 1 | `019e7f9d-c10a-7b43-ab18-06dd624470ba` | <10 minutes | completed | reviewer found no blockers | completed |

### Reviewer Outputs

#### closure-output

##### Summary
The closure reviewer directly inspected the five requested targets, checked the relevant diff, and re-ran targeted tests. The four accepted blocking findings from Round 1 are closed, and no new blocker was found.

##### Closure Verdict
`COMMENT`: Accepted blocking findings appear closed in the reviewed code and tests.

##### Blocking Findings
- none

##### Non-blocking Risks
- Duplicate entries for the same current task id still rely on upstream ordering.
  - Broken assumption: current-task records are unique.
  - Failure scenario: duplicate current-task records with conflicting `failure_mode` values could pick the first record.
  - Trigger condition: malformed or future upstream aggregate with duplicate task ids.
  - Impact: low; HarnessLab invokes Terminal-Bench one task at a time and current tests cover cross-task contamination.
  - Proof needed: add a duplicate-current-task precedence test if upstream output ever shows this shape.
- Review artifact still marked pending/open.
  - Broken assumption: closure review result has already been recorded.
  - Failure scenario: report remains stale after closure.
  - Trigger condition: main agent fails to update report.
  - Impact: audit trail confusion.
  - Proof needed: main agent updates this report.

##### Missing Tests
- No blocker-level test gap remains for the four accepted findings.
- Low-risk residual gap: no parser test for duplicate current-task records with conflicting failure modes.

##### Missing Logs / Observability
- No structured event says whether final classification came from official `failure_mode` mapping or fallback `benchmark/test_failed`; raw official JSON is preserved in verifier stdout and now asserted by integration tests.

##### Evidence
- `crates/harnesslab-cli/src/runner/external.rs:371` and `crates/harnesslab-cli/src/runner/external.rs:439` - only `agent_timeout` and `test_timeout` are recognized; unknown modes fall back.
- `crates/harnesslab-cli/src/runner/external.rs:138` and `crates/harnesslab-cli/src/runner/external.rs:434` - parser receives and filters by current task id.
- `crates/harnesslab-cli/src/runner/external_tests.rs:93` - other-task failure mode does not contaminate current task.
- `crates/harnesslab-report/src/lib.rs:122` and `crates/harnesslab-report/src/lib.rs:206` - report renders canonical snake_case failure text.
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs:34` - HTML asserts `execution/agent_timeout` and not `benchmark/test_failed`.
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs:45` - repeated official agent timeouts abort run and interrupt pending work.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| closure-implementation-adversary | none | n/a | n/a | n/a | reviewer found no blockers | none | none |
| closure-implementation-adversary | Duplicate current-task records rely on ordering | malformed aggregate could include duplicate task ids | non-blocking | defer | No evidence current Terminal-Bench one-task run emits duplicates; current task id filtering closes accepted blocker | documented residual risk | add test if official output shows duplicate task ids |
| closure-implementation-adversary | Review artifact pending | report needed final update | non-blocking | accept | closure output returned | updated report status to passed | none |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 2
- Blocking re-review launch records:
  - `019e7f9d-c10a-7b43-ab18-06dd624470ba`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

Passed. The accepted Round 1 blocking findings were fixed, validated by local gates, and passed a fresh internal closure review.
