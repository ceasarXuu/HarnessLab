# Subagent VS Review: Terminal-Bench Progress Watchdog

- Created: 2026-06-02T16:39:10+0800
- Updated: 2026-06-02T16:52:56+0800
- Report schema: adversarial-v1
- Task: rerun the real 80-task Terminal-Bench through HarnessLab, monitor failures, and fix engineering issues before wasting the full bench.
- Report path: `vs_review/2026-06-02-terminal-bench-progress-watchdog-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: progress watchdog implementation review

### Review Input

#### Objective
Ensure HarnessLab's Terminal-Bench no-output watchdog does not treat stale `run.log` creation as fresh progress at the watchdog boundary, so stuck Docker setup/build phases are stopped promptly without breaking valid progress-file behavior.

#### Review Target
Implementation, tests, traceability, and operations documentation for the host process no-output watchdog progress-file sampling.

#### Target Locations
- `crates/harnesslab-infra/src/process.rs`
- `crates/harnesslab-infra/src/process_tests.rs`
- `crates/harnesslab-infra/src/process_progress.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/development-operations.md`

#### Change Introduction
`HostProcessExecutor` now checks registered progress files on every no-output-watchdog loop iteration instead of only when the quiet period has already exceeded the watchdog timeout. The progress-file contract was adjusted so growth resets the no-output window from the actual observed write time, not from the later watchdog boundary. A new `C-SBOX-020` regression test proves early progress is sampled before the boundary.

#### Risk Focus
- The executor may now emit progress activity events earlier or more often than intended.
- Continuous progress sampling may perturb activity grace timing or existing no-output semantics.
- Updated tests may encode timing assumptions that are flaky or too implementation-specific.
- The old `C-SBOX-017` contract changed semantics and must still prove a meaningful user-facing behavior.
- Documentation and registry entries must match the implemented behavior.

#### Assumptions To Attack
- File metadata polling every loop is acceptable and does not create a performance or correctness regression.
- `last_output_ms` reset at observation time is close enough to actual write time for watchdog purposes.
- Progress-file growth should not protect a process until hard timeout unless growth continues.
- Activity grace remains bounded to one watchdog window after the last real progress.
- The tests would fail on the pre-fix behavior and pass only for the intended behavior.

#### Adversarial Lenses
- implementation correctness
- failure and timeout state
- concurrency and process lifecycle
- test validity and flake resistance
- observability and operations documentation

#### Verification Status
- `scripts/test-after-change.sh --select C-SBOX-017`: passed
- `scripts/test-after-change.sh --select C-SBOX-018`: passed
- `scripts/test-after-change.sh --select C-SBOX-020`: passed
- `cargo test -p harnesslab-infra`: passed
- `scripts/verify-test-registry.sh`: passed
- `git diff --check`: passed
- The full real Terminal-Bench rerun has not yet been restarted after this fix.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Focus on counterexamples and missing proof, not style.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 10 minutes | one bounded 5 minute extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | This is a process-state and timeout behavior change in `HostProcessExecutor`. | correctness, process lifecycle, regression risk |
| test-validity-adversary | The fix depends on timing-sensitive tests and a renamed contract. | self-deceptive tests, flake risk, traceability |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | `multi_agent_v1.spawn_agent` role `code-reviewer` | `019e877d-4681-7751-97a6-aad8493f32dc` | spawn_agent result nickname `Russell` | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |
| test-validity-adversary | `multi_agent_v1.spawn_agent` role `test-engineer` | `019e877d-f006-7b22-8680-d769d565dd20` | spawn_agent result nickname `Mendel`; first spawn attempt failed before launch due agent thread limit | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| implementation-adversary-round-1 | implementation-adversary | 1 | `019e877d-4681-7751-97a6-aad8493f32dc` | ~10 minutes | completed | reviewer returned blocking and non-blocking findings | completed |
| test-validity-adversary-round-1 | test-validity-adversary | 1 | `019e877d-f006-7b22-8680-d769d565dd20` | ~10 minutes | completed | reviewer returned blocking and non-blocking findings | completed |

### User Decision After Failed Review

- Required if primary and replacement attempts both fail.
- Decision: n/a
- User-visible reason: n/a

### Reviewer Outputs

#### implementation-adversary-round-1

##### Summary
The core code direction is correct: `process.rs` samples progress paths before the watchdog boundary. The review did not pass because test evidence did not yet prove the intended behavior.

##### Blocking Findings
- `C-SBOX-020` could pass if progress was ignored entirely.
  - Broken assumption: `elapsed < 1.8s` proves early progress was sampled.
  - Failure scenario: an implementation that ignores the progress write and kills at the original 1s no-output boundary still passes.
  - Trigger condition: progress-file sampling is missed or removed.
  - Impact: the new regression test would allow the exact bug family to return.
  - Proof needed: add a lower-bound assertion past the original no-output boundary and assert progress event ordering before no-progress.
- `C-SBOX-018` selector and registry did not run the progress-growth/activity-grace test.
  - Broken assumption: `scripts/test-after-change.sh --select C-SBOX-018` proved progress growth resets activity grace.
  - Failure scenario: `c_sbox_018_progress_growth_resets_activity_grace` can fail or be deleted while the selector remains green.
  - Trigger condition: CI or agents rely on registered selectors rather than hand-picked cargo names.
  - Impact: the test engineering gate is misleading.
  - Proof needed: add or remap a registered selector for the progress/activity interaction.

##### Non-blocking Risks
- Progress and activity deferrals share the same 30s event rate-limit bucket.
  - Broken assumption: earlier progress sampling does not affect observability.
  - Failure scenario: a progress event suppresses a near-immediate activity event.
  - Trigger condition: progress deferral followed by activity grace inside the rate-limit window.
  - Impact: event stream may be less diagnostic.
  - Proof needed: add event-order coverage or separate rate-limit buckets if this becomes confusing in real runs.
- `ProgressWatcher` returns after the first changed path.
  - Broken assumption: all progress files are sampled equally in one loop.
  - Failure scenario: simultaneous changes across multiple files are observed over multiple loops.
  - Trigger condition: multiple registered progress paths grow in the same sampling interval.
  - Impact: slight extra watchdog delay and noisy `last_progress`.
  - Proof needed: add a multi-progress-path unit test or update watcher semantics.

##### Required Fixes
- Harden `C-SBOX-020` with a lower bound and progress-before-no-progress event assertion.
- Add a first-class selector and registry ID for `c_sbox_018_progress_growth_resets_activity_grace`.

##### Missing Tests
- Real Terminal-Bench regression for early single `run.log` write followed by silence.
- Event-order test for progress then activity grace.
- Multi-progress-path test.

##### Missing Logs / Observability
- Progress events do not include observed timing or quiet-window values.
- Docs claim early sampling, but logs do not directly expose the timing.

##### Evidence
- `process.rs` moved `changed_path()` before the quiet-window check.
- `C-SBOX-020` originally lacked a lower bound and event assertion.
- `C-SBOX-018` selector ran `c_sbox_018_no_output_activity_has_bounded_grace`, not the progress/activity test.

#### test-validity-adversary-round-1

##### Summary
The implementation change has the right shape, and `C-SBOX-020` distinguishes late-boundary sampling from early sampling. The proof packet was incomplete because a progress/activity test was not registered.

##### Blocking Findings
- `C-SBOX-018` selector and registry did not cover progress-growth/activity-grace behavior.
  - Broken assumption: the cited `C-SBOX-018` run proved the changed progress semantics.
  - Failure scenario: the progress/activity test can regress while `C-SBOX-018` remains green.
  - Trigger condition: relying on the registered selector as traceable gate evidence.
  - Impact: test evidence can drift from the actual regression surface.
  - Proof needed: register the progress/activity test or fold it into the existing traceable contract.

##### Non-blocking Risks
- `C-SBOX-020` is wall-clock based.
  - Broken assumption: the threshold is stable on loaded CI machines.
  - Failure scenario: a correct implementation fails under scheduler jitter.
  - Trigger condition: slow process spawn, shell sleep, or poll wakeups.
  - Impact: intermittent false negatives.
  - Proof needed: repeated focused runs or a more deterministic test seam.

##### Required Fixes
- Register the untracked progress-growth/activity-grace test.
- Update verification evidence to cite exact test IDs.
- Harden `C-SBOX-020` proof with repeated-run evidence or stronger assertions.

##### Missing Tests
- Terminal-Bench integration test where a fake official runner creates `run.log` once early and then goes silent.
- Registry-covered progress/activity interaction test.
- Negative test for a pre-existing non-growing `run.log`.

##### Missing Logs / Observability
- Progress-file events do not record observed age or quiet-window values.
- No explicit event shows the quiet-window reset timestamp/value after progress growth.

##### Evidence
- `process.rs` samples `changed_path()` before `quiet_for_ms >= timeout`.
- `process_progress.rs` detects length growth.
- `C-SBOX-020` is the boundary-focused unit test.
- `C-SBOX-018` selector did not run the progress/activity test.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | `C-SBOX-020` could pass if progress was ignored | A direct no-progress kill at the original boundary could still satisfy only `< 1.8s` | blocking | accept | Original test had no lower bound and no event assertion | Changed `C-SBOX-020` to use a 2s watchdog, write progress at 0.5s, assert elapsed is `>= 2.3s` and `< 3.2s`, and assert `progress file path=` appears before `external_runner_no_progress` | Round 2 closure review |
| implementation-adversary | `C-SBOX-018` selector drift | Registered selector did not run progress/activity test | blocking | accept | `scripts/test-after-change.sh` mapped `C-SBOX-018` only to `c_sbox_018_no_output_activity_has_bounded_grace` | Added first-class `C-SBOX-021` selector and registry entry for `c_sbox_018_progress_growth_resets_activity_grace` | Round 2 closure review |
| test-validity-adversary | Progress/activity test was unregistered | Traceable proof did not cover changed behavior | blocking | accept | Same selector drift as above | Added `C-SBOX-021`; reran it successfully | Round 2 closure review |
| test-validity-adversary | Missing Terminal-Bench early single-run.log integration | Unit-level proof did not exercise adapter path | blocking | accept | Real issue happened through Terminal-Bench adapter progress path | Added `INT-039` for a fake official runner that writes `run.log` once early then goes silent; asserts `external_runner_no_progress`, progress event, no hard timeout | Round 2 closure review |
| implementation-adversary | Progress/activity event shared rate limit | A progress event may suppress nearby activity diagnostics | non-blocking | defer | Not required to fix the current stale-progress timing bug; no failure observed in current real Docker validation | Keep as future observability hardening if real runs show ambiguous event streams | none |
| implementation-adversary | Multiple progress paths return one changed path per loop | Simultaneous growth across multiple paths may be observed over multiple loops | non-blocking | defer | Terminal-Bench currently registers one `run.log` progress path | Track as future generic executor hardening | none |
| implementation-adversary | Progress events lack observed timing fields | Real-run audit requires inference from elapsed and event order | non-blocking | defer | Current events plus strengthened tests are enough for this fix; adding structured timing fields is larger observability work | Track as future log schema hardening | none |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 2
- Blocking re-review launch records:
  - test-validity-adversary `019e8787-7f6f-7222-aceb-a30134cb35a8`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

Round 1 accepted blocking findings were fixed and passed a fresh Round 2 closure review. The task may proceed to commit, rebuild, and the real 80-task Terminal-Bench rerun.

## Round 2: accepted blocking closure review

### Review Input

#### Objective
Verify that accepted blocking findings from Round 1 are actually closed before the real 80-task Terminal-Bench rerun restarts.

#### Review Target
Closure fixes for `C-SBOX-020`, the progress/activity selector drift, and the missing Terminal-Bench early `run.log` integration regression.

#### Target Locations
- `vs_review/2026-06-02-terminal-bench-progress-watchdog-review.md`
- `crates/harnesslab-infra/src/process_tests.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/development-operations.md`

#### Change Introduction
`C-SBOX-020` now has a lower-bound assertion beyond the original no-output boundary and validates progress event ordering before no-progress. `C-SBOX-021` was added as the first-class selector and registry entry for `c_sbox_018_progress_growth_resets_activity_grace`. `INT-039` was added for Terminal-Bench adapter behavior when the official runner writes `run.log` once early and then goes silent.

#### Risk Focus
- Whether Round 1 blocking findings are closed by registered, runnable tests.
- Whether `C-SBOX-020` can still pass when progress is ignored.
- Whether `C-SBOX-021` actually runs the progress/activity grace test.
- Whether `INT-039` genuinely exercises Terminal-Bench adapter progress paths.

#### Assumptions To Attack
- Selector names and registry entries match the actual tests.
- The new assertions prove the intended timing behavior and cannot pass on the known bad alternatives.
- The integration test verifies `external_runner_no_progress` rather than hard timeout.

#### Adversarial Lenses
- test validity
- traceability
- closure evidence
- failure semantics

#### Verification Status
- `scripts/test-after-change.sh --select C-SBOX-020`: passed
- `scripts/test-after-change.sh --select C-SBOX-021`: passed
- `scripts/test-after-change.sh --select INT-039`: passed
- `scripts/verify-test-registry.sh`: passed, `registry ok: 15 requirements, 122 tests`
- `cargo check -p harnesslab-cli --tests`: passed
- `scripts/verify-terminal-bench-docker-activity-grace-expiry.sh`: passed
- `scripts/verify-terminal-bench-docker-activity-watchdog.sh`: passed

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Review only closure of accepted blocking findings.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 10 minutes | one bounded 5 minute extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| test-validity-adversary | Round 1 accepted blockers were primarily test traceability and test-strength defects. | closure validity, regression proof |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| test-validity-adversary | `multi_agent_v1.spawn_agent` role `test-engineer` | `019e8787-7f6f-7222-aceb-a30134cb35a8` | spawn_agent result nickname `Lovelace` | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| closure-test-validity-round-2 | test-validity-adversary | 1 | `019e8787-7f6f-7222-aceb-a30134cb35a8` | < 10 minutes | completed | closure reviewer found no remaining blocking gaps | completed |

### Reviewer Outputs

#### closure-test-validity-round-2

##### Summary
Closure passed. No remaining blocking gaps were found for the three accepted Round 1 findings. Code, selector wiring, registry entries, and operations docs align with expected closure.

##### Closure Findings
- `C-SBOX-020` is closed: it now enforces a lower bound beyond the original 2s no-output boundary and asserts `progress file path=` appears before `external_runner_no_progress`.
- The progress-growth/activity-grace registration gap is closed: `C-SBOX-021` maps directly to `process::tests::c_sbox_018_progress_growth_resets_activity_grace` and has an active registry entry.
- The missing Terminal-Bench adapter regression is closed: `INT-039` is registered, selected, and implemented as an integration test that writes `run.log` once early then sleeps, asserting no-progress rather than hard timeout.

##### Remaining Blocking Findings
- None.

##### Non-blocking Risks
- Closure reviewer did not independently rerun the commands; static evidence supports closure, and main-agent verification results are recorded in Round 2 input.
- `C-SBOX-020` and `C-SBOX-021` remain wall-clock-sensitive tests; this remains a known non-blocking risk.

##### Required Fixes
- None for closure.

##### Missing Tests
- None that block closure of accepted Round 1 findings.
- Deferred broader gaps remain: multi-progress-path coverage and richer progress/activity observability tests.

##### Missing Logs / Observability
- No closure blocker.

##### Evidence
- `crates/harnesslab-infra/src/process_tests.rs` - hardened `C-SBOX-020` and progress/activity test.
- `scripts/test-after-change.sh` - new `C-SBOX-021` and `INT-039` selector mappings.
- `tests/TEST_REGISTRY.toml` - new `C-SBOX-021` and `INT-039` active registry entries.
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs` - `INT-039` adapter-path assertions.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| test-validity-adversary | Round 1 accepted blockers are closed | Closure reviewer found no remaining blocking gaps | n/a | accept | Static evidence plus completed local verification supports closure | Marked review passed | none |
| test-validity-adversary | Wall-clock sensitivity remains | Correct implementation could still be affected by extreme scheduler jitter | non-blocking | defer | Existing tests are consistent with surrounding executor timing tests and passed focused/local regression | Track as future deterministic test seam if flake appears | none |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 2
- Blocking re-review launch records:
  - test-validity-adversary `019e8787-7f6f-7222-aceb-a30134cb35a8`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: n/a
- Allowed to proceed: yes
