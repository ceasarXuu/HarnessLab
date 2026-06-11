# Subagent VS Review: Universal Benchmark Adapter Protocol Phase 2

- Created: 2026-06-08T02:16:58+0800
- Updated: 2026-06-11T22:10:56+0800
- Report schema: adversarial-v1
- Task: Make the benchmark adapter layer a universal protocol component so new benchmarks can be integrated behind adapter-owned contracts without upper layers branching on benchmark identity.
- Report path: `vs_review/2026-06-08-universal-benchmark-adapter-protocol-phase-2-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: Phase 2 Identity And Registry Foundation

### Review Input

#### Objective

Verify that Phase 2 implementation is real code/proof work, not just planning documentation: protocol identity, canonical authority schema, built-in adapter binding registry validation, and active selector gates for `ADAPT-PROTOCOL-001/002`.

#### Review Target

Code implementation, selector routing, registry metadata, documentation state, and validation evidence for the Phase 2 identity and registry foundation slice.

#### Target Locations

- `crates/harnesslab-core/src/adapter_protocol.rs`
- `crates/harnesslab-core/src/lib.rs`
- `crates/harnesslab-adapters/src/protocol_registry.rs`
- `crates/harnesslab-adapters/src/lib.rs`
- `scripts/test-after-change.sh`
- `scripts/verify-planned-adapter-selectors.sh`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `tests/FROZEN_SELECTOR_MANIFEST.toml`
- `xtask/src/adapter_claims.rs`
- `docs/adapter-protocol.md`
- `docs/plans/2026-06-04-benchmark-adapter-phase-1-boundary.md`
- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`

#### Change Introduction

The implementation adds core protocol identity newtypes, `AdapterProtocolAuthority`, `TaskRuntimeBinding`, a legacy runner-kind authority shim, and an adapter-side `AdapterRegistry` with built-in binding validation. It activates `ADAPT-PROTOCOL-001` and `ADAPT-PROTOCOL-002` from planned proof placeholders into exact selected cargo tests, updates claim validation expectations, and refreshes the frozen selector manifest.

#### Risk Focus

- Protocol ids may be under-specified or accept invalid values that make future horizontal benchmark extension ambiguous.
- Registry validation may not actually prevent binding conflicts, mode/capability mismatches, or legacy compatibility drift.
- Selector activation may be self-deceptive if tests only check the newly written implementation internals.
- The new protocol registry may still leave benchmark-specific branch behavior outside the adapter layer.
- Documentation may overclaim completion beyond Phase 2 identity/registry foundation.

#### Assumptions To Attack

- A new benchmark can be represented by protocol ids and binding metadata without changing upper-layer benchmark dispatch.
- Legacy `ExternalRunnerKind` can be safely mapped to protocol authority during migration.
- `ADAPT-PROTOCOL-001/002` prove actual implementation behavior and not only doc/status changes.
- Adding `protocol_registry.rs` remains covered by adapter boundary checks.
- The frozen selector manifest still prevents silent route/file-pattern drift.

#### Adversarial Lenses

- architecture
- implementation
- testing
- maintenance
- compatibility

#### Verification Status

- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-001`: passed.
- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-002`: passed.
- `CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh`: passed with `active=18 planned=11`.
- `scripts/verify-test-registry.sh`: passed.
- `cargo test -p xtask adapter_claims -- --nocapture`: passed.
- `cargo check -p harnesslab-core -p harnesslab-adapters`: passed.
- `cargo check -p xtask`: passed.
- `cargo fmt --all --check`: passed after formatting.
- `git diff --check`: passed.
- Line counts: core protocol 229, adapter protocol registry 231, old registry 250, xtask adapter claims 435.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Try to falsify the implementation and validation claims.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| complex | 12 minutes | one bounded 6 minute extension only if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-adversary | Protocol boundary and benchmark extension are the central product risk. | abstraction, module boundaries, future extension |
| implementation-adversary | New Rust types, registry validation, and legacy shims can fail under invalid or mixed states. | correctness, compatibility, error handling |
| test-validity-adversary | The main claim depends on selector activation and meta-test coverage. | self-deceptive tests, gate drift, missing failure paths |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-adversary | `multi_agent_v1.spawn_agent` | `019ea34e-d0eb-7a61-893c-71fc1b88a60a` | spawn result nickname `Ptolemy` | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| implementation-adversary | `multi_agent_v1.spawn_agent` | `019ea34e-f63c-7831-bb23-31c4717eebce` | spawn result nickname `Epicurus` | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| test-validity-adversary | `multi_agent_v1.spawn_agent` | `019ea34f-15ca-71b1-96f2-4e45a1ab7ba7` | spawn result nickname `Lorentz` | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| architecture-output | architecture-adversary | 1 | `019ea34e-d0eb-7a61-893c-71fc1b88a60a` | 12 minutes | completed | reviewer returned structured findings | completed |
| implementation-output | implementation-adversary | 1 | `019ea34e-f63c-7831-bb23-31c4717eebce` | 12 minutes | completed | reviewer returned structured findings | completed |
| test-validity-output | test-validity-adversary | 1 | `019ea34f-15ca-71b1-96f2-4e45a1ab7ba7` | 12 minutes | completed | reviewer returned structured findings | completed |

### Reviewer Outputs

#### architecture-output

##### Summary

Phase 2 has real code for identity newtypes and a metadata registry, but it
does not yet establish a horizontally extensible runtime protocol foundation.
The live production path still serializes and dispatches by `ExternalRunnerKind`,
the new `TaskRuntimeBinding` is unused outside declaration/docs/tests, and
registry validation is materially weaker than the protocol spec claims.

##### Blocking Findings

- Live dispatch is still enum-first, not protocol-binding-first.
  - Broken assumption: new benchmarks can route through `TaskRuntimeBinding.adapter_id` without upper-layer benchmark branching.
  - Failure scenario: a third benchmark is added to the protocol registry, but `TaskPlan.external_runner.kind` cannot represent it and `runtime_adapter_for` cannot dispatch it without adding a new enum variant and match arm.
  - Trigger condition: new benchmark integration after Phase 2.
  - Impact: horizontal extension is not achieved; upper layers still need benchmark-specific code changes.
  - Proof needed: a new-path fixture where a task carries `TaskRuntimeBinding`/`adapter_id` and preflight/execute/cleanup resolve through an adapter-id registry with no new `ExternalRunnerKind`.
