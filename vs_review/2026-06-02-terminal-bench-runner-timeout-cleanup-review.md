# Subagent VS Review: Terminal-Bench Runner Timeout And Cleanup

- Created: 2026-06-02T12:16:25+08:00
- Updated: 2026-06-02T12:16:25+08:00
- Report schema: adversarial-v1
- Task: Fix real Terminal-Bench execution failures discovered during a full claude-ds run, then rerun through HarnessLab without mock or smoke shortcuts.
- Report path: `vs_review/2026-06-02-terminal-bench-runner-timeout-cleanup-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: open

## Round 1: Post-Fix Implementation Review

### Review Input

#### Objective
Challenge the recent Terminal-Bench execution timeout, run-health, and cleanup changes before another real benchmark run.

#### Review Target
Code implementation, test strategy, operational cleanup behavior, and documentation.

#### Target Locations
- `crates/harnesslab-core/src/model.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_timeout.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_cleanup.rs`
- `crates/harnesslab-cli/src/runner/monitor.rs`
- `crates/harnesslab-cli/src/runner/cleanup.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `crates/harnesslab-cli/src/runner/monitor_tests.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_watchdog_contract.rs`
- `tests/TEST_REGISTRY.toml`
- `scripts/test-after-change.sh`
- `docs/development-operations.md`
- `docs/architecture.md`
- `docs/mvp-development-spec.md`

#### Change Introduction
- Added `FailureCode::ExternalRunnerTimeout` and mapped Terminal-Bench official runner `TerminationReason::Timeout` to `execution/external_runner_timeout` instead of `execution/agent_timeout`.
- Increased Terminal-Bench outer process timeout default from `agent_timeout + test_timeout + 600` to `agent_timeout + test_timeout + 1800` for first Docker build and setup budget.
- `RunMonitor` records `external_runner_timeouts` and aborts immediately on that execution failure.
- `RunSandboxCleanup` now calls Terminal-Bench `cleanup_run_resources(run_dir, run_id)`, which combines recorded compose project snapshots with lowercased run-id compose project scanning.
- Tests and docs were updated.

#### Risk Focus
- Whether cleanup fallback actually cleans active Terminal-Bench compose resources after abnormal/internal exit, including run id case conversion and unrecorded active tasks.
- Whether immediate abort on `external_runner_timeout` creates bad behavior in multi-task runs or reports.
- Whether `ExternalRunnerTimeout` propagates consistently to `health_impact`, run-health, exit codes, report labels, warnings, docs, and tests.
- Whether the longer hard timeout is sufficient without hiding real hangs or undermining the user's desire to stop early on engineering failures.
- Whether tests deterministically prove the real failure mode observed in the latest real run: QEMU Docker build hit outer 1200s and was mislabeled `agent_timeout`.

#### Assumptions To Attack
- Compose project matching by lowercased run id is enough to catch active task compose projects.
- Docker compose resources can be safely removed by project label without the original Terminal-Bench env vars.
- An external runner hard timeout should always abort the run immediately even after prior successes.
- Timeout and warning precedence cannot drop official benchmark context.
- The no-output, progress, and hard-timeout tests cannot pass while the real bug remains.

#### Adversarial Lenses
- requirements
- state
- concurrency
- failure
- data
- maintenance
- testing
- observability

#### Verification Status
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `cargo test -p harnesslab-cli --all-features terminal_bench -- --nocapture` passed.
- `scripts/test-after-change.sh --select ORCH-019` passed.
- `scripts/test-after-change.sh --select TB-004` passed.
- `scripts/test-after-change.sh --select INT-026` passed.
- `scripts/test-after-change.sh --select INT-028` passed.
- `scripts/test-after-change.sh --select INT-031` passed three consecutive runs after hardening the test.
- `scripts/verify-test-registry.sh` passed with 116 active tests.
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
| complex | 12 minutes | 8 minutes if alive and incomplete | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| code-reviewer | Multi-file execution semantics and test evidence need adversarial implementation review. | Timeout classification, cleanup fallback, run-health abort, test determinism |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | `multi_agent_v1.spawn_agent` | `019e868c-71be-7af2-9385-7e6c38f3a6e2` / Ramanujan | spawn_agent tool result | `fork_context=false` | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round1-code-reviewer | code-reviewer | 1 | `019e868c-71be-7af2-9385-7e6c38f3a6e2` | 12 minutes policy window | completed | reviewer returned `REQUEST CHANGES` | completed |

