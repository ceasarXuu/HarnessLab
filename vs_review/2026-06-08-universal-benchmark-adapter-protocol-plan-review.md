# Subagent VS Review: Universal Benchmark Adapter Protocol Implementation Plan

- Created: 2026-06-08T00:08:33+0800
- Updated: 2026-06-08T00:19:50+0800
- Report schema: adversarial-v1
- Task: adversarial review of the universal benchmark adapter protocol implementation plan
- Report path: `vs_review/2026-06-08-universal-benchmark-adapter-protocol-plan-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed; Round 1 blockers accepted and fixed in plan v0.2, Round 2 closure re-review passed

## Round 1: Plan Review

### Review Input

#### Objective

Review whether the implementation plan can genuinely turn the benchmark adapter
layer into a universal protocol boundary: any benchmark that completes the
adapter protocol should be callable by generic run, replay, report, doctor,
registry, and selector flows without those upper layers knowing which benchmark
is behind the adapter.

#### Review Target

- Engineering plan quality.
- Architecture completeness.
- Protocol isolation claim.
- Phase decomposition and gate strength.
- Conformance and proof strategy.
- Migration and rollback risk.

#### Target Locations

- `prd/2026-06-07-universal-benchmark-adapter-protocol.md`
- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `crates/harnesslab-cli/src/runner/external/runtime_adapter.rs`
- `crates/harnesslab-cli/src/runtime_compatibility.rs`
- `crates/harnesslab-cli/src/runner/replay.rs`
- `crates/harnesslab-adapters/src/registry.rs`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `xtask/src/adapter_claims.rs`

#### Change Introduction

The new plan proposes a high-risk architecture migration from a mature built-in
adapter model toward a universal adapter protocol. It identifies current
coupling around `ExternalRunnerKind`, runtime compatibility, replay authority,
doctor/readiness, report artifact semantics, and selector governance. It splits
execution into discovery, protocol spec, identity/registry foundation, unified
adapter interface, generic upper-layer refactor, conformance/scaffold, existing
adapter migration, third-adapter proof, and governance/release phases.

#### Risk Focus

- The plan may still allow hidden benchmark-specific branches outside adapter
  modules.
- The plan may under-specify serialized compatibility and replay migration.
- The conformance suite may prove schema shape rather than real black-box
  behavior.
- The third-adapter proof may be too toy-like to prove horizontal extension.
- The phase gates may be too vague to prevent premature closure.
- Doctor/report/readiness may remain benchmark-aware through indirect metadata
  or hard-coded interpretations.
- Existing Terminal-Bench and SWE-bench Pro behavior may regress during
  protocol migration.
- The plan may overclaim "universal" while explicitly deferring out-of-tree
  packaging.

#### Assumptions To Attack

- In-repo protocol isolation is enough to prove universal component behavior.
- A string id plus capabilities can replace `ExternalRunnerKind` without
  breaking old run replay.
- Static no-branch guards can reliably distinguish adapter-owned code from
  generic-layer leaks.
- Existing `ADAPT-*` and `SWEPRO-*` selectors can be preserved while moving to
  the protocol.
- Adapter-declared doctor/report metadata is enough for upper layers to remain
  benchmark-agnostic.
- The plan's phase gates are concrete enough for engineers to execute without
  inventing missing architecture.

#### Adversarial Lenses

- architecture and API compatibility
- migration and replay safety
- test validity and conformance quality
- observability and artifact privacy
- maintainability and adapter author experience
- release and rollback
- documentation claim discipline

#### Verification Status

- Plan document created.
- No code implementation has started for this protocol migration.
- `git diff --check` has not yet been run after creating the plan/review files.

#### Reviewer Instructions

- Fresh internal subagent session.
- Do not inherit main-agent context, chat history, reasoning, drafts, or
  conclusions.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Report blocking findings, non-blocking risks, required fixes, missing tests,
  missing logs/observability, and concrete counterexamples for major findings.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| complex | 15 minutes | one bounded extension if alive | 2 | accepted blockers require fresh closure re-review |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-reviewer | Challenge whether the plan actually creates a protocol boundary and removes upper-layer benchmark awareness. | architecture, compatibility, migration |
| test-engineer | Challenge conformance gates, selectors, proof strength, and third-adapter validation. | test adequacy, evidence quality |
| maintainer-experience-reviewer | Challenge whether adapter authors can use the protocol without hidden system knowledge or excessive integration cost. | maintainability, author UX, docs |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-reviewer | `multi_agent_v1.spawn_agent` | `019ea2d8-6a55-70b0-a17e-437596785d65` | spawn tool result | false | Round 1 review input plus target paths | main-agent history, reasoning, drafts, conclusions, full diff | yes |
| test-engineer | `multi_agent_v1.spawn_agent` | `019ea2d8-9826-76d3-8215-5f95c083d56c` | spawn tool result | false | Round 1 review input plus target paths | main-agent history, reasoning, drafts, conclusions, full diff | yes |
| maintainer-experience-reviewer | `multi_agent_v1.spawn_agent` | `019ea2d8-bebc-7063-8778-c7ebedd6103b` | spawn tool result | false | Round 1 review input plus target paths | main-agent history, reasoning, drafts, conclusions, full diff | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| architecture-reviewer | architecture-reviewer | 1 | `019ea2d8-6a55-70b0-a17e-437596785d65` | under 15 minutes | completed | reviewer returned request-changes findings | completed |
| test-engineer | test-engineer | 1 | `019ea2d8-9826-76d3-8215-5f95c083d56c` | under 15 minutes | completed | reviewer returned request-changes findings | completed |
| maintainer-experience-reviewer | maintainer-experience-reviewer | 1 | `019ea2d8-bebc-7063-8778-c7ebedd6103b` | under 15 minutes | completed | reviewer returned request-changes findings | completed |

### Reviewer Outputs

#### architecture-reviewer

##### Summary

`REQUEST CHANGES`. The plan has the right direction but was not yet strong
enough to prove benchmark-specific integration cost stops at the adapter layer.
Largest gaps: serialized authority migration, registry/identity semantics, and
generic doctor/report/readiness contracts.

##### Blocking Findings

- Serialized replay authority is still enum-shaped, and the plan did not define
  a canonical migration contract for old-only, new-only, dual-written, and
  mixed snapshot sets.
- Runtime dispatch was planned as id-based, but registry semantics were
  ambiguous because the plan allowed "`AdapterId` or `BenchmarkId` plus
  capability" rather than one concrete binding model.
- Doctor/readiness genericity was asserted but lacked a typed readiness record
  with severity, privacy boundary, remediation, probe timing, and capability
  linkage.

##### Non-blocking Risks

- Third-adapter proof may be too synthetic.
- Out-of-tree packaging is deferred and must not be overclaimed.
- Static no-branch guard was originally too late in the plan.

##### Required Fixes

- Define canonical serialized protocol identity and migration matrix before
  Phase 2 closes.
- Replace ambiguous dispatch wording with one binding model.
- Specify readiness/report/artifact schemas.
- Make registry metadata precise and behavior-free.
- Strengthen Phase 7 proof.

##### Missing Tests / Observability

- Old-only, new-only, and mixed replay fixtures.
- Negative test proving new adapters cannot add `ExternalRunnerKind` variants.
- Registry conflict tests.
- Static guard over doctor/report/runner/replay/selector routing.
- Protocol dispatch, replay authority, readiness, and registry diagnostics
  event fields.

#### test-engineer

##### Summary

`REQUEST CHANGES`. The plan was directionally strong but had too much prose and
not enough enforceable evidence around `ADAPT-PROTOCOL-*`, no-branch guards,
negative fixtures, and exact migration pass/fail thresholds.

##### Blocking Findings

- `ADAPT-PROTOCOL-*` gates were named but not registered or enforceable.
- The no-branch guard was underspecified and not executable.
- Conformance evidence was too shallow to prove behavior rather than schema.
- Existing regression selectors could still be weakened during migration
  because the plan did not freeze an exact selector manifest.

##### Non-blocking Risks

- Third-adapter proof remains underdefined unless it exercises behavior not
  already covered by Terminal-Bench/SWE-bench Pro and includes forbidden-diff
  proof.

##### Required Fixes

- Register and route `ADAPT-PROTOCOL-001..009` or the final selector family.
- Extend `xtask` claim validation to the new plan/prefixes.
- Implement no-branch guard before Phase 7.
- Freeze exact regression selector manifests.
- Define black-box positive/negative fixtures with pass/fail thresholds.

##### Missing Tests / Observability

- Protocol registry rows, negative adapter fixtures, no-branch bypass tests,
  generic doctor/report metadata tests, protocol replay drift tests, and
  selector-weakening tests.
- Required protocol dispatch fields and gate artifacts.

#### maintainer-experience-reviewer

##### Summary

`REQUEST CHANGES`. The plan had the right architecture intent but did not yet
standardize the adapter-author experience tightly enough; too much integration
knowledge remained implicit.

##### Blocking Findings

- Adapter author journey was not operationally specified.
- Conformance proof shape was too high-level to prevent shallow or inconsistent
  adapters.
- Stable vs experimental promotion remained ambiguous.
- Artifact, readiness, error, and reporting schemas were not standardized
  enough.

##### Non-blocking Risks

- Third adapter may be too toy-like.
- Out-of-tree packaging is deferred and docs must not imply external adapter
  support.
- No-branch guard allowlists can become brittle if not specified.

##### Required Fixes

- Add adapter-author journey: generate, implement, validate, register, prove,
  promote.
- Define exact scaffold outputs and permitted edit set.
- Convert `ADAPT-PROTOCOL-*` from names into registry-backed gates.
- Add stable promotion evidence schema before Phase 6/7.
- Reuse or supersede older concrete event/artifact/error contracts with
  versioned protocol schemas.

##### Missing Tests / Observability

- Golden scaffold test, negative conformance fixtures, forbidden-diff guard,
  official-proof archive validator, doctor/report schema tests.
- Adapter registration/discovery, conformance, readiness probe,
  artifact-declaration, and promotion evidence logs.

### Main Agent Response

| Reviewer | Finding | Severity | Decision | Action Taken | Validation |
|---|---|---:|---|---|---|
| architecture-reviewer | Serialized replay authority lacked canonical migration contract. | blocking | accept | Added `AdapterProtocolAuthority`, explicit fields, canonical old-only/new-only/dual-written/mixed replay matrix, and required fixtures. | Round 2 closure review passed. |
| architecture-reviewer | Dispatch/binding model ambiguous. | blocking | accept | Added `TaskRuntimeBinding` as the sole dispatch binding model and rewrote Phase 2 to use `adapter_id` from binding. | Round 2 closure review passed. |
| architecture-reviewer / maintainer-experience-reviewer | Readiness/report/artifact/error schemas underspecified. | blocking | accept | Added versioned readiness probe, artifact declaration, failure mapping, and report metadata schemas. | Round 2 closure review passed. |
| test-engineer | `ADAPT-PROTOCOL-*` gates not registered/enforceable. | blocking | accept | Added `ADAPT-PROTOCOL-001..012` selector plan with required registration surfaces and phase activation expectations. | Round 2 closure review passed. |
| test-engineer | No-branch guard underspecified and too late. | blocking | accept | Added explicit no-branch guard contract, forbidden patterns, allowlisted paths, bypass fixtures, and moved guard before Phase 4/Phase 7 gates. | Round 2 closure review passed. |
| test-engineer | Conformance proof too shallow. | blocking | accept | Added black-box positive/negative fixture requirements, exact expected failure messages, golden scaffold test, and behavioral conformance gates. | Round 2 closure review passed. |
| test-engineer | Regression selectors could be weakened during migration. | blocking | accept | Added frozen regression selector manifest requirement with exact ids, commands, expected counts/artifacts, and weakening guard. | Round 2 closure review passed. |
| maintainer-experience-reviewer | Adapter author journey not operational. | blocking | accept | Added adapter-author journey, scaffold outputs, permitted edit set, and forbidden generic behavior edits. | Round 2 closure review passed. |
| maintainer-experience-reviewer | Stable/experimental promotion ambiguous. | blocking | accept | Added stable promotion evidence schema and conditional-stable-blocked rule before migration phases. | Round 2 closure review passed. |
| all reviewers | Third-adapter proof may be too toy-like. | non-blocking | accept | Strengthened Phase 7 to require readiness, runtime failure injection, public/private artifacts, replay materials, report rendering, adapter-local failure mapping, and forbidden-diff proof. | Round 2 closure review passed. |
| architecture-reviewer / maintainer-experience-reviewer | Out-of-tree packaging deferred but may be overclaimed. | non-blocking | accept | Kept out-of-tree packaging explicitly out of scope and limited release language to in-repo protocol isolation. | Round 2 closure review passed. |

### Closure Status After Round 1

- Blocking findings found: yes
- Accepted blocking findings fixed: yes, in plan v0.2
- Blocking re-review completed: no
- Blocking re-review passed: pending
- Blocked reason: accepted blocking fixes require Round 2 closure review
- Allowed to proceed: no

## Round 2: Accepted Blocking Fix Closure Review

### Review Input

#### Objective

Verify that the accepted Round 1 blockers are fixed in
`docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`
version 0.2 and that the plan is now strong enough to guide implementation
without overclaiming universal benchmark adapter support.

#### Review Target

- Canonical serialized authority and replay migration matrix.
- Concrete `TaskRuntimeBinding` model.
- Versioned readiness, artifact, failure, and report schemas.
- Adapter author journey and permitted edit set.
- Stable promotion evidence schema.
- No-branch guard contract.
- Frozen regression selector manifest.
- Registered `ADAPT-PROTOCOL-001..012` selector plan.
- Phase updates that make these contracts actionable.

#### Target Locations

- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`
- `vs_review/2026-06-08-universal-benchmark-adapter-protocol-plan-review.md`

