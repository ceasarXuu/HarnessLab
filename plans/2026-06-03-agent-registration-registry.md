# Agent Registration Registry Implementation Plan

- Date: 2026-06-03
- Status: completed
- Scope: agent profile registry, materialization, runner integration, user docs, test engineering

## Goal

Make agent registration the low-friction first user experience for HarnessLab. A user or another agent should be able to register a CLI harness such as `claude-ds`, inspect valid fields, run preflight validation, execute a benchmark through HarnessLab formal commands, and get reproducible run artifacts and an HTML report.

## Product Requirements

1. Agent profiles remain at full CLI harness granularity. Users compare named profiles, including profiles that differ only by setup, skills, tools, hooks, model labels, or command options.
2. Registration is a readable single TOML file under the HarnessLab home. No extra code should be required for common Codex, Claude Code, OpenCode, Pi Coding Agent, and custom CLI cases.
3. `harnesslab init` creates editable profile templates with semantic sections, not raw undocumented setup shell as the primary path.
4. `harnesslab agent schema` exposes field descriptions, accepted values, examples, required/default status, and warnings in JSON/text output.
5. `harnesslab doctor` surfaces field-level errors and materialization blockers before a run starts.
6. Non-default `skills`, `tools`, or `hooks` policies must either be materialized by the selected agent kind or fail before benchmark execution.
7. Runs persist a reproducible materialized runtime snapshot while public artifacts redact known secrets.
8. Terminal-Bench and SWE-bench Pro adapters consume the materialized profile instead of interpreting raw profile policy independently.
9. Reports show the effective setup, skills, tools, hooks, and profile configuration used for the run.
10. Test coverage must stay above 95% line coverage and include black-box CLI flows, real adapter verification scripts, traceability registry checks, and anti-self-deception gates.

## Implementation Slices

### 1. Core Profile Model

- Add semantic profile fields for `[setup]`, `[skills]`, `[tools]`, `[hooks]`.
- Preserve legacy profile compatibility for old `labels.sandbox_setup_command` snapshots.
- Validate field conflicts, invalid names, unsupported values, missing input variables, and policy overlap.
- Expose profile field reference metadata for CLI schema output and docs.

Evidence:

- `crates/harnesslab-core/src/agent_profile.rs`
- `crates/harnesslab-core/src/agent_profile_reference.rs`
- `crates/harnesslab-core/src/config.rs`
- `crates/harnesslab-core/src/config_tests.rs`

### 2. CLI Registry Experience

- Add `harnesslab agent schema`.
- Update `harnesslab init` templates and agent README.
- Add `doctor` materialization checks and field-level diagnostics.
- Keep profile editing file-based; no interactive wizard is required for M0.

Evidence:

- `crates/harnesslab-cli/src/agent_registry/`
- `crates/harnesslab-cli/src/app.rs`
- `crates/harnesslab-cli/src/doctor.rs`
- `crates/harnesslab-cli/tests/agent_registry_contract.rs`
- `crates/harnesslab-cli/tests/doctor_contract.rs`
- `crates/harnesslab-cli/tests/init_contract.rs`

### 3. Runtime Materialization And Snapshots

- Convert semantic profile configuration into a materialized runtime profile before planning and execution.
- Reject unsupported non-default capability policies before run directory creation.
- Persist redacted materialized snapshots in the run directory.
- Include materialized summaries in run metadata and reports.

Evidence:

- `crates/harnesslab-cli/src/agent_registry/materializer.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-cli/src/runner/sandbox_setup.rs`
- `crates/harnesslab-report/src/lib.rs`
- `crates/harnesslab-cli/tests/run_output_contract.rs`

### 4. Benchmark Adapter Integration

- Pass materialized profile setup to Terminal-Bench bridge environment.
- Preserve benchmark verdicts separately from HarnessLab execution failures.
- Classify setup failure, no progress, hard timeout, cleanup failure, official agent timeout, verifier timeout, and parse error distinctly.
- Keep SWE-bench Pro agent sandbox provisioning compatible with materialized profiles.

Evidence:

- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_env.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_result.rs`
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro/agent.rs`
- `integrations/terminal_bench/harnesslab_tb_agent.py`
- `integrations/terminal_bench/harnesslab_tb_agent_registry_test.py`

### 5. Watchdog And Observability Hardening

