# Subagent VS Review: Terminal-Bench Agent Timeout Cleanup

- Created: 2026-06-02T05:05:00+08:00
- Updated: 2026-06-02T05:43:00+08:00
- Report schema: adversarial-v1
- Task: Prevent real Terminal-Bench runs with registered CLI agents from leaving orphan local agent subprocesses after timeout.
- Report path: `vs_review/2026-06-02-terminal-bench-agent-timeout-cleanup-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: Timeout Cleanup Implementation And Tests

### Review Input

#### Objective

Prevent real Terminal-Bench runs using `terminal_bench_agent_import_path = "harnesslab_tb_agent:HarnessLabCommandAgent"` from leaving orphan local CLI agent subprocesses when the registered agent command times out. HarnessLab must be able to stop early on engineering failures, clean runtime resources, and report meaningful benchmark failure classes.

#### Review Target

Implementation, tests, and operations documentation for registered Terminal-Bench agent timeout cleanup.

#### Target Locations

- `integrations/terminal_bench/harnesslab_tb_agent.py`
- `integrations/terminal_bench/harnesslab_tb_agent_test.py`
- `scripts/test-after-change.sh`
- `docs/development-operations.md`

#### Change Introduction

`run_registered_agent` now uses `subprocess.Popen(start_new_session=True)` instead of `subprocess.run`; it applies an internal timeout 5 seconds earlier than the official Terminal-Bench agent timeout, kills process groups and descendants on timeout, returns `FailureMode.AGENT_TIMEOUT`, and logs prompt plus partial stdout/stderr/error details. Tests add detached child process cleanup and `perform_task` timeout mapping/logging coverage. Docs record the real-run operational lesson.

#### Risk Focus

- Process tree cleanup correctness on macOS/Unix when child processes create their own process groups.
- Interaction with official Terminal-Bench `--global-agent-timeout-sec`.
- Preserving benchmark semantics without hiding infrastructure bugs.
- Accidental process-kill blast radius from pgid/pid signaling.
- Logging usefulness versus sensitive command/output exposure.
- Race or self-deception in orphan-process tests.
- Maintainability of a near-500-line Python adapter file.

#### Assumptions To Attack

- Collecting descendants with `ps -axo pid=,ppid=,pgid=` before signaling is sufficient.
- `timeout - 5` reliably beats Terminal-Bench outer timeout without changing semantics.
- `process_alive` using `os.kill(pid, 0)` is adequate for cleanup diagnostics.
- Partial stdout/stderr and prompt logging are safe and useful.
- One detached child process test covers Claude-like process trees well enough.
- No additional `uvx --from terminal-bench tb run` integration test is needed before the real full rerun.

#### Adversarial Lenses

- implementation
- failure
- concurrency
- process lifecycle
- test validity
- observability
- security
- maintenance

#### Verification Status

- `PYTHONPATH="$PWD/integrations/terminal_bench" uvx --from terminal-bench python -m unittest "$PWD/integrations/terminal_bench/harnesslab_tb_agent_test.py"` passed: 17 tests.
- `scripts/test-after-change.sh` passed: clippy, 255 nextest tests, Python unittest, registry, traceability, secret scan, coverage 95.09% lines / 83.48% branches.
- A real full benchmark run was intentionally terminated before this fix after observing an orphan `claude`/shell `find / -name maze_game.sh` process and one residual Terminal-Bench compose container.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Output sections: Summary, Blocking Findings, Non-blocking Risks, Required Fixes, Missing Tests, Missing Logs / Observability, Evidence.
- For every blocking or major finding include broken assumption, failure scenario, trigger condition, impact, and proof needed.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 10 minutes | 5 minutes once if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| code-reviewer / implementation-adversary | Timeout cleanup touches real process lifecycle, failure classification, signal blast radius, and logging. | implementation, process lifecycle, observability, security |
| test-engineer / test-validity-adversary | The bug was only visible under real run conditions, so tests must prove they cannot self-deceive about orphan cleanup. | test validity, race conditions, regression coverage |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| code-reviewer / implementation-adversary | `multi_agent_v1.spawn_agent` | `019e84f7-fd2a-7b00-a3bb-9bb6efae2c09` (`Peirce`) | spawn_agent result in current Codex thread | fork_context=false | Round 1 code-reviewer packet | main-agent history, reasoning, drafts, conclusions, full diff beyond target locations | yes |
| test-engineer / test-validity-adversary | `multi_agent_v1.spawn_agent` | `019e84f8-44ba-7692-8012-0c27f6a1addc` (`Meitner`) | spawn_agent result in current Codex thread | fork_context=false | Round 1 test-engineer packet | main-agent history, reasoning, drafts, conclusions, full diff beyond target locations | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| pending-code-reviewer | code-reviewer / implementation-adversary | 1 | `019e84f7-fd2a-7b00-a3bb-9bb6efae2c09` | pending | pending | waiting for reviewer output | pending |
| pending-test-engineer | test-engineer / test-validity-adversary | 1 | `019e84f8-44ba-7692-8012-0c27f6a1addc` | pending | pending | waiting for reviewer output | pending |

### Reviewer Outputs

Pending.

### Main Agent Response

Pending.

### Closure Status

- Blocking findings found: pending
- Accepted blocking findings fixed: pending
- Blocking re-review completed: pending
- Blocking re-review passed: pending
- Blocking re-review round links:
  - pending
- Blocking re-review launch records:
  - pending
- Rejected findings backed by evidence: pending
- Deferred findings documented: pending
- Blocked reason: pending
- Allowed to proceed: no

## Final Conclusion

Superseded by the amended closure record below.

## Amended Closure Record

### Round 1 Reviewer Outputs

#### code-reviewer / implementation-adversary

Blocking findings accepted:

- Reparented or double-forked descendants could survive a one-shot PPID-tree cleanup.
- Subtracting 5 seconds from the configured agent timeout changed benchmark semantics.
- Timeout with incomplete cleanup was collapsed into `AGENT_TIMEOUT`, hiding infrastructure failures.

Non-blocking risks accepted or noted:

- Raw prompt and partial stdout/stderr increase diagnostic artifact sensitivity.
- Zombie processes need to be treated as not-running for cleanup diagnostics.
- The Python adapter was close to the 500-line project limit and mixed process control with adapter logic.

#### test-engineer / test-validity-adversary

Blocking findings accepted:

- Unit tests did not exercise the real `tb run` import-path adapter path.
- Timeout-budget behavior was under-tested.

Non-blocking risks accepted:

- Detached child tests needed stronger process readiness checks.
- Argument and file input timeout paths needed coverage.
- Realistic multi-process CLI-agent shapes needed stronger coverage.

### Main Agent Response To Round 1

Accepted and implemented:

- Split process lifecycle code into `integrations/terminal_bench/harnesslab_tb_process.py`.
- Removed the early `timeout - 5` semantic shortcut. The adapter now enforces the full configured timeout.
- Added `terminal_bench_official_agent_timeout()` so official Terminal-Bench import-path runs receive a 30-second cleanup grace while `HARNESSLAB_AGENT_TIMEOUT_SEC` remains the user-configured budget.
- Added cleanup result classification: cleanup success maps to `AGENT_TIMEOUT`; cleanup failure maps to `UNKNOWN_AGENT_ERROR` in the adapter.
- Added Python tests for full timeout-budget semantics, detached descendants, reparented descendants, argument timeout, file timeout, and cleanup-failure classification.
- Added Rust tests for official cleanup grace and import-agent timeout env behavior.
- Added a real validation script: `scripts/verify-terminal-bench-import-timeout-cleanup.sh`.

Validation after Round 1 fixes:

- Python adapter tests: 22 passed.
- `cargo fmt --all --check`: passed.
- Targeted Rust tests for cleanup grace/import env: passed.
- Real validation script initially exposed a new result-classification bug: official Terminal-Bench reported `parse_error` after the adapter had already timed out cleanly, and HarnessLab misclassified the task as `test_failed`.

### Round 2 Reviewer Outputs

#### code-reviewer / closure-adversary

Blocking finding accepted:

- Reparented detached subprocess cleanup still relied on the run token appearing in argv. Production only guarantees the token is exported in the environment.

Non-blocking risks accepted:

- `int_026` and `int_028` still had order-sensitive `warnings[0]` assertions.
- Cleanup-failure evidence was still mostly in raw adapter logs.

#### test-engineer / closure-adversary

Blocking finding accepted:

- Real import-path timeout cleanup validation was still manual-only and not part of the required gate.

Non-blocking risks accepted:

- The real script needed artifact-level proof of import path, timeout grace, events, and cleanup success.
- Rust result parsing needed tests for false-positive prevention.

### Main Agent Response To Round 2

Accepted and implemented:

- Extended process discovery to use `ps eww` and find descendants that inherit `HARNESSLAB_AGENT_RUN_TOKEN` through the environment, not only through argv text.
- Changed the reparented-child Python test so the daemon child inherits env token only and does not receive token in argv.
- Changed the real validation script so the spawned child no longer receives the token in argv.
- Added script assertions for:
  - `--agent-import-path`
  - `harnesslab_tb_agent:HarnessLabCommandAgent`
  - `--global-agent-timeout-sec 33`
  - `external_runner_configured`
  - `process_timeout_sec=606`
  - `agent_error.log` contains `agent command timed out;`
  - `agent_error.log` contains `succeeded=True`
  - no marker process remains
  - no compose container/network remains
- Scoped adapter-timeout result override to the current official result directory instead of the whole attempt tree.
- Added Rust tests that official failure mode wins over adapter timeout logs and stale adapter logs do not override `parse_error`.
- Changed `int_024`, `int_026`, and `int_028` warning assertions from vector-index checks to membership checks.
- Added `scripts/verify-terminal-bench-import-timeout-cleanup.sh` to `scripts/test-after-change.sh`.

Validation after Round 2 fixes:

- Python adapter tests: 22 passed.
- Targeted Rust tests: 6 passed.
- `scripts/verify-terminal-bench-import-timeout-cleanup.sh`: passed using real HarnessLab CLI, real Terminal-Bench, Docker, import-path adapter, env-only child token discovery, and residue checks.
- `scripts/test-after-change.sh`: passed end-to-end:
  - 260 Rust tests passed.
  - 22 Python tests passed.
  - real Terminal-Bench import timeout cleanup script passed.
  - registry check passed.
  - traceability generated.
  - secret scan passed.
  - coverage passed at 95.09% lines and 83.33% branches.

Follow-up test stability fix:

- During a final full-gate rerun, coverage execution exposed that `int_027_terminal_bench_repeated_no_progress_aborts_run` still counted writes to a marker file created by a fake runner process that is intentionally killed by the no-output watchdog.
- The test now asserts HarnessLab-owned outputs instead: execution failure summary, interrupted summary, run-health no-progress/stall counts, one `run_health_aborted` task, five `external_runner_no_progress` tasks, and expected run events.
- Narrow fresh test review was launched for this test-only change.
- Reviewer `019e8528-d0b1-7e71-8742-e9808e49d6f1` (`Anscombe`) completed with no blocking findings and confirmed the change preserves the repeated no-progress abort contract while removing a flaky killed-process side effect.
- Validation after this fix:
  - `cargo fmt --all --check`: passed.
  - `cargo nextest run -p harnesslab-cli int_027_terminal_bench_repeated_no_progress_aborts_run`: passed.
  - `cargo +nightly-2026-05-26 llvm-cov test --workspace --all-features --exclude xtask --branch --no-report --test terminal_bench_failure_contract int_027_terminal_bench_repeated_no_progress_aborts_run`: passed.
  - `scripts/test-after-change.sh`: passed end-to-end again with 260 Rust tests, 22 Python tests, real import-path cleanup, registry, traceability, secret scan, and coverage at 95.09% lines / 83.33% branches.

### Round 3 Closure Review

Reviewer launch records:

| Reviewer | Internal Mechanism | Session / Job ID | Context Forked | Read-only | Status |
|---|---|---|---|---|---|
| code-reviewer / closure-adversary | `multi_agent_v1.spawn_agent` | `019e851c-1166-7342-bf2e-2b2602f401aa` (`Cicero`) | false | yes | completed |
| test-engineer / closure-adversary | `multi_agent_v1.spawn_agent` | `019e851c-a836-7c91-b41e-bb521f885564` (`Pascal`) | false | yes | completed |

#### code-reviewer / closure-adversary output

Summary: approved. No blocking correctness gaps found. The accepted blocking findings are fixed in code, covered by automated tests, and exercised by the real import-path validation script wired into the required gate. Recommendation: proceed with the full real `claude-ds` Terminal-Bench run.

Blocking findings: none.

Non-blocking risks:

- The review artifact needed final status updates before closure.

Evidence cited by reviewer:

- Env-only descendant discovery uses a per-run token injected through environment and `ps eww` token matching.
- Signal fanout skips the current process group.
- Real validation is in `scripts/test-after-change.sh`.
- Terminal-Bench result parsing preserves official failure modes and scopes fallback timeout log scanning to the current official result directory.
- Warning assertions use membership rather than vector order.
- Reviewer reran targeted Python tests, targeted Terminal-Bench failure contracts, the real import-timeout cleanup script, and Python compilation.

#### test-engineer / closure-adversary output

Summary: approved. No blocking test-adequacy findings remain. Accepted blockers are closed in code and in the automated slow gate.

Blocking findings: none.

Non-blocking risks:

- `scripts/test-after-change.sh` is intentionally environment-heavy because it now requires Docker, local benchmark data, `uvx --from terminal-bench`, pinned Rust tooling, and coverage tooling.
- Cleanup success is still evidenced mainly through `agent_error.log` strings and post-run residue checks rather than a structured cleanup artifact.

Evidence cited by reviewer:

- Real import-path timeout cleanup script is part of `scripts/test-after-change.sh`.
- Python cleanup discovers env-token descendants through `ps eww`.
- Reparented-child test no longer passes token in argv.
- Result classification tests cover official failure precedence and stale-log isolation.
- Targeted warning assertions use membership checks.

Closure status:

- Blocking findings found in previous rounds: yes.
- Accepted blocking findings fixed: yes.
- Required re-review launched: yes.
- Required re-review completed: yes.
- Blocking re-review passed: yes.
- Allowed to proceed to full real bench: yes.