- The new protocol registry is metadata-only and is not wired into descriptor or runtime adapter construction.
  - Broken assumption: registry foundation concentrates benchmark-specific adaptation into the adapter layer.
  - Failure scenario: registering a binding does not make the benchmark discoverable by `production_descriptors`, selectable by `adapter_for_with_root`, or executable by CLI runtime dispatch.
  - Trigger condition: new adapter registration through protocol metadata.
  - Impact: the registry can pass its self-test while the real system remains hardcoded.
  - Proof needed: descriptor lookup and runtime lookup consume one protocol registry source of truth, keyed by `BenchmarkId`/`AdapterId`.
- Protocol IDs validate only through constructors; deserialization can bypass canonical validation.
  - Broken assumption: protocol authority schemas are validated as serialized authority.
  - Failure scenario: a snapshot or registry payload deserializes invalid ids into transparent newtypes without calling `new`.
  - Trigger condition: serialized snapshot/replay/registry input.
  - Impact: canonical authority can contain invalid IDs even though constructor tests pass.
  - Proof needed: custom deserialization validation and negative serde round-trip tests.
- Registry validation under-enforces the spec and can both accept invalid bindings and reject valid future ones.
  - Broken assumption: `ADAPT-PROTOCOL-002` proves binding conflict and mode/capability correctness.
  - Failure scenario: `deterministic-sample` accepts an adapter with no core capabilities; `official-runner` rejects a docker-only official adapter.
  - Trigger condition: registering future modes or execution capabilities.
  - Impact: the registry shape is misleading as a protocol foundation.
  - Proof needed: registry tests for mandatory core capabilities, execution capability alternatives, invalid combinations, stable constraints, and duplicate benchmark policy.

##### Non-blocking Risks

- `AdapterBindingDescriptor.protocol_version` is ignored when building authority.
- The `ExternalRunnerKind` legacy mapping is acceptable only if guarded from becoming the new extension path.
- `ADAPT-PROTOCOL-001/002` are active but not frozen selector IDs.

##### Required Fixes

- Add adapter-id runtime registry and new-path dispatch from `TaskRuntimeBinding`.
- Keep `ExternalRunnerKind` only as explicit legacy compatibility.
- Make protocol registry construct or reference actual descriptor/runtime factories.
- Enforce serde-time id validation.
- Align registry validation with selected-mode and capability rules.

##### Missing Tests

- New-path dispatch without `ExternalRunnerKind`.
- Third-adapter/no-upper-branch fixture.
- Negative serde tests.
- Registry fixtures for missing core capabilities, execution-capability alternatives, duplicate benchmark policy, and stable promotion constraints.
- Tests proving `TaskRuntimeBinding` is emitted/consumed.

##### Missing Logs / Observability

- No runtime event or preflight field shows protocol binding resolution.
- No observable `legacy_shim_used` flag.
- No registry resolution log for selected benchmark/mode to adapter id.

##### Evidence

- `crates/harnesslab-core/src/benchmark.rs:165`
- `crates/harnesslab-core/src/benchmark.rs:176`
- `crates/harnesslab-cli/src/runner/external/runtime_adapter.rs:66`
- `crates/harnesslab-cli/src/runner/external/runtime_adapter.rs:69`
- `crates/harnesslab-adapters/src/protocol_registry.rs:21`
- `crates/harnesslab-adapters/src/registry.rs:103`
- `crates/harnesslab-core/src/adapter_protocol.rs:4`
- `crates/harnesslab-core/src/adapter_protocol.rs:95`
- `docs/adapter-protocol.md:132`

#### implementation-output

##### Summary

Phase 2 has real Rust code and selector activation for `ADAPT-PROTOCOL-001/002`,
but blocking gaps remain in schema validation and registry semantics. Constructor
validation is bypassed by serde deserialization, so the canonical serialized
authority is not actually validated at input boundaries.

##### Blocking Findings

- Invalid protocol ids deserialize successfully.
  - Broken assumption: id newtypes are validated schema types.
  - Failure scenario: invalid JSON authority ids deserialize because transparent newtypes derive `Deserialize`.
  - Trigger condition: reading serialized authority, snapshot, or registry input.
  - Impact: malformed authority can enter replay/dispatch snapshots.
  - Proof needed: negative serde tests and validated custom `Deserialize`.
- Registry validation does not enforce the documented selected-mode capability contract.
  - Broken assumption: `ADAPT-PROTOCOL-002` validates the normative mode/capability matrix.
  - Failure scenario: bindings missing core capabilities or mode requirements pass.
  - Trigger condition: registering non-built-in or future adapter bindings.
  - Impact: adapters can register as protocol-compatible while missing core protocol capabilities.
  - Proof needed: table-driven negative registry tests.
- Legacy runner kind compatibility is not checked for drift.
  - Broken assumption: dual-written legacy enum authority maps to the same adapter.
  - Failure scenario: a binding can combine `terminal-bench` with `SweBenchPro` legacy kind.
  - Trigger condition: registry construction with mismatched legacy metadata.
  - Impact: migration diagnostics and replay consistency checks can trust contradictory authority.
  - Proof needed: registry rejection tests for mismatched legacy kind and protocol authority.
- `TaskRuntimeBinding` serialized shape does not match the protocol document and is untested.
  - Broken assumption: Phase 2 has a correct `TaskRuntimeBinding` schema.
  - Failure scenario: docs require top-level identity fields but Rust nests them under `authority`.
  - Trigger condition: downstream implementation from docs or JSON snapshots.
  - Impact: documented and implemented schema become incompatible.
  - Proof needed: align docs and Rust schema, then add golden JSON round-trip tests.

##### Non-blocking Risks

- `built_in_protocol_registry()` panics on invalid built-in metadata.
- `AdapterVersion::new("legacy")` passes despite docs saying semver-compatible.
- Active protocol selectors are not in the frozen manifest.

