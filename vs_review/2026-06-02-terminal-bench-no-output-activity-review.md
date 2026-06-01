# Terminal-Bench No-Output Activity Review

## Status

In progress.

## Review Target

Code implementation, test strategy, documentation, and operations guidance for the Terminal-Bench no-output watchdog fix.

## Round 1 Input

Objective: find blocking or high-impact issues in the current fix for Terminal-Bench no-output monitoring. The product goal is to run real Terminal-Bench tasks through HarnessLab without misclassifying long but active Docker setup/build stages as external runner stalls, while still killing genuinely silent or stalled official runners.

Target locations:

- `crates/harnesslab-infra/src/process.rs`
- `crates/harnesslab-infra/src/process_tests.rs`
- `crates/harnesslab-infra/src/docker.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `scripts/test-after-change.sh`
- `scripts/verify-terminal-bench-import-timeout-cleanup.sh`
- `docs/development-operations.md`
- `docs/architecture.md`
- `docs/mvp-development-spec.md`

Change introduction: `HostProcessExecutor` now supports `no_output_activity_patterns`. At the no-output watchdog boundary it checks whether the child process group contains a non-zombie command matching a configured pattern; if so, it refreshes the watchdog clock instead of returning `NoProgress`. Terminal-Bench passes Docker setup/build patterns such as `docker compose`, `docker-buildx`, `docker pull`, and `docker exec`. Existing callers use an empty pattern list.

Risks to challenge:

- Whether this preserves no-progress behavior for truly silent official runners.
- Whether pattern matching can accidentally mask a real stall until hard timeout.
- Whether process-group activity checks are reliable on macOS/Linux and avoid unrelated process matches.
- Whether the public `ExecSpec` change preserves existing call sites and Docker sandbox behavior.
- Whether the tests prove the real failure mode observed during a real `terminal-bench full` run.
- Whether logs and docs are clear enough for future incident triage.

Verification before review:

- `cargo test -p harnesslab-infra c_sbox_003_no_output_activity_pattern_defers_to_hard_timeout`
- `cargo test -p harnesslab-infra c_sbox_003_host_exec_no_output_timeout_is_structured`
- `cargo test -p harnesslab-cli --test terminal_bench_failure_contract`

Reviewer instructions: fresh session, no inherited main-agent context, read target files directly, do not modify files, cite evidence paths and line numbers where possible, and include concrete counterexamples for findings.

## Round 1 Launch Records

| Reviewer | Tool | Session | Freshness | Input |
|---|---|---|---|---|
| code-reviewer | `multi_agent_v1.spawn_agent` | `019e8541-a66c-7860-99b2-f683a29c3719` / Carver | `fork_context=false`; packet excluded main conversation and hidden reasoning | Round 1 code/architecture packet |
| test-engineer | `multi_agent_v1.spawn_agent` | `019e8541-a7d0-7320-a68c-225bffe7d1d9` / Mill | `fork_context=false`; packet excluded main conversation and hidden reasoning | Round 1 test strategy packet |

## Round 1 Reviewer Outputs

### Code Reviewer: Carver

Summary: the initial fix addressed the observed no-output symptom directionally, but it had three blocking design gaps.

Blocking findings:

- Activity deferral reset the watchdog clock to a full new no-output window. Counterexample: a Docker setup process is active at the watchdog boundary, exits one second later, and the official runner then hangs silently; the initial implementation would grant a complete extra timeout window instead of detecting the stall promptly.
- The `docker exec` activity pattern was too broad. Counterexample: a hung agent execution or interactive container command could keep a `docker exec` process alive and mask a real agent-stage stall as setup activity.
- Deferral was not observable enough. Counterexample: a later hard timeout would not explain whether no-output was deferred by Docker activity or whether the watchdog simply failed to fire.

Non-blocking risks:

- Process-group based matching should avoid arbitrary substring matches because shell command text can contain Docker-looking words without running Docker.

Required fixes:

- Re-probe activity on a short cadence without resetting the full no-output window.
- Remove `docker exec` from the default activity allowlist.
- Emit rate-limited events when the watchdog defers.

Missing tests:

- Activity disappearing should kill promptly.
- The allowlist should assert `docker exec` is absent.

Missing logs / observability:

- Deferral events should include the matched process identifier and pattern.

### Test Engineer: Mill

Summary: the initial test set did not yet prove the real Terminal-Bench failure mode and left several self-deception paths.

Blocking findings:

- Synthetic fake-Docker tests did not prove the real `tb run` + Docker setup/build path. Counterexample: a fake `uvx` script can pass while actual Terminal-Bench writes logs internally, spawns Docker differently, or remains silent during setup.
- Broad substring matching could produce false positives. Counterexample: `/bin/sh -c 'echo docker buildx; sleep 10'` contains the target words but is not Docker activity.
- There was no end-to-end proof that activity deferral still respects hard timeout and cleanup. Counterexample: a permanently active Docker command could hide a stuck official runner until an unbounded wait.
- New tests were not registered in the test registry, so future traceability could drift.
- Observability did not prove why the watchdog deferred.

Required fixes:

- Add a real Docker/Terminal-Bench integration check through the HarnessLab CLI.
- Add negative matching tests for shell text that only mentions Docker commands.
- Add hard-timeout and activity-disappearance tests.
- Register the tests and required artifacts in `tests/TEST_REGISTRY.toml`.

Missing logs / observability:

- Add structured deferral events for activity/progress.

## Main-Agent Responses

Accepted all Round 1 blocking findings.

- Activity deferral no longer resets the full no-output window. `HostProcessExecutor` now checks the hard timeout first, then probes activity on a short cadence. If process activity disappears and no progress file has advanced, the next probe returns `NoProgress` promptly. Evidence: `crates/harnesslab-infra/src/process.rs`; tests `C-SBOX-015` and `C-SBOX-016`.
- `docker exec` was removed from the Terminal-Bench default activity allowlist. Evidence: `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`; test `terminal_bench_no_output_activity_patterns_are_setup_scoped`.
- Activity matching moved into `process_activity.rs` and matches executable tokens/prefixes rather than arbitrary substrings. Evidence: tests `matches_docker_activity_on_executable_tokens` and `ignores_shell_text_that_mentions_docker_activity`.
- Terminal-Bench now registers official `run.log` as a progress file. This handles real `tb run` behavior where progress is written to the official log instead of child stdout/stderr. Evidence: `crates/harnesslab-infra/src/process_progress.rs`, `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`, test `C-SBOX-017`.
- Deferral events are emitted as `external_runner_activity`, including either matched process metadata or progress-file path. Evidence: `crates/harnesslab-infra/src/process.rs`, `docs/development-operations.md`.
- Added real integration validation `scripts/verify-terminal-bench-docker-activity-watchdog.sh`. It uses real `uvx --from terminal-bench tb run`, real Docker, a copied local `hello-world` task with a unique silent Dockerfile layer, a registered import-path agent, and formal HarnessLab CLI. It asserts success, `external_runner_activity`, no `external_runner_no_progress`, and no residual compose resources.
- Added registry entries and selectors for `C-SBOX-015`, `C-SBOX-016`, `C-SBOX-017`, `INT-029`, and `INT-030`.

Validation after fixes:

- `scripts/test-after-change.sh --select C-SBOX-017` passed.
- `scripts/test-after-change.sh --select INT-029` passed.
- `scripts/test-after-change.sh` passed.
- Coverage gate passed: lines 95.22% (6719/7056), branches 83.24% (591/710).

## Round 2 Closure Input

Objective: verify whether accepted blocking findings from Round 1 are actually closed. Focus only on the Terminal-Bench no-output watchdog implementation, tests, logs, docs, and real Docker validation.

Closure target locations:

- `crates/harnesslab-infra/src/process.rs`
- `crates/harnesslab-infra/src/process_activity.rs`
- `crates/harnesslab-infra/src/process_progress.rs`
- `crates/harnesslab-infra/src/process_tests.rs`
- `crates/harnesslab-infra/src/docker.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `crates/harnesslab-cli/tests/support/terminal_bench.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `scripts/verify-terminal-bench-docker-activity-watchdog.sh`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/development-operations.md`
- `docs/architecture.md`
- `docs/mvp-development-spec.md`

