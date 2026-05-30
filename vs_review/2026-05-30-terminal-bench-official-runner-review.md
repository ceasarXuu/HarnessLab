# Terminal-Bench Official Runner Review

## Round 1 Input

Objective: HarnessLab must become a real harness evaluation lab, not demo/test-data. Terminal-Bench smoke/full should use official local dataset and official `tb run`, with run/report/replay artifacts.

Review target:

- `crates/harnesslab-core/src/benchmark.rs`
- `crates/harnesslab-adapters/src/terminal_bench.rs`
- `crates/harnesslab-adapters/src/registry.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/src/runner/external.rs`
- `crates/harnesslab-cli/tests/benchmark_contract.rs`
- `crates/harnesslab-cli/tests/external_smoke_contract.rs`
- `tests/TEST_REGISTRY.toml`

Risk focus:

- External runner contract and old snapshot compatibility.
- Terminal-Bench missing/corrupted/relative/real dataset handling.
- Official `tb run` path handling, run-id constraints, agent mapping labels, Docker/Colima behavior, outputs, failure classification, usage, report/replay artifacts.
- Avoiding self-deceptive tests.

Verification before review:

- `cargo test` passed.
- Real Terminal-Bench smoke run passed with official `tb run`, local `.benchmarks/terminal-bench/.../hello-world`, `tb-oracle`, score `1`, agent/verifier logs, official results, HarnessLab `results.json`, and `report.html`.

## Reviewer Launch Records

| Round | Role | Mechanism | Session | Context | Input | Read-only |
|---|---|---|---|---|---|---|
| 1 | code-reviewer | `multi_agent_v1.spawn_agent` | `019e7783-97ed-7ad1-a7e0-89cfa5a89471` / Lagrange | fresh session, `fork_context=false`; no main chat history | navigation packet in Round 1 Input | yes |

## Reviewer Outputs

### Round 1 / code-reviewer / Lagrange

Summary: request changes. The reviewer accepted the general external-runner direction but found that failure-source handling and dataset readiness could still produce self-deceptive run outcomes.

Blocking findings:

- `tb run` exit code `0` without an official `results.json` caused HarnessLab to treat the whole run as failed before writing a task-level result. Broken assumption: successful process exit implies benchmark result availability. Failure scenario: official runner exits cleanly after setup but produces no parseable score, leaving no usable `results.json`/`report.html` for the user.
- `tb run` nonzero exit with a valid official `results.json` was misclassified as execution failure. Broken assumption: process exit has higher authority than official benchmark output. Failure scenario: Terminal-Bench reports task failure via official results while the wrapper exits nonzero, making ranking/failure class wrong.

Non-blocking risks:

- Terminal-Bench dataset discovery used filesystem iteration order, so multiple local `terminal-bench-core-*` directories could pick different datasets on different machines.
- Smoke readiness could report ready even if the specific `hello-world` smoke task was missing.
- External-runner tests and report links did not yet prove the official result/lossy-log edge cases enough to guard against regression.

## Main-Agent Responses

- Accept blocking finding 1. Action: `execute_terminal_bench` now treats missing or unparsable official `results.json` as a task-level execution failure and still writes HarnessLab `results.json`, task `result.json`, event logs, and `report.html`. Regression test: `int_011_terminal_bench_zero_exit_without_results_stays_task_failure`.
- Accept blocking finding 2. Action: Terminal-Bench official `results.json` is now the score source of truth when present, even if `tb run` exits nonzero; nonzero process status is recorded but does not override a parseable official benchmark result. Regression test: `int_011_terminal_bench_nonzero_with_results_uses_benchmark_result`.
- Accept non-blocking dataset ordering risk. Action: Terminal-Bench data discovery now sorts candidate core directories before choosing the latest matching directory. Regression test: `c_bench_005_terminal_bench_chooses_core_dataset_deterministically`.
- Accept non-blocking smoke readiness risk. Action: Terminal-Bench smoke is ready only when the real `hello-world` task exists; partial local data without that task reports corrupted. Regression test: `c_bench_005_terminal_bench_smoke_requires_hello_world`.
- Accept test/report evidence gap. Action: added external smoke contracts for Terminal-Bench parse/failure cases, SWE-bench Pro official-evaluator contract paths, report links to agent/verifier logs, and `external_benchmark_runtime` traceability entries.

Validation after fixes:

- `cargo test -p harnesslab-cli --test external_smoke_contract`: 5 passed.
- `scripts/test-after-change.sh`: PASS, 148 tests passed, registry ok with 15 requirements / 72 tests, secret scan ok, coverage ok at 95.97% line and 81.66% branch, new-file coverage ok.