##### Required Fixes

- Add validated serde deserialization for all protocol id newtypes.
- Make registry validation match the documented matrix.
- Validate `legacy_runner_kind` against the canonical shim mapping.
- Align `TaskRuntimeBinding` docs and Rust schema with golden tests.

##### Missing Tests

- Invalid serde payloads for protocol id fields.
- `TaskRuntimeBinding` JSON round-trip.
- Legacy kind mismatch fixtures.
- Registry fixtures for selected modes, core capabilities, and stable restrictions.

##### Missing Logs / Observability

- No evidence that binding selection emits protocol identity and legacy shim fields.

##### Evidence

- `crates/harnesslab-core/src/adapter_protocol.rs:4`
- `crates/harnesslab-core/src/adapter_protocol.rs:20`
- `crates/harnesslab-core/src/adapter_protocol.rs:50`
- `crates/harnesslab-adapters/src/protocol_registry.rs:132`
- `docs/adapter-protocol.md:65`

#### test-validity-output

##### Summary

`ADAPT-PROTOCOL-001/002` are active and exact selector routes, but the evidence
does not prove the full objective. The main gaps are frozen-selector drift
coverage, overbroad `ADAPT-PROTOCOL-002` registry claims, and lack of
integration proof that the protocol registry is used by upper layers.

##### Blocking Findings

- Frozen selector guard does not cover `ADAPT-PROTOCOL-001/002`.
  - Broken assumption: the frozen manifest prevents selector route/file-pattern drift for newly active protocol gates.
  - Failure scenario: protocol selectors remain outside `REQUIRED_FROZEN_IDS` and the manifest.
  - Trigger condition: selector route or registry weakening after activation.
  - Impact: validation can claim drift protection while new selectors are outside the frozen baseline.
  - Proof needed: add protocol selectors to frozen ids and manifest.
- `ADAPT-PROTOCOL-002` does not enforce the documented registry contract.
  - Broken assumption: selector proves registry conflict and binding validation as documented.
  - Failure scenario: missing core capabilities, stable-without-evidence, or duplicate benchmark ids can pass.
  - Trigger condition: future adapter registration.
  - Impact: selector can pass while registry violates normative protocol docs.
  - Proof needed: validator and negative tests for documented mode/capability rules.
- Active protocol tests are implementation-internal and do not prove upper-layer protocol routing.
  - Broken assumption: passing `001/002` proves the adapter layer is already a universal protocol component.
  - Failure scenario: a new benchmark still requires editing upper-layer enum dispatch.
  - Trigger condition: third benchmark integration.
  - Impact: evidence is valid only for identity structs and registry unit checks, not generic dispatch.
  - Proof needed: integration smoke resolving a binding through protocol ids and reaching runtime without `ExternalRunnerKind`.

##### Non-blocking Risks

- `docs/adapter-protocol.md` still labels the document as Phase 1 specification.
- Normalized id semantics may need stricter negative cases.

##### Required Fixes

- Add protocol selectors to frozen selector baseline.
- Tighten `AdapterRegistry` validation.
- Add protocol-specific claim-validator negative tests.
- Clarify Phase 2 evidence wording.

##### Missing Tests

- Negative fixtures for missing core capabilities.
- Stable adapter without promotion evidence.
- Duplicate benchmark id policy.
- Protocol-id binding resolution integration smoke.
- Static/meta guard proving active protocol selectors cannot be omitted from frozen coverage.

##### Missing Logs / Observability

- Validation failures are plain strings.
- No frozen manifest artifact records protocol selector route/file-pattern locks.

##### Evidence

- `scripts/test-after-change.sh:218`
- `tests/TEST_REGISTRY.toml:3322`
- `xtask/src/frozen_selector_ids.rs:1`
- `crates/harnesslab-adapters/src/protocol_registry.rs:132`
- `docs/adapter-protocol.md:93`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| architecture-adversary | Live dispatch is still enum-first | New benchmarks cannot route through adapter id without upper-layer enum edits. | blocking | accept | Phase 2 plan explicitly requires runtime registry keyed by `adapter_id`. | Added `TaskPlan.runtime_binding`, `RuntimeTaskSnapshot.runtime_binding`, `runtime_adapter_for_adapter_id`, and a protocol-bound preflight proof in `ADAPT-RUNTIME-001`. | Round 2 closure review. |
| architecture-adversary | Protocol registry is metadata-only | Binding registration does not affect descriptor/runtime lookup. | blocking | accept | Current registry code is sidecar metadata. | Added runtime adapter lookup by protocol `adapter_id`; descriptor registry remains adapter-side protocol metadata for this slice. | Round 2 closure review. |
| architecture-adversary | Serde bypasses id validation | Invalid serialized ids can enter authority. | blocking | accept | Transparent newtypes derived `Deserialize`. | Added custom validated `Deserialize` for protocol id newtypes and `AdapterProtocolAuthority`; added invalid JSON negative tests. | Round 2 closure review. |
| architecture-adversary | Registry validation under-enforces spec | Invalid bindings can pass and valid alternatives can fail. | blocking | accept | Current selected-mode validator only checks minimal labels. | Added core capability checks, execution-capability alternative validation, legacy mapping validation, and negative registry fixtures. | Round 2 closure review. |
| implementation-adversary | Invalid ids deserialize successfully | Constructor-only validation is not schema validation. | blocking | accept | Same as architecture serde finding. | Added serde validation. | Round 2 closure review. |
| implementation-adversary | Mode/capability validation incomplete | `ADAPT-PROTOCOL-002` overclaims. | blocking | accept | Same as registry matrix finding. | Added stricter validator and negative tests. | Round 2 closure review. |
| implementation-adversary | Legacy kind compatibility drift | Binding can attach mismatched legacy kind. | blocking | accept | Registry does not compare shim authority. | Registry now compares binding identity/mode with `legacy_runner_kind_authority`. | Round 2 closure review. |
| implementation-adversary | `TaskRuntimeBinding` schema mismatch | Docs and Rust disagree on flat vs nested authority. | blocking | accept | Rust nests authority; docs list flat fields. | Updated docs to nested `authority` schema and added golden round-trip assertions. | Round 2 closure review. |
| test-validity-adversary | Protocol selectors not frozen | Active selector drift is not locked by frozen manifest. | blocking | accept | `ADAPT-PROTOCOL-*` absent from frozen ids/manifest. | Added `ADAPT-PROTOCOL-001/002` to `REQUIRED_FROZEN_IDS` and regenerated manifest; guard reports `total=86`. | Round 2 closure review. |
| test-validity-adversary | `ADAPT-PROTOCOL-002` does not prove doc contract | Unit test does not cover normative rules. | blocking | accept | Same as registry matrix finding. | Strengthened validator/tests. | Round 2 closure review. |
| test-validity-adversary | Active tests do not prove upper-layer routing | Tests are internal to core/adapters. | blocking | accept | Runtime remains enum-first. | Added adapter-id runtime preflight dispatch proof under `ADAPT-RUNTIME-001`. | Round 2 closure review. |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: no
- Blocking re-review passed: no
- Blocking re-review round links:
  - Round 2 pending