Closure questions:

- Is active Docker setup/build still credibly misclassified as no-progress before hard timeout?
- Can shell text that merely mentions Docker still mask a stall?
- Does excluding `docker exec` avoid masking hung agent execution?
- Does progress-file monitoring introduce a new broad masking issue?
- Does `INT-029` prove the real Terminal-Bench integration path rather than a fake bypass?
- Are logs, tests, docs, and registry entries sufficient for incident triage and drift prevention?

## Round 2 Launch Records

| Reviewer | Tool | Session | Freshness | Input |
|---|---|---|---|---|
| code-reviewer | `multi_agent_v1.spawn_agent` | `019e8565-2b52-7fb2-8f7f-1f238f3de5b0` / Plato | `fork_context=false`; packet excluded main conversation and hidden reasoning | Round 2 closure code/architecture packet |
| test-engineer | `multi_agent_v1.spawn_agent` | `019e8565-2cca-79a2-ae97-65afddfbe394` / Beauvoir | `fork_context=false`; packet excluded main conversation and hidden reasoning | Round 2 closure test strategy packet |

## Round 2 Reviewer Outputs

### Code Reviewer: Plato

Summary: closure pass. The accepted Round 1 blockers are closed: hard timeout is checked before deferral, activity defers on a short re-probe cadence, substring matching no longer masks shell text, `docker exec` is excluded, the real validation goes through `harnesslab -> uvx terminal-bench -> Docker`, and tests/docs/events are sufficient for primary incident triage.