### Reviewer Outputs

#### round1-code-reviewer

Summary: `clippy` and targeted Terminal-Bench tests passed, and `external_runner_timeout` was wired through normal result/report paths, but the reviewer did not consider the implementation safe for a real rerun because cleanup fallback did not cover stale abnormal-exit compose resources.

Blocking Findings:

- [HIGH] Stale-run cleanup cannot recover unrecorded active Terminal-Bench compose projects after a prior abnormal exit. Counterexample: a run dies during its first active task before `post_task`; no `terminal-bench-compose-projects.json` exists, so the next pre-run cleanup skips the sibling. Even with a snapshot, the code passed the current run id to sibling cleanup, so unrecorded active projects from the stale run are missed.
- [HIGH] Token matching does not mirror official Terminal-Bench run-id normalization. Counterexample: a valid HarnessLab `run_id` containing `_` or `.` is converted by Terminal-Bench to a dash-normalized lowercase token, but cleanup only scanned raw and lowercase forms, so fallback label scanning can miss valid compose projects.

Non-blocking Risks:

- Immediate abort on `external_runner_timeout` looked internally consistent across health impact, run health, execution exit code, and report rendering.
- The `+1800s` outer timeout is a policy choice that still needs empirical validation on cold-cache/QEMU-heavy tasks.

Required Fixes:

- Make stale-run cleanup use each stale run's own Terminal-Bench token and do not require an existing snapshot before attempting fallback cleanup.
- Normalize cleanup match tokens exactly like `official_run_id()`.
- Re-run review after those fixes before the real benchmark.

Missing Tests:

- Add a contract for a previous Terminal-Bench run that crashed before `post_task` with no snapshot.
- Add a contract that a stale sibling run uses its own run id instead of the current run id.
- Add a direct contract for hard-timeout classification as `external_runner_timeout`.

Missing Logs / Observability:

- Run-level cleanup events should log scan token, snapshot count, matched count, and project list.

### Main Agent Response

- accept: Stale-run cleanup now discovers Terminal-Bench sibling runs from `run.json`, existing snapshot files, or a Terminal-Bench-shaped run directory name. It passes each stale run's own `scan_run_id` into cleanup instead of the current run id. Evidence: `crates/harnesslab-cli/src/runner/cleanup.rs`, tests `cleanup_007_terminal_bench_pre_run_considers_stale_run_without_snapshot` and `cleanup_008_terminal_bench_pre_run_uses_stale_run_json_id`.
- accept: Cleanup token generation now includes the raw run id, lowercase run id, and the official Terminal-Bench-style normalized token that lowercases ASCII alphanumerics and replaces other characters with `-`. Evidence: `crates/harnesslab-cli/src/runner/external/terminal_bench_cleanup.rs`, test `terminal_bench_cleanup_match_tokens_include_official_normalized_token`.
- accept: Hard-timeout classification is now explicitly covered by `terminal_bench_hard_timeout_maps_to_external_runner_timeout`, `int_028_terminal_bench_hard_timeout_overrides_official_result`, and the stabilized `int_031_terminal_bench_progress_deferral_still_hard_times_out`.
- accept: Run-level cleanup events now include `scan_run_id`, `tokens`, `projects`, `snapshot_projects`, `matched_projects`, and removed count.
- accept: The first full nextest rerun exposed a flaky `INT-031` timing window where scheduler load could still produce `external_runner_no_progress`. The test was fixed to use a wider no-output threshold and faster continuous progress writes without changing product behavior.

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: pending
- Blocking re-review passed: pending
- Blocking re-review round links:
  - Round 2 pending