- Blocking re-review launch records:
  - pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: blocking closure re-review pending
- Allowed to proceed: no

## Round 2: Blocking Fix Closure

### Review Input

#### Objective

Verify that the accepted Round 1 blocking findings for Phase 2 identity and
registry foundation are fixed with current repo evidence.

#### Review Target

Closure review for protocol serde validation, `TaskRuntimeBinding` schema,
adapter protocol registry validation, frozen selector coverage for active
protocol selectors, and adapter-id runtime preflight dispatch.

#### Target Locations

- `crates/harnesslab-core/src/adapter_protocol.rs`
- `crates/harnesslab-core/src/benchmark.rs`
- `crates/harnesslab-core/src/runtime.rs`
- `crates/harnesslab-adapters/src/protocol_registry.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_adapter.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_adapter_tests.rs`
- `crates/harnesslab-cli/src/runner/external.rs`
- `xtask/src/frozen_selector_ids.rs`
- `tests/FROZEN_SELECTOR_MANIFEST.toml`
- `tests/TEST_REGISTRY.toml`
- `docs/adapter-protocol.md`

#### Change Introduction

The implementation now validates protocol ids during serde deserialization,
keeps `TaskRuntimeBinding` as a nested `authority` schema with golden
round-trip assertions, tightens built-in registry validation against core
capabilities, selected-mode execution requirements, and legacy shim drift,
freezes `ADAPT-PROTOCOL-001/002`, and adds a protocol `adapter_id` preflight
dispatch path with `legacy_shim_used` observability.

#### Risk Focus

- Fixes may be too narrow or still test implementation internals only.
- Runtime dispatch may still require `ExternalRunnerKind` for the new path.
- Protocol selector freeze may not actually be enforced by the frozen manifest.
- Registry validation may still diverge from the documented protocol matrix.
- Added `runtime_binding` fields may break compatibility or snapshot semantics.

#### Assumptions To Attack

- Invalid serialized ids cannot enter authority.
- `ADAPT-PROTOCOL-002` now rejects meaningful invalid bindings.
- Active protocol selectors are frozen with route/file-pattern guards.
- A task with `runtime_binding` and no `external_runner` can resolve preflight by `adapter_id`.
- Existing legacy tasks still work.

#### Adversarial Lenses

- architecture
- implementation
- testing
- compatibility

#### Verification Status

- `cargo fmt --all --check && git diff --check`: passed.
- `cargo check -p harnesslab-core -p harnesslab-adapters -p harnesslab-cli -p xtask`: passed.
- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-001`: passed.
- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-002`: passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-001`: passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-002`: passed.
- `scripts/verify-test-registry.sh`: passed.
- `cargo test -p xtask adapter_claims -- --nocapture`: passed.
- `CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh`: passed with `active=18 planned=11`.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Focus on whether accepted Round 1 blockers are actually closed.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 15 minutes | one bounded 8 minute extension only if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-adversary | Round 1 architecture blockers included runtime dispatch and registry foundation. | protocol boundary, runtime dispatch, horizontal extension |
| implementation-adversary | Round 1 implementation blockers included serde, registry, legacy drift, schema alignment. | correctness, compatibility, error handling |
| test-validity-adversary | Round 1 test blockers included frozen selector omission and overbroad selector claims. | proof strength, selector drift, missing negative cases |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-adversary | `multi_agent_v1.spawn_agent` | `019eb6d3-d593-7b93-86cf-95f2c5d0aa93` | spawn result nickname `Leibniz` | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| implementation-adversary | `multi_agent_v1.spawn_agent` | `019eb6d3-f778-7190-b94e-1628d2f1d9b9` | spawn result nickname `Feynman` | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| test-validity-adversary | `multi_agent_v1.spawn_agent` | `019eb6d4-17f7-77e3-8ee5-fde3d6c5cd49` | spawn result nickname `Kepler` | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| architecture-output | architecture-adversary | 1 | `019eb6d3-d593-7b93-86cf-95f2c5d0aa93` | 15 minutes | completed | reviewer returned blocking findings | accepted blockers |
| implementation-output | implementation-adversary | 1 | `019eb6d3-f778-7190-b94e-1628d2f1d9b9` | 15 minutes | completed | reviewer returned blocking findings | accepted blockers |
| test-validity-output | test-validity-adversary | 1 | `019eb6d4-17f7-77e3-8ee5-fde3d6c5cd49` | 15 minutes | completed | reviewer returned blocking findings | accepted blockers |

### Reviewer Outputs

#### architecture-output

##### Summary

Round 1 fixes improved schema validation, selector freeze, and preflight
observability, but the implementation still did not close the Phase 2 runtime
authority claim. The new binding path was preflight-oriented and production
adapters were not yet emitting enough protocol authority for replay to trust the
new path.

##### Blocking Findings