- Detect progress from official benchmark run logs even if the file was created before the no-output watchdog baseline.
- Continue distinguishing progress-file growth from bounded Docker setup/build activity.
- Emit events that explain last progress, last activity, current activity, and no-progress decisions.
- Avoid silent long waits when official runners stall.

Evidence:

- `crates/harnesslab-infra/src/process_progress.rs`
- `crates/harnesslab-infra/src/process_no_output.rs`
- `crates/harnesslab-cli/tests/terminal_bench_failure_contract.rs`
- `scripts/verify-terminal-bench-docker-activity-watchdog.sh`

### 6. Documentation And Playbook

- Provide a user-facing agent profile reference.
- Update the claude-ds Terminal-Bench playbook with the new registration style.
- Record development operation decisions for the stable gate runner, Docker setup, benchmark data, and real-run checks.

Evidence:

- `docs/agent-profile-reference.md`
- `docs/playbooks/terminal-bench-claude-ds.md`
- `docs/development-operations.md`
- `docs/prd.md`

### 7. Test Engineering

- Maintain requirement/test registry coverage for new agent profile requirements.
- Keep the full change gate as the authoritative local validation command.
- Ensure real verification scripts use HarnessLab formal commands and do not bypass the framework.
- Keep file-size constraints under the repository rule of 500 lines per Rust/Python code file.

Evidence:

- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `artifacts/test-traceability.json`
- `scripts/test-after-change.sh`
- `scripts/verify-terminal-bench-registered-setup.sh`
- `scripts/verify-terminal-bench-python-adapter.sh`
- `scripts/verify-terminal-bench-import-timeout-cleanup.sh`
- `scripts/verify-terminal-bench-docker-activity-watchdog.sh`

## Acceptance Criteria

| ID | Criterion | Evidence |
|---|---|---|
| AGT-REG-001 | Profile model supports setup, skills, tools, hooks and validates field-level errors. | `cargo test -p harnesslab-core agt_reg_001 -- --nocapture` |
| AGT-REG-002 | Doctor reports field-level diagnostics and materialization blockers. | `cargo test -p harnesslab-cli agt_reg_002 -- --nocapture` |
| AGT-REG-003 | Init and schema expose readable registration templates and field reference data. | `cargo test -p harnesslab-cli agt_reg_003 -- --nocapture` and `cargo test -p harnesslab-cli int_001_init_empty_home_creates_config_and_profiles -- --nocapture` |
| AGT-REG-004 | Runs persist redacted materialized runtime snapshots and reports include summaries. | `cargo test -p harnesslab-cli agt_reg_004 -- --nocapture` and `cargo test -p harnesslab-report rpt_001 -- --nocapture` |
| AGT-REG-005 | Terminal-Bench bridge executes materialized setup through HarnessLab. | `scripts/test-after-change.sh --select AGT-REG-005` |
| AGT-REG-006 | Unsupported non-default capability policies fail before run directory creation. | `cargo test -p harnesslab-cli agt_reg_006 -- --nocapture` |
| INT-REAL | Real Terminal-Bench cleanup/watchdog scripts pass through HarnessLab formal commands. | `scripts/test-after-change.sh` |
| COV | Coverage remains at least 95% line and 70% branch. | `scripts/test-after-change.sh` |
| TRACE | Requirement/test registry remains complete. | `scripts/verify-test-registry.sh` and `scripts/generate-test-traceability.sh` |

## Validation Result

Latest full local gate:

```bash
scripts/test-after-change.sh
```

Observed pass signals:

- `PASS terminal-bench import timeout cleanup`
- `PASS terminal-bench registered setup`
- `bridge setup command hash matches materialized snapshot`
- `PASS terminal-bench import success cleanup`
- `PASS terminal-bench docker activity watchdog`
- `PASS terminal-bench docker activity grace expiry`
- `registry ok: 21 requirements, 145 tests`
- `secret scan ok`
- `coverage ok: lines 95.47% (8028/8409), branches 83.22% (744/894), modules 2`
- `new-file coverage ok: 7 new production Rust files are present in coverage data`
- `PASS scripts/test-after-change.sh`

## Closure Result

- `subagent-vs-review` adversarial review completed in `vs_review/2026-06-03-agent-registration-registry-review.md`.
- Accepted blocking findings were fixed and re-reviewed by fresh internal subagents.
- Added regression coverage for materialized Terminal-Bench setup identity, setup secret redaction in public command snapshots, and setup-failure warning classification.
- Final full gate passed after the accepted fixes.
