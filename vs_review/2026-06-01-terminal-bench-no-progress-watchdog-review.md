# Terminal-Bench No-Progress Watchdog Review

## Review Target

- Objective: stop silent Terminal-Bench official runner stalls before the outer hard timeout, classify them separately from agent failures, and keep the behavior auditable through tests, logs, reports, and traceability.
- Target files:
  - `crates/harnesslab-infra/src/process.rs`
  - `crates/harnesslab-core/src/model.rs`
  - `crates/harnesslab-cli/src/runner/external.rs`
  - `crates/harnesslab-cli/src/runner/external/terminal_bench_timeout.rs`
  - `crates/harnesslab-cli/src/runner/monitor.rs`
  - `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
  - `scripts/test-after-change.sh`
  - `tests/TEST_REGISTRY.toml`
  - `docs/development-operations.md`
  - `docs/mvp-development-spec.md`
  - `docs/architecture.md`

## Review Input Packet

Reviewers were asked to challenge:

- process-tree killing correctness and log stream behavior
- false positives from stdout/stderr based watchdogs
- Terminal-Bench failure classification and report rendering
- schema/docs compatibility for new serialized enum variants
- run-health handling for repeated runner stalls
- whether tests prove the real CLI contract rather than isolated units

Verification state before closure review:

- `scripts/test-after-change.sh` passed
- 239 Rust tests passed, 0 skipped
- 7 Python tests passed
- registry ok: 15 requirements, 81 tests
- coverage: 95.46% lines, 84.02% branches

## Reviewer Launch Records

| Round | Reviewer | Agent id | Role | Context | Input | Status |
|---|---|---:|---|---|---|---|
| 1 | Linnaeus | `019e7fd1-a91b-7b73-b987-02d53cd3eefd` | code-reviewer | fresh, no forked main context | read-only adversarial code review packet | completed |
| 1 | Kuhn | `019e7fd1-d024-7c22-baec-fffc4552658e` | test-engineer | fresh, no forked main context | read-only adversarial test review packet | completed |
| 2 | Dirac | `019e7fdd-3b36-78d2-b7ec-396f3a613fa1` | code-reviewer | fresh, no forked main context | read-only closure review packet | completed |
| 3 | Dalton | `019e7fea-4787-7bb2-9687-f6f9fa5ae450` | code-reviewer | fresh, no forked main context | read-only mechanical split review packet | completed |

Excluded context: full conversation history, hidden reasoning, persuasion briefs, and implementation conclusions. Reviewers were instructed not to modify files.

## Round 1 Outputs

### Finding 1: Missing End-to-End Watchdog Proof

- Severity: blocking
- Reviewer: Kuhn
- Evidence: only unit-level no-progress process and mapping tests existed; no CLI black-box test proved `results.json`, `events.jsonl`, `report.html`, and logs for the Terminal-Bench external runner path.
- Main-agent response: accept
- Resolution:
  - Added `int_021_terminal_bench_silent_official_runner_is_no_progress`.
  - Test drives the HarnessLab CLI path with fake `uvx`, forces a silent official runner, and asserts `execution/external_runner_no_progress`, `agent.termination_reason = no_progress`, `events.jsonl`, preserved stdout, report text, and exit code.

### Finding 2: Registry and Selector Drift

- Severity: blocking
- Reviewer: Kuhn
- Evidence: no dedicated selector or registry entry for the no-output watchdog behavior.
- Main-agent response: accept
- Resolution:
  - Added `C-SBOX-013` selector and registry entry for host no-output watchdog.
  - Added `INT-021` selector and registry entry for Terminal-Bench no-progress behavior.
  - `verify-test-registry` passed after updates.

### Finding 3: Fixed 300s Silence Cap Can False-Fail Valid Runs

- Severity: blocking
- Reviewer: Linnaeus
- Evidence: default watchdog capped silence at 300 seconds, assuming all valid Terminal-Bench phases emit logs within five minutes.
- Main-agent response: accept
- Resolution:
  - Removed fixed 300-second default cap.
  - Default watchdog is now `max(agent_timeout, test_timeout) + 60`, lower-bounded at 120 seconds and bounded only by the outer hard timeout.
  - Added bounded debug override `HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC`.
  - Updated operations docs and timeout tests.

### Finding 4: Schema/Docs Drift for New Variants

- Severity: non-blocking
- Reviewer: Linnaeus
- Evidence: `no_progress` and `external_runner_no_progress` were serialized but not documented.
- Main-agent response: accept
- Resolution:
  - Updated schema docs to list `no_progress` and `external_runner_no_progress`.
  - Updated architecture docs to include `no_progress`.

### Finding 5: Run-Health Gap for Repeated Runner Stalls

- Severity: residual risk
- Reviewer: Linnaeus
- Evidence: run-health only counted `agent_timeout` for early abort.
- Main-agent response: accept
- Resolution:
  - `external_runner_no_progress` now counts as an execution stall.
  - Added unit coverage for aborting after repeated external runner no-progress failures before any non-timeout completion.

## Round 2 Closure Output

Closure reviewer Dirac accepted closure of all previously blocking findings.

Residual low issue:

- `docs/architecture.md` used `signal` instead of the actual serialized `signaled`.
- Main-agent response: accept.
- Resolution: changed the example to `signaled`.

## Closure Status

Status: closed.

Accepted blocking findings were fixed and re-reviewed by a fresh internal reviewer. No remaining blocking findings.

## Post-Review Engineering Hygiene

After closure, the main agent found two touched Rust files exceeded the repository 500-line guidance:

- `crates/harnesslab-cli/src/runner/external.rs`
- `crates/harnesslab-cli/src/runner/monitor.rs`

Resolution:

- Split Terminal-Bench external runner implementation into `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`.
- Split monitor unit tests into `crates/harnesslab-cli/src/runner/monitor_tests.rs`.
- Rechecked touched Rust file lengths; all touched Rust files are under 500 lines.

## Round 3 Mechanical Split Review Output

Reviewer Dalton found no refactor-related issues.

Evidence:

- `external.rs` delegates Terminal-Bench work through the new private module without changing the parent-facing runner flow.
- `external_tests.rs` is rewired through `super::terminal_bench`.
- `monitor.rs` loads tests from `monitor_tests.rs` with the expected `super::*` scope.

Verification run by reviewer:

- `cargo test -p harnesslab-cli --lib --no-run`
- `cargo test -p harnesslab-cli --lib runner::external::tests::terminal_bench_result_maps_official_agent_timeout`
- `cargo test -p harnesslab-cli --lib runner::monitor::tests::monitor_aborts_after_timeout_threshold_before_any_success`
- `cargo test -p harnesslab-cli --lib runner::monitor::tests::monitor_aborts_after_external_runner_no_progress_threshold_before_any_success`

Round 3 status: approved. No blocking or non-blocking findings.