- Adapter-id dispatch was still not proven through execute/cleanup/replay.
  - Failure scenario: a task with only `TaskRuntimeBinding` can preflight, but
    execution still fails as missing legacy runner spec.
  - Proof needed: execute reaches the selected runtime adapter by protocol
    `adapter_id`; cleanup discovery and replay authority use the same binding.
- Protocol registry remained too disconnected from live runtime authority.
  - Failure scenario: registry validation passes while contradictory
    `runtime_binding` authority is accepted at runtime.
  - Proof needed: runtime validates task authority against the built-in protocol
    registry before selecting an adapter.
- Production adapters did not yet emit protocol binding into task/runtime and
  attempt snapshots.
  - Failure scenario: replay cannot prove whether an attempt used protocol
    authority or legacy enum authority.
  - Proof needed: Terminal-Bench and SWE-bench Pro plans dual-write
    `TaskRuntimeBinding`; attempt snapshots include and fingerprint
    `protocol_authority`.

#### implementation-output

##### Summary

The serde and registry unit-level blockers were materially improved, but runtime
input validation was still incomplete and could accept contradictory
`TaskRuntimeBinding` values outside the registry tests.

##### Blocking Findings

- Runtime accepted contradictory protocol authority.
  - Failure scenario: `TaskRuntimeBinding` declares an adapter id or benchmark id
    inconsistent with the registry and execution still proceeds.
  - Proof needed: preflight/execute path validates `binding.authority` through
    `built_in_protocol_registry().validate_authority`.
- Registry validation still overclaimed the full protocol matrix.
  - Failure scenario: documentation implies artifact/redaction/report
    constraints are enforced by `ADAPT-PROTOCOL-002`, but the validator only
    covers identity, capability, mode, stability, and legacy mapping.
  - Proof needed: either enforce the whole matrix now or scope the 002
    documentation to the current registry gate and defer artifact/report rules
    to their planned gates.

#### test-validity-output

##### Summary

The active selectors were no longer merely placeholders, but the proof still had
two self-deception risks: execution was not covered by the protocol-only path,
and `ADAPT-PROTOCOL-002` could be interpreted as stronger than its actual test
coverage.

##### Blocking Findings

- `ADAPT-RUNTIME-001` proved protocol-bound preflight but not execution.
  - Failure scenario: protocol-only tasks pass preflight tests but fail at
    `execute_external_task`.
  - Proof needed: a protocol-only execution test that reaches the runtime adapter
    and returns an adapter-owned structured failure instead of missing runner
    authority.
- `ADAPT-PROTOCOL-002` documentation and validator were mismatched.
  - Failure scenario: future adapter authors believe artifact/redaction/report
    constraints are part of current registry validation.
  - Proof needed: table and selector description distinguish registry-level
    capability/mode validation from later conformance gates.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| architecture-adversary | Execute/cleanup/replay not protocol-bound | Preflight can pass while execute/replay still require legacy runner authority. | blocking | accept | Runtime dispatch was not consistently using `TaskRuntimeBinding`. | Added `runtime_adapter_for_task`; execute, preflight, cleanup target discovery, internal-error snapshots, and replay validation now resolve by task protocol binding when present. | Round 3 closure review. |
| architecture-adversary | Runtime authority not registry-validated | Contradictory binding can reach runtime. | blocking | accept | Registry tests alone do not protect runtime input. | `runtime_adapter_for_task` validates `binding.authority` through `built_in_protocol_registry().validate_authority`; added negative preflight test for protocol authority mismatch. | Round 3 closure review. |
| architecture-adversary | Production adapters do not emit protocol authority | Replay cannot prove new-path authority. | blocking | accept | Task plans and snapshots lacked live protocol binding evidence. | Terminal-Bench and SWE-bench Pro task plans now dual-write `runtime_binding`; task runtime snapshots carry it; external-runtime public/private snapshots include and fingerprint `protocol_authority`. | Round 3 closure review. |
| implementation-adversary | Runtime accepts contradictory binding | Same contradictory authority can bypass registry unit fixtures. | blocking | accept | Runtime path needed its own validation boundary. | Added registry validation at runtime selection boundary and mismatch test. | Round 3 closure review. |
| implementation-adversary | `ADAPT-PROTOCOL-002` overclaims full matrix | Registry validator does not enforce artifact/redaction/report metadata. | blocking | accept | Full artifact/report gates are planned later and should not be implied now. | Rewrote `docs/adapter-protocol.md` selected-mode table to separate registry-rejected combinations from deferred conformance gates. | Round 3 closure review. |
| test-validity-adversary | Protocol-only execution unproved | A protocol-only task may fail before adapter execution. | blocking | accept | Preflight alone was insufficient. | Added protocol-only execution test that reaches Terminal-Bench adapter and returns structured `ExternalRunnerSetupFailed`; `ADAPT-RUNTIME-001` passes. | Round 3 closure review. |
| test-validity-adversary | Selector traceability incomplete after replay changes | Runtime/replay helper changes can affect snapshot selectors without file-pattern coverage. | blocking | accept | Current work touched replay and test snapshot helper. | Added `replay.rs`, `external.rs`, and `tests/support/runtime_snapshot.rs` to affected selector file patterns and regenerated frozen manifest. | Round 3 closure review. |

### Additional Verification After Fixes

