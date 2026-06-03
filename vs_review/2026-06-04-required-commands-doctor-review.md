# Subagent VS Review: setup required commands doctor check

- Created: 2026-06-04T00:35:00+0800
- Updated: 2026-06-04T00:48:00+0800
- Report schema: adversarial-v1
- Task: Implement Slice F setup.required_commands doctor check.
- Report path: `vs_review/2026-06-04-required-commands-doctor-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: implementation and test review

### Review Input

#### Objective
Verify that `setup.required_commands` is now a real doctor precheck with field-level diagnostics for command syntax, host availability, and setup-provider expectations.

#### Review Target
Required-command doctor implementation, validation reuse, and contract tests.

#### Target Locations
- `crates/harnesslab-core/src/agent_profile.rs`
- `crates/harnesslab-cli/src/lib.rs`
- `crates/harnesslab-cli/src/doctor.rs`
- `crates/harnesslab-cli/src/doctor_setup.rs`
- `crates/harnesslab-cli/tests/doctor_setup_contract.rs`
- `crates/harnesslab-cli/tests/doctor_contract.rs`

#### Change Introduction
The change adds a dedicated `agent.<name>.setup.required_commands` doctor check. It reports each declared command with `field`, `command`, `valid_name`, `host_available`, `setup_preset`, `provider`, `status`, and `message`. Invalid command syntax is an error. Missing commands with `setup.preset = "none"` are errors. Builtin setup-provider commands such as `claude` for `kind = "claude-code"` are ok and explain that builtin setup can provide them. Custom setup commands are treated as sandbox-dependent rather than false host errors. The core command-name validator is exported so doctor and profile validation share one rule.

#### Acceptance
- `required_commands = ["definitely-missing-harnesslab-required-command"]` with `preset = "none"` is an error.
- `required_commands = ["claude"]` with `kind = "claude-code"` and builtin setup explains that builtin setup can provide it.
- `required_commands = ["foo | bar"]` is an exact field error.

#### Verification Status
- Passed: `cargo fmt --all && cargo fmt --all --check`
- Passed: `cargo test -p harnesslab-cli agt_reg_011 -- --nocapture`
- Passed: `cargo test -p harnesslab-core agt_reg_001 -- --nocapture`
- Passed: `cargo test -p harnesslab-cli agt_reg_002 -- --nocapture`
- Passed: `cargo test -p harnesslab-cli agt_reg_010 -- --nocapture`
- Passed: `git diff --check`
- Passed: touched implementation and new test files are under 500 lines.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Focus on correctness, setup-provider semantics, test adequacy, false positives, and line-limit regressions.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | multi_agent_v1.spawn_agent code-reviewer | 019e8e21-6cec-7cf2-b8c9-3e0c427b9f6e | spawn_agent result nickname=Dewey | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |
| test-validity-adversary | multi_agent_v1.spawn_agent test-engineer | 019e8e21-9e4f-7390-9806-2cf7506f65b1 | spawn_agent result nickname=Mill | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |

### Reviewer Outputs

#### implementation-adversary

- Result: PASS.
- Blocking findings: none.
- Acceptance verdict: satisfied.
- Non-blocking finding: committed tests did not initially lock `preset = "custom"` sandbox-dependent classification.
- Manual verification by reviewer: with host path constrained, builtin `claude-code` + `required_commands = ["claude"]` reported `provider = "builtin_setup"` and `status = "ok"`; custom setup + missing command reported `provider = "custom_setup"` and `status = "ok"`.

#### test-validity-adversary

- Result: PASS.
- Blocking findings: none.
- Acceptance verdict: satisfied.
- Evidence: missing/no-setup, builtin provider, invalid command syntax, exact field details, local-machine robustness, and separate under-500-line test file were all verified.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | Custom setup classification was manually verified but not contract-locked. | Future changes could turn custom setup missing host command into a false host error while acceptance still appears covered. | low | accepted | Custom setup is a planned non-host provider state. | Added `agt_reg_011_doctor_marks_custom_setup_required_command_as_sandbox_dependent`, asserting `provider = "custom_setup"`, `status = "ok"`, `host_available = false`, and custom-setup message. | Covered by `cargo test -p harnesslab-cli agt_reg_011 -- --nocapture`. |
| implementation-adversary | Review artifact pending. | Code/test may pass while `/vs_review` remains open. | low | accepted | Report was still open. | Finalized this report. | none |
| test-validity-adversary | No missing-proof gap. | n/a | n/a | accepted | Fresh test-validity review passed. | None required. | none |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: n/a
- Blocking re-review completed: n/a
- Blocking re-review passed: n/a
- Rejected findings backed by evidence: none
- Deferred findings documented: none
- Blocked reason: none
- Allowed to proceed: yes

## Final Conclusion

Passed. Fresh internal implementation and test-validity reviews found no blocking issues. The only non-blocking coverage suggestion, custom setup sandbox-dependent classification, was codified with an additional `AGT-REG-011` contract test.
