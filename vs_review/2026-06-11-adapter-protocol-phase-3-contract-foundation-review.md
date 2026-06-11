# Subagent VS Review: Adapter Protocol Phase 3 Contract Foundation

- Created: 2026-06-11T23:40:00+0800
- Updated: 2026-06-11T23:40:00+0800
- Report schema: adversarial-v1
- Task: Continue implementing `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md` by starting Phase 3 unified adapter interface work.
- Report path: `vs_review/2026-06-11-adapter-protocol-phase-3-contract-foundation-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: open

## Round 1: Phase 3 Data And Runtime Contract Foundation

### Review Input

#### Objective

Verify that this slice materially advances Phase 3 of the universal benchmark
adapter protocol plan: adapter-owned protocol subcontracts for data lifecycle,
runtime lifecycle, readiness probes, and failure taxonomy, plus active
`ADAPT-PROTOCOL-003/004` proof gates.

#### Review Target

Code implementation, selector routing, registry metadata, boundary docs, and
validation evidence for the Phase 3 contract foundation slice.

#### Target Locations

- `crates/harnesslab-adapters/src/protocol_contract.rs`
- `crates/harnesslab-adapters/src/protocol_contract_builtins.rs`
- `crates/harnesslab-adapters/src/protocol_contract_tests.rs`
- `crates/harnesslab-adapters/src/lib.rs`
- `scripts/test-after-change.sh`
- `scripts/verify-planned-adapter-selectors.sh`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `tests/FROZEN_SELECTOR_MANIFEST.toml`
- `xtask/src/adapter_claims.rs`
- `xtask/src/frozen_selector_ids.rs`
- `docs/adapter-protocol.md`
- `docs/plans/2026-06-04-benchmark-adapter-phase-1-boundary.md`
- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`

#### Change Introduction

The implementation adds protocol-facing adapter descriptor records with data
lifecycle, runtime lifecycle, readiness probe, and central failure mapping
subcontracts. Built-in Terminal-Bench and SWE-bench Pro protocol descriptors are
constructed from the existing protocol registry and production benchmark
descriptors. `ADAPT-PROTOCOL-003` and `ADAPT-PROTOCOL-004` are activated from
planned placeholders into concrete contract tests with negative cases. Selector
inventory, frozen manifest, boundary docs, and plan/protocol docs are updated.

#### Risk Focus

- The new protocol contract may be metadata-only and not sufficiently tied to
  real adapter behavior.
- Contract validation may only prove the current hardcoded built-ins, not a
  reusable horizontal protocol.
- Runtime lifecycle and failure taxonomy gates may be too weak to catch missing
  adapter readiness/failure responsibilities.
- Selector activation may be self-deceptive if file patterns or xtask route
  specs miss key implementation files.
- The new production modules may violate existing adapter data-boundary
  constraints or line-count constraints.

#### Assumptions To Attack

- Terminal-Bench and SWE-bench Pro can be represented through the same
  `ProtocolAdapterDescriptor` structure.
- Missing declared capabilities, missing readiness probes, and duplicate failure
  mappings are rejected before a benchmark is treated as protocol-conformant.
- `ADAPT-PROTOCOL-003/004` are real active gates and no longer planned proof
  placeholders.
- This slice does not overclaim full Phase 3 completion; artifact/report/doctor
  work remains planned.

#### Adversarial Lenses

- architecture
- implementation
- testing
- maintenance
- observability

#### Verification Status

