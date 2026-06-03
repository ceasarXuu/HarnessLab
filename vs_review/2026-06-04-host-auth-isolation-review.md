# Subagent VS Review: host auth isolation

- Created: 2026-06-04T01:12:00+0800
- Updated: 2026-06-04T01:55:00+0800
- Report schema: adversarial-v1
- Task: Implement Slice G host auth isolation.
- Report path: `vs_review/2026-06-04-host-auth-isolation-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: implementation and test review

### Review Input

#### Objective
Verify host agent execution no longer inherits the parent process environment unless explicitly allowed by profile auth policy, while Docker request environment behavior remains compatible.

#### Review Target
Explicit process environment support, resolved agent host environment, Docker env compatibility, and host auth black-box tests.

#### Target Locations
- `crates/harnesslab-infra/src/process.rs`
- `crates/harnesslab-infra/src/process_tests.rs`
- `crates/harnesslab-infra/tests/process_signal_contract.rs`
- `crates/harnesslab-infra/src/docker.rs`
- `crates/harnesslab-cli/src/runner/sandbox.rs`
- `crates/harnesslab-cli/tests/host_auth_contract.rs`
- Existing `ExecSpec` call sites in CLI runner/external/verifier/version probe paths.

#### Change Introduction
`ExecSpec` now carries `env_clear` and explicit `env_vars`. `HostProcessExecutor` applies `env_clear()` and then sets `env_vars`. Agent host execution sets `env_clear = true` and builds an explicit environment from a launch baseline plus task env plus `auth.inherit_env` only when `auth.inherit = true`. Docker request env remains compatible with previous `docker -e` behavior: task env vars are passed, and inherited env names are included when `auth.inherit = true`.

#### Acceptance
- Host task with ambient `HARNESSLAB_HOST_AUTH_SECRET=...` and `auth.inherit=false` cannot read the secret.
- Host task with `auth.inherit=true` and `inherit_env=["HARNESSLAB_HOST_AUTH_SECRET"]` can read it.
- Docker env request behavior remains unchanged.

#### Verification Status
- Passed: `cargo fmt --all && cargo fmt --all --check`
- Passed: `cargo test -p harnesslab-infra agt_reg_012 -- --nocapture`
- Passed: `cargo test -p harnesslab-cli agt_reg_012 -- --nocapture`
- Passed: `cargo test -p harnesslab-cli docker_request -- --nocapture`
- Passed: `cargo test -p harnesslab-cli run_005_docker_request_uses_run_network_and_task_sandbox_spec -- --nocapture`
- Passed: `git diff --check`
- Passed: touched implementation and new test files are under 500 lines.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Focus on environment leakage, auth semantics, Docker compatibility, test adequacy, and false positives.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | multi_agent_v1.spawn_agent code-reviewer | 019e8e40-ec7f-7ea3-9800-3f56c555e405 | spawn_agent result nickname=Nietzsche | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |
| test-validity-adversary | multi_agent_v1.spawn_agent test-engineer | 019e8e41-1b4d-7523-9fae-7593bad4603b | spawn_agent result nickname=Mendel | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |

### Reviewer Outputs

#### implementation-adversary

- Result: REQUEST CHANGES.
- Blocking finding: Docker-backed agent runs reused the agent `env_clear = true` policy for the host-side `docker exec` CLI wrapper. This could drop `DOCKER_HOST`, `DOCKER_CONTEXT`, `DOCKER_CONFIG`, TLS variables, and similar client configuration during exec even though sandbox creation still used the ambient Docker client environment.
- Test finding: Docker request assembly was covered, but host-side Docker exec wrapper environment behavior was not covered.

#### test-validity-adversary

- Result: PASS.
- Blocking findings: none.
- Evidence: host ambient-secret isolation and declared-env inheritance are black-box tested; infra-level `env_clear/env_vars` is directly tested; Docker request behavior remains covered by existing tests.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | Docker host-side exec inherited agent env clearing. | Docker create could succeed while docker exec fails because host Docker client config env was stripped. | blocking | accepted | `DockerCliProvider::exec` wrapped the scrubbed agent `ExecSpec` and forwarded `env_clear` to the host CLI. | Added `DockerCliProvider::exec_host_spec` that always uses `env_clear = false` and empty `env_vars` for the host Docker CLI wrapper. | Round 2 re-review required. |
| implementation-adversary | Missing regression for Docker exec wrapper env policy. | Request assembly tests do not cover host-side docker exec client environment. | medium | accepted | Existing tests checked `docker_create_request`, not wrapper `ExecSpec`. | Added `agt_reg_012_docker_exec_host_spec_preserves_docker_client_environment`, asserting Docker exec host spec does not clear env and does not carry agent-only env vars. | Round 2 re-review required. |
| test-validity-adversary | No blocking test gap. | n/a | n/a | accepted | Fresh test-validity review passed. | None required. | none |

## Round 2: Docker exec env closure

### Review Input

#### Objective
Close the Round 1 Docker runtime compatibility blocker.

#### Accepted Blockers To Close
1. Docker host-side `docker exec` CLI must not reuse agent env clearing.
2. A regression test must cover the wrapper env policy.

#### Updated Verification Status
- Passed: `cargo fmt --all && cargo fmt --all --check`
- Passed: `cargo test -p harnesslab-infra agt_reg_012 -- --nocapture`
- Passed: `cargo test -p harnesslab-cli agt_reg_012 -- --nocapture`
- Passed: `cargo test -p harnesslab-cli docker_request -- --nocapture`
- Passed: `cargo test -p harnesslab-cli run_005_docker_request_uses_run_network_and_task_sandbox_spec -- --nocapture`
- Passed: `git diff --check`

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| docker-exec-env-closure | multi_agent_v1.spawn_agent code-reviewer | 019e8e54-d94d-7522-a511-d287a8304116 | spawn_agent result nickname=James | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| docker-exec-env-closure | docker-exec-env-closure | 1 | 019e8e54-d94d-7522-a511-d287a8304116 | 3m | completed | reviewer returned REQUEST CHANGES | completed |

### Reviewer Outputs

#### docker-exec-env-closure

- Result: REQUEST CHANGES.
- Summary: code closure passed because `DockerCliProvider::exec` now wraps with `env_clear = false`, clears wrapper `env_vars`, and keeps `exec_host_spec` private.
- Blocking finding: the regression test targeted the private helper instead of the public `DockerCliProvider::exec` behavior under review. A fake docker test should prove the public wrapper path sees ambient Docker client env and does not see agent-only env.
- Non-blocking risks: none.
- Required fixes: replace or supplement the helper-only test with a public-path regression.
- Missing tests: Docker exec wrapper env behavior was not covered by CLI tests or the helper-only infra test.
- Missing logs / observability: none.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| docker-exec-env-closure | Regression test targeted private helper rather than public `DockerCliProvider::exec`. | A future change could bypass `exec_host_spec` or alter `exec` wiring while the helper-only test still passes. | blocking | accepted | Round 2 reviewer cited `crates/harnesslab-infra/src/docker_tests.rs` helper-only coverage. | Removed the helper-only regression and added `crates/harnesslab-infra/tests/docker_exec_env_contract.rs`, which runs `DockerCliProvider::exec` through a fake ambient `docker` on `PATH` and verifies `DOCKER_HOST` is visible while `HARNESSLAB_AGENT_ONLY` is absent. | Round 3 re-review required. |

### Updated Verification After Response

- Passed: `cargo fmt --all && cargo fmt --all --check`
- Passed: `cargo test -p harnesslab-infra agt_reg_012 -- --nocapture`
- Passed: `git diff --check`
- Passed: `wc -l crates/harnesslab-infra/tests/docker_exec_env_contract.rs crates/harnesslab-infra/src/docker_tests.rs crates/harnesslab-infra/src/docker.rs` confirmed all three files are under 500 lines.

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: no
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: Round 2 found the test was at the wrong behavioral boundary.
- Allowed to proceed: no

## Round 3: public Docker exec regression closure

### Review Input

#### Objective
Close the Round 2 blocker that the Docker exec regression must exercise the public `DockerCliProvider::exec` path.

#### Accepted Blockers To Close
1. Docker host-side `docker exec` CLI must keep ambient Docker client env.
2. Agent-only explicit env from the scrubbed agent spec must not leak into the host Docker CLI wrapper.
3. The regression must exercise `DockerCliProvider::exec`, not only the private helper.

#### Updated Verification Status
- Passed: `cargo fmt --all && cargo fmt --all --check`
- Passed: `cargo test -p harnesslab-infra agt_reg_012 -- --nocapture`
- Passed: `git diff --check`
- Passed: touched files are under 500 lines.

#### Target Locations
- `crates/harnesslab-infra/src/docker.rs`
- `crates/harnesslab-infra/tests/docker_exec_env_contract.rs`
- `crates/harnesslab-infra/src/docker_tests.rs`
- `vs_review/2026-06-04-host-auth-isolation-review.md`

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| public-docker-exec-regression-closure | multi_agent_v1.spawn_agent code-reviewer | 019e8e60-6ccb-7fe3-8a02-9e72cd5ddab1 | spawn_agent result nickname=Newton | fork_context=false | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| public-docker-exec-regression-closure | public-docker-exec-regression-closure | 1 | 019e8e60-6ccb-7fe3-8a02-9e72cd5ddab1 | 5m | completed | reviewer returned PASS | completed |

### Reviewer Outputs

#### public-docker-exec-regression-closure

- Result: PASS.
- Summary: `DockerCliProvider::exec` rebuilds the host-side Docker CLI wrapper with `env_clear = false` and empty `env_vars`, so ambient Docker client env is preserved and agent-only explicit env is not forwarded. The new regression drives public `DockerCliProvider::exec` through a fake `docker` on `PATH`, verifies `DOCKER_HOST` survives, and exits nonzero if `HARNESSLAB_AGENT_ONLY` leaks. The old helper-only regression is gone from `src/docker_tests.rs`.
- Blocking findings: none.
- Non-blocking risk: the report previously listed `cargo test -p harnesslab-infra docker_exec_public_wrapper_helper -- --nocapture`; this was misleading because the helper is meaningful only when invoked by the `agt_reg_012` parent test with `HARNESSLAB_RUN_DOCKER_EXEC_HELPER=1`.
- Required fixes: none.
- Missing tests: none.
- Missing logs / observability: none.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| public-docker-exec-regression-closure | Misleading verification line listed standalone helper test. | A reader could treat the helper invocation as public-path proof, although the parent `agt_reg_012` test is the real proof. | low | accepted | The helper returns immediately unless `HARNESSLAB_RUN_DOCKER_EXEC_HELPER=1`; the parent `agt_reg_012` test injects that env and fake `docker` path. | Removed the standalone helper command from verification sections; kept `cargo test -p harnesslab-infra agt_reg_012 -- --nocapture` as the authoritative Docker exec proof. | none |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

Passed. Slice G host auth isolation can proceed: host agents run with explicit environment policy, Docker exec host CLI environment is preserved, agent-only env does not leak into the Docker wrapper, and accepted blocking findings received fresh closure review.
