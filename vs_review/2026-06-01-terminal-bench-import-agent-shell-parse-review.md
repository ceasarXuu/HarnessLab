# Terminal-Bench Import Agent Shell Parse Review

## Review Target

- Objective: prevent HarnessLab's Terminal-Bench import agent bridge from sending invalid natural-language model output into the task tmux shell, where it can produce shell syntax errors and wait until timeout without signaling completion.
- Target files:
  - `integrations/terminal_bench/harnesslab_tb_agent.py`
  - `integrations/terminal_bench/harnesslab_tb_agent_test.py`

## Review Input Packet

Reviewers should challenge:

- whether `/bin/sh -n` is an appropriate non-executing guard before sending model output to Terminal-Bench tmux
- whether invalid natural-language output is classified as `FailureMode.PARSE_ERROR` without hiding agent failures
- whether valid plain shell and fenced shell output still execute normally
- whether the new test captures the observed real-run failure mode: `All 11 tests passed (file existence check + 10 maze content checks).`
- whether logs preserve enough evidence to diagnose parse failures

Verification state before review:

- `uvx --from terminal-bench python integrations/terminal_bench/harnesslab_tb_agent_test.py` passed: 9 tests.

## Reviewer Launch Records

| Round | Reviewer | Agent id | Role | Context | Input | Status |
|---|---|---:|---|---|---|---|
| 1 | Hooke | `019e824c-90ab-7e63-b5a4-5d615d039282` | code-reviewer | fresh, no forked main context | read-only adversarial code review packet | completed |
| 1 | Newton | `019e824c-da12-7080-9f5b-1f17b669da71` | test-engineer | fresh, no forked main context | read-only adversarial test review packet | completed |
| 2 | Euler | `019e8256-67c4-7523-95f7-02431d6d226d` | code-reviewer | fresh, no forked main context | read-only closure review packet | completed |
| 3 | Lagrange | `019e825f-1aac-7530-81a6-d6fb12abbafc` | code-reviewer | fresh, no forked main context | read-only second closure review packet | completed |
| 4 | Pauli | `019e8265-fa36-73a1-bc84-57cccc38e4c2` | code-reviewer | fresh, no forked main context | read-only third closure review packet | completed |
| 5 | Parfit | `019e826c-095f-7ca2-9d5b-1cc749a1a2bb` | test-engineer | fresh, no forked main context | read-only wrapper-test closure packet | completed |
| 6 | Russell | `019e826f-0c5f-7ee1-becd-3f01861e6c5a` | test-engineer | fresh, no forked main context | read-only tightened-wrapper-test closure packet | completed |
| 7 | Locke | `019e8273-d6f1-72e3-bd3e-f00f861ce8a3` | test-engineer | fresh, no forked main context | read-only quoted-shell-path closure packet | completed |

Excluded context: full conversation history, hidden reasoning, persuasion briefs, and implementation conclusions. Reviewers are instructed not to modify files.

## Round 1 Outputs

### Finding 1: Missing End-to-End Fenced Invalid Output Test

- Severity: blocking
- Reviewer: Newton
- Evidence: the original test covered plain invalid natural-language output, but not the full `perform_task()` path when the same invalid output is wrapped in shell fences.
- Main-agent response: accept.
- Resolution:
  - Added `test_perform_task_rejects_fenced_invalid_shell_before_tmux`.
  - The test asserts `FailureMode.PARSE_ERROR`, no tmux command, preserved `agent_output.txt`, preserved extracted `container_script.sh`, and `script_syntax_error.log`.

### Finding 2: Host Shell Syntax Check Does Not Match Task Shell

- Severity: blocking
- Reviewer: Hooke
- Evidence: the original guard used host `/bin/sh -n`, while Terminal-Bench executes in a tmux shell inside the task container. A script can parse on host `/bin/sh` but fail in the task shell.
- Main-agent response: accept.
- Resolution:
  - Added `resolve_execution_shell()` to read the tmux session `default-shell` from the task container.
  - Changed syntax validation to run inside the task container against that shell.
  - Changed final execution to send a wrapper that writes the script to a temporary container file and executes it with the same shell.
  - The wrapper avoids running the model output directly as tmux input and keeps Terminal-Bench's completion sentinel reachable.
  - Added `test_perform_task_uses_container_tmux_shell_for_syntax_check`.