- `cargo fmt --all --check`: passed.
- `cargo check -p harnesslab-core -p harnesslab-adapters -p xtask`: passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-001`: passed.
- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-003`: passed.
- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-004`: passed.
- `scripts/verify-test-registry.sh`: passed.
- `cargo run -p xtask -- verify-frozen-selector-manifest`: passed through registry script.
- `CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh`: passed with `active=20 planned=9`.
- Line counts: `protocol_contract.rs` 288, `protocol_contract_builtins.rs` 273, `protocol_contract_tests.rs` 68, `xtask/src/adapter_claims.rs` 455.

#### Reviewer Instructions

- Fresh internal subagent session.
- Do not inherit main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Try to falsify the implementation and validation claims.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 10 minutes | one bounded 5 minute extension only if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-adversary | Phase 3 is primarily about whether the new protocol surface is a useful adapter-owned boundary instead of another metadata list. | abstraction, boundaries, horizontal extension |
| test-validity-adversary | The slice activates planned selectors, so selector routing and negative coverage need direct challenge. | self-deceptive gates, file-pattern drift, missing failure paths |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| closure-reviewer | `multi_agent_v1.spawn_agent` | `019eb72e-5e33-7763-a52f-6bb93958d736` | spawn result nickname `Nash` | fork_context=false | Round 3 Adapter-owned Descriptor Closure Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| closure-output | closure-reviewer | 1 | `019eb72e-5e33-7763-a52f-6bb93958d736` | 10 minutes | completed | reviewer found no blockers for the accepted Round 3 scope | completed |

### Reviewer Output

#### closure-output

##### Summary

Round 3 passed. The reviewer found the remaining accepted blocker closed:
protocol descriptor construction is now adapter-owned, and the central built-in
collector only aggregates descriptors exposed by adapter instances.

##### Blocking Findings

None for the accepted Round 3 blocker.

##### Non-blocking Risks

- `tests/TEST_REGISTRY.toml` and the implementation plan document are over 500
  lines. This does not reopen the code-file size rule because they are not code
  files, but it is a future maintainability concern.
- This closure remains scoped to descriptor ownership and contract-foundation
  coverage. Artifact/report/doctor/static-no-branch proof remains later work.

### Main Agent Response

| Reviewer | Finding | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|
| closure-reviewer | No blocking findings for adapter-owned descriptor construction | n/a | accept closure | Reviewer verified `BenchmarkAdapter::protocol_descriptor`, adapter-owned `terminal_bench_protocol.rs` and `swe_bench_pro_protocol.rs`, aggregator-only `protocol_contract_builtins.rs`, and selector coverage. | Marked Round 3 passed. | Continue later Phase 3/4 items. |
| closure-reviewer | Oversized non-code files | non-blocking | defer | AGENTS rule is scoped to code files; splitting registry/docs is a separate maintenance task and not required to close this code slice. | Recorded risk. | Consider future registry/doc modularization. |

### Additional Verification For Round 3 Closure

- `cargo fmt --all --check && git diff --check`: passed.
- `cargo check -p harnesslab-core -p harnesslab-adapters -p xtask`: passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-001`: passed.
- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-003`: passed.
- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-004`: passed.
- `scripts/verify-test-registry.sh`: passed.

### Closure Status

- Blocking findings found: no in Round 3
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes for Round 3
- Blocking re-review passed: yes
- Blocking re-review launch records:
  - `019eb72e-5e33-7763-a52f-6bb93958d736`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: oversized non-code registry/plan files; live
  runtime artifact/report/doctor/static-no-branch proof remains later planned
  work
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

This Phase 3 contract-foundation slice passed adversarial review after accepted
blockers were fixed. It does not complete the full
`2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`; it
advances that plan by activating `ADAPT-PROTOCOL-003/004` as honest
contract-foundation gates and moving protocol descriptor ownership into adapter
modules. Remaining planned selectors are still required for artifact
declaration/redaction, replay mixed-authority fixtures, generic
doctor/report/readiness integration, static no-branch guard, scaffold, migration
preservation, third-adapter proof, and stable-promotion evidence.
| closure-reviewer | `multi_agent_v1.spawn_agent` | `019eb726-6542-7e91-9168-c30e24c43600` | spawn result nickname `Arendt` | fork_context=false | Round 2 Contract Foundation Closure Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| closure-output | closure-reviewer | 1 | `019eb726-6542-7e91-9168-c30e24c43600` | 10 minutes | completed | reviewer found one remaining blocking finding | accepted blocker |

### Reviewer Output

#### closure-output

##### Summary