- `cargo fmt --all --check`: passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-003`: passed after
  protocol-aware replay fingerprint/source-path fix.
- `scripts/test-after-change.sh --select SWEPRO-005`: passed after syncing
  test snapshot fingerprint helper with `protocol_authority`.

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: in progress; current main-agent fixes landed
  in worktree and require full validation plus Round 3 fresh closure review
- Blocking re-review completed: yes for Round 2
- Blocking re-review passed: no
- Blocking re-review round links:
  - Round 2 recorded above
- Blocking re-review launch records:
  - `019eb6d3-d593-7b93-86cf-95f2c5d0aa93`
  - `019eb6d3-f778-7190-b94e-1628d2f1d9b9`
  - `019eb6d4-17f7-77e3-8ee5-fde3d6c5cd49`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: artifact/redaction/report matrix enforcement is
  explicitly deferred to later protocol conformance gates in
  `docs/adapter-protocol.md`
- Blocked reason: Round 3 fresh closure review pending after accepted Round 2
  blocker fixes
- Allowed to proceed: no, not until full validation and Round 3 closure pass

## Final Conclusion

Phase 2 has progressed from identity/registry-only work into live
runtime/replay authority binding, but it is not final-closed yet. Round 2 found
accepted blockers, current fixes are in the worktree, and Round 3 fresh closure
review is required before this review can be marked passed.

## Round 3: Runtime Authority Closure Review

### Review Input

Round 3 asked fresh read-only subagents to falsify the post-Round-2 fixes:
canonical registry authority, protocol-bound runtime dispatch, production
adapter `runtime_binding` emission, external-runtime `protocol_authority`
fingerprinting, replay authority validation, selector traceability, and
documentation scope.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-adversary | `multi_agent_v1.spawn_agent` | `019eb6e9-b29d-7743-9c60-e384e0123558` | spawn result nickname `Anscombe` | fork_context=false | Round 3 architecture closure packet | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| implementation-adversary | `multi_agent_v1.spawn_agent` | `019eb6e9-f996-79c1-850e-91a032a0c8be` | spawn result nickname `Ptolemy` | fork_context=false | Round 3 implementation closure packet | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| test-validity-adversary | `multi_agent_v1.spawn_agent` | `019eb6ea-4171-7772-a45f-d89c7eb4b0e1` | spawn result nickname `Russell` | fork_context=false | Round 3 test-validity closure packet | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| architecture-output | architecture-adversary | 1 | `019eb6e9-b29d-7743-9c60-e384e0123558` | 5 minutes | completed | reviewer returned blocking findings | accepted blockers |
| implementation-output | implementation-adversary | 1 | `019eb6e9-f996-79c1-850e-91a032a0c8be` | 5 minutes | completed | reviewer returned blocking findings | accepted blockers |
| test-validity-output | test-validity-adversary | 1 | `019eb6ea-4171-7772-a45f-d89c7eb4b0e1` | 5 minutes | completed | reviewer returned blocking findings | accepted blockers |

### Reviewer Outputs

#### architecture-output

##### Summary

Round 3 did not pass. The reviewer found that runtime adapter lookup still used
a concrete `adapter_id` match, dual-written tasks still preferred legacy
dataset/source refs, and production adapters emitted legacy-shim authority
rather than canonical registry authority.

##### Blocking Findings

- Generic runtime still hardcoded concrete adapter identity instead of resolving
  through one registration mechanism.
- Runtime/replay did not use a single authority model when both
  `external_runner` and `runtime_binding` were present.
- Production adapters emitted `legacy_runner_kind_authority(...)`, producing
  `adapter_version = legacy` / `stability = Legacy`, while runtime observability
  still used short implementation ids.

#### implementation-output

##### Summary

Round 3 requested changes. The reviewer found that `runtime_binding` was not
actually authoritative when legacy fields were also present, and
`validate_authority()` accepted forged `adapter_version`, reduced capability
sets, altered stability, and changed compatibility metadata.

##### Blocking Findings

- Dual-written tasks could silently use stale `external_runner` refs while
  claiming protocol authority.
- Built-in registry validation checked only identity/mode/protocol shape, not
  canonical authority equality.

##### Non-blocking Risks

- Cleanup discovery still skips invalid protocol-bound tasks silently.
- Some source-text sentinels remain brittle, though behavior coverage now
  exists.

#### test-validity-output

##### Summary

Round 3 did not pass. The reviewer found that snapshot tests did not assert
non-null live `protocol_authority`, selector file patterns did not include the
adapter builder files that emit `runtime_binding`, and deterministic-mode docs
still outpaced validator coverage.

##### Blocking Findings

- `ADAPT-RUNTIME-003` / `SWEPRO-005` could pass with `protocol_authority = null`.
- Selector traceability missed `crates/harnesslab-adapters/src/terminal_bench.rs`
  and `crates/harnesslab-adapters/src/swe_bench_pro.rs`.
- `ADAPT-PROTOCOL-002` did not test deterministic mode with execution
  capabilities.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| architecture-adversary | Runtime lookup hardcoded concrete adapter ids | Third adapter still requires editing generic CLI match. | blocking | accept | `runtime_adapter_for_adapter_id` used concrete string match. | Replaced concrete id match with a runtime adapter registration table searched by each adapter's canonical `adapter_id()`. | Round 4 closure review. |
| architecture-adversary | Dual-written tasks prefer legacy refs | Stale legacy dataset/source can override protocol refs. | blocking | accept | `runtime_dataset_ref` / `runtime_source_ref` preferred `external_runner`. | Runtime helpers now use `runtime_binding` when present and fail closed on dual-write dataset/source mismatch. | Round 4 closure review. |
| architecture-adversary | Production adapters emit legacy-shim authority | Runtime snapshots prove legacy authority self-consistency, not canonical registry truth. | blocking | accept | Adapter builders called `legacy_runner_kind_authority`. | Terminal-Bench and SWE-bench Pro now derive task authority from `built_in_protocol_registry().binding_for_adapter_id(...).authority()`. | Round 4 closure review. |
| implementation-adversary | Registry accepts forged authority metadata | Forged version/capabilities/stability can be logged and fingerprinted as truth. | blocking | accept | `validate_authority` only checked benchmark/mode/protocol and revalidated the forged capability set. | `validate_authority` now compares adapter_version, stability, legacy_runner_kind, and exact capability set against the registry descriptor. | Round 4 closure review. |
| implementation-adversary | Cleanup discovery silently drops invalid protocol tasks | Protocol config breakage can hide cleanup enumeration. | non-blocking | defer | Current cleanup API returns `Vec` and preflight catches invalid runtime bindings before run; changing cleanup error surface is broader than Phase 2 authority closure. | Deferred to Phase 4 generic upper-layer cleanup/report/doctor error-surface work. | Track under ADAPT-PROTOCOL-007/008. |
| test-validity-adversary | Live snapshot tests do not assert protocol_authority | Authority emission can regress to null while legacy runner keeps tests green. | blocking | accept | Snapshot tests asserted fingerprint but not non-null authority. | Added public/private `protocol_authority` assertions for Terminal-Bench and SWE-bench Pro plus a replay negative test that null authority fails after checksums/anchors are recomputed. | Round 4 closure review. |
| test-validity-adversary | Selector traceability misses adapter builders | Edits to runtime_binding emission may not schedule authority tests. | blocking | accept | Runtime authority is emitted in adapter builder files. | Added `terminal_bench.rs` to `ADAPT-RUNTIME-003` and `swe_bench_pro.rs` to `SWEPRO-005` file patterns; regenerated frozen manifest. | Round 4 closure review. |
| test-validity-adversary | Deterministic docs/validator mismatch | Execution-only deterministic capability drift was not rejected. | blocking | accept | Validator rejected official/patch/cleanup/report but not execution capabilities. | Deterministic mode now rejects execution capabilities too; added negative registry test. | Round 4 closure review. |
| architecture-adversary | Plan still documents flat TaskRuntimeBinding | Later phases could implement stale schema. | non-blocking | accept | Plan section 8.6 had flat fields. | Updated plan to nested `authority` field list. | Round 4 closure review. |

### Additional Verification After Round 3 Fixes

- `cargo fmt --all --check && git diff --check`: passed.
- `cargo check -p harnesslab-core -p harnesslab-adapters -p harnesslab-cli -p xtask`: passed.
- `scripts/verify-test-registry.sh`: passed.
- `cargo run -p xtask -- verify-frozen-selector-manifest`: passed.
- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-001`: passed.
- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-002`: passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-001`: passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-002`: passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-003`: passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-006`: passed.
- `scripts/test-after-change.sh --select SWEPRO-005`: passed.
- `cargo test -p xtask adapter_claims -- --nocapture`: passed.
- `CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh`: passed with `active=18 planned=11`.

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: in progress; current fixes validated locally
  and require Round 4 fresh closure review
- Blocking re-review completed: yes for Round 3
- Blocking re-review passed: no
- Blocking re-review round links:
  - Round 3 recorded above
- Blocking re-review launch records:
  - `019eb6e9-b29d-7743-9c60-e384e0123558`
  - `019eb6e9-f996-79c1-850e-91a032a0c8be`
  - `019eb6ea-4171-7772-a45f-d89c7eb4b0e1`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: cleanup discovery error surfacing deferred to
  Phase 4 generic upper-layer error surface work
- Blocked reason: Round 4 fresh closure review pending after accepted Round 3
  blocker fixes
- Allowed to proceed: no, not until Round 4 closure pass

## Current Final Conclusion

Phase 2 is substantially stronger after Round 3 fixes: production adapters now
emit canonical registry authority, runtime/replay fail closed on dual-write ref
drift, live snapshots assert and fingerprint non-null protocol authority, and
selector traceability covers the adapter builders. It is still not review-closed
until a Round 4 fresh closure review passes.

## Round 4: Authority Closure Re-Review

### Review Input

Round 4 asked fresh read-only reviewers to verify accepted Round 3 blockers:
canonical adapter id registration, canonical registry-derived task authority,
dual-write dataset/source drift handling, exact registry authority validation,
live `protocol_authority` assertions and replay blocker, selector traceability,
and docs alignment.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | `multi_agent_v1.spawn_agent` | `019eb6f9-3aef-7260-986b-b410e3ac32a4` | spawn result nickname `Herschel` | fork_context=false | Round 4 implementation closure packet | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| test-validity-adversary | `multi_agent_v1.spawn_agent` | `019eb6f9-79b0-70e1-9691-037c70a8898b` | spawn result nickname `Dewey` | fork_context=false | Round 4 test-validity closure packet | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| implementation-output | implementation-adversary | 1 | `019eb6f9-3aef-7260-986b-b410e3ac32a4` | 8 minutes | completed_after_extension | reviewer returned one blocking finding | accepted blocker |
| test-validity-output | test-validity-adversary | 1 | `019eb6f9-79b0-70e1-9691-037c70a8898b` | 8 minutes | completed_after_extension | reviewer returned no blockers | completed |

### Reviewer Outputs

#### implementation-output

##### Summary

Round 4 requested changes. Most accepted Round 3 items were closed, but
dual-written SWE tasks could still proceed if `runtime_binding.task_ref` existed
and legacy `external_runner.source_path` was omitted.

##### Blocking Findings

- Dual-written SWE source-ref drift was not fail-closed when the legacy
  `source_path` was missing.
  - Broken assumption: `runtime_source_ref` blocks all dual-write mismatches.
  - Failure scenario: a SWE task has protocol `task_ref` and legacy
    `external_runner.source_path = None`; runtime silently uses protocol ref.
  - Impact: partial legacy drift can pass unnoticed on source-bearing adapters.
  - Proof needed: require source_path presence/equality when both authorities
    are present and add a negative test.

#### test-validity-output

##### Summary

No blocking finding survived for the accepted Round 3 closure scope. The
reviewer confirmed live `protocol_authority` assertions, replay null-authority
failure, selector/manifest traceability, and focused verification passing.

##### Non-blocking Risks

- Possible transient `ADAPT-RUNTIME-005` timing flake was observed by the
  reviewer once but passed immediately in isolation and in the full sweep.
- `xtask` route spec does not independently mirror the full `SWEPRO-005`
  file-pattern set, though registry and frozen manifest cover it.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | SWE dual-write source omission not fail-closed | Protocol `task_ref` could silently replace missing legacy `source_path`. | blocking | accept | Source-bearing dual-write tasks must prove old/new refs are consistent. | Moved ref resolution into `runtime_authority.rs`; `runtime_source_ref` now requires legacy source_path to exist and equal protocol task_ref when both authorities are present; added a negative SWE dual-write omission test. | Round 5 closure review. |
| implementation-adversary | Cleanup still uses runner_kind | Cleanup is not fully adapter-id-native. | non-blocking | defer | Matches later no-branch cleanup migration scope. | Tracked under Phase 4/ADAPT-PROTOCOL-008. | none |
| implementation-adversary | Docs metadata stale | `docs/adapter-protocol.md` date and review status not yet final. | non-blocking | accept | Metadata should reflect final closure after Round 5. | Will update after Round 5 result. | Round 5 closure. |
| test-validity-adversary | Possible ADAPT-RUNTIME-005 flake | One stressed run reportedly missed `external_runner_activity`, then passed in isolation/full sweep. | non-blocking | defer | No reproduction in main full sweep; not blocking Phase 2 authority closure. | Keep as residual risk; rerun if it recurs. | none |

### Additional Verification After Round 4 Fix

- `cargo fmt --all --check && git diff --check`: passed.
- `cargo check -p harnesslab-core -p harnesslab-adapters -p harnesslab-cli -p xtask`: passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-001`: passed.
- `scripts/test-after-change.sh --select SWEPRO-005`: passed.
- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-002`: passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-003`: passed.
- `scripts/verify-test-registry.sh`: passed.
- `cargo run -p xtask -- verify-frozen-selector-manifest`: passed.

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: in progress; current fix validated locally
  and requires Round 5 fresh closure review
