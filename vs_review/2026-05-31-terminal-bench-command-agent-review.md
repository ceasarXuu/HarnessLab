# Terminal-Bench Command Agent Review

## Review Input

- Objective: verify the implementation that lets a registered HarnessLab custom agent run Terminal-Bench through an import agent using the registered command.
- Review target: Terminal-Bench external runner command construction, Python import agent, and contract tests.
- Target locations:
  - `crates/harnesslab-cli/src/runner/external.rs`
  - `crates/harnesslab-cli/src/runner/external_tests.rs`
  - `crates/harnesslab-cli/tests/terminal_bench_contract.rs`
  - `integrations/terminal_bench/harnesslab_tb_agent.py`
- Change introduction: HarnessLab now exports registered agent profile fields to the official `tb run` process, supports a `terminal_bench_agent_pythonpath` label, and ships `HarnessLabCommandAgent` for Terminal-Bench import-path execution.
- Risk focus: env quoting, command injection surfaces, secret exposure in logs/snapshots, input-mode fidelity, timeout behavior, Terminal-Bench API compatibility, and whether tests can miss the registered-command path.
- Assumptions to attack: profile command is safe to run via shell, Python import agent can execute host-side CLI and send the produced script to the task session, no secrets are persisted, smoke success is not caused by bypassing HarnessLab CLI, and official Terminal-Bench result parsing still controls the score.
- Verification status: `cargo test --workspace` passed; real HarnessLab command run with registered `claude-ds` against Terminal-Bench smoke succeeded with score 1.0 and `report.html`.
- Reviewer instructions: use a fresh session, inspect the target files directly, do not modify files, cite file and line evidence where possible, and focus on falsifying the implementation rather than confirming it.

## Reviewer Launch Records

| Round | Reviewer role | Mechanism | Agent id | Context isolation | Input packet | Status |
| --- | --- | --- | --- | --- | --- | --- |
| 1 | code-reviewer | `multi_agent_v1.spawn_agent` | `019e7e50-2cf2-7020-87a7-b57c28a453de` | `fork_context=false`; only this review packet was sent | Review Input above | pending |
| 1 | test-engineer | `multi_agent_v1.spawn_agent` | `019e7e50-5694-7961-b57c-e2228d93b258` | `fork_context=false`; only this review packet was sent | Review Input above | pending |

## Timeout Records

| Round | Reviewer role | Timeout policy | Attempt | Status |
| --- | --- | --- | --- | --- |
| 1 | code-reviewer | normal, 10 minutes | primary | completed |
| 1 | test-engineer | normal, 10 minutes | primary | completed |

## Reviewer Outputs

### Round 1 Code Reviewer

Final verdict: fail.

Blocking findings:

- `HARNESSLAB_AGENT_TIMEOUT_SEC` used raw profile timeout instead of the effective benchmark/run agent timeout. This could make the import agent kill a registered CLI earlier than the outer `tb run` timeout.
- `tty` profiles were valid HarnessLab profiles but unsupported by the Python import agent, creating a runtime-only failure after `tb run` started.

Non-blocking risks:

- The import agent writes `prompt.txt`, `agent_output.txt`, and `container_script.sh`, expanding the artifact surface for raw model output.
- The Rust import-agent contract verified env propagation into fake `uvx`, but did not prove the Python import agent was imported and executed.

### Round 1 Test Engineer

Final verdict: fail.

Blocking findings:

- Existing contract tests could pass even if `HarnessLabCommandAgent` was never imported or executed, because fake `uvx` wrote `results.json` directly.
- Python adapter behavior was effectively untested despite containing the core MVP logic for subprocess input modes, errors, parse failures, and terminal dispatch.
- Repo evidence did not yet include a retained real run for the target profile path: registered custom agent plus `terminal_bench_agent_import_path` plus registered command.

Non-blocking risks:

- Malformed Terminal-Bench `results.json` was not covered.
- Env export coverage did not assert every exported profile field.
- Adapter `argument` and `file` input modes were untested.

## Main-Agent Responses

- Accepted: effective timeout mismatch. Fixed by passing the computed `agent_timeout` into `terminal_bench_agent_env`; added Rust unit coverage in `external_tests.rs`.
- Accepted: `tty` mismatch. Fixed by mapping `InputMode::Tty` to `stdin` for the Terminal-Bench import agent; added Rust unit coverage in `external_tests.rs`.
- Accepted: Python adapter untested. Added `integrations/terminal_bench/harnesslab_tb_agent_test.py` covering stdin, argument, file, nonzero, parse error, fenced extraction, and terminal dispatch.
- Accepted: real-run evidence incomplete. A new real HarnessLab CLI run must be executed after the fixes and recorded before closure.
- Deferred non-blocking: malformed `results.json` contract. Existing missing/nonzero result coverage protects current report semantics; malformed JSON coverage should be added when expanding Terminal-Bench parser hardening.
- Deferred non-blocking: raw prompt/model output artifact surface. Current MVP needs trajectory evidence; future redaction policy should classify these artifacts explicitly.

## Closure Status

Round 1 failed. Accepted blocking findings require a fresh follow-up review after fixes and real-run evidence are complete.

## Fix Evidence Before Round 2