Round 2 requested changes. The reviewer verified fixes for readiness schema,
failure mapping schema, `ADAPT-PROTOCOL-004` proof scoping, and selector
file-pattern coverage. The remaining blocker was that the actual
`ProtocolAdapterDescriptor` construction still lived in central helper code,
with adapters only delegating to those helpers.

##### Blocking Findings

- Accepted blocker 1 was only partially closed: descriptors were callable via
  `BenchmarkAdapter::protocol_descriptor`, but actual contract construction was
  still centrally fabricated.
  - Broken assumption: exposing a trait method is enough to make the protocol
    surface adapter-owned.
  - Failure scenario: a third adapter or Phase 4 runtime refactor still edits a
    central fabrication table instead of only adapter-owned code.
  - Trigger: new adapter onboarding or per-adapter readiness/failure changes.
  - Impact: horizontal extension remains blocked.
  - Proof needed: each adapter module owns its `ProtocolAdapterDescriptor` or
    equivalent protocol object; central collector only aggregates instances.

##### Non-blocking Risks

- Rust schema remains narrower than full protocol docs in some fields such as
  `required_tools`, `privacy_scope`, and `adapter_detail_code`. This is a future
  integration risk but not blocking for the rescoped contract-foundation slice.

### Main Agent Response

| Reviewer | Finding | Severity | Decision | Action Taken | Follow-up |
|---|---|---|---|---|---|
| closure-reviewer | Protocol descriptor construction still centralized | blocking | accept | Moved Terminal-Bench descriptor construction into `terminal_bench_protocol.rs` and SWE-bench Pro descriptor construction into `swe_bench_pro_protocol.rs`; `protocol_contract_builtins.rs` is now only a 14-line aggregator that calls adapter instances. Added test assertions that descriptors are exposed from `TerminalBenchAdapter::protocol_descriptor()` and `SweBenchProAdapter::protocol_descriptor()`. | Round 3 closure review. |
| closure-reviewer | Missing explicit readiness negative cases | non-blocking | accept | Added negative coverage for `official.runner`, `patch.evaluator`, `host.agent_execution`, `run_as.readiness`, `cleanup.verdict_override`, and incomplete readiness fields. Synthetic `*.readiness` remains covered by validator logic but not through built-in exact registry validation. | none |

### Additional Verification After Round 2 Fix