- Blocking re-review completed: yes for Round 4
- Blocking re-review passed: no
- Blocking re-review round links:
  - Round 4 recorded above
- Blocking re-review launch records:
  - `019eb6f9-3aef-7260-986b-b410e3ac32a4`
  - `019eb6f9-79b0-70e1-9691-037c70a8898b`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: cleanup runner-kind migration under Phase 4;
  possible `ADAPT-RUNTIME-005` flake as residual risk
- Blocked reason: Round 5 fresh closure review pending after accepted Round 4
  blocker fix
- Allowed to proceed: no, not until Round 5 closure pass

## Current Final Conclusion After Round 4

Round 4 closed the Round 3 evidence concerns except for one accepted SWE
dual-write source omission blocker. The fix is implemented and validated
locally, but Phase 2 still waits on Round 5 closure review.

## Round 5: SWE Source Authority Closure Review

### Review Input

Round 5 asked a fresh read-only reviewer to verify the accepted Round 4 blocker:
SWE dual-written tasks must fail closed when protocol `runtime_binding.task_ref`
exists but legacy `external_runner.source_path` is missing or inconsistent. The
review also checked that runtime/replay authority helpers, snapshot authority
fingerprints, selector routing, and registry validation still compile and pass
focused proof gates after the Round 4 fix.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | `multi_agent_v1.spawn_agent` | `019eb705-7d53-79b3-8fe4-9e33b33b118f` | spawn result nickname `Peirce` | fork_context=false | Round 5 SWE source authority closure packet | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| implementation-output | implementation-adversary | 1 | `019eb705-7d53-79b3-8fe4-9e33b33b118f` | 6 minutes | completed | reviewer approved closure with no blockers | completed |