### Non-Blocking Risks

- Valid-script coverage was shallow.
  - Main-agent response: accept.
  - Resolution: added `test_perform_task_sends_valid_multiline_script_to_terminal_session`.
- Parse-failure logs did not include shell identity or return code.
  - Main-agent response: accept.
  - Resolution: `script_syntax_error.log` now includes shell path and exit code; `execution_shell.txt` records the selected shell.
- `/bin/sh` checker availability was untested.
  - Main-agent response: accept.
  - Resolution: added `test_shell_syntax_error_reports_missing_shell`.

Validation after fixes:

- `uvx --from terminal-bench python integrations/terminal_bench/harnesslab_tb_agent_test.py` passed: 13 tests.

## Round 2 Closure Output

Closure reviewer Euler accepted closure of Finding 1 but rejected closure of Finding 2.

### Finding 3: Shell Lookup Was Global, Not Session-Targeted

- Severity: blocking
- Reviewer: Euler
- Evidence: the implementation called `tmux show-options -gv default-shell`, which reads the global tmux setting, while the closure note claimed it read the task session shell. A session-specific default-shell override could still differ.
- Main-agent response: accept.
- Resolution:
  - Changed shell lookup to `tmux show-options -t <session> -v default-shell`.
  - Updated the fake container test to assert the session-targeted lookup.

### Finding 4: Wrapper Should Not Depend On Pane Shell Grammar

- Severity: non-blocking risk
- Reviewer: Euler
- Evidence: the wrapper sent to tmux was POSIX shell syntax and could still be parsed by a non-POSIX pane shell before Terminal-Bench's sentinel is appended.
- Main-agent response: accept.
- Resolution:
  - Wrapped the container command as `/bin/sh -lc <quoted-wrapper>`.
  - The model script is still validated and executed by the resolved task shell inside that wrapper.

### Finding 5: Shell Resolution Fallback Was Silent

- Severity: observability risk
- Reviewer: Euler
- Evidence: fallback to `/bin/sh` did not explain why shell resolution failed.
- Main-agent response: accept.
- Resolution:
  - `resolve_execution_shell()` now returns the shell and an optional fallback reason.
  - `execution_shell_resolution.log` is written when fallback occurs.

Validation after Round 2 fixes:

- `uvx --from terminal-bench python integrations/terminal_bench/harnesslab_tb_agent_test.py` passed: 14 tests.

## Round 3 Closure Output

Closure reviewer Lagrange accepted the fenced-output coverage and wrapper direction, but rejected closure of Finding 2 for non-root Terminal-Bench sessions.

### Finding 6: Shell Lookup And Syntax Check Bypassed Session User

- Severity: blocking
- Reviewer: Lagrange
- Evidence: Terminal-Bench runs tmux operations with the configured session user, but the implementation called `container.exec_run(...)` without preserving `session._user`. Docker defaults that to root, which can miss a non-root tmux server and fall back to `/bin/sh`.
- Main-agent response: accept.
- Resolution:
  - Added `execution_user()` and `exec_in_session()`.
  - Shell lookup now calls `session.container.exec_run(..., user=session._user)`.
  - Container-side syntax validation also calls `session.container.exec_run(..., user=session._user)`.
  - Fallback logs now include the session user.
  - The fake container records exec users, and the regression test asserts all container-side execs use `agent-user`.

Validation after Round 3 fixes:

- `uvx --from terminal-bench python integrations/terminal_bench/harnesslab_tb_agent_test.py` passed: 14 tests.

## Round 4 Closure Output

Closure reviewer Pauli approved the accepted blocking fixes.

- Blocking findings: none.
- Main-agent response: accept closure.
- Evidence:
  - Shell lookup is session-targeted.
  - Shell lookup and container syntax validation preserve `session._user`.
  - Invalid model output is converted to `FailureMode.PARSE_ERROR` before tmux input.
  - The wrapper logs the selected shell, fallback reason, syntax error details, raw output, extracted script, and final command.
- Non-blocking risk: Pauli noted there was no direct successful-path test proving the final execution wrapper uses a non-default resolved shell.
- Main-agent response: accept.
- Resolution:
  - Added `test_perform_task_uses_resolved_shell_in_execution_wrapper`.