## Round 2 Closure Input

Objective: verify closure of accepted Round 1 blocking findings and the expanded external benchmark path before this work is committed.

Review target:

- `crates/harnesslab-cli/src/runner/external.rs`
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-adapters/src/terminal_bench.rs`
- `crates/harnesslab-adapters/src/swe_bench_pro.rs`
- `crates/harnesslab-core/src/benchmark.rs`
- `crates/harnesslab-cli/tests/external_smoke_contract.rs`
- `crates/harnesslab-cli/tests/benchmark_contract.rs`
- `tests/TEST_REGISTRY.toml`
- `tests/REQUIREMENTS.toml`
- `crates/harnesslab-report/src/lib.rs`

Change introduction: Terminal-Bench and SWE-bench Pro now plan real local benchmark data through `TaskPlan.external_runner`, route execution through external official runners, preserve official outputs, and map parse/no-diff/evaluator failures into task-level HarnessLab results and reports.

Risk focus:

- Round 1 blocking closure: official results source-of-truth and missing-results task failure.
- SWE-bench Pro official-evaluator contract, patch capture, no-diff classification, local Docker workspace preparation.
- Artifact/report completeness for debugging real failures.
- Test registry/coverage evidence not becoming self-referential.

Assumptions to attack:

- External process success/failure and official benchmark success/failure can be cleanly separated.
- Real local benchmark data paths are deterministic and replayable.
- Agent labels cannot silently route an unsupported harness through an official runner.
- Missing official outputs are visible as task failures rather than run orchestration crashes.
- New tests would fail if external runner behavior regressed.

Verification status:

- `cargo test -p harnesslab-cli --test external_smoke_contract`: 5 passed.
- `scripts/test-after-change.sh`: PASS, 148 tests passed, coverage 95.97% line / 81.66% branch.
- Manual real benchmark evidence exists under ignored `.benchmarks/_harnesslab-home-terminal-real/runs/` for Terminal-Bench and SWE-bench Pro smoke paths.

Reviewer instructions: fresh session, read targets directly, do not modify files, cite evidence paths and line numbers where possible, and focus on remaining blockers or evidence gaps. Do not inherit main-agent context beyond this packet.

## Round 2 Reviewer Launch Records

| Round | Role | Mechanism | Session | Context | Input | Read-only |
|---|---|---|---|---|---|---|
| 2 | code-reviewer | `multi_agent_v1.spawn_agent` | `019e779a-72ad-7273-a59e-7a9ee8db26b1` / Epicurus | fresh session, `fork_context=false`; no main chat history | navigation packet in Round 2 Closure Input | yes |

## Round 2 Reviewer Outputs

### Round 2 / code-reviewer / Epicurus

Summary: request changes. Terminal-Bench Round 1 closure looked closed, but SWE-bench Pro still had two failure-handling blockers.

Blocking findings:

- SWE-bench Pro setup failures still aborted the whole run instead of becoming task-level failures. Counterexample: fake Docker returning nonzero produced a run-level exit with only `run.json`, snapshots and `events.jsonl`, but no `results.json` or `report.html`.
- SWE-bench Pro evaluator success without official `eval_results.json` was misclassified as `benchmark/test_failed`. Counterexample: fake evaluator exited `0` and wrote no `eval_results.json`; HarnessLab emitted model-style benchmark failure instead of an evaluator/output failure.

Non-blocking risks:

- SWE external-runner observability was weaker than Terminal-Bench because metadata, workspace prep and evaluator phases did not emit phase events.
- Registry traceability was muddy because `INT-011` was titled as a Terminal-Bench test while also covering SWE runtime assertions.

Reviewer verification:

- `cargo test -p harnesslab-cli --test external_smoke_contract`: 5 passed.
- `cargo test -p harnesslab-cli --test benchmark_contract`: 5 passed.
- `cargo test -p harnesslab-adapters`: 19 passed.
- `cargo test -p harnesslab-core`: 34 passed.
- `cargo test -p harnesslab-report`: 2 passed.

## Round 2 Main-Agent Responses

- Accept blocking finding 1. Action: `swe_bench_pro::execute` now catches metadata extraction and workspace-prep failures, appends `external_runner_setup_failed`, synthesizes a task-level `execution/workspace_prep_failed` result, writes verifier stderr, and returns a normal `TaskAttemptResult` so the orchestrator still writes run-level `results.json` and `report.html`. Regression test: `int_011_swe_bench_pro_workspace_failure_stays_task_failure`.
- Accept blocking finding 2. Action: `run_evaluator` no longer collapses missing or invalid official `eval_results.json` to score `0.0`; it records the parse failure in verifier stderr/events and classifies the task as `execution/evaluator_error`. Regression test: `int_011_swe_bench_pro_missing_eval_results_is_evaluator_error`.
- Accept observability risk. Action: SWE external runner now emits `external_runner_started`, `external_runner_workspace_started`, `external_runner_evaluator_started`, `external_runner_setup_failed`, and `external_result_parse_failed` events.
- Accept registry traceability risk. Action: renamed `INT-011` to `External benchmark smoke contracts`.

Validation after fixes:

- `cargo test -p harnesslab-cli --test external_smoke_contract`: 7 passed.
- `scripts/test-after-change.sh`: PASS, 150 tests passed, registry ok with 15 requirements / 72 tests, secret scan ok, coverage ok at 95.95% line and 81.62% branch, new-file coverage ok.

## Round 3 Closure Input

Objective: verify closure of the Round 2 accepted blocking findings and confirm no high-impact blocker remains before commit.

Review target:

- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs`
- `crates/harnesslab-cli/tests/external_smoke_contract.rs`
- `tests/TEST_REGISTRY.toml`
- `vs_review/2026-05-30-terminal-bench-official-runner-review.md`

