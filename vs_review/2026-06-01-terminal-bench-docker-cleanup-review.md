# Subagent VS Review: Terminal-Bench Docker Cleanup

- Created: 2026-06-01T00:00:00+08:00
- Updated: 2026-06-01T00:00:00+08:00
- Report schema: adversarial-v1
- Task: Fix the root cause behind Terminal-Bench Docker network pool exhaustion by cleaning compose resources owned by HarnessLab runs.
- Report path: `vs_review/2026-06-01-terminal-bench-docker-cleanup-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: Root-Cause Fix Review

### Review Input

#### Objective
Stop Terminal-Bench runs from leaking Docker compose containers/networks that exhaust Docker address pools during or after real benchmark execution.

#### Review Target
Initial implementation of Terminal-Bench compose cleanup in:
- `crates/harnesslab-infra/src/docker.rs`
- `crates/harnesslab-infra/src/docker/compose.rs`
- `crates/harnesslab-infra/src/docker_tests.rs`
- `crates/harnesslab-cli/src/runner/external.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_cleanup.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_env.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_timeout.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`

#### Risk Focus
- Cleanup might guess Terminal-Bench compose project names and miss real leaked resources.
- Cleanup might delete unrelated Docker compose resources.
- Pre-task cleanup might degrade to a warning and still launch into dirty Docker state.
- Tests might prove only helper behavior, not the run path.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | multi_agent_v1.spawn_agent | 019e7f59-ab3e-7b73-b4ef-a16e8baf6ea3 | tool call | fork_context=false | Round 1 Root-Cause Fix Review | main-agent history and conclusions | yes |

### Reviewer Outputs

#### code-reviewer-output

Blocking findings:
- Cleanup was keyed off a locally guessed compose project name, not an authoritative Terminal-Bench/Docker identity.
- Known pre-task cleanup failures were downgraded to warnings, so execution could continue on dirty Docker state.

Missing tests / observability:
- No cleanup-path test proving `execute_terminal_bench()` calls cleanup before and after task execution.
- No real-label validation for compose project discovery.
- Docker cleanup tests only covered happy path.
- Cleanup events needed project/resource detail, not just a removed count.

### Main Agent Response

| Finding | Decision | Fix / Evidence |
|---|---|---|
| Guessed compose project names | accepted | Replaced guessed project deletion with Docker-label discovery via `compose_projects_matching`, exact project deletion via `cleanup_compose_projects`, and snapshot persistence in `terminal-bench-compose-projects.json`. |
| Pre-task cleanup warning-only path | accepted | `cleanup_task_resources(..., required=true)` now returns an error before `uvx` launch if Docker label scan/cleanup fails. |
| Missing run-path coverage | accepted | Added `terminal_bench_cleanup_contract.rs` covering actual label discovery, post-task cleanup, event output, and snapshot persistence. |
| Missing cleanup failure coverage | accepted | Added Docker provider error-path tests and a contract test proving pre-task cleanup failure blocks agent launch. |
| Weak observability | accepted | Cleanup events now include token, matched exact projects, container/network counts, and removed resources. |

## Round 2: Safety Boundary Review

### Review Input

Review focused on whether run-level cleanup of historical sibling runs was safe and whether it could delete unrelated compose resources.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | multi_agent_v1.spawn_agent | 019e7f65-2aa7-77b1-abbd-6e4ef6cc1f94 | tool call | fork_context=false | Round 2 Safety Boundary Review | main-agent history and conclusions | yes |

### Reviewer Outputs

#### safety-code-reviewer-output

Blocking finding:
- Completed-sibling pre-run cleanup was still overbroad: it treated every sibling run directory with `results.json` as a Terminal-Bench cleanup token and matched Docker projects by substring.

Missing tests:
- No negative coverage proving non-Terminal-Bench sibling runs and near-collision labels are ignored.
- No contract test proving pre-task cleanup failure blocks launch.

### Main Agent Response

| Finding | Decision | Fix / Evidence |
|---|---|---|
| Overbroad sibling cleanup | accepted | Run-level cleanup now only replays exact compose project labels from sibling `terminal-bench-compose-projects.json` snapshots. It no longer uses sibling run IDs as Docker match tokens. |
| Missing negative sibling coverage | accepted | Added `cleanup_006_terminal_bench_pre_run_ignores_siblings_without_project_snapshot`. |
| Missing pre-task failure contract | accepted | Added `int_011_terminal_bench_pre_task_cleanup_failure_blocks_agent_launch`. |

## Round 3: Closure Review

### Review Input

Final closure review limited to the prior blocking findings: authoritative label discovery, exact-project replay, sibling cleanup scope, and pre-task failure behavior.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer | multi_agent_v1.spawn_agent | 019e7f6f-8fa7-7100-8cd4-3d2aa2b844db | tool call | fork_context=false | Round 3 Closure Review | main-agent history and conclusions | yes |

### Reviewer Outputs

#### closure-code-reviewer-output

Summary:
- APPROVE. No blocking or non-blocking findings in the requested closure scope.

Evidence:
- Per-task cleanup discovers compose projects from authoritative Docker labels and deletes exact projects.
- Run-level cleanup only replays sibling runs containing `terminal-bench-compose-projects.json`.
- Pre-task cleanup failure returns before `external_runner_started` and before `HostProcessExecutor::exec`.
- Targeted cleanup, failure, and exact-label tests cover the closure path.

## Verification

- `scripts/test-after-change.sh`: PASS
- Rust tests: 227/227 passed in nextest run
- Python adapter tests: 7/7 passed
- Coverage gate: lines 95.39% (6228/6529), branches 83.33% (505/606)
- New production Rust file coverage gate: PASS

## Final Conclusion

Passed. Terminal-Bench Docker cleanup is allowed to proceed.