- `cargo fmt --all`: passed.
- `cargo check -p harnesslab-core -p harnesslab-adapters -p xtask`: passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-001`: passed.
- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-003`: passed.
- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-004`: passed.
- `scripts/verify-test-registry.sh`: passed.

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: in progress; current fix validated locally
  and requires Round 3 fresh closure review
- Blocking re-review completed: yes for Round 2
- Blocking re-review passed: no
- Blocking re-review launch records:
  - `019eb726-6542-7e91-9168-c30e24c43600`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: full live runtime artifact proof and broader
  schema fields remain planned for later selectors
- Blocked reason: Round 3 fresh closure review pending
- Allowed to proceed: no, not until Round 3 closure pass

## Round 3: Adapter-owned Descriptor Closure Review

### Review Input

Round 3 asks a fresh read-only reviewer to verify the single accepted Round 2
blocker: protocol descriptor construction must now be adapter-owned, with the
central built-in collector only aggregating descriptors exposed by adapter
instances.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-adversary | `multi_agent_v1.spawn_agent` | `019eb71b-536a-7791-80c1-757913e70d47` | spawn result nickname `Faraday` | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |
| test-validity-adversary | `multi_agent_v1.spawn_agent` | `019eb71b-7d0c-7a63-a271-830d88198605` | spawn result nickname `Hypatia` | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| architecture-output | architecture-adversary | 1 | `019eb71b-536a-7791-80c1-757913e70d47` | 10 minutes | completed | reviewer returned blocking architecture findings | accepted blockers |
| test-validity-output | test-validity-adversary | 1 | `019eb71b-7d0c-7a63-a271-830d88198605` | 10 minutes | completed | reviewer returned blocking test-validity findings | accepted blockers |

### Reviewer Outputs

#### architecture-output

##### Summary

The reviewer rejected closure. The slice was real wiring progress, but the
initial protocol surface was a centralized metadata catalog, not adapter-owned
callable subcontracts. Readiness and failure mapping schemas were too weak for
the Phase 3 claims, and `ADAPT-PROTOCOL-003/004` wording overclaimed
black-box/runtime proof.

##### Blocking Findings

- Protocol surface was not adapter-owned or callable.
  - Broken assumption: string-shaped operation metadata is enough to become the
    Phase 3 adapter interface.
  - Failure scenario: Phase 4 still has no protocol methods to call and must
    keep using `BenchmarkAdapter` or add more central switches.
  - Trigger: third-adapter onboarding or upper-layer refactor.
  - Impact: horizontal extension remains blocked.
  - Proof needed: adapters supply protocol-owned descriptors or trait objects
    from adapter modules, not central fabrication.
- Readiness schema was incomplete and enforcement too weak.
  - Broken assumption: probes with only id/capability/phase/remediation cover
    adapter-owned readiness.
  - Failure scenario: adapters can declare Docker, official runner, or patch
    evaluator capabilities while passing without required readiness coverage.
  - Trigger: new adapter or drift in built-in readiness probes.
  - Impact: false green conformance and no viable generic readiness model.
  - Proof needed: full schema fields and capability-specific enforcement.
- Failure taxonomy contract was too shallow.
  - Broken assumption: adapter_code/class/code/message validates meaningful
    central failure taxonomy.
  - Failure scenario: generic runtime/report/health code cannot distinguish
    phase/subphase or health impact.
  - Trigger: Phase 4/6 failure reporting work.
  - Impact: failure handling remains benchmark-specific.
  - Proof needed: add phase/subphase/health/private diagnostics fields and
    negative tests.
- `ADAPT-PROTOCOL-003/004` were active but not black-box/integration proof.
  - Broken assumption: selected lib tests prove live runtime behavior.
  - Failure scenario: live runtime behavior drifts while descriptor tests stay
    green.
  - Trigger: regression in CLI runtime paths or event/snapshot emission.
  - Impact: false Phase 3 closure evidence.
  - Proof needed: add live integration proof or downgrade wording/status.

##### Non-blocking Risks

- Central mode matching in `protocol_contract_builtins.rs` can drift from actual
  adapter behavior.
- Docs were mostly careful, but "black-box" and "runtime lifecycle conformance"
  wording was stronger than the implemented gate.

#### test-validity-output

##### Summary

The reviewer confirmed `ADAPT-PROTOCOL-003/004` were active/frozen at the
metadata level, but rejected them as credible proof gates for the original
claims. The main issues were runtime artifact overclaiming, self-referential
white-box tests, and incomplete file-pattern coverage.

##### Blocking Findings

- `ADAPT-PROTOCOL-004` claimed runtime/integration proof but routed to a lib
  unit test, with `required_artifacts` not enforced.
  - Broken assumption: registry metadata requiring runtime proof implies runtime
    artifacts are produced and checked.
  - Failure scenario: selector stays green without `events.jsonl`,
    `external-runtime.*.json`, `result.json`, or `run-health.json`.
  - Trigger: running `scripts/test-after-change.sh --select ADAPT-PROTOCOL-004`.
  - Impact: downstream docs/reviews are misled.
  - Proof needed: real integration harness or rescope the active selector.
- `ADAPT-PROTOCOL-003/004` were self-referential white-box metadata tests.
  - Broken assumption: cloned generated descriptors prove black-box adapter
    behavior.
  - Failure scenario: adapter/runtime behavior drifts while descriptor tests
    pass.
  - Trigger: preserving tables while breaking consumers.
  - Impact: false confidence in Phase 3 protocol conformance.
  - Proof needed: drive tests through adapter-owned protocol consumers or
    fixture adapters.
- `file_patterns` omitted real descriptor sources.
  - Broken assumption: selector routing covers the source files that feed
    protocol descriptors.
  - Failure scenario: edits to `registry.rs`, `terminal_bench.rs`, or
    `swe_bench_pro.rs` skip `ADAPT-PROTOCOL-003/004`.
  - Trigger: descriptor-generation changes.
  - Impact: gate bypass through routing drift.
  - Proof needed: add true descriptor sources to registry and frozen manifest.

##### Missing Tests / Observability

- Negative cases for duplicate adapter ids, empty splits, wrong operation ids,
  empty contract shapes, missing readiness probes, invalid failure mappings.
- No runner-level enforcement for `required_artifacts`; this was resolved by
  removing the false artifact claim from this contract-foundation selector.

### Main Agent Response

| Reviewer | Finding | Severity | Decision | Action Taken | Follow-up |
|---|---|---|---|---|---|
| architecture-adversary | Protocol surface not adapter-owned/callable | blocking | accept | Added `BenchmarkAdapter::protocol_descriptor`; Terminal-Bench and SWE-bench Pro now expose protocol descriptors from their adapter implementations. `built_in_protocol_adapter_descriptors()` collects from adapter instances instead of centrally fabricating descriptor ownership. | Closure review. |
| architecture-adversary | Readiness schema incomplete | blocking | accept | Expanded `ProtocolReadinessProbe` with severity, status contract, public message, remediation, and private detail contract; validator now requires probes for `readiness.basic`, Docker, official runner, patch evaluator, host execution, run-as, cleanup, and `*.readiness` capabilities. | Closure review. |
| architecture-adversary | Failure taxonomy too shallow | blocking | accept | Expanded `ProtocolFailureMapping` with adapter phase, adapter subphase, health impact, and private diagnostics contract; added negative tests for incomplete and invalid mappings. | Closure review. |
| architecture-adversary | `003/004` overclaim black-box/integration proof | blocking | accept | Rescoped docs/registry wording to contract foundation, removed false runtime artifact requirements from `ADAPT-PROTOCOL-004`, and kept live runtime artifact proof explicitly planned for later selectors. | Closure review. |
| test-validity-adversary | `004` runtime artifacts were dead metadata | blocking | accept | Changed `ADAPT-PROTOCOL-004` requirement to `expected_layers = ["contract"]`, `required_runtime_proof = false`, and `required_artifacts = []`; docs now state live runtime proof remains later work. | Closure review. |
| test-validity-adversary | `003/004` self-referential tests | blocking | accept | Tests now call descriptors exposed by adapter instances and assert stronger schema/negative cases; the remaining scope is explicitly contract foundation, not live black-box proof. | Closure review. |
| test-validity-adversary | File patterns omitted descriptor sources | blocking | accept | Added `registry.rs`, `terminal_bench.rs`, and `swe_bench_pro.rs` to `ADAPT-PROTOCOL-003/004` file patterns and xtask active route specs; regenerated frozen manifest. | Closure review. |

### Additional Verification After Round 1 Fixes

- `cargo fmt --all --check && git diff --check`: passed.
- `cargo check -p harnesslab-core -p harnesslab-adapters -p xtask`: passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-001`: passed.
- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-003`: passed.
- `scripts/test-after-change.sh --select ADAPT-PROTOCOL-004`: passed.
- `scripts/verify-test-registry.sh`: passed.
- `cargo run -p xtask -- print-frozen-selector-manifest > tests/FROZEN_SELECTOR_MANIFEST.toml`: completed.

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: in progress; current fixes validated locally
  and require Round 2 fresh closure review
- Blocking re-review completed: no
- Blocking re-review passed: no
- Blocking re-review launch records: pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: live runtime artifact proof remains planned for
  later protocol selectors rather than claimed by this contract-foundation slice
- Blocked reason: Round 2 fresh closure review pending
- Allowed to proceed: no, not until Round 2 closure pass

## Round 2: Contract Foundation Closure Review

### Review Input

Round 2 asks fresh read-only reviewers to verify the accepted Round 1 fixes:
adapter-owned protocol descriptor exposure, stronger readiness and failure
schemas, capability-specific readiness enforcement, negative tests, corrected
`ADAPT-PROTOCOL-004` proof scope, and expanded selector file-pattern coverage.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