Validation after Round 4 fix:

- `uvx --from terminal-bench python integrations/terminal_bench/harnesslab_tb_agent_test.py` passed: 15 tests.
- `uvx --from mypy mypy --ignore-missing-imports integrations/terminal_bench/harnesslab_tb_agent.py integrations/terminal_bench/harnesslab_tb_agent_test.py` passed.
- `uvx ruff check integrations/terminal_bench/harnesslab_tb_agent.py integrations/terminal_bench/harnesslab_tb_agent_test.py` passed.

## Round 5 Closure Output

Closure reviewer Parfit approved the new wrapper test as sufficient, but flagged a non-blocking looseness: the test checked for the shell path as a substring rather than decoding the `/bin/sh -lc` payload.

- Blocking findings: none.
- Main-agent response: accept non-blocking hardening.
- Resolution:
  - Tightened `test_perform_task_uses_resolved_shell_in_execution_wrapper` to parse the outer command with `shlex.split()`.
  - The test now asserts the exact outer wrapper prefix `["/bin/sh", "-lc"]`.
  - The test now inspects the decoded payload and requires exactly one execution line shaped as the resolved shell plus `/tmp/harnesslab-agent-run-<hex>.sh`.

Validation after Round 5 hardening:

- `uvx --from terminal-bench python integrations/terminal_bench/harnesslab_tb_agent_test.py` passed: 15 tests.
- `uvx --from mypy mypy --ignore-missing-imports integrations/terminal_bench/harnesslab_tb_agent.py integrations/terminal_bench/harnesslab_tb_agent_test.py` passed.
- `uvx ruff check integrations/terminal_bench/harnesslab_tb_agent.py integrations/terminal_bench/harnesslab_tb_agent_test.py` passed.

## Round 6 Closure Output

Closure reviewer Russell approved the tightened wrapper test. Russell noted a non-blocking brittleness for shell paths containing spaces or quoting-sensitive characters.

- Blocking findings: none.
- Main-agent response: accept non-blocking hardening.
- Resolution:
  - Changed the wrapper execution test to use a fake shell path containing spaces.
  - The assertion now matches the `shlex.quote()`-escaped resolved shell path in the decoded payload.

Validation after Round 6 hardening:

- `uvx --from terminal-bench python integrations/terminal_bench/harnesslab_tb_agent_test.py` passed: 15 tests.
- `uvx --from mypy mypy --ignore-missing-imports integrations/terminal_bench/harnesslab_tb_agent.py integrations/terminal_bench/harnesslab_tb_agent_test.py` passed.
- `uvx ruff check integrations/terminal_bench/harnesslab_tb_agent.py integrations/terminal_bench/harnesslab_tb_agent_test.py` passed.

## Round 7 Closure Output

Closure reviewer Locke approved the quoted-shell-path hardening.

- Blocking findings: none.
- Main-agent response: accept closure.
- Evidence:
  - The test now uses a fake resolved shell path containing spaces.
  - The test parses the outer command with `shlex.split()`.
  - The test asserts the decoded payload contains exactly one execution line whose prefix is the `shlex.quote()`-escaped resolved shell.
- Non-blocking residual risk: Locke noted a single-quote shell path is not covered. The implementation uses `shlex.quote()`, and this is outside the reported regression class.
- Main-agent response: defer. Track as optional future hardening if shell path quoting logic is refactored.

Final validation:

- `uvx --from terminal-bench python integrations/terminal_bench/harnesslab_tb_agent_test.py` passed: 15 tests.
- `uvx --from mypy mypy --ignore-missing-imports integrations/terminal_bench/harnesslab_tb_agent.py integrations/terminal_bench/harnesslab_tb_agent_test.py` passed.
- `uvx ruff check integrations/terminal_bench/harnesslab_tb_agent.py integrations/terminal_bench/harnesslab_tb_agent_test.py` passed.
- `scripts/test-after-change.sh` passed, including 239 Rust tests, 15 Python bridge tests, registry check, traceability check, secret scan, line coverage 95.32%, and branch coverage 83.92%.

## Closure Status

Status: closed. No unresolved blocking findings.