#### Verification Status

- Plan v0.2 updated after Round 1.
- `git diff --check`: passed after review response.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| closure-reviewer | `multi_agent_v1.spawn_agent` | `019ea2e0-e185-7502-91e5-f5245e1d22be` | spawn tool result | false | Round 2 closure input plus target paths | main-agent history, reasoning, drafts, conclusions, full diff | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round-2-closure-reviewer | closure-reviewer | 1 | `019ea2e0-e185-7502-91e5-f5245e1d22be` | under 10 minutes | completed | reviewer returned PASS with no blocking findings | completed |

### Reviewer Outputs

#### round-2-closure-reviewer

##### Summary

`PASS`. Plan v0.2 addresses the accepted Round 1 blockers at plan level and
does not claim implementation is complete. The review report also remains
accurate about the closure flow.

##### Blocking Findings

None.

##### Non-blocking Risks

- Selector timing could be worded even tighter because `ADAPT-PROTOCOL-*`
  registration is said to be required before Phase 5 closes while several
  selectors become active earlier. This is mitigated by the plan rule that every
  active protocol proof must already exist in requirements, registry,
  `scripts/test-after-change.sh`, and `xtask` claim validation.
- Schemas are still plan-level tables, not executable schema files yet. This is
  acceptable because Phase 1 requires producing the versioned protocol spec
  before implementation proceeds.

##### Required Fixes

None.

##### Missing Tests / Logs

None for plan closure. Implementation tests/logs are future phase gates.

##### Evidence

- Canonical replay authority and old/new/mixed matrix:
  `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`
- Concrete `TaskRuntimeBinding` dispatch model:
  `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`
- Versioned readiness/artifact/failure/report schemas:
  `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`
- Adapter author journey, scaffold outputs, permitted/forbidden edits:
  `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`
- Stable/experimental promotion evidence schema:
  `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`
- No-branch guard, frozen regression selector manifest, and
  `ADAPT-PROTOCOL-001..012` selector plan:
  `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`

### Final Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Residual non-blocking risks: selector timing wording and executable schema
  files are deferred to implementation phase gates and are not claimed complete.
- Allowed to proceed: yes