Blocking findings:

- None.

Non-blocking risks:

- Progress-file deferral was broader than the documented "growth" contract because same-size non-empty mtime updates counted as progress. Counterexample: a helper could periodically rewrite a same-size status line to `run.log` and keep the watchdog deferring without forward progress.
- Activity probe failures are silently treated as no activity. Counterexample: if `ps` fails in an unusual target environment, operators do not get a warning that activity detection was unavailable.

Required fixes:

- None required for closure.

Missing tests:

- Optional negative contract for same-size `run.log` rewrite if growth-only semantics are intended.

Closure verdict: pass.

### Test Engineer: Beauvoir

Summary: closure pass. Activity matching, `docker exec` removal, activity disappearance handling, progress-file deferral, real `INT-029`, and registry integration are strong enough to close the accepted blocking findings.

Blocking findings:

- None.

Non-blocking risks:

- Progress-file deferral event content was not directly asserted.
- A combination case was missing where official results exist, progress keeps the no-output watchdog from firing, and hard timeout still wins.
- `INT-029` has environment timing risk because real Docker cold cache can be slow.

Required fixes:

- None required for closure.

Missing tests:

- Optional assertion for `progress file path=`.
- Optional combination integration test for progress deferral followed by hard timeout.

Closure verdict: pass.

## Round 2 Main-Agent Responses

- Accept Plato's progress-file broadness risk. Fixed `ProgressWatcher` so only file length growth, or creation with non-empty content, counts as progress. Same-size rewrites no longer defer. Added negative test `ignores_same_size_content_rewrite`.
- Accept Beauvoir's progress deferral observability test gap. Extended `C-SBOX-017` to assert `external_runner_activity` contains `progress file path=`.
- Accept Beauvoir's hard-timeout combination test gap. Added `INT-031` in `crates/harnesslab-cli/tests/terminal_bench_watchdog_contract.rs`: official runner writes a successful `results.json`, grows `run.log` to defer no-output, then hard timeout terminates the process and HarnessLab reports execution failure with `external_runner_timeout`, not `external_runner_no_progress`.
- Accept registry drift risk. Added `INT-031` to `scripts/test-after-change.sh` and `tests/TEST_REGISTRY.toml`; registry now reports 95 active tests.
- Defer Plato's `ps` warning suggestion as non-blocking. Existing behavior intentionally treats activity probe failure as absence of activity so no-progress/hard-timeout safety still applies; warning observability can be added with a narrow follow-up if the environment shows `ps` instability.

Post-response validation:

- `cargo test -p harnesslab-infra process_progress` passed.
- `scripts/test-after-change.sh --select C-SBOX-017` passed.
- `scripts/test-after-change.sh --select INT-031` passed.
- `scripts/verify-test-registry.sh` passed with 95 tests.
- `scripts/test-after-change.sh` passed:
  - Rust nextest: 274 passed.
  - Python unittest: 22 passed.
  - Real Terminal-Bench import timeout cleanup: passed.
  - Real Terminal-Bench Docker activity watchdog: passed.
  - Registry, traceability, and secret scan: passed.
  - Coverage: lines 95.22% (6709/7046), branches 83.33% (590/708).
  - New-file coverage: 2 new production Rust files present.

## Closure Status

Passed. Both fresh closure reviewers reported no blocking findings, optional test gaps were addressed, and full validation passed after the follow-up fixes.