- Blocking re-review launch records:
  - Round 2 pending
- Rejected findings backed by evidence: none
- Deferred findings documented: none
- Blocked reason: accepted blocking findings require fresh closure review
- Allowed to proceed: no

## Validation Evidence Before Closure Review

- `cargo fmt --all --check`: passed
- `scripts/test-after-change.sh --select ORCH-020`: passed
- `scripts/test-after-change.sh --select ORCH-021`: passed
- `scripts/test-after-change.sh --select TB-004`: passed
- `scripts/test-after-change.sh --select INT-031`: passed three consecutive runs after timing hardening
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed
- `scripts/verify-test-registry.sh`: passed with 15 requirements and 118 tests
- `cargo nextest run --workspace --all-features`: first run exposed flaky `INT-031`; second run passed 297/297

## Round 2: Accepted Blocking Fix Closure Review

### Review Input

#### Objective
Decide whether the two accepted Round 1 blocking findings are actually closed before a real Terminal-Bench benchmark rerun.

#### Review Target
Accepted blocking fixes only: stale Terminal-Bench cleanup recovery, official token normalization, cleanup observability, and deterministic watchdog tests.

#### Target Locations
- `crates/harnesslab-cli/src/runner/cleanup.rs`
- `crates/harnesslab-cli/src/runner/cleanup_tests.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_cleanup.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `crates/harnesslab-cli/src/runner/monitor.rs`
- `crates/harnesslab-cli/src/runner/monitor_tests.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_watchdog_contract.rs`
- `tests/TEST_REGISTRY.toml`
- `scripts/test-after-change.sh`
- `docs/development-operations.md`
- `docs/architecture.md`
- `docs/mvp-development-spec.md`
- `vs_review/2026-06-02-terminal-bench-runner-timeout-cleanup-review.md`

#### Closure Questions
- Can pre-run stale cleanup recover a prior Terminal-Bench run that died before `post_task` and has no `terminal-bench-compose-projects.json` snapshot?
- When a stale sibling has `run.json`, does cleanup scan with the stale run's own `run_id` rather than the current run id?
- Does cleanup scan include the same dash-normalized lowercase token that Terminal-Bench `official_run_id()` uses, not just raw/lowercase strings?
- Are cleanup events sufficient to diagnose which token/project path was attempted?
- Are tests deterministic enough, especially `INT-031` progress deferral versus hard timeout under concurrent nextest load?

#### Verification Status
- `cargo fmt --all --check` passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- `scripts/test-after-change.sh --select ORCH-020` passed.
- `scripts/test-after-change.sh --select ORCH-021` passed.
- `scripts/test-after-change.sh --select TB-004` passed.
- `scripts/test-after-change.sh --select INT-031` passed three consecutive runs after timing hardening.
- `cargo nextest run --workspace --all-features` passed 297/297 after timing hardening.

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| code-reviewer | Accepted blocking implementation fixes require independent closure review before proceeding. | Cleanup recovery, token normalization, test determinism, observability |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | `multi_agent_v1.spawn_agent` | `019e86a1-a344-7b03-b19e-ff602f045188` / Sartre | spawn_agent tool result | `fork_context=false` | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round2-code-reviewer | code-reviewer | 1 | `019e86a1-a344-7b03-b19e-ff602f045188` | 12 minutes policy window | completed | reviewer returned `PASS` | completed |

### Reviewer Outputs

#### round2-code-reviewer

Summary: Accepted Round 1 blockers look closed. Stale Terminal-Bench sibling cleanup now discovers runs from `run.json`, snapshot presence, or Terminal-Bench-shaped directory names; passes each stale run's own `scan_run_id`; and scans raw, lowercase, and official dash-normalized tokens. Hard timeout mapping and run-health abort are wired consistently.

Blocking Findings:

- None.

Non-blocking Risks:

- [LOW] The fallback accepting any sibling basename containing `terminal-bench` is too broad when both `run.json` and snapshot are missing. Counterexample: `runs/debug-terminal-bench-notes/` could trigger substring-based compose scanning.
- [LOW] `int_031_terminal_bench_progress_deferral_still_hard_times_out` remains wall-clock based, though the widened timing passed repeated runs and full nextest.

Required Fixes:

- None before the real rerun.

Missing Tests:

- A combined end-to-end stale pre-run cleanup test with no snapshot, stale `run.json`, real Docker label scanning, and normalized-token matching is still a small coverage gap. Current coverage is split across unit and post-task integration tests.

Missing Logs / Observability:

- None blocking. Run-level cleanup events now include `run`, `scan_run_id`, `tokens`, `projects`, `snapshot_projects`, `matched_projects`, and removed count.

Final recommendation: PASS.

### Main Agent Response

- accept: The low-risk broad basename fallback was tightened after Round 2. Sibling directories without `run.json` and without `terminal-bench-compose-projects.json` are only considered when they look like a HarnessLab Terminal-Bench run directory with `-terminal-bench-` and a UTC timestamp-like suffix. Evidence: `looks_like_terminal_bench_run_dir` and `cleanup_009_terminal_bench_pre_run_ignores_loose_name_match`.
- accept: `INT-031` remains wall-clock based but passed three consecutive single runs, full nextest, and coverage. The separate coverage-only flake in `c_sbox_018_progress_growth_resets_activity_grace` was fixed by writing progress before the no-output threshold boundary and retaining a bounded timing assertion.

## Round 3: Post-Closure Low-Risk Fix Review

### Review Input

#### Objective
Review only the post-closure low-risk fixes: tighter Terminal-Bench stale cleanup basename fallback, ORCH-022 registry wiring, and coverage-flake hardening for process activity grace.

#### Target Locations
- `crates/harnesslab-cli/src/runner/cleanup.rs`
- `crates/harnesslab-cli/src/runner/cleanup_tests.rs`
- `crates/harnesslab-infra/src/process_tests.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `vs_review/2026-06-02-terminal-bench-runner-timeout-cleanup-review.md`

