# Subagent VS Review: run_as enforcement

- Created: 2026-06-04T02:32:00+0800
- Updated: 2026-06-04T03:18:00+0800
- Report schema: adversarial-v1
- Task: Implement Slice H run user enforcement.
- Report path: `vs_review/2026-06-04-run-as-enforcement-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: implementation and test review

### Review Input

#### Objective
Verify `setup.run_as` is no longer silently ignored for host agent tasks while Docker/sandbox execution still materializes the requested user behavior.

#### Review Target
Run-as enforceability boundary, run precheck, doctor visibility, and regression tests.

#### Target Locations
- `crates/harnesslab-cli/src/agent_registry/materializer.rs`
- `crates/harnesslab-cli/src/agent_registry/mod.rs`
- `crates/harnesslab-cli/src/runner/external.rs`
- `crates/harnesslab-cli/src/doctor.rs`
- `crates/harnesslab-cli/src/doctor_run_as.rs`
- `crates/harnesslab-cli/tests/agent_registry_contract.rs`
- `crates/harnesslab-cli/tests/doctor_run_as_contract.rs`
- Existing Docker wrapping test in `crates/harnesslab-cli/src/runner/sandbox_tests.rs`

#### Change Introduction
`run_as_requires_sandbox` centralizes the rule that `setup.run_as = "root"` or `"harnesslab"` requires sandboxed execution, while `"current"` is host-compatible. The plan precheck now rejects host tasks with sandbox-required run_as before creating a run directory. Doctor emits a dedicated `agent.<name>.setup.run_as` warning for non-current values so users see that host tasks cannot switch users. Docker command rendering remains unchanged and continues to wrap sandboxed commands for `harnesslab`.

#### Acceptance
- Host benchmark profile with `run_as = "harnesslab"` fails before task execution and before run directory creation.
- Doctor warns for non-current `setup.run_as` and reports host support details.
- Doctor accepts `setup.run_as = "current"` for host compatibility.
- Docker task rendering still includes the requested user behavior.

#### Verification Status
- Passed: `cargo fmt --all && cargo fmt --all --check`
- Passed: `cargo test -p harnesslab-cli agt_reg_012 -- --nocapture`
- Passed: `cargo test -p harnesslab-cli docker_command_prefixes_builtin_and_custom_setup -- --nocapture`
- Passed: `cargo test -p harnesslab-cli run_005_docker_request_uses_run_network_and_task_sandbox_spec -- --nocapture`
- Passed: `git diff --check`
- Passed: touched implementation and test files are under 500 lines.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Focus on false positives for Terminal-Bench/SWE-bench external runner tasks, missed host paths, silent fallback, doctor severity, and test adequacy.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 8m | 4m bounded | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Run precheck changes execution eligibility and could block the wrong benchmark mode. | correctness, compatibility, permission boundary |
| test-validity-adversary | Acceptance depends on black-box failure timing and Docker regression coverage. | test adequacy, false confidence |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | multi_agent_v1.spawn_agent code-reviewer | 019e8e7e-0527-7533-9326-5d1c0f8705ea | spawn_agent result nickname=Dirac | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |
| test-validity-adversary | multi_agent_v1.spawn_agent test-engineer | 019e8e7e-43c8-7e53-8e1a-cce3b9d42993 | spawn_agent result nickname=Pasteur | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| implementation-adversary | implementation-adversary | 1 | 019e8e7e-0527-7533-9326-5d1c0f8705ea | 8m | completed | reviewer returned REQUEST CHANGES | completed |
| test-validity-adversary | test-validity-adversary | 1 | 019e8e7e-43c8-7e53-8e1a-cce3b9d42993 | 8m | completed | reviewer returned REQUEST CHANGES | completed |

### Reviewer Outputs

#### implementation-adversary

- Result: REQUEST CHANGES.
- Summary: the precheck is placed before run directory creation for fresh runs and is applied on resume/replay, but it originally classified enforceability by outer task image only.
- Blocking finding: external-runner subpaths can still execute agent work on the host even when the outer task image is sandbox-like. Terminal-Bench import-agent runs through a host process and the bridge falls back to current user on non-root hosts. SWE-bench Pro gold applies the patch on the host. Non-current `setup.run_as` would still be silently ignored in those paths.
- Non-blocking risk: doctor warning was generic and did not distinguish host-agent external subpaths from sandboxed agent execution.
- Required fixes: reject non-current run_as for Terminal-Bench import-path and SWE gold host-agent subpaths, or implement real user switching there. Update doctor to surface path-specific blockers.
- Missing tests: Terminal-Bench import-path and SWE gold run_as contracts.
- Missing logs / observability: doctor should expose why current user is required.

#### test-validity-adversary

- Result: REQUEST CHANGES.
- Summary: host-side precheck and doctor coverage were materially covered, but Docker run_as materialization was only asserted for the builtin setup branch.
- Blocking finding: Docker regression coverage did not prove `setup.run_as` stays materialized independently of setup preset. A regression that wraps only builtin profiles but skips custom/none setup could pass.
- Non-blocking risk: `root` run_as branch was unpinned.
- Required fixes: add Docker render regression for a non-builtin profile with `setup.run_as = "harnesslab"`.
- Missing tests: host rejection and doctor coverage for `setup.run_as = "root"`; Docker render test for non-builtin setup paths.
- Missing logs / observability: none.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | External-runner host-agent subpaths still ignored non-current run_as. | Outer task image can be sandbox-like while the agent substep is host-executed, so `run_as=harnesslab/root` remains unenforceable. | blocking | accepted | Terminal-Bench import path and SWE gold are host-agent subpaths. | `validate_run_as_for_plan` now checks actual agent execution reason: host task, Terminal-Bench import agent host path, and SWE-bench Pro gold host path. | Round 2 re-review required. |
| implementation-adversary | Doctor warning did not distinguish path-specific blockers. | Users could see only a generic sandbox warning for a profile that will always be blocked on a configured host-agent subpath. | non-blocking | accepted | `doctor_run_as` originally only emitted generic `sandbox_required`. | Doctor now reports `blocked_host_agent_paths` and uses error status/severity for configured Terminal-Bench import-path or SWE gold with non-current run_as. | Round 2 re-review required. |
| test-validity-adversary | Docker render coverage only asserted run_as for builtin setup. | Non-builtin or legacy setup could stop applying the requested-user wrapper without failing tests. | blocking | accepted | Existing `docker_command_prefixes_builtin_and_custom_setup` only asserted `runuser` on Codex branch. | Added assertions that custom/legacy setup render paths also include `runuser -u harnesslab`. | Round 2 re-review required. |
| test-validity-adversary | Root branch was unpinned. | `run_as=root` could be mistakenly treated as host-compatible. | non-blocking | accepted | Tests only covered `harnesslab` and `current`. | Added host-run rejection and doctor warning coverage for `run_as = "root"`. | Round 2 re-review required. |

### Updated Verification After Response

- Passed: `cargo fmt --all && cargo fmt --all --check`
- Passed: `cargo test -p harnesslab-cli agt_reg_012 -- --nocapture`
- Passed: `cargo test -p harnesslab-cli docker_command_prefixes_builtin_and_custom_setup -- --nocapture`
- Passed: `git diff --check`
- Passed: touched implementation and test files are at or under 500 lines.

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: no
- Blocking re-review passed: pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: accepted blocking findings need fresh closure review
- Allowed to proceed: no

## Round 2: closure review

### Review Input

#### Objective
Verify the Round 1 accepted blockers are closed.

#### Accepted Blockers To Close
1. Precheck must reject non-current `setup.run_as` for every actual host-agent path: host tasks, Terminal-Bench import-path, and SWE-bench Pro gold.
2. Doctor must expose path-specific host-agent blockers, not only a generic sandbox warning.
3. Docker render tests must prove requested-user wrapping survives non-builtin setup paths.
4. `root` run_as must be pinned as host-incompatible.

#### Updated Verification Status
- Passed: `cargo fmt --all && cargo fmt --all --check`
- Passed: `cargo test -p harnesslab-cli agt_reg_012 -- --nocapture`
- Passed: `cargo test -p harnesslab-cli docker_command_prefixes_builtin_and_custom_setup -- --nocapture`
- Passed: `git diff --check`
- Passed: touched files are at or under 500 lines.

#### Target Locations
- `crates/harnesslab-cli/src/runner/external.rs`
- `crates/harnesslab-cli/src/doctor_run_as.rs`
- `crates/harnesslab-cli/src/runner/sandbox_tests.rs`
- `crates/harnesslab-cli/tests/agent_registry_contract.rs`
- `crates/harnesslab-cli/tests/doctor_run_as_contract.rs`
- `crates/harnesslab-cli/tests/external_smoke_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_run_as_contract.rs`
- `crates/harnesslab-cli/tests/support/terminal_bench.rs`
- `crates/harnesslab-cli/tests/support/swe.rs`

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| run-as-closure | multi_agent_v1.spawn_agent code-reviewer | 019e8e94-eb2a-7160-b5a3-a1a9a2f372a4 | spawn_agent result nickname=Tesla | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| run-as-closure | run-as-closure | 1 | 019e8e94-eb2a-7160-b5a3-a1a9a2f372a4 | 8m | completed | reviewer returned PASS | completed |

### Reviewer Outputs

#### run-as-closure

- Result: PASS.
- Summary: Round 2 blockers are closed. The precheck rejects sandbox-required `setup.run_as` for host tasks, Terminal-Bench import-path, and SWE-bench Pro gold before run directory creation or replay execution. Terminal-Bench builtin and SWE non-gold sandbox paths are not falsely blocked. Doctor escalates configured host-agent subpaths to `error` and exposes `blocked_host_agent_paths`. Docker render coverage now proves `runuser -u harnesslab` survives non-builtin setup paths. `root` is pinned host-incompatible through the shared helper and tests.
- Blocking findings: none.
- Non-blocking risk: host-path label mapping is duplicated between precheck and doctor and could drift later.
- Required fixes: none.
- Missing tests: low-risk hardening only; no direct `root` contract for Terminal-Bench import-path or SWE gold, though shared gating covers it.
- Missing logs / observability: none.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| run-as-closure | Host-path label mapping is duplicated between runner precheck and doctor. | A future label/path rule could be updated in one place but not the other. | low | defer | Current behavior is covered by runner and doctor contracts, and no blocking issue remains. Refactoring after PASS would require another review cycle. | No code change in this slice after closure review. | Consider centralizing host-agent path classification in a later cleanup slice. |
| run-as-closure | No direct `root` contract for Terminal-Bench import-path or SWE gold. | Future branch-specific code could diverge from shared `run_as_requires_sandbox`. | low | defer | `root` is covered for host tasks and doctor, and import/gold paths share the same `run_as_requires_sandbox` gate. | No code change in this slice after closure review. | Add branch-specific root contracts if this area changes again. |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

Passed. Slice H run_as enforcement can proceed: non-current run_as is blocked for actual host-agent execution paths, sandbox/Docker run_as materialization remains covered, doctor surfaces path-specific blockers, and accepted blocking findings received fresh closure review.