### Reviewer Output

#### implementation-output

##### Summary

Round 5 passed. The reviewer found that SWE dual-write source authority now
fails closed when the legacy `source_path` is missing, runtime authority helpers
preserve protocol-first semantics with legacy consistency checks, replay compares
and fingerprints `protocol_authority`, and selector/manifest validation covers
the touched surfaces.

##### Blocking Findings

None.

##### Non-blocking Risks

- Legacy-only SWE preflight still reports a `runtime binding missing
  source_path` message in one path even though behavior remains fail-closed.
  This is wording only and is not part of Phase 2 authority correctness.
- A positive protocol-only SWE execution fixture is still useful for later
  confidence. Existing Phase 2 negative and snapshot tests prove fail-closed
  dual-write authority behavior, while broader benchmark conformance remains
  Phase 3/4 work.

### Additional Verification For Round 5 Closure

- `cargo fmt --all --check`: passed.
- `git diff --check`: passed.
- `cargo check -p harnesslab-core -p harnesslab-adapters -p harnesslab-cli -p xtask`: passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-001`: passed.
- `scripts/test-after-change.sh --select SWEPRO-005`: passed.
- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-002`: passed.
- `scripts/test-after-change.sh --select ADAPT-RUNTIME-003`: passed.
- `scripts/verify-test-registry.sh`: passed.
- `cargo run -p xtask -- verify-frozen-selector-manifest`: passed.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | No blocking findings | n/a | n/a | accept closure | Round 5 reviewer approved the accepted Round 4 fix and focused proof gates. | Marked Phase 2 review passed and updated protocol/plan metadata. | Continue Phase 3/4 from the implementation plan. |
| implementation-adversary | Legacy-only SWE preflight wording can be clearer | A fail-closed legacy-only missing-source case may mention runtime binding. | non-blocking | defer | Behavior is correct; changing code after closure would require another review loop for wording only. | Recorded as a residual wording cleanup. | Phase 3/4 polish or next SWE adapter touch. |
| implementation-adversary | Positive protocol-only SWE fixture missing | Existing tests emphasize authority fail-closed behavior more than positive SWE protocol-only execution. | non-blocking | defer | Phase 2 required authority binding and replay correctness; full conformance matrix is later work. | Recorded as later conformance coverage. | Phase 3 adapter conformance suite. |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes for Round 5
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 5 recorded above
- Blocking re-review launch records:
  - `019eb705-7d53-79b3-8fe4-9e33b33b118f`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: legacy-only SWE preflight wording; positive
  protocol-only SWE conformance fixture; cleanup/report/doctor branch removal
  under Phase 4
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

Phase 2 is review-closed and passed. This slice implemented real adapter
protocol runtime authority, not only documentation: canonical protocol authority
types, validated registry binding, production task dual-write, protocol-id
runtime dispatch, protocol-aware preflight telemetry, fail-closed dual-write
dataset/source checks, external-runtime snapshot authority fingerprints, replay
authority validation, active selector gates, frozen manifest refresh, and focused
regression coverage are all in place.

This does not mean the whole adapter architecture plan is complete. Remaining
planned work stays in Phase 3/4: full adapter conformance interfaces, generic
doctor/report/cleanup behavior, richer artifact/failure schemas, and additional
positive benchmark fixtures for future horizontal adapter onboarding.
