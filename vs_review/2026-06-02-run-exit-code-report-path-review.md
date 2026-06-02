# Subagent VS Review: run exit code and report path semantics

- Created: 2026-06-02T14:42:12+0800
- Updated: 2026-06-02T15:04:17+0800
- Report schema: adversarial-v1
- Task: Make HarnessLab real benchmark runs report valid experiment results without treating benchmark verdicts as command execution failures.
- Report path: `vs_review/2026-06-02-run-exit-code-report-path-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: implementation and validation review

### Review Input

#### Objective
Ensure a completed HarnessLab run with benchmark-level failures still produces a valid experiment result, exits successfully at the CLI command layer, includes a usable report path in machine-readable output, and preserves non-zero exits for true execution/run failures.

#### Review Target
Implementation, test plan, documentation, and command-output contract for run exit-code mapping and report path persistence.

#### Target Locations
- `crates/harnesslab-core/src/model.rs`
- `crates/harnesslab-core/src/model_tests.rs`
- `crates/harnesslab-cli/src/output.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/src/runner/run_output.rs`
- `crates/harnesslab-cli/src/runner_tests.rs`
- `crates/harnesslab-cli/tests/cli_contract.rs`
- `crates/harnesslab-cli/tests/external_smoke_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/architecture.md`
- `docs/mvp-development-spec.md`
- `docs/test-engineering.md`

#### Change Introduction
The implementation now treats benchmark verdicts (`benchmark_failure`, `partial_success`) as valid run outcomes with CLI exit code `0`. True run-level failure, execution failure, empty results, and interruption remain non-zero. JSON run output and `results.json` now carry `report_path`. Duplicated run-output emission in new run and replay paths was moved into `runner/run_output.rs`.

#### Risk Focus
- Benchmark failures might be accidentally hidden as success without enough task-level failure evidence.
- Execution failures or interrupted runs might incorrectly return `0`.
- Replay/new-run output paths might diverge or lose report path data.
- Tests might only validate fake benchmarks while missing Terminal-Bench and SWE-Bench Pro contracts.
- Test registry selectors might point at stale test names after renames.
- The documentation might describe a different exit-code contract than the implementation.

#### Assumptions To Attack
- The latest effective attempt per task is still used before exit-code derivation.
- `report_path` is available in both stdout JSON and persisted `results.json`.
- `benchmark_failure` and official Terminal-Bench `agent_timeout` are result verdicts, not command failures.
- SWE-Bench Pro no-diff is a benchmark verdict, while sandbox creation, git diff failure, and missing evaluator output remain execution failures.
- `run resume` and `run replay` still print stable output and do not require separate duplicated logic.
- All edited Rust code files remain at or below the 500-line project limit.

#### Adversarial Lenses
- implementation
- testing
- failure
- data
- maintenance
- observability

#### Verification Status
- `cargo fmt --all --check` passed.
- `scripts/test-after-change.sh --select ORCH-003` passed.
- `scripts/test-after-change.sh --select INT-003` passed.
- `scripts/test-after-change.sh --select INT-004` passed.
- `scripts/test-after-change.sh --select INT-022` passed.
- `scripts/test-after-change.sh --select INT-023` passed.
- `scripts/verify-test-registry.sh` passed before the last test-name-only edits and will be rerun before closure.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed before the last test-name-only edits and will be rerun before closure.
- `cargo test -p harnesslab-core --all-features` passed.
- `cargo test -p harnesslab-cli --all-features` passed after updating stale benchmark-exit expectations.
- Known unverified before closure: final clippy, final registry check, final build, review triage.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Focus on falsifying the implementation and validation claims above.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 10 minutes | one bounded 5 minute extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary / code-reviewer | The change touches run command semantics, result persistence, replay/new-run output flow, and multiple benchmark contract tests. | implementation correctness, failure mapping, test validity, maintenance |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary / code-reviewer | `multi_agent_v1.spawn_agent` role `code-reviewer` | `019e8712-6163-7180-90df-3c092bfb0e58` | spawn tool result in current Codex thread | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, hidden conversation, full diff persuasion brief | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| bohr-round-1 | implementation-adversary / code-reviewer | 1 | `019e8712-6163-7180-90df-3c092bfb0e58` | ~10 minutes | completed | reviewer returned findings | completed |

### Reviewer Outputs

#### bohr-round-1

##### Summary
No blocking runtime bug found. Reviewer confirmed the implemented path maps `benchmark_failure` and `partial_success` to CLI exit `0`, keeps `execution_failure` at `1`, run-level/empty-result failure at `3`, interruption at `130`, and persists/emits `report_path`.

##### Blocking Findings
- none

##### Non-blocking Risks
- [MEDIUM] Stdout JSON could make benchmark-level failure runs indistinguishable from full-pass runs if callers only read top-level `status`.
  - Broken assumption: `status = "success"` plus `report_path` is enough task-level failure evidence.
  - Failure scenario: automation treats a benchmark-failure run as a clean pass.
  - Trigger condition: completed run where exit code is `0` because failures are benchmark-level only.
  - Impact: silent false positives in machine integrations.
  - Proof needed: document command-health status or include summary/verdict fields in stdout JSON and test them.
- [MEDIUM] SWE-Bench Pro spec contradicted the implemented execution-vs-benchmark boundary for git-diff and missing-evaluator-output failures.
  - Broken assumption: docs match true execution failure behavior.
  - Failure scenario: future maintainer reclassifies adapter/workspace/evaluator-output failures as benchmark verdicts.
  - Trigger condition: adapter refactor driven from stale docs.
  - Impact: execution failures could incorrectly return `0`.
  - Proof needed: update SWE-Bench Pro spec and acceptance text.
- [MEDIUM] `report_path` coverage was narrow; only one fake-benchmark test asserted stdout/results parity.
  - Broken assumption: shared output helper alone is enough durable coverage.
  - Failure scenario: replay or external benchmark JSON drops `report_path`.
  - Trigger condition: future edits to replay wiring or external benchmark helpers.
  - Impact: machine clients lose direct report artifact pointer.
  - Proof needed: add replay, Terminal-Bench, and SWE-Bench assertions.

##### Required Fixes
- Update command-output contract docs for top-level `status`.
- Update SWE-Bench Pro spec so true execution failures remain documented as execution failures.
- Document `report_path` in stdout JSON and persisted `results.json`.

##### Missing Tests
- Add replay contract for stdout `report_path` / persisted `results.json` parity.
- Add Terminal-Bench and SWE-Bench assertions on stdout `report_path`.
- Add top-level JSON semantics coverage for benchmark-level failures.

##### Missing Logs / Observability
- `run_finished` did not record derived exit code, summary buckets, or `report_path`.

##### Evidence
- `crates/harnesslab-cli/src/output.rs`
- `crates/harnesslab-cli/src/runner/run_output.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/tests/cli_contract.rs`
- `crates/harnesslab-cli/tests/external_smoke_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `docs/mvp-development-spec.md`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| bohr-round-1 | Top-level JSON status is ambiguous | Automation may treat command `status = success` as all tasks passed. | non-blocking medium | accept | `RunOutput` previously exposed status/run/report only. | Added `exit_code`, `verdict`, `results_path`, and `summary` to `run --json`; added `run_output_contract.rs` assertions for benchmark failure. | none |
| bohr-round-1 | SWE-Bench Pro docs contradicted execution-vs-benchmark boundary | Future adapter work could classify HarnessLab plumbing failures as benchmark failures. | non-blocking medium | accept | Existing tests assert git-diff and missing eval output are execution failures. | Updated `docs/mvp-development-spec.md` failure table and patch adapter acceptance criteria. | none |
| bohr-round-1 | `report_path` coverage too narrow | Replay or external benchmark flows could lose JSON report path without tests catching it. | non-blocking medium | accept | Existing helper tests parsed `run_dir` and checked artifacts but not stdout parity. | Added replay, Terminal-Bench, and SWE-Bench assertions for stdout `report_path` / `results_path` and persisted `results.json.report_path`. | none |
| bohr-round-1 | `run_finished` lacked final outcome details | Postmortem from `events.jsonl` could not distinguish exit 0 with benchmark failures. | non-blocking medium | accept | Existing event message was only `run finished`. | Added `exit_code`, summary buckets, `total_score`, and `report_path` to `run_finished` event message; tested in `run_output_contract.rs`. | none |
| bohr-round-1 | No blocking findings | n/a | n/a | accept | Reviewer explicitly reported no blocking runtime bugs. | No blocking re-review required by the review protocol. | none |

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
- Deferred findings documented: n/a
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

Passed after accepting and fixing all non-blocking review findings. Final validation completed:

- `cargo test -p harnesslab-core --all-features`
- `cargo test -p harnesslab-cli --all-features`
- `scripts/verify-test-registry.sh`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- `cargo build --workspace --all-features`
- Rust file line cap check for `>500` lines
