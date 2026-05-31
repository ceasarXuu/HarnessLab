# Subagent VS Review: real benchmark run paths

- Created: 2026-05-31T16:21:50+0800
- Updated: 2026-05-31T18:40:00+0800
- Report schema: adversarial-v1
- Task: Run real Terminal-Bench and SWE-bench Pro paths instead of fake/oracle-only paths.
- Review mode: fresh internal subagents, no inherited main-agent context.
- Status: passed
- Allowed to proceed: yes

## Scope

The implementation moves HarnessLab from fake-only checks toward real benchmark execution:

- Terminal-Bench runs through official `uvx --from terminal-bench tb run`.
- SWE-bench Pro uses the official SWE image for workspace preparation, agent execution, and evaluation.
- Non-gold SWE agents run through the shared Docker sandbox instead of a host-only shortcut.
- Built-in Codex, Claude Code, and OpenCode profiles can provision missing CLIs inside sandbox images through `sandbox_setup_command`.
- Real benchmark artifacts remain under ignored `.benchmarks` / run output directories and are not tracked by git.

## Review Rounds

| Round | Reviewer | Session / Job ID | Result | Main Response |
|---|---|---|---|---|
| 1 | Code reviewer | `019e7d1f-eb50-7830-a4f4-081f4a0ac57a` | REQUEST CHANGES | Accepted all blockers. Replaced host SWE execution with Docker sandbox, accepted `{{instruction_file}}`, skipped generic bootstrap for external tasks, tightened Terminal-Bench model validation, persisted git diff status/stderr. |
| 2 | Code reviewer | `019e7d35-a30f-7720-a23c-c1f4e5feea11` | PASS | No blocking findings. Medium follow-ups were kept as tracked hardening items. |
| 3 | Code reviewer | `019e7d40-82ad-7ec0-b377-b52ab6f5eccc` | PASS with medium risks | Accepted observability/test concerns. Added sandbox manifest, stricter fake Docker checks, and real run evidence. |
| 4 | Code reviewer | `019e7d54-a9cf-70e0-9cd6-edaeafeda377` | REQUEST CHANGES | Accepted blockers. Added in-sandbox CLI provisioning, applied benchmark network overrides to `RunSpec`, gated Terminal-Bench `--model`, and added stricter contracts. |
| 5 | Code reviewer | `019e7d85-edda-7e60-b382-25fe44190ab8` | REQUEST CHANGES | Accepted both findings. This report was rewritten to remove stale claims and benchmark timeout overrides now affect actual execution. |
| 6 | Code reviewer | `019e7d93-c4af-7d42-8706-6b190de45d67` | PASS | No blocking findings. Reviewer verified accepted blockers are closed, quality gates pass, no Rust file exceeds 500 lines, and the review artifact does not overclaim real-run evidence. |

## Accepted Findings And Fixes

| Finding | Severity | Decision | Fix |
|---|---|---|---|
| Non-gold SWE ran on host, not sandbox. | High | Accept | `swe_bench_pro::agent` now uses `runner::sandbox::run_agent` with `jefzda/sweap-images:<dockerhub_tag>`. |
| File mode rejected documented `{{instruction_file}}`. | High | Accept | Profile validation accepts `{{instruction_file}}`; renderer replaces both file placeholders. |
| Generic patch bootstrap could dirty official SWE workspace. | Medium | Accept | `prepare_workspace()` skips generic bootstrap for external tasks. |
| Explicit Terminal-Bench `codex` / `opencode` labels could omit model. | Medium | Accept | Validation now requires `terminal_bench_model` or `model` for those built-ins. |
| Official SWE image lacked CLI executable for built-in agents. | High | Accept | Built-in profiles include `sandbox_setup_command`; sandbox render prefixes setup before the agent command. |
| Benchmark network override was ignored. | High | Accept | `RunSpec.execution.network` now uses `plan.run_config_overrides.network` before falling back to global config. |
| Terminal-Bench `--model` leaked to non-model built-ins. | Medium | Accept | `--model` is emitted only for `codex` and `opencode`. |
| Fake Docker tests could miss missing Codex setup. | High | Accept | Fake Docker can require Codex setup and fails if the generated command does not provision Codex first. |
| Review artifact contained stale evidence and wrong event names. | High | Accept | This file now records exact current event names: `external_runner_agent_sandbox_starting`, `external_runner_agent_sandbox_completed`, and `external_runner_agent_sandbox_failed`. |
| Timeout override was recorded but not operational. | Medium | Accept | Terminal-Bench command timeouts, Terminal-Bench process timeout, shared sandbox agent timeout, and SWE sandbox manifest now use `RunSpec.execution.timeout_sec` when present. |

## Current Evidence

### Automated Verification

- `scripts/test-after-change.sh`: PASS before the final timeout wiring change.
  - Tests: 193 passed.
  - Coverage: line 95.08% (5278/5551), branch 81.66% (423/518).
- Targeted verification after timeout wiring:
  - `cargo test -p harnesslab-cli --test terminal_bench_contract -- --nocapture`: 9 passed.
  - `cargo test -p harnesslab-cli --test external_smoke_contract -- --nocapture`: 10 passed.
  - `cargo test -p harnesslab-cli runner::sandbox::tests::agent_timeout_uses_task_override_marker -- --nocapture`: 1 passed.
- Hygiene checks already performed:
  - `git diff --check`: clean.
  - Rust file length check: no file over 500 lines after refactor.
  - `git ls-files .benchmarks | wc -l`: 0.
  - No lingering Docker container for the interrupted Codex SWE run label.

### Real Benchmark Runs

| Run | Benchmark | Agent | Outcome | Evidence |
|---|---|---|---|---|
| `swe-gold-swe-bench-pro-smoke-20260531T091356689048Z` | SWE-bench Pro smoke | gold patch profile | Completed, score 1.0 | Official workspace/evaluator path completed. |
| `tb-oracle-terminal-bench-smoke-20260531T09154650793Z` | Terminal-Bench smoke | oracle | Completed, score 1.0 | Official `tb run` produced resolved result. |
| `codex-default-terminal-bench-smoke-20260531T09085269503Z` | Terminal-Bench smoke | Codex built-in | Completed as benchmark failure, score 0 | Official `tb run` executed and returned `unknown_agent_error`; this is a real failed benchmark result, not a HarnessLab fake. |
| `codex-default-swe-bench-pro-smoke-20260531T094541861007Z` | SWE-bench Pro smoke | Codex default | Manually interrupted, no scored result | Real Codex executed inside the official SWE image for more than 20 minutes and modified the repository. This proves the missing-executable blocker was passed, but it is not a completed score. |

## Current Known Limits

- Non-oracle Codex SWE-bench Pro produced real in-container execution evidence, but no completed scored result because the run was manually interrupted after exceeding an interactive wait budget.
- Token/cost collection remains optional and depends on profile configuration.
- The report intentionally does not claim cross-run ranking; this milestone is single-run real benchmark execution.

## Closure

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Final closure review: Round 6, `019e7d93-c4af-7d42-8706-6b190de45d67`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Allowed to proceed: yes
