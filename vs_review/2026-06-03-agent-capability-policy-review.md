# Subagent VS Review: agent capability policy resolver

- Created: 2026-06-03T20:04:11+0800
- Updated: 2026-06-03T22:17:11+0800
- Report schema: adversarial-v1
- Task: Complete agent registration gaps so capability policy semantics are real, diagnosable, and test-backed.
- Report path: `vs_review/2026-06-03-agent-capability-policy-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: implementation review

### Review Input

#### Objective
Verify the implemented capability policy resolver, materializer integration, and doctor diagnostics for agent registration before the change is committed.

#### Review Target
Code implementation, architecture boundaries, and test strategy for capability policy resolution.

#### Target Locations
- `crates/harnesslab-core/src/capability_policy.rs`
- `crates/harnesslab-core/src/agent_profile.rs`
- `crates/harnesslab-core/src/config.rs`
- `crates/harnesslab-cli/src/agent_registry/capability_catalog.rs`
- `crates/harnesslab-cli/src/agent_registry/materializer.rs`
- `crates/harnesslab-cli/src/doctor_capabilities.rs`
- `crates/harnesslab-cli/src/doctor.rs`
- `crates/harnesslab-cli/tests/doctor_contract.rs`
- `crates/harnesslab-core/src/config_tests.rs`
- `crates/harnesslab-cli/src/runner/external_tests.rs`
- `crates/harnesslab-cli/src/runner/sandbox_setup.rs`

#### Change Introduction
The change introduces a core `ResolvedCapabilityPolicy` model. It defines capability set algebra as explicit allow first, inherited defaults second, empty set when inheritance is off and no allow list exists, then subtract deny. CLI-side catalogs define available/default/enforcement support per agent kind. `fake` can enforce policies; real kinds currently accept only default policies until verified materializers exist. Doctor now emits per-domain policy checks with available/default/effective sets and exact error paths.

#### Risk Focus
- The resolver may compute the effective set correctly in tests but expose contradictory validation or materialization behavior in real profiles.
- Dynamic `ProfileValidationError.field` and `accepted_values` type changes may break downstream assumptions or output contracts.
- CLI catalogs may encode defaults that create false confidence for real agent kinds.
- Doctor may produce duplicated or misleading errors that confuse users.
- Runtime snapshots may serialize a larger `capabilities` structure without tests proving the shape remains useful and redacted.

#### Assumptions To Attack
- `allow` should be the source set when non-empty, even if `inherit=false`.
- Unknown allow/deny entries should be blocking doctor errors with exact field paths.
- Non-default policies for unsupported real agent kinds should block before benchmark execution.
- `include_paths` only belongs to skills and should not silently affect tools/hooks.
- Adding structured capability data to `MaterializedAgentProfile` does not break existing runner/test construction.

#### Adversarial Lenses
- implementation
- architecture
- testing
- observability
- maintenance

#### Verification Status
- Passed: `cargo fmt --all && cargo fmt --all --check`
- Passed: `cargo test -p harnesslab-core agt_reg_001 -- --nocapture`
- Passed: `cargo test -p harnesslab-core agt_reg_008 -- --nocapture`
- Passed: `cargo test -p harnesslab-cli agt_reg_002 -- --nocapture`
- Passed: `cargo test -p harnesslab-cli agt_reg_004 -- --nocapture`
- Passed: `cargo test -p harnesslab-cli agt_reg_006 -- --nocapture`
- Passed: `cargo test -p harnesslab-cli agt_reg_008 -- --nocapture`
- Passed: `git diff --check`
- Passed: touched code files are below 500 lines by `wc -l`.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Focus on high-impact correctness, architecture, test, and diagnostic failures.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| complex | 12 minutes | one 6 minute extension if active | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | The change alters validation, resolver behavior, materialization blocking, and doctor JSON. | correctness, invalid inputs, compatibility |
| architecture-adversary | The change introduces a new core model plus CLI catalogs and runtime snapshots. | boundaries, dependency direction, maintainability |
| test-validity-adversary | The change claims AGT-REG-008 closure and updates older tests after type changes. | weak assertions, missing failure paths, self-deceptive tests |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | multi_agent_v1.spawn_agent code-reviewer | 019e8d5f-7302-7c93-847c-1b34b1023f28 | spawn_agent result nickname=Godel | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |
| architecture-adversary | multi_agent_v1.spawn_agent architect | 019e8d5f-b5f1-7251-a449-1a15c75e6c75 | spawn_agent result nickname=Zeno | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |
| test-validity-adversary | multi_agent_v1.spawn_agent test-engineer | 019e8d5f-f715-70f1-ad8a-028de97c531e | spawn_agent result nickname=Einstein | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| test-validity-output | test-validity-adversary | 1 | 019e8d5f-f715-70f1-ad8a-028de97c531e | <12m | completed | completed via wait_agent notification | completed |
| implementation-output | implementation-adversary | 1 | 019e8d5f-7302-7c93-847c-1b34b1023f28 | <12m | completed | completed via wait_agent notification | completed |
| architecture-output | architecture-adversary | 1 | 019e8d5f-b5f1-7251-a449-1a15c75e6c75 | <12m | completed | completed via wait_agent notification | completed |

### Reviewer Outputs

#### test-validity-output

##### Summary
The reviewer found one blocking correctness issue: the resolver claims set algebra, but duplicate allow entries survive into `effective`. This can mislead doctor/runtime snapshots and future materializers.

##### Blocking Findings
- Duplicate capability entries violate the claimed set semantics.
  - Broken assumption: capability policy is set algebra.
  - Failure scenario: `tools.allow = ["bash", "bash"]` produces duplicated `effective` values.
  - Trigger condition: repeated value in `allow`.
  - Impact: CLI-visible output and future enforcement can become non-idempotent.
  - Proof needed: black-box doctor/materialization test that rejects duplicates or proves deduped output.

##### Non-blocking Risks
- Default unsupported real agent kinds look healthy in text doctor output.
- Runtime snapshot contract is substring-only, not structural.

##### Required Fixes
- Make capability resolution actually set-based.
- Add a black-box contract for duplicate allow behavior.
- Decide whether unsupported real kinds should emit a visible text warning for default policies.

##### Missing Tests
- Duplicate allow black-box test.
- Unknown deny doctor test.
- Non-skill include_paths doctor/materialization test.
- Unsupported real kind default-policy details test.

##### Missing Logs / Observability
- Run-time materialization rejection is a plain pre-run anyhow string.
- Text-mode doctor hides capability details available in JSON.

##### Evidence
- `crates/harnesslab-core/src/capability_policy.rs:166` - allow entries were iterated from raw vectors.
- `crates/harnesslab-core/src/agent_profile.rs:193` - validation did not reject duplicate entries.
- `crates/harnesslab-cli/tests/doctor_contract.rs:478` - unknown allow was covered but unknown deny was not.

#### implementation-output

##### Summary
The reviewer found no blocking issue, but requested a contract change because error-state doctor JSON can expose a runnable-looking `effective` set while materialization rejects the same policy.

##### Blocking Findings
- none

##### Non-blocking Risks
- Blocked policies still serialize a runnable-looking `effective` set.
  - Broken assumption: `details.effective` represents a policy that can be materialized and used.
  - Failure scenario: `kind = "custom"` with `[tools] inherit = false, allow = ["bash"]` shows `effective=["bash"]` while materialization is error.
  - Trigger condition: unsupported kind with a non-default but otherwise known policy.
  - Impact: resolver, doctor, and materializer appear to disagree.
  - Proof needed: contract that blocked policies do not expose `effective` as runnable, or rename candidate output.

##### Required Fixes
- Reserve `effective` for materializable policy results, or move error-state algebra output to `candidate_effective`.
- Add a doctor contract for unsupported-but-known allow.

##### Missing Tests
- Non-skill `include_paths` CLI propagation.
- Error-state `effective` relationship.

##### Missing Logs / Observability
- Run only surfaces the first materialization error and produces no structured pre-run artifact.

##### Evidence
- `crates/harnesslab-core/src/capability_policy.rs:153` - unsupported errors are attached after candidate computation.
- `crates/harnesslab-cli/src/doctor_capabilities.rs:22` - status depended only on errors.
- `crates/harnesslab-cli/src/agent_registry/materializer.rs:30` - materializer rejects on resolver errors.

#### architecture-output

##### Summary
The reviewer found two blocking architecture issues: `fake` was marked enforced without runtime consumption, and unsupported real kinds were reported as `ok` while exposing candidate defaults as if verified.

##### Blocking Findings
- `fake` capability policy currently only parses; it is not runtime-enforced.
  - Broken assumption: `CapabilityEnforcement::Enforced` means the execution path consumes `effective`.
  - Failure scenario: a fake profile with non-default capabilities passes materialization and run continues, but no command/env/mount/scheduler behavior uses the resolved set.
  - Trigger condition: any fake profile with non-default policy.
  - Impact: metadata validation is misreported as runtime enforcement.
  - Proof needed: either implement runtime consumption or mark fake unsupported until a consumer exists.
- Unsupported materializer default policies are reported as `ok` with speculative capability sets.
  - Broken assumption: default policy for unsupported real kinds can be exposed as reliable `available/default/effective`.
  - Failure scenario: a default codex/claude/opencode profile shows ok policy/materialization checks while `enforcement.mode = "unsupported"`.
  - Trigger condition: any non-fake kind with default policy.
  - Impact: internal catalog guesses become user-facing contracts.
  - Proof needed: warning status or hidden/informational candidate values plus a contract test.

##### Non-blocking Risks
- `AgentProfile::validate()` can be mistaken for full runnability validation.
- `ConfigError::InvalidField` flattens dynamic accepted values into CLI-oriented text.
- allow/deny overlap can appear in validation and resolver outputs.

##### Required Fixes
- Correct enforcement semantics before commit.
- Adjust doctor contract so unsupported domains are not `ok` with authoritative effective/default/available values.
- Clarify validation boundaries.

##### Missing Tests
- Run-level test if fake remains enforced.
- Unsupported kind default-policy doctor contract.
- Core/CLI two-stage validation boundary test.

##### Missing Logs / Observability
- No structured runtime event indicating capability domain applied vs skipped/unsupported.
- No explicit materializer verified status field.

##### Evidence
- `crates/harnesslab-cli/src/agent_registry/capability_catalog.rs:44` - fake was marked enforced.
- `crates/harnesslab-cli/src/runner/sandbox.rs:30` - run path does not consume materialized capabilities.
- `crates/harnesslab-cli/src/doctor_capabilities.rs:20` - doctor status did not consider unsupported enforcement.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| test-validity-adversary | Duplicate capability entries violate set semantics | Duplicate allow entries can produce duplicate effective entries | blocking | accept | Set semantics must be normalized or rejected | Added `candidate_effective`, deduped candidate/effective construction, and core duplicate allow test | Round 2 required |
| implementation-adversary | Blocked policies serialize runnable-looking effective set | Unsupported custom allow can show effective while materialization errors | medium | accept | Error-state result needs a candidate/effective distinction | `effective` is now empty unless resolver has no errors and enforcement is verified; candidate values live in `candidate_effective` | covered by Round 2 |
| architecture-adversary | Fake marked enforced without runtime enforcement | Fake policies pass materialization but run path does not consume capabilities | blocking | accept | Enforcement must mean runtime consumer exists | Removed fake `Enforced` catalog path; fake non-default policies now block until runtime enforcement exists | Round 2 required |
| architecture-adversary | Unsupported defaults reported ok with speculative sets | Unsupported non-fake defaults expose ok/effective despite no verified materializer | blocking | accept | Unsupported should not be an ok verified policy | Doctor policy status is warning for unsupported default policies; materialization warnings include unverified materializer reasons; `effective` remains empty | Round 2 required |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 2 and Round 3
- Blocking re-review launch records:
  - Round 2 launch records and Round 3 launch record
- Rejected findings backed by evidence: n/a
- Deferred findings documented: no
- Blocked reason: n/a
- Allowed to proceed: yes

## Round 2: blocking fix re-review

### Review Input

#### Objective
Verify that the accepted Round 1 blocking findings are actually fixed and that the fixes do not create new high-impact regressions.

#### Review Target
Re-review of capability policy resolver semantics, unsupported enforcement reporting, and fake agent materialization behavior after Round 1 fixes.

#### Target Locations
- `crates/harnesslab-core/src/capability_policy.rs`
- `crates/harnesslab-cli/src/agent_registry/capability_catalog.rs`
- `crates/harnesslab-cli/src/agent_registry/materializer.rs`
- `crates/harnesslab-cli/src/doctor_capabilities.rs`
- `crates/harnesslab-cli/tests/doctor_contract.rs`
- `crates/harnesslab-core/src/config_tests.rs`
- `vs_review/2026-06-03-agent-capability-policy-review.md`

#### Change Introduction
The fix separates `candidate_effective` from `effective`; `effective` is non-empty only when the policy has no resolver errors and enforcement is verified. Candidate/effective values are deduplicated to preserve set semantics. The fake catalog no longer declares enforcement, so fake non-default policies block until a runtime consumer exists. Unsupported default policies now produce warning status in doctor and materialization warnings instead of `ok` with authoritative-looking effective sets.

#### Risk Focus
- The Round 1 blockers may only be hidden rather than fixed.
- `candidate_effective`/`effective` semantics may be inconsistent across resolver, doctor, and materializer.
- Unsupported default policies may still appear as successful materialization.
- Tests may assert the new behavior narrowly without proving CLI-visible contracts.

#### Assumptions To Attack
- Duplicate allow entries are normalized in resolver output.
- Error or unsupported states never expose runnable-looking `effective` values.
- Fake non-default policies block until runtime enforcement exists.
- Unsupported default policies are visible as warnings and carry explicit unverified materializer data.

#### Adversarial Lenses
- implementation
- architecture
- testing

#### Verification Status
- Passed after fixes: `cargo fmt --all && cargo fmt --all --check`
- Passed after fixes: `cargo test -p harnesslab-core agt_reg_008 -- --nocapture`
- Passed after fixes: `cargo test -p harnesslab-cli agt_reg_008 -- --nocapture`
- Passed after fixes: `cargo test -p harnesslab-cli agt_reg_006 -- --nocapture`
- Passed after fixes: `cargo test -p harnesslab-cli agt_reg_004 -- --nocapture`
- Passed after fixes: `git diff --check`

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Focus on whether accepted blockers are truly closed.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 15 minutes | one 8 minute extension if active | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Re-checks duplicate set semantics and effective/candidate behavior. | correctness |
| architecture-adversary | Re-checks enforcement truthfulness and unsupported materializer status. | boundaries, maintainability |
| test-validity-adversary | Re-checks whether new tests actually prove the fixes. | validation quality |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | multi_agent_v1.spawn_agent code-reviewer | 019e8dc3-cda3-7ea1-8dc6-01c5480dd160 | spawn_agent result nickname=Descartes | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |
| architecture-adversary | multi_agent_v1.spawn_agent architect | 019e8dc4-050c-72b0-8bb6-a810fa3b3312 | spawn_agent result nickname=Confucius | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |
| test-validity-adversary | multi_agent_v1.spawn_agent test-engineer | 019e8dc4-406c-7632-a202-1b9847ccfb3d | spawn_agent result nickname=Bacon | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round2-implementation-output | implementation-adversary | 1 | 019e8dc3-cda3-7ea1-8dc6-01c5480dd160 | <15m | completed | completed via wait_agent notification | completed |
| round2-test-validity-output | test-validity-adversary | 1 | 019e8dc4-406c-7632-a202-1b9847ccfb3d | <15m | completed | completed via wait_agent notification | completed |
| round2-architecture-output | architecture-adversary | 1 | 019e8dc4-050c-72b0-8bb6-a810fa3b3312 | <15m | completed | completed via wait_agent notification | completed |

### Reviewer Outputs

#### round2-implementation-output

##### Summary
The reviewer found the Round 1 blocking issues substantively closed: set outputs are deduped, fake non-default policy blocks, unsupported defaults warn with empty effective. It found no blocking issue, but requested warning severity alignment and CLI duplicate-allow coverage.

##### Blocking Findings
- none

##### Non-blocking Risks
- Warning status still serialized with `severity = "error"`.
- Duplicate allow was proven at resolver level but not CLI black-box level.

##### Required Fixes
- Align capability policy severity with warning status.
- Add CLI duplicate allow contract.

##### Missing Tests
- Unsupported default severity assertion.
- Duplicate allow doctor/materialization black-box test.

##### Missing Logs / Observability
- Text doctor omits detailed capability fields.

##### Evidence
- `crates/harnesslab-cli/src/doctor_capabilities.rs` - status was warning but severity was hard-coded error.
- `crates/harnesslab-core/src/capability_policy.rs` - duplicate allow was tested in core.

#### round2-test-validity-output

##### Summary
The reviewer confirmed core semantics are present but found one blocking contract issue: unsupported default capability checks still serialize warning status with error severity, so machine consumers still see error-like output.

##### Blocking Findings
- Unsupported default capability checks still serialize as error-severity warnings.
  - Broken assumption: unsupported default policies are downgraded to warning-level machine contract.
  - Failure scenario: `agent.default-codex.tools.policy` and materialization report `status="warning"` but `severity="error"`.
  - Trigger condition: unsupported default policy with no resolver errors.
  - Impact: downstream JSON consumers can still treat warning-only unsupported materializers as hard errors.
  - Proof needed: doctor contract asserting both status and severity are warning.

##### Non-blocking Risks
- Duplicate allow dedupe is not black-box pinned.
- Fake non-default blocking is not run-level pinned.
- Runtime snapshot shape is not structurally pinned.

##### Required Fixes
- Policy doctor checks with warning status must have warning severity.
- Materialization warning-only checks must have warning severity.
- Contract tests must assert both.

##### Missing Tests
- Duplicate allow doctor/materialization black-box test.
- Fake non-default run contract.
- Structural runtime snapshot capability fields.

##### Missing Logs / Observability
- No runtime event for capability domain applied/skipped/unsupported.

##### Evidence
- `crates/harnesslab-cli/src/doctor_capabilities.rs` - severity was hard-coded error.
- `crates/harnesslab-cli/src/doctor.rs` - materialization warning status still used error severity.
- `crates/harnesslab-cli/tests/doctor_contract.rs` - unsupported default test only asserted status before the fix.

#### round2-architecture-output

##### Summary
The reviewer confirmed the core semantic fixes are mostly real, but found the same blocking severity mismatch for unsupported-default warning semantics.

##### Blocking Findings
- Unsupported-default warning semantics are still serialized as error severity.
  - Broken assumption: warning status means warning-level machine contract.
  - Failure scenario: default codex profile emits warning status but error severity for policy and materialization checks.
  - Trigger condition: unsupported default capability catalog.
  - Impact: JSON consumers still see an error-shaped result.
  - Proof needed: status/severity black-box assertions.

##### Non-blocking Risks
- All current capability catalogs are unsupported, creating warning noise.
- Runner-facing summaries remain lossy.

##### Required Fixes
- Change capability policy warning severity to warning.
- Change materialization warning severity to warning.
- Add contract tests.

##### Missing Tests
- Unsupported default policy/materialization severity.
- End-to-end warning check should not serialize severity error.

##### Missing Logs / Observability
- Materialization warnings are unstructured strings.
- Runner/report summaries omit candidate/materializer verified details.

##### Evidence
- `crates/harnesslab-cli/src/doctor_capabilities.rs` - warning status with error severity before fix.
- `crates/harnesslab-cli/src/doctor.rs` - materialization warning with error severity before fix.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | Warning status still uses error severity | Warning downgrade is incomplete for JSON consumers | medium | accept | Machine consumers may key on severity | Changed policy warning severity to warning; added duplicate allow black-box | Round 3 required |
| test-validity-adversary | Unsupported default capability checks still serialize as error-severity warnings | Warning-only unsupported defaults still look like hard errors to JSON consumers | blocking | accept | Both policy and materialization checks had `status=warning`, `severity=error` | Changed policy/materialization warning severity to warning and asserted both in contract | Round 3 required |
| architecture-adversary | Unsupported-default warning semantics still serialized as error severity | Warning status was not warning-level machine contract | blocking | accept | Same policy/materialization severity mismatch | Changed policy/materialization warning severity to warning and asserted both in contract | Round 3 required |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: pending
- Blocking re-review passed: pending
- Blocking re-review round links:
  - pending
- Blocking re-review launch records:
  - pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: no
- Blocked reason: pending
- Allowed to proceed: pending

## Round 3: severity closure re-review

### Review Input

#### Objective
Verify that the Round 2 accepted blocking finding is fixed: unsupported default capability policy/materialization warnings must serialize warning severity, not error severity.

#### Review Target
Narrow re-review of doctor severity behavior and black-box test coverage after severity fix.

#### Target Locations
- `crates/harnesslab-cli/src/doctor_capabilities.rs`
- `crates/harnesslab-cli/src/doctor.rs`
- `crates/harnesslab-cli/tests/doctor_contract.rs`
- `vs_review/2026-06-03-agent-capability-policy-review.md`

#### Change Introduction
Policy checks now set `severity = "warning"` when computed status is warning. Materialization checks now set warning severity when materialization succeeds with warnings. The unsupported default doctor contract now asserts both policy and materialization `status`/`severity` are warning. The duplicate allow doctor contract now uses repeated allow entries and asserts deduped `candidate_effective`.

#### Risk Focus
- Warning status might still pair with error severity somewhere in the capability policy or materialization doctor path.
- Tests might only cover one domain or only status.
- The report closure might overstate the review status.

#### Assumptions To Attack
- Unsupported default policy warnings are warning severity in JSON.
- Unsupported default materialization warnings are warning severity in JSON.
- Duplicate allow is black-box pinned through doctor JSON.

#### Adversarial Lenses
- testing
- implementation

#### Verification Status
- Passed after severity fix: `cargo fmt --all && cargo fmt --all --check`
- Passed after severity fix: `git diff --check`
- Passed after severity fix: `cargo test -p harnesslab-cli agt_reg_008 -- --nocapture`
- Passed after severity fix: `cargo test -p harnesslab-cli agt_reg_006 -- --nocapture`
- Passed after severity fix: `cargo test -p harnesslab-cli agt_reg_004 -- --nocapture`

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Focus only on whether Round 2's accepted blocking severity finding is closed.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 8 minutes | one 4 minute extension if active | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| test-validity-adversary | The accepted blocker is a JSON contract/test coverage issue. | status/severity contract |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| test-validity-adversary | multi_agent_v1.spawn_agent test-engineer | 019e8dd6-ce9f-7ea1-aa84-9403c91513c4 | spawn_agent result nickname=Mencius | fork_context=false | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless discovered by reviewer | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round3-test-validity-output | test-validity-adversary | 1 | 019e8dd6-ce9f-7ea1-aa84-9403c91513c4 | <8m | completed | completed via wait_agent notification | completed |

### Reviewer Outputs

#### round3-test-validity-output

##### Summary
Round 2's accepted blocking severity finding is closed. Policy and materialization warning paths now serialize warning severity, and the black-box unsupported default contract asserts both status and severity.

##### Blocking Findings
- none

##### Non-blocking Risks
- none for the Round 2 severity-closure scope

##### Required Fixes
- none

##### Missing Tests
- none required to close the Round 2 severity finding

##### Missing Logs / Observability
- none for this scope

##### Evidence
- `crates/harnesslab-cli/src/doctor_capabilities.rs` - policy warning status maps to warning severity.
- `crates/harnesslab-cli/src/doctor.rs` - materialization warning status maps to warning severity.
- `crates/harnesslab-cli/tests/doctor_contract.rs` - unsupported default policy/materialization assert status and severity warning.
- `crates/harnesslab-cli/tests/doctor_contract.rs` - duplicate allow black-box contract uses repeated allow entries and asserts deduped candidate output.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| test-validity-adversary | none | n/a | n/a | n/a | Round 3 found no blocking findings | n/a | n/a |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: n/a
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 3
- Blocking re-review launch records:
  - test-validity-adversary 019e8dd6-ce9f-7ea1-aa84-9403c91513c4
- Rejected findings backed by evidence: n/a
- Deferred findings documented: no
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

Passed. Round 1 blocking findings were fixed, Round 2 identified one remaining severity-contract blocker, and Round 3 re-review found that blocker closed with no remaining blocking findings.
