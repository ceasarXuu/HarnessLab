# Universal Benchmark Adapter Protocol Implementation Plan

- Created: 2026-06-08
- Updated: 2026-06-11
- Version: 0.6
- Status: In Progress
- Owner / Responsible: implementation owner TBD
- Related Systems: `harnesslab-core`, `harnesslab-adapters`, `harnesslab-cli`, `harnesslab-report`, `xtask`, `tests/REQUIREMENTS.toml`, `tests/TEST_REGISTRY.toml`
- Related Links:
  - `prd/2026-06-07-universal-benchmark-adapter-protocol.md`
  - `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
  - `docs/plans/2026-06-06-benchmark-adapter-phase-8-full-gate-closure.md`
  - `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-phase-0-branch-inventory.md`
  - `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-phase-0-frozen-selector-manifest.md`
  - `docs/adapter-protocol.md`
  - `tests/FROZEN_SELECTOR_MANIFEST.toml`
  - `vs_review/2026-06-07-benchmark-adapter-blocker-fix-review.md`
- Risk Level: High
- Plan Type: Full

## 1. Background

The current adapter layer is mature enough for built-in benchmark support, but
it is not yet a universal protocol boundary. Existing evidence shows strong
coverage for `ADAPT-DATA-001..005`, `ADAPT-RUNTIME-001..006`, and
`SWEPRO-001..005`, but the implementation still has benchmark-specific coupling
outside a pure protocol boundary.

Known current-state examples:

- Runtime adapter lookup is keyed by `ExternalRunnerKind` and a closed match in
  `crates/harnesslab-cli/src/runner/external/runtime_adapter.rs`.
- Runtime compatibility still branches on `ExternalRunnerKind` in
  `crates/harnesslab-cli/src/runtime_compatibility.rs`.
- Replay validates adapter version by `runner.kind` in
  `crates/harnesslab-cli/src/runner/replay.rs`.
- Doctor/readiness checks include benchmark-specific knowledge outside adapter
  metadata.
- Data adapter behavior lives in `harnesslab-adapters`, while runtime adapter
  behavior lives inside `harnesslab-cli`.

The product requirement is stricter than "add new built-in benchmarks faster":
all benchmark-specific integration cost must be contained inside the adapter
implementation. Once an adapter satisfies the protocol, generic run, replay,
doctor, report, registry, and selector flows must be able to call it without
knowing which benchmark it is.

## 2. Problem Definition

### Current Behavior

New benchmark support currently depends on several benchmark-aware surfaces:

- core model enum variants
- adapter registry entries
- data adapter implementation
- CLI runtime adapter implementation
- runtime compatibility branches
- replay version/material validation
- doctor/readiness checks
- selector and adapter-claim routing
- documentation and review artifacts

### Expected Behavior

A benchmark becomes callable when it implements the standard adapter protocol
and passes conformance gates. Generic upper layers can:

- discover the adapter by opaque id
- inspect descriptor and capabilities
- prepare and list tasks
- execute runtime phases
- classify failures
- write public/private artifacts
- validate replay materials
- render reports
- produce doctor/readiness diagnostics

without adding benchmark-specific branches.

### Gap

The present implementation has proven abstractions but not a complete protocol
contract. The system still treats benchmark identity as an enum and has
benchmark-family knowledge outside adapter-owned modules.

## 3. Goals

- Define a universal benchmark adapter protocol that covers data, runtime,
  scoring, artifacts, replay, readiness, reporting, and conformance.
- Replace benchmark-id branching in non-adapter layers with protocol dispatch
  and capability checks.
- Provide a conformance suite that rejects incomplete adapters before runtime.
- Migrate Terminal-Bench and SWE-bench Pro onto the protocol without behavior
  regressions.
- Add a sample third benchmark adapter that proves horizontal extension with no
  generic-layer code changes.
- Preserve the existing Phase 8 evidence discipline: selectors, registry
  checks, redaction tests, replay tests, and `/vs_review/` closure.

## 4. Non-goals

- No public marketplace in this plan.
- No untrusted arbitrary adapter sandboxing in this plan.
- No remote adapter installation workflow in this plan.
- No guarantee that official benchmark preservation is automatic; adapters must
  provide standardized official-behavior proof.
- No silent compatibility fallback from legacy enum behavior once the new
  protocol gate is active.

## 5. Complexity And Risk Assessment

| Area | Risk | Reason |
|---|---:|---|
| Core model compatibility | High | `ExternalRunnerKind` is serialized into plans, snapshots, and replay authority. |
| Runtime dispatch | High | Execution, cleanup, failure mapping, and artifact writing are core run paths. |
| Replay | High | Incorrect migration can allow unsafe live replanning or reject valid old runs. |
| Doctor/readiness | Medium | Benchmark-specific checks must move without degrading diagnostics. |
| Reporting | Medium | Generic rendering must avoid benchmark-specific assumptions. |
| Test governance | High | False conformance would make "protocol-complete" meaningless. |
| Existing benchmark behavior | High | Terminal-Bench and SWE-bench Pro evidence must not regress. |

## 6. Constraints And Assumptions

| Assumption | Verification Method | If Assumption Fails |
|---|---|---|
| First release can keep adapters in-repo while enforcing protocol isolation. | User/product review before implementation. | Add an out-of-tree packaging phase and compatibility API review. |
| Existing serialized runs may need compatibility handling during migration. | Inspect current snapshots and replay tests in Phase 0. | Add an explicit legacy replay compatibility shim with bounded deprecation policy. |
| Terminal-Bench and SWE-bench Pro can both express their behavior through one protocol. | Migrate both adapters and run existing selectors. | Split protocol into mandatory core plus capability extensions. |
| A sample third benchmark can prove horizontal extension without external dependencies. | Implement a small deterministic benchmark adapter in Phase 7. | Use a real third benchmark and move environment proof earlier. |
| Adapter authors can accept central failure taxonomy for stable adapters. | Review taxonomy with current and sample adapters. | Add adapter-local detail codes mapped to central public codes. |

## 7. Current State Summary

| Surface | Current State | Protocol Gap |
|---|---|---|
| Data adapter | `harnesslab-adapters` owns descriptor, prepare, list, plan, snapshot for data families. | Data and runtime are not one protocol surface. |
| Runtime adapter | CLI-local `BenchmarkRuntimeAdapter` dispatches by `ExternalRunnerKind`. | Closed enum dispatch blocks id-based horizontal extension. |
| Runtime compatibility | Branches on concrete runner kind. | Readiness must become adapter-declared capabilities/checks. |
| Replay | Uses stored snapshots, but current adapter version lookup is still kind-based. | Replay authority must use adapter id/version and protocol materials. |
| Doctor | Contains benchmark-specific readiness knowledge. | Doctor must consume adapter readiness descriptors. |
| Report | Mostly generic, but public artifact semantics are not adapter-declared enough. | Reports must render adapter-declared public artifacts and scores. |
| Test registry | Strong current selector discipline. | Needs adapter protocol conformance registry and "no upper-layer branch" guards. |

## 8. Target Architecture

### 8.1 Protocol Layers

The universal adapter protocol should be split into mandatory core and optional
capability extensions:

| Protocol Area | Required For All Adapters | Capability Extension |
|---|---|---|
| Descriptor | id, display name, schema version, adapter version, stability, supported splits | official-runner proof, patch-style benchmark, cleanup override |
| Data lifecycle | inspect, prepare, list tasks, snapshot task, create task plan | remote data auth, split discovery |
| Runtime lifecycle | preflight, execute, cleanup, failure mapping | host-agent execution, docker orchestration, sandbox runner |
| Artifacts | public/private artifact declarations, redaction refs, runtime snapshots | custom report panels, cleanup report |
| Replay | adapter id/version authority, required runtime materials, drift policy | adapter-specific live material validation |
| Readiness | adapter-declared checks, blockers, remediation | environment-specific probes |
| Scoring | score extraction, outcome/failure classification | benchmark-specific metric breakdown |

### 8.2 Identity Model

Move from enum-first identity to protocol identity:

- `BenchmarkId`: stable string id, for example `terminal-bench`.
- `AdapterId`: stable adapter implementation id, for example
  `harnesslab.terminal-bench.runtime`.
- `AdapterVersion`: semantic/protocol version for replay authority.
- `AdapterProtocolVersion`: central protocol version understood by generic
  layers.
- `LegacyExternalRunnerKind`: compatibility-only concept for old snapshots, not
  the primary runtime dispatch key.

### 8.3 Generic Dispatch Rule

Non-adapter layers may branch on:

- protocol version
- declared capability
- stability level
- generic failure class/code
- generic artifact type

Non-adapter layers must not branch on:

- `terminal-bench`
- `swe-bench-pro`
- a concrete adapter implementation type
- legacy `ExternalRunnerKind`, except inside an explicit legacy migration shim

### 8.4 Conformance Gates

Every protocol-complete adapter must pass:

- descriptor schema conformance
- data lifecycle conformance
- runtime lifecycle conformance
- artifact public/private conformance
- redaction conformance
- failure taxonomy conformance
- replay authority conformance
- readiness/doctor conformance
- report artifact conformance
- no upper-layer benchmark branch guard

Stable promotion additionally requires:

- official benchmark preservation proof
- documentation
- adversarial review
- rollback/fallback notes
- sample run artifact archive

### 8.5 Canonical Serialized Authority Contract

The protocol migration must define one canonical authority object before Phase 2
can close. The object is named `AdapterProtocolAuthority` in this plan; the
implementation may choose the exact Rust type name, but it must preserve these
fields and semantics:

| Field | Required | Meaning |
|---|---|---|
| `benchmark_id` | yes | Stable benchmark family id, for example `terminal-bench`. |
| `adapter_id` | yes | Stable implementation id. This is the runtime dispatch key. |
| `protocol_version` | yes | Version of the universal adapter protocol understood by generic layers. |
| `adapter_version` | yes | Adapter implementation version used for replay drift detection. |
| `selected_mode` | yes | Adapter-selected mode, for example `official-runner`, `patch-evaluator`, or `deterministic-sample`. |
| `capabilities` | yes | Sorted capability ids used by generic layers for feature gating. |
| `stability` | yes | `experimental`, `stable`, or `legacy`. |
| `legacy_runner_kind` | compatibility-only | Present only when reading or dual-writing old `ExternalRunnerKind` authority. |

Canonical read/write rules:

| Snapshot Set | Canonical Authority | Replay Behavior |
|---|---|---|
| New-only protocol fields in benchmark, task-runtime, preflight, and external-runtime snapshots | `AdapterProtocolAuthority` | Use protocol authority; ignore legacy enum except for diagnostic comparison. |
| Dual-written protocol plus legacy enum fields | `AdapterProtocolAuthority` | Use protocol authority and verify legacy enum maps to the same adapter only as a consistency check. |
| Old-only legacy fields with no protocol authority anywhere in the source run | Named legacy shim | Use explicit legacy shim, emit replay warning, and block if legacy mapping is unknown. |
| Mixed protocol benchmark/task authority but legacy-only attempt runtime snapshots | None | Fail closed with `protocol_authority_incomplete`; do not silently infer attempt authority. |
| Mixed old benchmark/task authority but protocol attempt snapshots | None | Fail closed with `protocol_authority_inconsistent`; do not combine authorities from different eras. |
| Protocol authority mismatch across public/private/runtime/task snapshots | None | Fail closed with `protocol_authority_mismatch`. |

Phase 2 must add fixtures for every row in this table. No implementation phase
may claim replay safety if any mixed-authority fixture is missing.

### 8.6 Task Runtime Binding Model

Dispatch must use one concrete binding object, not "`AdapterId` or
`BenchmarkId` plus capability". The plan standardizes the dispatch object as
`TaskRuntimeBinding`.

Required binding fields:

- `authority`
  - `benchmark_id`
  - `adapter_id`
  - `protocol_version`
  - `adapter_version`
  - `selected_mode`
  - `capabilities`
  - `stability`
  - optional `legacy_runner_kind`
- `dataset_ref`
- `task_ref`
- `artifact_contract_id`
- `readiness_contract_id`

Selection rule:

- CLI/user input selects a `benchmark_id` and split.
- The adapter registry resolves that benchmark id to exactly one enabled
  default adapter binding for the requested mode.
- Runtime dispatch uses `adapter_id`.
- Generic layers may inspect capabilities but must not switch on concrete
  benchmark id or adapter id.
- Multiple adapters for one benchmark are allowed only when registry metadata
  selects one unambiguous default or the user explicitly selects a mode/adapter.

Registry conflict tests required before Phase 2 closes:

- duplicate `benchmark_id`
- duplicate `adapter_id`
- same benchmark with two default adapters for one mode
- adapter requiring unsupported protocol version
- capability set that does not satisfy the selected mode
- disabled or experimental adapter selected as stable

The central registry may contain descriptor and binding data. It must not
contain behavior switches such as "if benchmark is X, run Y code" outside
adapter-owned modules and the named legacy shim.

### 8.7 Versioned Protocol Schemas

Phase 1 must define versioned schemas for the following records. Phase 3 and
Phase 4 cannot close until generic layers consume these schemas instead of
benchmark-specific interpretation.

#### Readiness Probe Schema

| Field | Required | Notes |
|---|---|---|
| `check_id` | yes | Stable adapter-local id. |
| `adapter_id` | yes | Adapter that produced the check. |
| `capability` | yes | Capability being checked. |
| `phase` | yes | `discovery`, `data_prepare`, `preflight`, `runtime`, `replay`, or `report`. |
| `severity` | yes | `info`, `warning`, `blocked`, or `fatal`. |
| `status` | yes | `ready`, `degraded`, `blocked`, or `unknown`. |
| `public_message` | yes | Safe user-facing explanation. |
| `private_detail_ref` | optional | Private diagnostics path or key. |
| `remediation` | required for blocked/fatal | Actionable next step. |
| `required_tools` | optional | Tool names and versions when relevant. |
| `privacy_scope` | yes | `public` or `private`. |

#### Artifact Declaration Schema

| Field | Required | Notes |
|---|---|---|
| `artifact_id` | yes | Stable id. |
| `scope` | yes | `attempt` or `run`. |
| `path` | yes | Relative artifact path within the declared scope. |
| `artifact_type` | yes | `runtime_snapshot`, `event_log`, `result`, `report_public`, `diagnostic_public`, `diagnostic_private`, or `adapter_custom`. |
| `visibility` | yes | `public` or `private`. |
| `producer_phase` | yes | Adapter phase/subphase. |
| `required_for_replay` | yes | Boolean. |
| `redaction_policy` | yes | `none`, `scan`, `structured`, or `private_only`. |
| `schema_version` | yes | Artifact schema version. |

#### Failure Mapping Schema

| Field | Required | Notes |
|---|---|---|
| `failure_class` | yes | Central class. |
| `failure_code` | yes | Central public code. |
| `adapter_phase` | yes | Protocol phase. |
| `adapter_subphase` | yes | Adapter-local stable subphase. |
| `adapter_detail_code` | optional | Adapter-local private/detail code mapped to central code. |
| `public_message` | yes | Redacted message. |
| `private_diagnostics` | optional | Private diagnostics ref. |
| `health_impact` | yes | Central health impact mapping. |

#### Report Metadata Schema

| Field | Required | Notes |
|---|---|---|
| `score_fields` | yes | Public score fields and units. |
| `public_artifacts` | yes | Artifact ids safe for report rendering. |
| `summary_fields` | yes | Generic summary labels. |
| `warnings` | optional | Public warning labels. |
| `detail_sections` | optional | Adapter-declared report sections with public artifact refs only. |

### 8.8 Adapter Author Journey And Permitted Edit Set

The adapter author experience is a product surface. Phase 5 cannot close until
the generated scaffold and docs define this journey:

1. `xtask adapter scaffold --benchmark-id <id> --adapter-id <id>`
2. Implement only generated adapter-owned files.
3. Fill descriptor, capability, readiness, artifact, failure, replay, and
   report schemas.
4. Add fixture data under the generated fixture directory.
5. Run `xtask adapter-conformance --adapter <adapter_id>`.
6. Run generated selector route.
7. Produce official behavior proof if promoting to stable.
8. Run `/vs_review/` closure for stable promotion.

The scaffold must declare:

| Output | Purpose |
|---|---|
| adapter module | Adapter-owned implementation only. |
| descriptor file | Identity, capabilities, stability, docs links. |
| fixtures directory | Positive and negative conformance fixtures. |
| generated test module | Adapter-specific conformance invocation. |
| registry metadata row | Descriptor/binding data only. |
| docs stub | Adapter author/user notes. |
| review stub | Stable promotion review checklist. |

Permitted manual edits for a new adapter:

- generated adapter module
- generated descriptor/metadata row
- generated fixtures
- generated docs stub
- generated review checklist
- generated selector registration

Forbidden for a protocol-complete adapter proof:

- generic runner behavior files
- generic replay behavior files
- generic doctor behavior files
- generic report behavior files
- generic no-branch guard weakening
- central behavior switches keyed by concrete benchmark id

Phase 7 must include a diff proof that the third adapter did not modify
forbidden files.

### 8.9 Stable Promotion Evidence Schema

Stable promotion cannot be deferred until the final release phase. Phase 1 must
define the evidence schema; Phase 6 and Phase 7 must use it.

Required fields:

| Field | Required | Notes |
|---|---|---|
| `adapter_id` | yes | Adapter being promoted. |
| `adapter_version` | yes | Version under review. |
| `protocol_version` | yes | Protocol version. |
| `conformance_command` | yes | Exact command and output path. |
| `official_tool_name` | conditional | Required for benchmarks with official runner/evaluator. |
| `official_tool_version` | conditional | Required when official tool exists. |
| `official_command` | conditional | Exact preservation command. |
| `environment` | yes | OS/tools/Docker/network assumptions. |
| `artifact_archive` | yes | Path to archived proof artifacts. |
| `result_comparison` | conditional | Official vs HarnessLab result comparison. |
| `known_conditions` | optional | Explicit environment-gated caveats. |
| `review_report` | yes | `/vs_review/` path. |
| `status` | yes | `experimental`, `stable`, or `conditional-stable-blocked`. |

If official proof is environment-gated, the adapter cannot be marked `stable`;
it must remain `experimental` or `conditional-stable-blocked`.

### 8.10 No-branch Guard Contract

The static no-branch guard must be implemented before Phase 4 starts removing
branches and must be mandatory before Phase 7.

Forbidden patterns outside allowlisted paths:

- concrete benchmark ids such as `terminal-bench`, `swe-bench-pro`, or the
  third adapter id in behavior code
- `ExternalRunnerKind::` in new-path generic code
- `match benchmark_id.as_str()`
- `match adapter_id.as_str()`
- direct imports of adapter modules from generic runner/replay/doctor/report
  code
- central registry behavior closures keyed by benchmark id

Allowlisted paths:

- adapter-owned modules
- descriptor/metadata files
- generated fixtures
- tests that assert the guard catches forbidden examples
- named legacy shim
- docs and review artifacts

Guard tests must include positive examples and negative bypass fixtures.

### 8.11 Frozen Regression Selector Manifest

Phase 0 must produce a frozen selector manifest before migration starts. It must
list exact ids, commands, expected test counts, required artifacts, and owning
contract for:

- `ADAPT-DATA-*`
- `ADAPT-RUNTIME-*`
- `TB-*`
- `SWEPRO-*`
- `INT-*` selectors that cover external runtime, replay, redaction, report, and
  doctor behavior
- `SEC-*` public/private redaction selectors

Any migration phase that removes, renames, weakens, or changes expected counts
for a frozen selector must fail until the plan records an equivalent or stronger
replacement.

### 8.12 Registered Protocol Selector Plan

The `ADAPT-PROTOCOL-*` family must be registered before Phase 5 closes. Planned
rows are not sufficient for implementation closure; every active protocol proof
must exist in `tests/REQUIREMENTS.toml`, `tests/TEST_REGISTRY.toml`,
`scripts/test-after-change.sh`, and `xtask` claim validation.

| Selector | Status At Introduction | Required Proof |
|---|---|---|
| `ADAPT-PROTOCOL-001` | planned in Phase 1, active in Phase 2 | Descriptor, identity, and protocol authority schema validation. |
| `ADAPT-PROTOCOL-002` | planned in Phase 1, active in Phase 2 | Registry conflict and binding resolution validation. |
| `ADAPT-PROTOCOL-003` | planned in Phase 1, active in Phase 3 | Data lifecycle protocol contract foundation. |
| `ADAPT-PROTOCOL-004` | planned in Phase 1, active in Phase 3 | Runtime lifecycle, readiness, and failure taxonomy contract foundation. |
| `ADAPT-PROTOCOL-005` | planned in Phase 1, active in Phase 3 | Artifact declaration, public/private, and redaction conformance. |
| `ADAPT-PROTOCOL-006` | planned in Phase 1, active in Phase 4 | Replay authority old/new/mixed fixture conformance. |
| `ADAPT-PROTOCOL-007` | planned in Phase 1, active in Phase 4 | Generic doctor/readiness/report metadata conformance. |
| `ADAPT-PROTOCOL-008` | planned in Phase 1, active before Phase 4 exit | Static no-branch guard with bypass fixtures. |
| `ADAPT-PROTOCOL-009` | planned in Phase 1, active in Phase 5 | Scaffold golden path and generated adapter conformance. |
| `ADAPT-PROTOCOL-010` | planned in Phase 1, active in Phase 6 | Existing adapter migration preservation manifest. |
| `ADAPT-PROTOCOL-011` | planned in Phase 1, active in Phase 7 | Third-adapter horizontal extension proof and forbidden-diff guard. |
| `ADAPT-PROTOCOL-012` | planned in Phase 1, active in Phase 8 | Stable promotion evidence archive validation. |

## 9. Phase Gate Overview

| Phase | Name | Close Gate |
|---:|---|---|
| 0 | Discovery And Branch Inventory | Complete inventory of benchmark-specific branches and serialized compatibility risks. |
| 1 | Protocol Specification | Versioned protocol spec and acceptance contracts reviewed. |
| 2 | Identity And Registry Foundation | Opaque benchmark/adapter id registry replaces enum-first dispatch in new paths. |
| 3 | Unified Adapter Interface | Data/runtime/readiness/replay contracts are exposed through one adapter protocol. |
| 4 | Generic Upper-layer Refactor | Runner, replay, doctor, report consume protocol/capabilities instead of benchmark ids. |
| 5 | Conformance Suite And Scaffold | New adapter template plus conformance gates reject incomplete adapters. |
| 6 | Existing Adapter Migration | Terminal-Bench and SWE-bench Pro pass protocol gates with no behavior regression. |
| 7 | Horizontal Extension Proof | A third adapter passes conformance without generic-layer code changes. |
| 8 | Stable Governance And Release | Stable/experimental policy, docs, review, rollback, and post-release checks close. |

## 10. Phased Execution Plan

### Phase 0: Discovery And Branch Inventory

#### Objective

Build an exact current-state inventory of benchmark-specific coupling and
serialized compatibility risks before changing architecture.

#### Entry Criteria

- Current `main` is clean.
- Latest adapter blocker closure commit is present.
- Existing selector guard passes or any failure is recorded as a baseline issue.

#### Entry Criteria Checks

| Entry Criterion | Check Method | Evidence / Output | Owner |
|---|---|---|---|
| Clean working tree | `git status --short` | Empty output | Implementation owner |
| Latest closure present | `git log --oneline -3` | Includes adapter blocker closure and PRD commits | Implementation owner |
| Current selector baseline | `scripts/verify-planned-adapter-selectors.sh` | `active=16 planned=1` or recorded failure | Implementation owner |

#### Design Approach

Use source scanning and targeted manual review to classify each benchmark-aware
branch as one of:

- adapter-owned and acceptable
- registry metadata and acceptable
- generic-layer leak to remove
- legacy compatibility shim to isolate

#### Implementation Tasks

- Search for `ExternalRunnerKind`, concrete benchmark ids, adapter module names,
  and benchmark-specific event/report/replay wording.
- Create a branch inventory table with owner surface, reason, and migration
  disposition.
- Inventory serialized fields in `RunSpec`, `TaskPlan`, `ExternalRunnerSpec`,
  benchmark snapshots, task runtime snapshots, and external-runtime snapshots.
- Identify all tests that currently rely on enum-specific behavior.

#### Deliverables

- `docs/plans/...-phase-0-branch-inventory.md`
- Updated implementation plan section if discovery changes phase order.

#### Testing And Validation

| Validation Item | Method | Passing Standard |
|---|---|---|
| Branch inventory completeness | `rg` commands recorded in the inventory | Every match is triaged or explicitly excluded |
| Serialized compatibility inventory | Snapshot/model review | Every persisted benchmark-kind field has migration handling |
| Baseline tests | Existing selector guard | Baseline is green or failures are documented before changes |

#### Exit Criteria

- No untriaged benchmark-specific branch remains in non-adapter layers.
- Compatibility risks are classified before code changes begin.

#### Review Plan

- One architecture reviewer checks whether inventory misses hidden coupling.
- One test reviewer checks whether current selectors prove the baseline.

#### Risks And Fallback

| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| Hidden branch missed | Later genericity claims become false | New benchmark needs generic-layer code | Add static no-branch guards before refactor | Reopen Phase 0 and block Phase 7 |
| Serialized field migration underestimated | Replay breaks old runs | Replay tests fail after identity migration | Add compatibility fixtures early | Keep legacy shim isolated until migration complete |

#### Gate To Next Phase

Proceed only after the inventory has explicit dispositions for every match and
reviewers agree no blocker remains.

### Phase 1: Protocol Specification

#### Objective

Define the adapter protocol contract before changing code.

#### Entry Criteria

- Phase 0 inventory is complete.
- Open product questions have a default or approved assumption.

#### Entry Criteria Checks

| Entry Criterion | Check Method | Evidence / Output | Owner |
|---|---|---|---|
| Inventory complete | Phase 0 doc | All branch dispositions present | Implementation owner |
| Product assumptions explicit | PRD update or plan assumption table | No hidden launch-slice assumption | Product owner |

#### Design Approach

Write a versioned protocol spec that names the required interfaces, artifacts,
capabilities, failure mapping, and conformance evidence. Keep the protocol
small enough for current adapters but strict enough to prevent generic-layer
leaks.

#### Implementation Tasks

- Define `BenchmarkId`, `AdapterId`, `AdapterVersion`, and
  `AdapterProtocolVersion` semantics.
- Define `AdapterProtocolAuthority`, dual-write/dual-read rules, and old/new/
  mixed replay migration matrix.
- Define `TaskRuntimeBinding` as the only dispatch binding model.
- Define versioned readiness, artifact, failure, and report metadata schemas.
- Specify descriptor fields and capability flags.
- Specify data lifecycle operations and required determinism.
- Specify runtime lifecycle operations and allowed phase/subphase taxonomy.
- Specify artifact declaration, public/private boundaries, redaction refs, and
  replay material authority.
- Specify doctor/readiness checks as adapter-declared probes.
- Specify report-facing public artifact metadata.
- Define experimental vs stable adapter requirements.
- Define stable promotion evidence schema and conditional-stable-blocked status.
- Define the first `ADAPT-PROTOCOL-001..012` requirement/selector registry rows
  as planned gates, with exact future activation phases.

#### Deliverables

- `docs/adapter-protocol.md`
- Protocol acceptance matrix mapped to PRD `AC-001..AC-009`.
- Decision log entry for in-repo protocol isolation vs out-of-tree packaging.

#### Testing And Validation

| Validation Item | Method | Passing Standard |
|---|---|---|
| Protocol covers existing adapters | Cross-check Terminal-Bench and SWE-bench Pro | No existing required behavior lacks a protocol field |
| Protocol blocks generic-layer branching | Review against Phase 0 inventory | Every non-adapter branch has a protocol replacement |
| Evidence is measurable | Acceptance matrix | Each AC has a concrete test or review artifact |
| Review blockers addressed | Round 1 plan review response | Serialized authority, binding, schema, conformance, branch guard, and promotion evidence blockers have explicit plan coverage |

#### Exit Criteria

- Protocol spec is review-ready.
- Required and optional capabilities are separated.
- Existing adapters can be mapped without one-off exceptions.

#### Review Plan

- Run adversarial review of protocol spec before implementation.

#### Risks And Fallback

| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| Protocol too broad | Implementation stalls | Too many mandatory fields | Split required core from capability extensions | Keep advanced features experimental |
| Protocol too weak | Upper layers keep branching | Phase 4 cannot remove branch | Add capability/failure/artifact fields | Re-review Phase 1 before Phase 2 |

#### Gate To Next Phase

Proceed only when the protocol spec has no accepted blocking review findings.

### Phase 2: Identity And Registry Foundation

#### Objective

Introduce opaque benchmark and adapter ids as the primary new-path identity,
while isolating `ExternalRunnerKind` as legacy compatibility.

#### Entry Criteria

- Phase 1 protocol spec is review-accepted.
- Serialized compatibility strategy is approved.

#### Entry Criteria Checks

| Entry Criterion | Check Method | Evidence / Output | Owner |
|---|---|---|---|
| Protocol accepted | Review report | No open blockers | Implementation owner |
| Compatibility fixtures ready | Test fixture inventory | Old snapshot/run fixtures selected | Test owner |

#### Design Approach

Add protocol identity types in `harnesslab-core`, then migrate new registry and
task/runtime metadata to id-based fields. Avoid breaking old replay by creating
an explicit legacy mapping shim.

#### Implementation Tasks

- Add typed `BenchmarkId`, `AdapterId`, `AdapterVersion`, and protocol version
  model.
- Add descriptor registry keyed by `BenchmarkId`.
- Add runtime registry keyed by `adapter_id` from `TaskRuntimeBinding`.
- Add binding resolution that maps CLI-selected `benchmark_id` and mode to one
  unambiguous `TaskRuntimeBinding`.
- Add explicit legacy mapping from `ExternalRunnerKind` to protocol ids.
- Update snapshots to include protocol identity fields.
- Add tests that new-path dispatch does not require `ExternalRunnerKind`.
- Add tests that legacy snapshots remain handled only through the shim.
- Add old-only, new-only, dual-written, and mixed-authority replay fixtures.
- Add duplicate/conflicting registry fixture tests.

#### Current Phase 2 Landing Notes

- Landed: validated protocol identity newtypes, nested
  `TaskRuntimeBinding.authority`, built-in protocol registry validation,
  explicit legacy shim authority, and active/frozen `ADAPT-PROTOCOL-001/002`.
- Landed: production Terminal-Bench and SWE-bench Pro plans dual-write
  `runtime_binding`; preflight, execute, cleanup discovery, internal-error
  snapshot writing, and replay authority checks resolve through the task's
  protocol binding when present.
- Landed: runtime observability includes protocol adapter id, protocol version,
  benchmark id, selected mode, stability, capabilities, and `legacy_shim_used`.
- Landed: external-runtime private/public fingerprints include
  `protocol_authority`, and replay fails closed on mismatched protocol authority
  or adapter version drift.
- Deliberately not claimed as complete in Phase 2: full artifact declaration,
  redaction policy, detailed failure taxonomy, readiness probe content, generic
  report metadata, and no-branch upper-layer enforcement. Those remain assigned
  to `ADAPT-PROTOCOL-005/006/007/008/010/012` in later phases.

#### Deliverables

- Core identity model.
- Protocol registry.
- Legacy compatibility shim.
- Identity migration tests.

#### Testing And Validation

| Validation Item | Method | Passing Standard |
|---|---|---|
| New ids serialize | Core model tests | Round-trip stable ids and versions |
| Legacy compatibility bounded | Replay fixture tests | Old runs pass or fail with explicit migration reason |
| No enum-first new path | Static guard | New protocol registry does not match on concrete benchmark ids outside shim |
| Mixed authority fails closed | Replay fixture tests | Mixed old/new snapshot sets produce explicit protocol authority blocker |
| Binding conflict rejected | Registry tests | Duplicate or ambiguous adapter bindings fail before runtime |

#### Exit Criteria

- New adapter identity is not enum-first.
- Old identity use is isolated and documented.

#### Review Plan

- Code review focused on serialization, replay compatibility, and migration
  boundaries.

#### Risks And Fallback

| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| Old replay breaks | User cannot replay prior runs | Replay fixtures fail | Keep legacy fields and shim | Delay removal of enum fields |
| New id aliases conflict | Ambiguous adapter dispatch | Registry duplicate test fails | Enforce canonical ids | Block adapter registration |

#### Gate To Next Phase

Proceed only when new identity tests and legacy replay compatibility tests pass.

### Phase 3: Unified Adapter Interface

#### Objective

Expose data, runtime, readiness, replay, artifact, and report contracts through
one protocol surface.

#### Entry Criteria

- Phase 2 registry and identity are green.
- Current data/runtime adapter traits are mapped to the protocol spec.

#### Entry Criteria Checks

| Entry Criterion | Check Method | Evidence / Output | Owner |
|---|---|---|---|
| Identity registry green | Phase 2 tests | All identity tests pass | Implementation owner |
| Trait mapping complete | Mapping table | Current adapter methods mapped or deprecated | Implementation owner |

#### Design Approach

Keep the implementation modular but make adapter ownership explicit. The public
protocol may expose one adapter descriptor with subcontracts rather than one
large trait. The key requirement is that generic layers call protocol methods,
not concrete benchmark modules.

#### Implementation Tasks

- Define protocol-facing adapter descriptor and subcontracts. **Started:** the
  adapter crate now exposes `ProtocolAdapterDescriptor` plus adapter-owned data
  lifecycle, runtime lifecycle, readiness probe, and central failure mapping
  records for built-in adapters.
- Move runtime compatibility checks into adapter-declared readiness/capability
  methods.
- Implement versioned readiness probe records and generic readiness event fields.
- Make runtime preflight consume adapter readiness results.
- Make cleanup target discovery adapter-owned without enum dispatch.
- Make failure mapping return central failure class/code plus adapter-local
  private diagnostics.
- Make artifact contracts adapter-declared before runtime writes.
- Make report metadata derive from adapter public artifact declarations.
- Enforce artifact declaration before public artifact exposure.
- Add adapter-local detail code mapping to central public failure codes.

#### Deliverables

- Protocol adapter interface. **Started:** `ADAPT-PROTOCOL-003/004` validate
  built-in descriptors through protocol conformance records.
- Adapter readiness contract. **Started:** runtime conformance requires
  readiness probes for declared readiness capabilities.
- Artifact declaration contract. **Started:** `ADAPT-PROTOCOL-005` validates
  adapter-owned artifact declarations, public/private runtime snapshot pairing,
  redaction policy, safe attempt-relative paths, and report public artifact
  references.
- Failure taxonomy contract. **Started:** runtime conformance rejects empty,
  duplicate, or non-failure mappings.

#### Testing And Validation

| Validation Item | Method | Passing Standard |
|---|---|---|
| Existing adapters implement protocol | Compile and selector tests | Terminal-Bench and SWE-bench Pro still run through protocol |
| Readiness no longer hard-coded | Static guard | `runtime_compatibility` has no concrete benchmark branch outside adapter-owned code |
| Artifact declaration enforced | Conformance tests | Descriptor cannot omit required public/private artifact declarations |
| Report metadata generic | Report contract tests | Report renders only declared public metadata/artifacts |
| Failure schema enforced | Negative fixture tests | Unclassified adapter failures fail conformance |

#### Current Phase 3 Landing Notes

- Landed: `ProtocolAdapterDescriptor` is exposed through adapter instances and
  concentrates built-in adapter data/runtime/readiness/failure subcontracts
  behind protocol metadata.
- Landed: `ADAPT-PROTOCOL-003` is active and validates data lifecycle operation
  coverage, binding/descriptor alignment, declared capability ownership, and
  negative mismatch cases.
- Landed: `ADAPT-PROTOCOL-004` is active and validates runtime lifecycle
  operation coverage, cleanup capability coupling, capability-specific
  readiness probe coverage, and central failure mapping schema fields.
- Landed: `ADAPT-PROTOCOL-005` is active and validates artifact declaration,
  public/private boundary, redaction policy, capability-required artifact
  families, and report public artifact references for built-in protocol
  adapters.
- Not yet landed: generic report metadata consumption, doctor readiness
  consumption, live runtime artifact reconciliation proof, and static no-branch
  enforcement.

#### Exit Criteria

- Generic runtime code calls protocol interfaces only.
- Adapter-specific readiness and artifact behavior lives in adapter-owned code.

#### Review Plan

- Architecture review focused on whether subcontracts are cohesive and not a
  disguised concrete-benchmark switch.

#### Risks And Fallback

| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| Interface becomes too large | Adapter authors face high cost | Sample adapter implementation is noisy | Split core and capability traits | Keep optional capability extensions |
| Readiness diagnostics degrade | Doctor output loses remediation | Doctor tests fail or messages become generic | Require adapter-declared remediation text | Keep old message as adapter metadata |

#### Gate To Next Phase

Proceed only when existing adapter behavior remains green and no generic-layer
benchmark branch is required for readiness/runtime/artifacts.

### Phase 4: Generic Upper-layer Refactor

#### Objective

Make runner, replay, doctor, report, selector routing, and registry validation
consume the protocol rather than concrete benchmark identities.

#### Entry Criteria

- Phase 3 protocol interfaces are implemented.
- Static branch inventory has target replacements.

#### Entry Criteria Checks

| Entry Criterion | Check Method | Evidence / Output | Owner |
|---|---|---|---|
| Protocol interfaces green | Phase 3 tests | Existing selectors pass | Implementation owner |
| Branch replacements mapped | Phase 0/3 docs | No unknown branch disposition | Implementation owner |

#### Design Approach

Refactor one upper layer at a time, preserving tests. The no-branch guard must
exist before this phase removes branches, then become mandatory before this
phase exits.

#### Implementation Tasks

- Runner: dispatch by protocol registry and task runtime binding.
- Replay: validate adapter id/version and declared replay materials.
- Doctor: collect readiness checks from adapter descriptors.
- Report: render adapter-declared public artifacts and score metadata.
- Selector registry: route conformance and adapter proof selectors by protocol
  metadata.
- Static guards: reject concrete benchmark branches in non-adapter code.
- Gate artifacts: write branch-guard result, selector inventory result, and
  protocol dispatch event field checks.

#### Deliverables

- Generic runner dispatch.
- Generic replay validation.
- Generic doctor/readiness integration.
- Generic report artifact integration.
- Static no-branch guard.

#### Testing And Validation

| Validation Item | Method | Passing Standard |
|---|---|---|
| Runner genericity | New protocol selector | New adapter id can run without generic branch |
| Replay genericity | Replay tests | Adapter id/version/material checks are protocol-based |
| Doctor genericity | Doctor tests | Readiness output comes from adapter metadata |
| Report genericity | Report tests | Public artifacts render from adapter declarations |
| No branch guard | `xtask` static check | Concrete benchmark ids absent outside allowlisted adapter/legacy files |
| Branch guard bypass tests | Negative fixtures | Intentional forbidden branches are rejected |
| Protocol dispatch observability | Event assertions | Dispatch events include benchmark id, adapter id, adapter version, protocol version, selected mode, capability, and legacy-shim usage |

#### Exit Criteria

- Non-adapter layers are benchmark-agnostic by automated guard.
- Existing selectors remain green.

#### Review Plan

- Code review plus adversarial test review focused on false genericity claims.

#### Risks And Fallback

| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| Static guard too brittle | Blocks legitimate metadata | Guard fails on docs/tests | Allowlist metadata files explicitly | Refine guard, do not remove it |
| Replay policy too strict | Valid replay blocked | Replay fixture fails | Add protocol-declared compatibility policy | Keep explicit degraded path only if user-visible |

#### Gate To Next Phase

Proceed only when no-branch guard and migrated upper-layer tests pass.

### Phase 5: Conformance Suite And Scaffold

#### Objective

Provide the standard tools that let adapter authors know when their benchmark
is protocol-complete.

#### Entry Criteria

- Generic upper-layer refactor is green.
- Protocol acceptance matrix is stable.

#### Entry Criteria Checks

| Entry Criterion | Check Method | Evidence / Output | Owner |
|---|---|---|---|
| Upper layers generic | Phase 4 guard | No branch guard passes | Implementation owner |
| Acceptance matrix stable | Docs review | AC to test mapping complete | Test owner |

#### Design Approach

Create an adapter conformance harness that can be run against any registered
adapter. It should test protocol behavior, not implementation internals.

#### Implementation Tasks

- Add `xtask adapter-conformance --adapter <id>`.
- Add descriptor schema validation.
- Add deterministic data lifecycle tests.
- Add runtime failure injection tests.
- Add public/private artifact redaction tests.
- Add replay material authority tests.
- Add doctor/readiness contract tests.
- Add report artifact contract tests.
- Add scaffold template for new adapter modules.
- Add docs for adapter authors.
- Register and route `ADAPT-PROTOCOL-001..012` across requirements, test
  registry, `scripts/test-after-change.sh`, and `xtask` claim validation.
- Add positive and negative fixtures with exact expected failure messages.
- Add golden scaffold test that generated adapter compiles and runs
  conformance.

#### Deliverables

- Conformance CLI.
- Adapter scaffold template.
- Adapter author guide.
- Conformance registry entries.

#### Testing And Validation

| Validation Item | Method | Passing Standard |
|---|---|---|
| Incomplete adapter rejected | Negative fixtures | Missing descriptor/lifecycle/artifact fields fail before run |
| Complete adapter accepted | Positive fixture | Sample complete fixture passes conformance |
| Redaction enforced | Fake secret fixture | No fake secret appears in public outputs |
| Replay enforced | Drift fixture | Missing/drifted materials block replay |
| Protocol selectors enforceable | Registry/xtask checks | `ADAPT-PROTOCOL-*` rows are active or planned according to phase and cannot be silently omitted |
| Scaffold author path works | Golden scaffold test | Generated adapter compiles and reports conformance status |

#### Exit Criteria

- A developer can run one command to validate adapter protocol completeness.
- Negative fixtures prove the suite rejects common incomplete adapters.

#### Review Plan

- Test-engineering review focused on whether conformance proves behavior or only
  checks superficial fields.

#### Risks And Fallback

| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| Conformance too shallow | Bad adapters pass | Third adapter needs generic-layer branch | Add black-box runtime fixtures | Block stable promotion |
| Scaffold encourages copy-paste bugs | New adapters inherit mistakes | Repeated adapter failures | Include minimal and full templates | Keep examples reviewed and tested |

#### Gate To Next Phase

Proceed only when conformance has positive and negative fixtures and passes
review.

### Phase 6: Existing Adapter Migration

#### Objective

Migrate Terminal-Bench and SWE-bench Pro onto the universal protocol while
preserving current behavior and evidence.

#### Entry Criteria

- Conformance suite exists.
- Terminal-Bench and SWE-bench Pro migration checklist is ready.

#### Entry Criteria Checks

| Entry Criterion | Check Method | Evidence / Output | Owner |
|---|---|---|---|
| Conformance suite green | Phase 5 command | Positive/negative fixtures pass | Test owner |
| Migration checklist ready | Adapter-specific checklist | Current selectors mapped to protocol gates | Implementation owner |

#### Design Approach

Migrate one existing adapter at a time. Do not start by deleting legacy paths;
first make the protocol path pass existing selectors, then retire duplicated
legacy code.

#### Implementation Tasks

- Terminal-Bench: migrate descriptor, data lifecycle, runtime, cleanup, events,
  snapshots, replay, readiness, and report declarations.
- SWE-bench Pro: migrate metadata/workspace/patch/evaluator phases, replay
  material authority, fake-tool proof, and Docker-gated official proof wording.
- Update existing `ADAPT-*`, `TB-*`, `SWEPRO-*`, `INT-*`, and `SEC-*` mappings
  to protocol selectors.
- Preserve the frozen regression selector manifest and record any replacement
  as equivalent or stronger before changing selector status.
- Produce stable-promotion evidence records for existing adapters, or leave
  them explicitly experimental/conditional if official proof is missing.
- Remove or isolate legacy enum dispatch after protocol selectors pass.
- Preserve public/private diagnostics and redaction behavior.

#### Deliverables

- Protocol-complete Terminal-Bench adapter.
- Protocol-complete SWE-bench Pro adapter.
- Updated selectors and registry entries.
- Migration evidence report.

#### Testing And Validation

| Validation Item | Method | Passing Standard |
|---|---|---|
| Terminal-Bench behavior | Existing TB/ADAPT selectors | No regression |
| SWE fake-tool behavior | `SWEPRO-001..005` | No regression |
| Runtime snapshots | `ADAPT-RUNTIME-003..006` | No regression |
| Redaction | Existing `SEC-*` plus conformance | No public fake secrets |
| Replay | Replay selectors | No silent live replanning |
| Frozen selector preservation | Selector manifest guard | No required behavior selector is removed, weakened, or silently replaced |
| Promotion evidence | Evidence archive validator | Stable/experimental/conditional status matches official proof availability |

#### Exit Criteria

- Both existing external benchmarks are protocol-complete.
- Legacy benchmark-specific non-adapter paths are removed or isolated.

#### Review Plan

- Independent code and test review after each adapter migration.

#### Risks And Fallback

| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| Behavior regression | Existing benchmark runs fail | Existing selector failure | Migrate one adapter at a time | Revert adapter migration slice |
| SWE official proof still gated | Full stable claim blocked | Docker unavailable | Keep explicit conditional status | Record proof in Docker-capable environment later |

#### Gate To Next Phase

Proceed only after both existing adapters pass conformance and existing
behavior selectors.

### Phase 7: Horizontal Extension Proof

#### Objective

Prove that a new benchmark can be added by implementing the protocol without
changing generic upper layers.

#### Entry Criteria

- Existing adapters are protocol-complete.
- No-branch guard is active.
- Scaffold and conformance docs are available.

#### Entry Criteria Checks

| Entry Criterion | Check Method | Evidence / Output | Owner |
|---|---|---|---|
| Existing adapters stable | Phase 6 selectors | All required selectors pass | Implementation owner |
| No-branch guard active | `xtask` guard | Guard passes before adding new adapter | Test owner |
| Scaffold ready | Adapter author guide | New adapter steps documented | Implementation owner |

#### Design Approach

Add a deterministic third benchmark adapter with intentionally different
semantics from existing adapters. The point is not benchmark value; the point is
extension proof. The adapter should exercise data readiness, runtime execution,
scoring, artifacts, replay, doctor, report, and failure classification.

#### Implementation Tasks

- Add sample third benchmark adapter under adapter-owned scope.
- Register descriptor metadata.
- Add conformance selector for the new adapter.
- Run generic run/replay/doctor/report flows.
- Verify no generic-layer file changed except registry metadata and tests.
- Add negative test proving a missing adapter method fails conformance.
- Exercise readiness, runtime failure injection, public/private artifacts,
  replay materials, report rendering, and adapter-local failure mapping.
- Produce forbidden-diff proof for generic runner/replay/doctor/report behavior
  files.

#### Deliverables

- Sample third adapter.
- `ADAPT-PROTOCOL-*` proof selectors.
- Horizontal extension evidence report.

#### Testing And Validation

| Validation Item | Method | Passing Standard |
|---|---|---|
| No generic code changes | Diff/guard | Generic runner/replay/doctor/report untouched except registry metadata if required |
| Third adapter runs | CLI integration test | Run completes through generic dispatch |
| Third adapter replays | Replay integration test | Replay validates protocol materials |
| Third adapter reports | Report smoke test | Public artifacts render generically |
| Third adapter diagnoses | Doctor/readiness test | Readiness is adapter-declared |
| Third adapter is nontrivial | Conformance fixtures | Exercises readiness, runtime failure, replay drift, report rendering, and public/private artifact boundaries |
| Forbidden diff proof | Diff guard | Generic behavior files are unchanged except allowed registry metadata/tests |

#### Exit Criteria

- A new adapter proves horizontal extension.
- The evidence directly satisfies PRD `AC-008`.

#### Review Plan

- Adversarial review focused on whether the sample adapter cheated by relying
  on hidden upper-layer changes.

#### Risks And Fallback

| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| Sample too simple | Does not prove real extension | Review rejects proof | Include replay, failure, report, and doctor coverage | Add a real third benchmark proof |
| Registry metadata still too invasive | Extension not seamless | Generic code changes required | Move more behavior into descriptor/capabilities | Reopen Phase 4 |

#### Gate To Next Phase

Proceed only when third-adapter proof passes and review finds no accepted
blocking genericity issue.

### Phase 8: Stable Governance And Release

#### Objective

Make the protocol operationally maintainable for future benchmark adapters.

#### Entry Criteria

- Horizontal extension proof is accepted.
- Existing adapters and sample adapter pass conformance.

#### Entry Criteria Checks

| Entry Criterion | Check Method | Evidence / Output | Owner |
|---|---|---|---|
| Extension proof accepted | Phase 7 review | No accepted blockers | Implementation owner |
| Conformance green | Full protocol gate | Existing and sample adapters pass | Test owner |

#### Design Approach

Define stable/experimental governance, release notes, rollback policy,
post-release monitoring, and adapter author documentation. Treat conformance as
the release gate, not as an optional test.

#### Implementation Tasks

- Define stable vs experimental adapter states.
- Add adapter promotion checklist.
- Add docs for implementing, validating, releasing, and debugging adapters.
- Add release rollback plan for protocol migration.
- Add post-release checks for run/replay/report/doctor flows.
- Run final adversarial review.

#### Deliverables

- Adapter author guide.
- Stable promotion checklist.
- Release and rollback notes.
- Final `/vs_review/` closure report.

#### Testing And Validation

| Validation Item | Method | Passing Standard |
|---|---|---|
| Full protocol gate | `scripts/test-after-change.sh` plus protocol conformance | All required gates pass |
| Docs accuracy | Doc review | Docs do not claim unsupported plugin/marketplace behavior |
| Rollback readiness | Rollback review | Clear path to revert protocol migration or disable unstable adapter |
| Final review | Fresh adversarial review | No accepted blockers remain |

#### Exit Criteria

- Protocol is ready to use as the standard path for future benchmark adapters.
- Remaining limitations are documented and not contradicted by release language.

#### Review Plan

- Fresh architecture, test, and operations review.

#### Risks And Fallback

| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| Release overclaims maturity | Future adapter authors hit hidden constraints | Review/docs mismatch | Explicit stable/experimental wording | Keep protocol in experimental |
| Rollback too hard | Regression blocks users | Main run/replay path fails | Feature flag or compatibility shim | Revert protocol migration commit series |

#### Gate To Close

Close only when final review passes, full gates pass, and all accepted blockers
have a fresh closure review.

## 11. Dependency Table

| Dependency | Type | Current Status | Blocking Risk | Handling Plan |
|---|---|---|---|---|
| Existing adapter selectors | test | Ready | Regression if selectors are weakened | Keep current selectors as required gates throughout migration |
| Docker-capable SWE official evaluator environment | environment | Unknown/currently unavailable locally | Full SWE stable proof cannot close | Keep Docker-gated proof explicit until environment is available |
| Adapter author UX expectations | person/product | Partially known | Scaffold may optimize wrong workflow | Validate with PRD review before Phase 5 |
| Legacy replay fixtures | data/test | Unknown | Migration can break prior runs silently | Build fixture inventory in Phase 0 |
| Static no-branch guard | tooling | Needs implementation | Genericity claim can become unenforced | Add in Phase 4 before third-adapter proof |

## 12. API And Compatibility Strategy

- Keep legacy `ExternalRunnerKind` readable until old run replay policy is
  explicitly resolved.
- New protocol snapshots must include `benchmark_id`, `adapter_id`,
  `adapter_version`, `protocol_version`, `selected_mode`, and `capabilities`.
- Replay must use `AdapterProtocolAuthority` as canonical whenever any protocol
  authority is present.
- Replay must reject mixed old/new authority sets unless every required
  protocol authority field is present and consistent.
- Legacy replay may map old enum variants only when the source run is old-only
  and the mapping goes through the named legacy shim.
- New adapters must not add new `ExternalRunnerKind` variants as the primary
  dispatch mechanism.
- Any compatibility fallback must emit explicit warning or blocker text; no
  silent live replanning.

## 13. Security And Data Handling Strategy

- Public/private artifact boundaries must be adapter-declared.
- Adapter-declared public artifacts must pass fake-secret redaction tests.
- Private diagnostics may contain command paths, source paths, and raw adapter
  errors; public diagnostics must contain structured labels only.
- Adapters must declare environment requirements without exposing secrets.
- Doctor output must avoid private paths unless JSON/private mode explicitly
  permits them.

## 14. Testing And Validation Strategy

Core gates:

- Existing `ADAPT-DATA-*`, `ADAPT-RUNTIME-*`, `TB-*`, `SWEPRO-*`, `INT-*`, and
  `SEC-*` selectors remain required.
- New `ADAPT-PROTOCOL-*` selectors must be registered in requirements, test
  registry, selector routing, and `xtask` claim validation before they can be
  used as closure evidence.
- Static no-branch guard rejects benchmark-specific generic-layer code.
- Negative fixtures prove incomplete adapters fail before runtime.
- Third-adapter proof validates horizontal extension.

Minimum new selector families:

| Selector Family | Purpose |
|---|---|
| `ADAPT-PROTOCOL-001` | Descriptor, identity, and protocol authority schema validation |
| `ADAPT-PROTOCOL-002` | Registry conflict and binding resolution validation |
| `ADAPT-PROTOCOL-003` | Data lifecycle protocol contract foundation |
| `ADAPT-PROTOCOL-004` | Runtime lifecycle, readiness, and failure taxonomy contract foundation |
| `ADAPT-PROTOCOL-005` | Artifact declaration, public/private boundary, and redaction conformance |
| `ADAPT-PROTOCOL-006` | Replay authority old/new/mixed fixture conformance |
| `ADAPT-PROTOCOL-007` | Generic doctor/readiness/report metadata conformance |
| `ADAPT-PROTOCOL-008` | Static no-branch guard with bypass fixtures |
| `ADAPT-PROTOCOL-009` | Scaffold golden path and generated adapter conformance |
| `ADAPT-PROTOCOL-010` | Existing adapter migration preservation manifest |
| `ADAPT-PROTOCOL-011` | Third-adapter horizontal extension proof and forbidden-diff guard |
| `ADAPT-PROTOCOL-012` | Stable promotion evidence archive validation |

## 15. Release, Rollback, And Fallback Strategy

- Release in slices matching phases.
- Commit each phase separately with green targeted gates.
- Keep legacy shim until protocol replay fixtures and old-run policy are
  accepted.
- If a phase regresses existing benchmark behavior, revert that phase commit
  rather than adding benchmark-specific fallback branches.
- If the universal protocol cannot support an existing benchmark behavior, stop
  and revise protocol capabilities before continuing.
- If the third-adapter proof requires generic-layer changes, treat that as a
  blocker and reopen Phase 4.

## 16. Observability And Success Metrics

| Metric | Target |
|---|---|
| Generic-layer benchmark id branches | Zero outside adapter registry metadata and legacy shim |
| Protocol conformance selectors | All pass for stable adapters |
| Third-adapter generic code changes | Zero generic runner/replay/doctor/report behavior changes |
| Public artifact redaction failures | Zero |
| Replay silent replanning | Zero |
| Adapter readiness blockers | Actionable message with remediation for every blocked required check |

## 17. Open Questions

- Should Phase 7 use a deterministic toy benchmark only, or also a real third
  benchmark?
- Should out-of-tree adapter packaging be introduced after Phase 8 or deferred
  until a separate marketplace/plugin requirement exists?
- Should stable adapters be required to map all failures to central failure
  codes, or may they expose adapter-local public detail codes?
- What is the deprecation policy for old runs that only contain
  `ExternalRunnerKind` and no protocol identity?

## 18. Decision Log

| Decision | Status | Rationale | Revisit Trigger |
|---|---|---|---|
| Protocol isolation is the target | Accepted | User clarified that adapter cost is acceptable, but must stay inside adapter layer. | User requests marketplace or untrusted adapter execution. |
| Start with in-repo protocol isolation | Proposed | Proves genericity without plugin distribution complexity. | User requires third-party out-of-tree adapters immediately. |
| Treat `ExternalRunnerKind` as legacy after migration | Proposed | Closed enum dispatch prevents horizontal extension. | Serialization compatibility blocks migration. |
| Add a third-adapter proof | Proposed | Directly proves PRD `AC-008`. | User chooses a real benchmark proof instead. |

## 19. Plan Quality Checklist

- Problem, goals, non-goals, assumptions, and risks are separated.
- Every phase has entry criteria, tasks, validation, exit criteria, review, and
  fallback.
- High-risk validation is moved early through branch inventory and protocol
  review.
- Existing benchmark behavior preservation remains a required gate.
- Genericity is backed by static guards and third-adapter proof, not just
  documentation.
- Rollback and legacy compatibility are explicit.
- Open questions are stated rather than hidden.

## 20. Change Log

| Version | Date | Change |
|---|---|---|
| 0.1 | 2026-06-08 | Initial full implementation plan for universal benchmark adapter protocol. |
| 0.2 | 2026-06-08 | Accepted Round 1 adversarial review blockers and added canonical authority migration matrix, concrete `TaskRuntimeBinding`, versioned readiness/artifact/failure/report schemas, adapter-author journey, stable promotion evidence schema, no-branch guard contract, frozen regression selector manifest, and registered `ADAPT-PROTOCOL-001..012` selector plan. |
| 0.3 | 2026-06-08 | Started implementation with Phase 0 branch inventory and frozen selector manifest deliverables; Phase 0 review is pending before Phase 1 protocol specification work. |
| 0.4 | 2026-06-08 | Added Phase 0 machine-enforced frozen selector lockfile and `xtask` guard; expanded inventory for integrations, serialized authority fields, registry dispatch classification, and readiness-owned label references. |
| 0.5 | 2026-06-08 | Hardened Phase 0 frozen guard with independent selector id baseline, exact router case locking, and execution file content hashes for the shared selector executor and external proof scripts. |
| 0.6 | 2026-06-08 | Started Phase 1 protocol specification with `docs/adapter-protocol.md` and registered `ADAPT-PROTOCOL-001..012` as planned requirement/test/selector gates. |
| 0.7 | 2026-06-08 | Started Phase 2 implementation by adding protocol identity/authority types, built-in adapter protocol registry binding validation, and active `ADAPT-PROTOCOL-001/002` selector routes. |
| 0.8 | 2026-06-11 | Extended Phase 2 implementation into runtime/replay authority binding: production adapters dual-write `TaskRuntimeBinding`, CLI preflight/execute/cleanup resolve by protocol `adapter_id`, external-runtime snapshots fingerprint `protocol_authority`, and `ADAPT-PROTOCOL-002` documentation is scoped to current registry validation while later artifact/redaction/report gates remain explicit. |
| 0.9 | 2026-06-11 | Closed Phase 2 adversarial review after Round 5: accepted blockers from prior rounds are fixed and validated; remaining items are non-blocking Phase 3/4 work such as SWE protocol-only positive coverage and cleanup/report/doctor branch removal. |
| 1.0 | 2026-06-11 | Started Phase 3 by adding protocol-facing adapter subcontracts and activating `ADAPT-PROTOCOL-003/004` for data lifecycle, runtime lifecycle, readiness probe, and central failure mapping contract foundation. |
| 1.1 | 2026-06-11 | Extended Phase 3 by adding adapter-owned artifact/report declaration contracts and activating `ADAPT-PROTOCOL-005` for public/private artifact boundary and redaction policy validation. |