#### Closure Questions
- Does `looks_like_terminal_bench_run_dir` accidentally reject real HarnessLab Terminal-Bench run ids, including replay runs and agent names with hyphens?
- Does it still overmatch obvious non-run directories?
- Is ORCH-022 sufficient for the prior nonblocking risk?
- Does the process test still prove progress growth resets activity grace, or did the timing relaxation make it toothless?
- Are registry and `--select` wiring correct?

#### Verification Status
- `cargo fmt --all --check`: passed after formatting.
- `scripts/test-after-change.sh --select ORCH-022`: passed.
- `cargo +nightly-2026-05-26 llvm-cov test -p harnesslab-infra --all-features process::tests::c_sbox_018_progress_growth_resets_activity_grace -- --exact --nocapture`: passed.
- `scripts/verify-test-registry.sh`: passed with 15 requirements and 119 tests.

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| code-reviewer | Post-review code changes require a fresh focused review before final validation. | Cleanup target matching, timing-test stability, registry wiring |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | `multi_agent_v1.spawn_agent` | `019e86ad-6d22-7f12-afe0-42a4e4b2ef52` / Singer | spawn_agent tool result | `fork_context=false` | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round3-code-reviewer | code-reviewer | 1 | `019e86ad-6d22-7f12-afe0-42a4e4b2ef52` | 12 minutes policy window | completed | reviewer returned `PASS` | completed |

### Reviewer Outputs

#### round3-code-reviewer

Summary: Scoped post-closure changes look safe overall. The reviewer found no blocking regression in tightened stale-cleanup fallback, ORCH-022 wiring, or watchdog test hardening. Real HarnessLab Terminal-Bench run ids, including replay ids and agent names with hyphens, are still accepted by the new fallback because generated run ids end in a timestamp suffix.

Blocking Findings:

- None.

Non-blocking Risks:

- [LOW] The fallback still accepted malformed pseudo-timestamps like `debug-terminal-bench-full-20260602TnotesZ` because it only checked the date prefix, `T`, alphanumeric tail, and trailing `Z`.

Required Fixes:

- None.

Missing Tests / Logs:

- Add one negative case for a pseudo-run basename with a fake timestamp suffix, for example `debug-terminal-bench-full-20260602TnotesZ`.
- No additional logging gap found.

Final recommendation: PASS.

### Main Agent Response

- accept: The pseudo-timestamp gap was closed. `looks_like_terminal_bench_run_dir` now requires `YYYYMMDDT<digits>Z`, and `cleanup_009_terminal_bench_pre_run_ignores_loose_name_match` now asserts that `debug-terminal-bench-full-20260602TnotesZ` is ignored. Evidence: `scripts/test-after-change.sh --select ORCH-022`, `scripts/verify-test-registry.sh`, and `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.

## Round 4: Final Micro-Fix Review

### Review Input

#### Objective
Review only the final micro-fix after Round 3: stricter timestamp suffix matching and the pseudo timestamp negative test.

#### Target Locations
- `crates/harnesslab-cli/src/runner/cleanup.rs`
- `crates/harnesslab-cli/src/runner/cleanup_tests.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`

#### Closure Questions
- Does the stricter timestamp still accept real HarnessLab run ids from `timestamp_id()`, including replay and hyphenated agent names?
- Does the new negative test cover the pseudo timestamp false-positive from Round 3?
- Is there any blocking issue before final validation and real benchmark rerun?

#### Verification Status
- `cargo fmt --all --check`: passed.
- `scripts/test-after-change.sh --select ORCH-022`: passed.
- `scripts/verify-test-registry.sh`: passed with 15 requirements and 119 tests.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| code-reviewer | Final post-review code change needs an independent read before final gate. | Cleanup target matching, regression risk |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | `multi_agent_v1.spawn_agent` | `019e86b8-5c89-73e1-9789-f690b7036ee9` / Averroes | spawn_agent tool result | `fork_context=false` | Round 4 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round4-code-reviewer | code-reviewer | 1 | `019e86b8-5c89-73e1-9789-f690b7036ee9` | 12 minutes policy window | completed | reviewer returned `PASS` | completed |

### Reviewer Outputs

#### round4-code-reviewer

Summary: Reviewed the final micro-change in `cleanup.rs` and `cleanup_tests.rs`, plus selector and registry wiring. Real HarnessLab run ids remain accepted because normal and replay runs end with `store::timestamp_id()`, which produces `YYYYMMDDT<digits>Z`; hyphenated agent names are unaffected because only the suffix after the last `-` is inspected. Renamed stale directories with a valid `run.json` bypass the heuristic entirely.

Blocking Findings:

- None.

Non-blocking Risks:

- [LOW] There is no direct regression test for a replay-shaped stale directory with a hyphenated agent name, though the logic is correct by construction.

Required Fixes:

- None.

Missing Tests / Logs:

- Optional: add a replay + hyphenated-agent directory-name case later.
- For the real benchmark rerun, inspect `terminal_bench_docker_cleanup` events with `scan_run_id=` to confirm stale-run targeting.

Final recommendation: PASS.

### Main Agent Response

- defer: The optional replay + hyphenated-agent directory-name case is not needed before the real benchmark rerun because real replay dirs satisfy the same final timestamp suffix check, and stale dirs with `run.json` bypass the heuristic. No blocking finding remains.

## Final Validation

- `cargo fmt --all --check`: passed.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`: passed.
- `scripts/test-after-change.sh --select ORCH-022`: passed.
- `scripts/verify-test-registry.sh`: passed with 15 requirements and 119 tests.
- `cargo +nightly-2026-05-26 llvm-cov test -p harnesslab-infra --all-features process::tests::c_sbox_018_progress_growth_resets_activity_grace -- --exact --nocapture`: passed.
- `cargo nextest run --workspace --all-features`: passed 298/298.
- `scripts/test-after-change.sh`: passed; coverage lines 95.28%, branches 83.03%.

## Final Conclusion

All blocking review findings are closed and re-reviewed. Round 4 has no blocking findings. Proceed to real benchmark rerun.
