# Subagent VS Review: event log concurrency

- Created: 2026-06-02T07:22:47+0800
- Updated: 2026-06-02T07:34:00+0800
- Report schema: adversarial-v1
- Task: repair HarnessLab engineering failures found during real Terminal-Bench full run before retrying the formal CLI benchmark.
- Report path: `vs_review/2026-06-02-event-log-concurrency-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: concurrent event logging fix

### Review Input

#### Objective
Ensure HarnessLab's formal CLI can run concurrent benchmark tasks without corrupting run-level `events.jsonl`, and ensure the local test gate exposes failures instead of hiding diagnostic output.

#### Review Target
Implementation, test strategy, test gate behavior, and documentation for the event log concurrency fix.

#### Target Locations
- `crates/harnesslab-infra/src/event.rs`
- `scripts/test-after-change.sh`
- `scripts/verify-test-after-change-select-output.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/mvp-development-spec.md`
- `.benchmarks/_harnesslab-home-terminal-real/runs/claude-ds-terminal-bench-full-20260601T230915085811Z/events.jsonl`

#### Change Introduction
The previous full Terminal-Bench run was terminated after detecting malformed `events.jsonl` lines caused by concurrent task workers appending events at the same time. The change serializes event appends with a Unix file lock around each complete JSONL record, adds a multi-process regression test (`LOG-005`), registers the test, and adds a meta check proving selected test failures print their cargo/rustc output.

#### Risk Focus
- Concurrent writers may still interleave, lose, duplicate, or partially write event records.
- The new test may only validate the helper path, not the real event append contract used by runs.
- The lock may create portability, lifecycle, or performance problems under parallel benchmark execution.
- The test gate may still hide useful output or create recursion/flakiness.
- Documentation and registry entries may drift from actual behavior.

#### Assumptions To Attack
- `libc::flock` on the append-opened file descriptor is sufficient to serialize all HarnessLab event writers on supported Unix targets.
- Writing the serialized JSON and newline as two `write_all` calls while holding the lock is safe.
- A test that launches separate libtest child processes exercises the same cross-process race seen in the real run.
- Non-Unix no-op locking is acceptable for the current supported runtime.
- Invalid `RUSTFLAGS` is a stable enough negative control for the test gate meta check.

#### Adversarial Lenses
- concurrency
- data consistency
- failure
- testing
- observability
- maintenance

#### Verification Status
- `cargo fmt --check` passed.
- `scripts/test-after-change.sh --select LOG-005` passed.
- `scripts/verify-test-after-change-select-output.sh` passed.
- `scripts/verify-test-registry.sh` passed.
- Full regression and renewed full Terminal-Bench run not yet executed after this fix.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Report blocking findings, non-blocking risks, required fixes, missing tests, and missing observability.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 10 minutes | 5 minutes once if active | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Event logging changed shared data consistency under concurrent benchmark execution. | concurrency, data consistency, failure |
| test-validity-adversary | The issue was caught only in a real run, so the regression and meta tests must not be self-deceptive. | testing, gate behavior, coverage |
| observability-adversary | The failure mode made the run record unreliable; future diagnosis depends on durable logs. | observability, artifact integrity |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | multi_agent_v1.spawn_agent code-reviewer | 019e857f-e4cf-7540-908c-c8f9e2842c88 | tool spawn result nickname Harvey | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions | yes |
| test-validity-adversary | multi_agent_v1.spawn_agent test-engineer | 019e8580-1c83-70d1-bea8-28f2ec2bf58c | tool spawn result nickname Herschel | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions | yes |
| observability-adversary | multi_agent_v1.spawn_agent verifier | 019e8580-d738-7af2-949d-ea5996b64859 | tool spawn result nickname Dalton | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| implementation-adversary-1 | implementation-adversary | 1 | 019e857f-e4cf-7540-908c-c8f9e2842c88 | within normal wait | completed | returned review output | completed |
| test-validity-adversary-1 | test-validity-adversary | 1 | 019e8580-1c83-70d1-bea8-28f2ec2bf58c | within normal wait | completed | returned review output | completed |
| observability-adversary-1 | observability-adversary | 1 | 019e8580-d738-7af2-949d-ea5996b64859 | within normal wait | completed | returned review output | completed |

### Reviewer Outputs

#### implementation-adversary-1

##### Summary
Verdict: comment. No blocking defect found in the Unix implementation. The reviewer confirmed the old real artifact showed concatenated JSON objects and blank fragments, and the new file-lock write path targets that corruption class.

##### Blocking Findings
- none

##### Non-blocking Risks
- Non-Unix builds still use a no-op lock.
  - Broken assumption: non-Unix no-op locking is acceptable.
  - Failure scenario: concurrent runs on a non-Unix target can reproduce JSONL interleaving.
  - Trigger condition: `concurrency > 1` on a non-Unix target.
  - Impact: run-level `events.jsonl` can corrupt.
  - Proof needed: document Unix-only support, hard-fail unsupported concurrency, or add a cross-platform lock.

##### Required Fixes
- none

##### Missing Tests
- Exact real workload rerun is still required before claiming closure.
- `LOG-005` originally checked parseable line count but not full logical uniqueness.
- `META-001` originally validated compiler/toolchain failure output, not a deliberately failing test body.

##### Missing Logs / Observability
- Some best-effort event paths discard `append_event` errors, so lock failures there may be silent.

##### Evidence
- `.benchmarks/_harnesslab-home-terminal-real/runs/claude-ds-terminal-bench-full-20260601T230915085811Z/events.jsonl:21` - old real corruption.
- `crates/harnesslab-infra/src/event.rs:21` - append path now locks before writing a full record.
- `crates/harnesslab-infra/src/event.rs:49` - Unix file-lock guard.
- `scripts/test-after-change.sh:117` - selected-test output is printed before exit.

#### test-validity-adversary-1

##### Summary
The reviewer confirmed the old failure was real and the unit regression targets the right primitive, but reported two blocking validation gaps: `META-001` did not exercise a real test assertion failure, and there was no fresh runtime proof from a concurrent CLI run.

##### Blocking Findings
- `META-001` does not prove failing-test diagnostics are preserved; it only proves compiler flag errors are echoed.
  - Broken assumption: invalid `RUSTFLAGS` is a stable enough negative control for selected test failure output.
  - Failure scenario: real assertion failures could be hidden while rustc invocation errors still print.
  - Trigger condition: the selected-test runner changes post-processing around cargo output.
  - Impact: local gate becomes poor diagnostic evidence for actual failing tests.
  - Proof needed: force a real selected test failure and assert panic/failure context is visible.
- The change lacks runtime proof for the product-level claim that concurrent CLI workers no longer corrupt run-level `events.jsonl`.
  - Broken assumption: a cross-process unit test is sufficient to close the real CLI corruption bug.
  - Failure scenario: real `harnesslab run` still corrupts `events.jsonl` through an uncovered append path or runtime interaction.
  - Trigger condition: fresh benchmark run with `concurrency > 1`.
  - Impact: the product regression that terminated the real run can recur.
  - Proof needed: fresh concurrent CLI benchmark run plus JSONL validation of the produced run-level `events.jsonl`.

##### Non-blocking Risks
- Original `LOG-005` checked line count and JSON parseability, but not per-write uniqueness/completeness.
- Non-Unix locking is no-op.

##### Required Fixes
- Strengthen `META-001` to exercise an actual selected test assertion failure.
- Produce fresh runtime proof from a concurrent CLI benchmark run and validate the new run's `events.jsonl`.

##### Missing Tests
- Full expected `(worker,index)` coverage in `LOG-005`.
- End-to-end concurrent run proof for run-level `events.jsonl`.

##### Missing Logs / Observability
- Paired success artifact was missing at review time.

##### Evidence
- `.benchmarks/_harnesslab-home-terminal-real/runs/claude-ds-terminal-bench-full-20260601T230915085811Z/events.jsonl:21` - malformed old artifact.
- `scripts/verify-test-after-change-select-output.sh:4` - original meta check used invalid `RUSTFLAGS`.
- `crates/harnesslab-infra/src/event.rs:105` - original unit regression around append primitive.

#### observability-adversary-1

##### Summary
The reviewer found one blocking observability/data-integrity gap: append serialization alone does not prevent existing malformed `events.jsonl` from being silently reused by `resume`, `replay`, or `report open`.

##### Blocking Findings
- No runtime integrity validation of an existing `events.jsonl` before resume/report reuse the run directory.
  - Broken assumption: making future appends atomic is enough to make every run artifact trustworthy.
  - Failure scenario: a historical corrupted run or mid-write crash leaves malformed JSONL; `run resume`, `run replay`, or `report open` can continue without surfacing corruption.
  - Trigger condition: any malformed existing `events.jsonl`.
  - Impact: incident responders and downstream commands trust an invalid run artifact.
  - Proof needed: integration tests that corrupt `events.jsonl` and prove `resume`, `replay`, and `report open` fail before reuse.

##### Non-blocking Risks
- Non-Unix builds have no lock protection.
- The operations runbook does not include a one-command JSONL integrity check.

##### Required Fixes
- Add a pre-consumption integrity check for `events.jsonl` before existing run directories are reused.
- Add integration tests covering malformed event-log rejection.
- Document or close the non-Unix lock gap if cross-platform support becomes expected.

##### Missing Tests
- Full regression and renewed real Terminal-Bench run were still pending.
- No end-to-end malformed run reuse test existed at review time.

##### Missing Logs / Observability
- No runtime integrity signal existed before the fix.

##### Evidence
- `crates/harnesslab-cli/src/runner.rs:249` - runner wrote events but did not parse existing logs.
- `crates/harnesslab-report/src/lib.rs:120` - report model is driven by results, not event parsing.
- `docs/mvp-development-spec.md:1124` - event lines must be valid JSON.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| test-validity-adversary | `META-001` only tested compiler failure output. | Compiler errors can still print while real test assertion output is hidden. | blocking | accept | `scripts/verify-test-after-change-select-output.sh` originally used invalid `RUSTFLAGS`. | Added `meta_001_selected_failure_outputs_assertion_context` and changed the meta script to force a real selected test panic; `scripts/verify-test-after-change-select-output.sh` passed. | Closure review Round 2. |
| test-validity-adversary | No runtime proof for concurrent CLI run-level `events.jsonl`. | Unit append tests may not prove the real CLI path. | blocking | accept | Real run evidence was malformed; fresh proof is still needed. | Planned post-fix formal concurrent Terminal-Bench run and JSONL validation before claiming MVP delivery. | Must complete after closure review and full gate. |
| test-validity-adversary | `LOG-005` did not prove uniqueness/completeness. | Valid line count can hide duplicate/lost logical events. | non-blocking | accept | Original test only checked count and event name. | `LOG-005` now asserts every `(worker,index)` pair appears exactly once; `scripts/test-after-change.sh --select LOG-005` passed. | none |
| observability-adversary | Existing malformed event logs can be reused by `resume`, `replay`, or `report open`. | Future atomic appends do not make historical/corrupted artifacts trustworthy. | blocking | accept | CLI reuse paths did not validate `events.jsonl`. | Added `validate_event_log`, wired it into `run resume`, `run replay`, and `report open`, and added `INT-032`, `INT-033`, `INT-034`; all targeted tests passed. | Closure review Round 2. |
| implementation-adversary | Non-Unix locking is no-op. | Concurrent runs on non-Unix could corrupt event logs. | non-blocking | defer | Current product/runtime target is local macOS/Linux Docker CLI; Windows support is not part of the MVP gate. | Left Unix implementation in place. | Track when non-Unix support becomes product scope. |
| implementation-adversary | Best-effort event paths may discard lock/write errors. | Some secondary observability events can be lost silently. | non-blocking | defer | Existing cleanup/process event paths are explicitly best-effort and should not mask primary task outcomes. | No code change in this patch. | Revisit if event durability becomes strict for cleanup/process auxiliary events. |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: pending
- Blocking re-review passed: pending
- Blocking re-review round links:
  - Round 2 pending
- Blocking re-review launch records:
  - Round 2 pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: closure review pending
- Allowed to proceed: no

## Round 2: blocking closure review

### Review Input

#### Objective
Confirm that the accepted Round 1 blocking findings are fixed before the run is allowed to proceed to full validation and real Terminal-Bench rerun.

#### Review Target
Closure fixes for selected-test failure diagnostics, event append completeness, and event-log integrity validation before existing run reuse.

#### Target Locations
- `crates/harnesslab-infra/src/event.rs`
- `crates/harnesslab-cli/src/app.rs`
- `crates/harnesslab-cli/tests/event_log_integrity_contract.rs`
- `scripts/test-after-change.sh`
- `scripts/verify-test-after-change-select-output.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/mvp-development-spec.md`
- `vs_review/2026-06-02-event-log-concurrency-review.md`

#### Change Introduction
After Round 1, `META-001` now forces a real selected test panic and checks the panic sentinel and failing test name; `LOG-005` now checks every expected worker/index event exactly once; `validate_event_log` rejects malformed JSONL and CLI reuse paths call it before `resume`, `replay`, and `report open`. New integration tests `INT-032`, `INT-033`, and `INT-034` cover those CLI paths.

#### Risk Focus
- Accepted blocking findings may only be partially fixed.
- New integrity checks may be wired to one CLI path but not all reuse paths.
- Tests may pass without proving the path fails before mutation or append.
- Registry/docs may drift from the new behavior.

#### Assumptions To Attack
- `META-001` now exercises a real assertion failure, not a compiler/tooling failure.
- `LOG-005` now detects lost or duplicated logical writes.
- Existing malformed `events.jsonl` is rejected before `run_resumed` can be appended.
- `run replay` and `report open` reject malformed source/run event logs before reuse.

#### Adversarial Lenses
- testing
- observability
- data consistency
- regression closure

#### Verification Status
- `scripts/test-after-change.sh --select LOG-005` passed.
- `scripts/test-after-change.sh --select LOG-006` passed.
- `scripts/verify-test-after-change-select-output.sh` passed.
- `cargo test -p harnesslab-cli --test event_log_integrity_contract -- --nocapture` passed.
- `scripts/test-after-change.sh --select INT-032` passed.
- `scripts/test-after-change.sh --select INT-033` passed.
- `scripts/test-after-change.sh --select INT-034` passed.
- `scripts/verify-test-registry.sh` passed.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Focus only on whether accepted Round 1 blocking findings are closed.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 10 minutes | 5 minutes once if active | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| test-validity-adversary | Round 1 accepted blocking testing findings require closure. | testing, anti-self-deception |
| observability-adversary | Round 1 accepted blocking observability finding requires closure. | artifact integrity, reuse paths |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| test-validity-adversary-closure | multi_agent_v1.spawn_agent test-engineer | 019e858b-e5a5-7fa1-81ba-f5f46f9f7070 | tool spawn result nickname Heisenberg | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions | yes |
| observability-adversary-closure | multi_agent_v1.spawn_agent verifier | 019e858c-0f19-7be0-9cfa-d9db20b715c5 | tool spawn result nickname Leibniz | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| test-validity-adversary-closure-1 | test-validity-adversary-closure | 1 | 019e858b-e5a5-7fa1-81ba-f5f46f9f7070 | within high-risk wait | completed | returned closure output | completed |
| observability-adversary-closure-1 | observability-adversary-closure | 1 | 019e858c-0f19-7be0-9cfa-d9db20b715c5 | within high-risk wait | completed | returned closure output | completed |

### Reviewer Outputs

#### test-validity-adversary-closure-1

##### Summary
Round 1 accepted blocking testing findings are closed at the code/test-gate level. `META-001` now uses a real selected-test panic check, `LOG-005` proves completeness across expected worker/index writes, and the real concurrent CLI runtime proof is still correctly tracked as pending rather than falsely claimed complete.

##### Blocking Findings
- none

##### Non-blocking Risks
- none within the accepted Round 1 testing-closure scope

##### Required Fixes
- none

##### Missing Tests
- Fresh concurrent real CLI runtime proof is still missing by design and must remain a separate validation gate before the original product-level regression is fully closed.

##### Missing Logs / Observability
- none in this closure slice

##### Evidence
- `crates/harnesslab-infra/src/event.rs:235` - `META-001` forces a real panic from the selected test body.
- `scripts/verify-test-after-change-select-output.sh:4` - meta gate checks the panic sentinel and failing test name.
- `scripts/test-after-change.sh:122` - selected-test runner prints captured cargo output before propagating failure.
- `crates/harnesslab-infra/src/event.rs:161` - `LOG-005` checks every expected worker/index pair.
- `crates/harnesslab-cli/tests/event_log_integrity_contract.rs:6` - resume malformed-log rejection is covered.
- `crates/harnesslab-cli/tests/event_log_integrity_contract.rs:30` - replay malformed-log rejection is covered.
- `crates/harnesslab-cli/tests/event_log_integrity_contract.rs:51` - report open malformed-log rejection is covered.

#### observability-adversary-closure-1

##### Summary
Pass. The accepted Round 1 blocking observability finding is fixed: `resume`, `replay`, and `report open` validate `events.jsonl` before reusing a run directory; the validator reports path/line context; and `resume` cannot append `run_resumed` before the failure because validation happens before `resume_run`.

##### Blocking Findings
- none

##### Non-blocking Risks
- none

##### Required Fixes
- none

##### Missing Tests
- none

##### Missing Logs / Observability
- none

##### Evidence
- `crates/harnesslab-cli/src/app.rs:174` - validates event log before `resume_run`.
- `crates/harnesslab-cli/src/app.rs:178` - validates event log before `replay_run`.
- `crates/harnesslab-cli/src/app.rs:213` - validates event log before `report open` uses the run directory.
- `crates/harnesslab-infra/src/event.rs:49` - validator rejects empty, blank, and malformed JSONL with path/line context.
- `crates/harnesslab-cli/tests/event_log_integrity_contract.rs:7` - resume test asserts failure and no `run_resumed` append.
- `crates/harnesslab-cli/tests/event_log_integrity_contract.rs:31` - replay test covers malformed source rejection.
- `crates/harnesslab-cli/tests/event_log_integrity_contract.rs:51` - report open test covers malformed run rejection.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| test-validity-adversary-closure | No blocking findings; code/test-gate closure passed. | n/a | n/a | accept | Reviewer verified `META-001`, `LOG-005`, and malformed-log tests. | Proceed to full local gate. | Real concurrent CLI benchmark proof remains required after full gate. |
| observability-adversary-closure | No blocking findings; event-log reuse integrity closure passed. | n/a | n/a | accept | Reviewer verified `resume`, `replay`, and `report open` validation before reuse. | Proceed to full local gate. | none |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 2
- Blocking re-review launch records:
  - test-validity-adversary-closure and observability-adversary-closure launch records above
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Validation

- `scripts/test-after-change.sh`: passed.
- Nextest: 281 tests passed.
- Python Terminal-Bench adapter tests: 22 tests passed.
- Real Terminal-Bench import timeout cleanup script: passed.
- Real Terminal-Bench Docker activity watchdog script: passed.
- Registry check: 101 active tests validated.
- Meta gate: `META-001` forced a real selected-test panic and verified output visibility.
- Traceability generation: passed.
- Secret scan: passed.
- Coverage gate: line 95.18% (6831/7177), branch 82.92% (597/720).
- New-file coverage gate: passed.

## Final Conclusion

Code/test-gate closure passed and full local validation passed. Renewed real full Terminal-Bench execution is still required before the benchmark execution claim is complete.