- Rust targeted tests: `cargo test -p harnesslab-cli terminal_bench` passed.
- Python adapter tests: `PYTHONPATH=<repo-root>/integrations/terminal_bench uvx --from terminal-bench python -m unittest <repo-root>/integrations/terminal_bench/harnesslab_tb_agent_test.py` passed with 7 tests.
- Real run: `claude-ds-terminal-bench-smoke-20260531T140204507202Z` completed through HarnessLab CLI.
- Real run score: `summary.total_score = 1`, `tasks[0].benchmark_score = 1`, `tasks[0].state = success`.
- Real report: `.benchmarks/_harnesslab-home-terminal-real/runs/claude-ds-terminal-bench-smoke-20260531T140204507202Z/report.html`.
- Import-agent evidence: `agent/command.txt` uses `--agent-import-path 'harnesslab_tb_agent:HarnessLabCommandAgent'`, exports `HARNESSLAB_AGENT_COMMAND='claude-ds -p --bare --output-format text'`, and exports `HARNESSLAB_AGENT_TIMEOUT_SEC='3600'`.
- Adapter evidence: Terminal-Bench `agent-logs/agent_output.txt` contains the model-produced script and `agent-logs/container_script.sh` contains the executed script.

## Round 2 Review Input

- Objective: verify that Round 1 accepted blocking findings were fixed and the MVP threshold evidence is sufficient.
- Review target: same files as Round 1, plus the new Python test file.
- Target locations:
  - `crates/harnesslab-cli/src/runner/external.rs`
  - `crates/harnesslab-cli/src/runner/external_tests.rs`
  - `crates/harnesslab-cli/tests/terminal_bench_contract.rs`
  - `integrations/terminal_bench/harnesslab_tb_agent.py`
  - `integrations/terminal_bench/harnesslab_tb_agent_test.py`
- Specific closure checks:
  - Effective timeout is exported to the import agent.
  - `tty` maps to stdin or is otherwise handled before runtime failure.
  - Python adapter core behavior is directly tested.
  - Real run evidence shows the registered `claude-ds` command was used through HarnessLab CLI and Terminal-Bench import agent.

## Round 2 Reviewer Launch Records

| Round | Reviewer role | Mechanism | Agent id | Context isolation | Input packet | Status |
| --- | --- | --- | --- | --- | --- | --- |
| 2 | code-reviewer | `multi_agent_v1.spawn_agent` | `019e7e59-43f8-7732-a6cf-76912aa5cbfd` | `fork_context=false`; only Round 2 packet was sent | Round 2 Review Input above | pending |
| 2 | test-engineer | `multi_agent_v1.spawn_agent` | `019e7e59-7cbb-7b93-824b-750d60d9372a` | `fork_context=false`; only Round 2 packet was sent | Round 2 Review Input above | pending |

## Round 2 Timeout Records

| Round | Reviewer role | Timeout policy | Attempt | Status |
| --- | --- | --- | --- | --- |
| 2 | code-reviewer | normal, 10 minutes | primary | completed |
| 2 | test-engineer | normal, 10 minutes | primary | completed |

## Round 2 Reviewer Outputs

### Round 2 Code Reviewer

Final verdict: pass-with-risks.

Closure findings:

- Effective timeout export is closed. The runner computes `agent_timeout` and passes it to `terminal_bench_agent_env`; the retained run shows `HARNESSLAB_AGENT_TIMEOUT_SEC='3600'` and `--global-agent-timeout-sec 3600`.
- `tty` handling is closed. `InputMode::Tty` normalizes to `stdin` before reaching the Python import agent.
- Python adapter direct testing is closed. The adapter tests cover stdin, argument, file, output extraction, nonzero exit, parse error, and terminal dispatch.
- Real-run evidence is closed. The retained run shows the registered `claude-ds` command, the import path `harnesslab_tb_agent:HarnessLabCommandAgent`, Terminal-Bench `agent_name = "harnesslab-command"`, model output, executed script, and final success.

New blocking findings: none.

Residual risks:

- Low: attempt-level `agent/command.txt` is report-safe rather than guaranteed literal runtime command for every future case.
- Low: `tty` has Rust-side normalization coverage but no end-to-end `tty` profile run.
- Low: malformed `results.json` parser behavior is still not directly tested.

### Round 2 Test Engineer

Final verdict: pass-with-risks.

Closure findings:

- Import-agent execution evidence is closed through combined Rust env propagation tests, Python adapter tests, and retained real-run artifacts.
- Python adapter behavior is closed with 7 direct unit tests.
- Real-run evidence is closed with the retained `claude-ds-terminal-bench-smoke-20260531T140204507202Z` run.
- Timeout override and `tty` handling are closed with direct Rust unit coverage.

New blocking findings: none.

Residual risks:

- The Rust contract still does not instantiate the real Python import agent inside a fake Terminal-Bench process; current closure depends on combined evidence rather than one automated end-to-end contract.
- Python tests use a `FakeSession`, so they verify the `send_command` shape rather than a real `TmuxSession`.
- Prompt/model/script artifacts are still persisted by design.

## Round 2 Main-Agent Responses

- Accepted residual risks as non-blocking for this MVP threshold because both reviewers agreed no blocking issue remains.
- Added the Python adapter unit test command to `scripts/test-after-change.sh` so the key non-Rust path is part of the standard gate.
- Verified `scripts/test-after-change.sh` after adding the Python gate. It passed with 199 Rust tests, 7 Python adapter tests, coverage line 95.09%, and branch 81.61%.

## Final Closure Status

Closed as pass-with-risks. The requested MVP threshold is met by retained run `claude-ds-terminal-bench-smoke-20260531T140204507202Z`: registered `claude-ds` custom agent, HarnessLab formal CLI, real Terminal-Bench smoke execution, valid score 1.0, and HTML report.