Change introduction: SWE-bench Pro external runner now converts metadata/workspace setup failures into task-level execution results, treats missing/malformed official evaluator output as `execution/evaluator_error`, emits phase events, and has regression coverage for both failure modes.

Risk focus:

- Round 2 blocking closure only.
- Whether task-level failure results are actually returned to the orchestrator and produce `results.json` / `report.html`.
- Whether missing official evaluator output can still be misreported as model benchmark failure.
- Whether the new tests are strong enough to fail on regression.

Verification status:

- `cargo test -p harnesslab-cli --test external_smoke_contract`: 7 passed.
- `scripts/test-after-change.sh`: PASS, 150 tests passed, coverage 95.95% line / 81.62% branch.

Reviewer instructions: fresh session, read targets directly, do not modify files, cite evidence paths and line numbers where possible. This is a closure review; focus on unresolved blockers, not style.

## Round 3 Reviewer Launch Records

| Round | Role | Mechanism | Session | Context | Input | Read-only |
|---|---|---|---|---|---|---|
| 3 | code-reviewer | `multi_agent_v1.spawn_agent` | `019e77a2-9fd4-7a90-965b-34a0d71eac0d` / Euclid | fresh session, `fork_context=false`; no main chat history | navigation packet in Round 3 Closure Input | yes |

## Round 3 Reviewer Outputs

### Round 3 / code-reviewer / Euclid

Summary: approve. No blocking issues remain for the two accepted Round 2 findings, and no new high-impact blocker was found in the closure scope.

Blocking findings:

- None.

Closure evidence:

- Round 2 setup-failure blocker is closed: setup-stage SWE failures return a normal `TaskAttemptResult` instead of bubbling an error, so the orchestrator can aggregate run-level `results.json` and `report.html`.
- Round 2 evaluator-output blocker is closed: missing or malformed official `eval_results.json` now sets a parse error, writes verifier stderr, emits `external_result_parse_failed`, and classifies as `execution/evaluator_error`.
- Registry traceability cleanup is in place.

Non-blocking risk:

- Metadata-extraction setup failure shared the same helper as workspace-prep failure, but direct coverage initially pinned only the workspace-prep branch.

Reviewer verification:

- `cargo test -p harnesslab-cli --test external_smoke_contract`: 7 passed.
- `cargo check -p harnesslab-cli --tests`: passed.

## Round 3 Main-Agent Responses

- Accept approval and close blocking review. No unresolved blocking findings remain.
- Accept non-blocking metadata coverage risk. Action: extended `int_011_swe_bench_pro_workspace_failure_stays_task_failure` to cover both `HARNESSLAB_FAKE_SWE_METADATA_FAIL` and `HARNESSLAB_FAKE_SWE_DOCKER_FAIL`, proving both setup-failure entry branches still produce task-level `execution/workspace_prep_failed`, run-level `results.json`, `report.html`, and `external_runner_setup_failed` events.

Final validation:

- `cargo test -p harnesslab-cli --test external_smoke_contract`: 7 passed.
- `scripts/test-after-change.sh`: PASS, 150 tests passed, registry ok with 15 requirements / 72 tests, secret scan ok, coverage ok at 96.02% line and 81.86% branch, new-file coverage ok.

Closure status: passed. No unresolved blocking findings remain.
