# Subagent VS Review: Benchmark Adapter Phase 0 Implementation

- Created: 2026-06-04T21:14:37+08:00
- Updated: 2026-06-04T22:24:11+08:00
- Report schema: adversarial-v1
- Task: Land Phase 0 of the Benchmark Adapter Layer Architecture Design.
- Report path: `vs_review/2026-06-04-benchmark-adapter-phase-0-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: Phase 0 Proof-Surface Review

### Review Input

#### Objective

Implement Phase 0: Contract Inventory from the benchmark adapter architecture
plan so later adapter phases cannot claim unregistered or unrouted proof.

#### Review Target

Code implementation, test registry rules, selector routing, and Phase 0
inventory documentation.

#### Target Locations

- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-04-benchmark-adapter-phase-0-inventory.md`
- `crates/harnesslab-adapters/src/registry.rs`
- `scripts/test-after-change.sh`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `xtask/src/main.rs`
- `crates/harnesslab-cli/tests/external_smoke_contract.rs`

#### Change Introduction

Phase 0 adds an active gap sentinel for the current `BenchmarkAdapter` trait,
registers planned adapter proof IDs, changes `INT-011` to a counted grouped
selector for ten SWE-bench Pro smoke/failure tests, and extends
`xtask verify-test-registry` so adapter IDs claimed in the architecture plan
must exist in requirements, test registry, and selector routing.

#### Risk Focus

- Planned proof IDs might be mistaken for implemented proof.
- The registry verifier might miss selector-routing gaps or overfit to one
  document shape.
- `INT-011` might still be misleading despite the grouped selector.
- The active gap sentinel might become brittle or block Phase 1 incorrectly.
- Requirement/test status handling might weaken existing registry guarantees.

#### Assumptions To Attack

- `status = "planned"` requirements and tests are safe because active coverage
  is still enforced for active requirements.
- Explicit planned selector failures are better than no route for future IDs.
- Scanning the architecture plan for claimed IDs is enough to catch future
  over-claims.
- The `ADAPT-DATA-000` source sentinel is an acceptable temporary proof that the
  current trait lacks independent data adapter methods.
- `INT-011` covering ten `int_011_swe_bench_pro_*` tests is the correct current
  grouped proof for SWE-bench Pro smoke/failure cases.

#### Adversarial Lenses

- architecture
- implementation correctness
- testing validity
- migration safety
- maintenance
- failure modes

#### Verification Status

- `scripts/test-after-change.sh --select ADAPT-DATA-000` passed.
- `scripts/test-after-change.sh --select META-002` passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-001` was asserted to exit
  64 with a planned-proof message.
- `scripts/test-after-change.sh --select INT-011` passed and ran ten tests.
- `cargo test -p harnesslab-adapters -- --nocapture` passed, 23 tests.
- `cargo test -p xtask -- --nocapture` passed, 4 tests.
- `git diff --check` passed.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Focus on falsifying the Phase 0 closure claim, not on implementing later
  phases.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 10 minutes | one bounded extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-adversary | Phase 0 changes migration boundaries and status semantics. | architecture and maintainability |
| test-validity-adversary | The main risk is false proof or misleading selector coverage. | test validity and evidence |
| implementation-adversary | The implementation touches Rust verifier logic, shell selector logic, and registry data. | correctness and failure handling |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-adversary | `multi_agent_v1.spawn_agent` / architect | `019e92c6-3fd5-7de1-8ddb-e0e98b09d18b` | spawn_agent result, nickname `Dewey` | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |
| test-validity-adversary | `multi_agent_v1.spawn_agent` / test-engineer | `019e92c6-864c-7052-a89d-f2a9a4a9aa97` | spawn_agent result, nickname `Turing` | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |
| implementation-adversary | `multi_agent_v1.spawn_agent` / code-reviewer | `019e92c6-d149-7193-ad89-020a55c6c4a5` | spawn_agent result, nickname `Ptolemy` | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| architecture-round-1 | architecture-adversary | 1 | `019e92c6-3fd5-7de1-8ddb-e0e98b09d18b` | 10 minutes | completed | Reviewer returned blocking architecture findings. | completed |
| test-validity-round-1 | test-validity-adversary | 1 | `019e92c6-864c-7052-a89d-f2a9a4a9aa97` | 10 minutes | completed | Reviewer returned blocking test-validity findings. | completed |
| implementation-round-1 | implementation-adversary | 1 | `019e92c6-d149-7193-ad89-020a55c6c4a5` | 10 minutes | completed | Reviewer returned blocking implementation findings. | completed |

### Reviewer Outputs

#### architecture-round-1

##### Summary

Phase 0 is not closure-ready. Registration exists and `INT-011` is count-routed,
but the registry gate is too weak to prevent future over-claims, `META-002`
claims an artifact its command does not produce, and `ADAPT-DATA-000` does not
match the plan's Phase 0 gap-test contract.

##### Blocking Findings

- `META-002` does not guarantee future adapter-proof claims are semantically safe.
  - Broken assumption: scanning the architecture plan is enough to catch future over-claims.
  - Failure scenario: future docs claim new IDs, use range syntax, or map a planned ID to unrelated passing behavior while `META-002` still passes.
  - Trigger condition: normal doc refactors or selector edits.
  - Impact: later phases can claim proof that is documented but not validated.
  - Proof needed: scan all claim docs, parse ranges, and enforce planned-ID route semantics.
- `META-002` weakens registry guarantees because `artifacts/test-traceability.json` is declared but not produced by its command.
  - Broken assumption: requirement/test status handling did not weaken registry guarantees.
  - Failure scenario: `META-002` passes and is treated as runtime proof without generating the artifact.
  - Trigger condition: running `META-002` standalone.
  - Impact: declarative artifact lists satisfy runtime proof without evidence.
  - Proof needed: make `META-002` generate traceability or remove the artifact claim.
- `ADAPT-DATA-000` does not meet the Phase 0 contract as written.
  - Broken assumption: a passing source sentinel is an acceptable substitute for the planned failing contract test.
  - Failure scenario: a green sentinel is treated as positive contract coverage.
  - Trigger condition: Phase 0 closure or Phase 1 activation review.
  - Impact: the current-gap gate is weaker than the plan says.
  - Proof needed: replace with negative boundary test or update plan/inventory to define the sentinel mechanism.

##### Non-blocking Risks

- `INT-011` registry row overstates artifact coverage.
- `INT-011` counts ten test functions, not necessarily ten distinct scenarios.

##### Required Fixes

- Strengthen adapter-claim validation beyond single-doc free-text scrape.
- Fix `META-002` command/artifact mismatch.
- Align `ADAPT-DATA-000` with the Phase 0 plan.
- Tighten `INT-011` artifact declarations or add missing assertions.

##### Missing Tests

- Add xtask tests for claimed-ID parsing, secondary-doc claims, and misrouted planned selectors.
- Add negative selector behavior checks for planned adapter selectors.
- Add assertions for `INT-011` profile/runtime artifacts if they remain declared.

##### Missing Logs / Observability

- `xtask verify-test-registry` should show which adapter IDs were found.
- The verifier should distinguish planned IDs routed to planned failure from mere case-label presence.

##### Evidence

- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-04-benchmark-adapter-phase-0-inventory.md`
- `xtask/src/main.rs`
- `tests/TEST_REGISTRY.toml`
- `scripts/verify-test-registry.sh`
- `scripts/test-after-change.sh`
- `crates/harnesslab-adapters/src/registry.rs`
- `crates/harnesslab-cli/tests/external_smoke_contract.rs`

#### test-validity-round-1

##### Summary

Phase 0 is not closure-ready. Registration exists and selected checks pass, but
the active sentinel is incomplete, the meta-verifier scans only one hard-coded
document, and selector validation is textual and untested.

##### Blocking Findings

- `ADAPT-DATA-000` does not cover the full documented data-adapter gap.
  - Broken assumption: the sentinel fully proves the missing data adapter methods.
  - Failure scenario: `create_task_plan` appears without the rest of the contract and the sentinel still passes.
  - Trigger condition: partial Phase 1 extraction.
  - Impact: current contract gap is no longer fully visible.
  - Proof needed: include `create_task_plan` in sentinel and inventory.
- `verify-test-registry` only scans one hard-coded plan file.
  - Broken assumption: scanning the architecture plan catches future over-claims.
  - Failure scenario: the Phase 0 inventory doc claims an unrouted ID and the gate passes.
  - Trigger condition: future docs update inventory but not architecture doc.
  - Impact: documentation can over-claim proof.
  - Proof needed: scan both target docs or use one canonical machine-checked source.
- Selector-routing validation is textual and has no dedicated tests.
  - Broken assumption: a selector case string proves correct behavior.
  - Failure scenario: case exists but routes to wrong command or exits 0.
  - Trigger condition: selector refactor or copy-paste error.
  - Impact: registered IDs execute no real proof.
  - Proof needed: tests for verifier logic and planned selector exit behavior.

##### Non-blocking Risks

- `INT-011` remains coarse because one counted function covers two workspace-failure env cases.
- Source-text sentinel can be brittle.

##### Required Fixes

- Include `create_task_plan` in `ADAPT-DATA-000`.
- Scan all Contract Inventory claim sources.
- Add route behavior tests and xtask verifier tests.

##### Missing Tests

- xtask verifier tests for success/failure, claim extraction, and route validation.
- negative test for inventory doc over-claim.
- table-driven planned selector exit checks.

##### Missing Logs / Observability

- `verify_test_registry` should emit claim sources and validated adapter IDs.
- Add an audit artifact mapping claimed adapter IDs to requirement, registry, and selector surfaces.

##### Evidence

- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-04-benchmark-adapter-phase-0-inventory.md`
- `crates/harnesslab-adapters/src/registry.rs`
- `xtask/src/main.rs`
- `scripts/test-after-change.sh`

#### implementation-round-1

##### Summary

Verdict: request changes. The claimed commands passed, but planned-status
support weakens registry guarantees globally, `create_task_plan` has no
independent proof surface, and `INT-011` still does not satisfy the runtime
artifact standard.

##### Blocking Findings

- Planned-status support weakens the existing registry contract.
  - Broken assumption: `status = "planned"` is safe because active coverage still holds for everything that matters.
  - Failure scenario: an existing critical requirement is marked planned and escapes active coverage/runtime proof enforcement.
  - Trigger condition: maintainer adds `status = "planned"` to any non-Phase-0 row.
  - Impact: registry metadata can suppress repo-wide guarantees.
  - Proof needed: restrict planned status to the Phase 0 adapter inventory.
- Phase 0 inventory is incomplete because `create_task_plan` has no proof surface.
  - Broken assumption: `ADAPT-DATA-001..004` plus compatibility covers the full data adapter contract.
  - Failure scenario: Phase 1 closes without independently proving task-plan creation.
  - Trigger condition: later phase closes against current IDs.
  - Impact: one explicit contract surface remains unregistered.
  - Proof needed: add a dedicated ID or map it explicitly.
- `INT-011` is count-routed but does not satisfy the runtime-proof artifact standard.
  - Broken assumption: counted routing removes the misleading-proof problem.
  - Failure scenario: key artifacts/events disappear while the current subset of assertions stays green.
  - Trigger condition: runtime artifact regression.
  - Impact: SWE-bench Pro proof still over-claims runtime coverage.
  - Proof needed: assert required artifacts/events or split the proof.

##### Non-blocking Risks

- `ADAPT-DATA-000` is a raw source-text sentinel.
- The meta-check originally proved only literal case-label presence.

##### Required Fixes

- Scope planned status to explicit Phase 0 adapter inventory.
- Add or map `create_task_plan` proof.
- Bring `INT-011` into runtime artifact compliance.

##### Missing Tests

- Add xtask verifier tests.
- Add negative test that non-adapter requirements cannot be planned.
- Add `INT-011` runtime artifact assertions.

##### Missing Logs / Observability

- Print claimed adapter IDs and active/planned breakdown.
- Add structured proof-inventory artifact.

##### Evidence

- `docs/test-engineering.md`
- `xtask/src/main.rs`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `crates/harnesslab-cli/tests/external_smoke_contract.rs`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| architecture | `META-002` semantic safety too weak | Single-doc free-text scan misses inventory claims, range syntax, and route semantics | blocking | accept | Reviewer evidence matched `xtask/src/main.rs` and inventory doc | Added multi-source claim scanning, range expansion, planned route semantic check, and adapter proof inventory artifact in `xtask/src/adapter_claims.rs` | Closure review |
| architecture | `META-002` artifact mismatch | `scripts/verify-test-registry.sh` did not generate declared traceability artifact | blocking | accept | Registry row declared `artifacts/test-traceability.json` | Updated `scripts/verify-test-registry.sh` to run `generate-test-traceability`; added `artifacts/adapter-proof-inventory.json` to `META-002` artifacts | Closure review |
| architecture | `ADAPT-DATA-000` does not align with plan | Plan expected a failing gap test, implementation used passing sentinel | blocking | accept | Plan and implementation wording conflicted | Updated architecture plan and inventory to define active gap sentinel; sentinel now checks `create_task_plan` too | Closure review |
| architecture | `INT-011` artifact overclaim | Registry listed artifacts not asserted | major | accept | Reviewer found missing assertions for profile/runtime artifacts | Added runtime artifact assertions in `int_011_swe_bench_pro_smoke_runs_external_evaluator_contract`; updated registry artifact list to actual asserted artifacts | Closure review |
| architecture | `INT-011` counts functions not scenarios | Function count can hide scenario bundling | major | defer | Phase 0 target is to avoid zero/under-counted selector; Phase 5 `SWEPRO-*` planned IDs split semantics | Kept counted group for Phase 0; scenario-level split remains in `SWEPRO-*` planned IDs | Phase 5 |
| test-validity | `create_task_plan` missing from sentinel | Partial contract extraction can evade sentinel | blocking | accept | Plan trait includes `create_task_plan` | Added `ADAPT-DATA-005`; updated sentinel, plan, inventory, requirements, registry, and selector | Closure review |
| test-validity | Only one claim doc scanned | Inventory doc can over-claim | blocking | accept | Inventory doc now independently claims controls | Scanner now reads architecture design and Phase 0 inventory docs | Closure review |
| test-validity | Selector route validation is textual and untested | Route can exist but execute wrong behavior | blocking | accept | Original check used `script.contains` | Added route parser, planned route semantic check, xtask unit tests, and `META-008` planned selector shell verifier | Closure review |
| implementation | Planned status weakens registry globally | Any normal requirement could be marked planned | blocking | accept | Original verifier skipped all planned requirements | Planned status now allowed only for adapter IDs claimed in scanned docs; added unit test and docs rule | Closure review |
| implementation | `create_task_plan` has no proof surface | Phase 1 can close without task-plan creation proof | blocking | accept | No `ADAPT-DATA-005` before fix | Added `ADAPT-DATA-005` across plan, requirements, registry, and selector | Closure review |
| implementation | `INT-011` artifact standard incomplete | Runtime artifacts can disappear while grouped test stays green | blocking | accept | Test asserted only subset of artifacts | Added `assert_swe_runtime_artifacts` for run-level and attempt-level artifacts, `run --json`, `run_finished`, patch and prediction outputs | Closure review |
| implementation | Raw source sentinel is brittle | Comments/refactors can affect source-text test | major | defer | Sentinel is explicitly temporary Phase 0 proof and fails when Phase 1 methods appear | Documented temporary sentinel; Phase 1 must replace with real contract tests | Phase 1 |

Validation evidence after fixes:

- `cargo fmt`: passed.
- `cargo test -p xtask -- --nocapture`: 9 passed.
- `scripts/test-after-change.sh --select META-008`: 15 planned adapter selectors exited 64 with planned-proof messages.
- `scripts/test-after-change.sh --select META-002`: passed, produced traceability and adapter proof inventory.
- `scripts/test-after-change.sh --select INT-011`: ran 10 tests and passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-000`: passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-005`: exited 64 with planned-proof message.
- `cargo test -p harnesslab-adapters -- --nocapture`: 23 passed.
- `git diff --check`: passed.

## Round 2: Accepted-Blocker Closure Re-Review

### Review Input

#### Objective

Falsify whether Round 1 accepted blocking findings for Benchmark Adapter Phase 0
are actually closed. This round is a closure review, not a broad restart of the
architecture track.

#### Review Target

Phase 0 proof infrastructure, selector behavior, registry semantics, SWE-bench
Pro grouped proof, plan/inventory docs, and the review report.

#### Target Locations

- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-04-benchmark-adapter-phase-0-inventory.md`
- `vs_review/2026-06-04-benchmark-adapter-phase-0-review.md`
- `xtask/src/main.rs`
- `xtask/src/adapter_claims.rs`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `scripts/test-after-change.sh`
- `scripts/verify-test-registry.sh`
- `scripts/verify-planned-adapter-selectors.sh`
- `docs/test-engineering.md`
- `crates/harnesslab-adapters/src/registry.rs`
- `crates/harnesslab-cli/tests/external_smoke_contract.rs`

#### Original Accepted Blocking Findings Under Closure Review

1. Adapter claim validation was semantically too weak: single-doc free-text
   scan, no range parsing, no route semantics.
2. `META-002` declared traceability artifacts without generating them.
3. `ADAPT-DATA-000` did not align with the Phase 0 contract and missed
   `create_task_plan`.
4. `verify-test-registry` scanned only one claim doc.
5. Selector validation was textual and untested.
6. Planned status could weaken global registry guarantees.
7. `create_task_plan` had no proof surface.
8. `INT-011` artifact/runtime proof was too weak.

#### Claimed Closure Evidence

- `cargo fmt`: passed.
- `cargo test -p xtask -- --nocapture`: 9 passed.
- `scripts/test-after-change.sh --select META-002`: passed; emitted
  `registry ok: 42 requirements, 168 tests`,
  `adapter proof claims ok: 16 ids from 2 sources`, and generated
  `artifacts/test-traceability.json`.
- `scripts/test-after-change.sh --select META-008`: passed; 15 planned adapter
  selectors exited planned-proof.
- `scripts/test-after-change.sh --select INT-011`: ran 10 tests and passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-000`: passed.
- `scripts/test-after-change.sh --select ADAPT-DATA-005` was checked to exit 64
  as planned proof.
- `cargo test -p harnesslab-adapters -- --nocapture`: 23 passed.
- `git diff --check`: passed.

#### Closure Risk Focus

- Planned status must be tightly scoped to explicit Phase 0 adapter inventory.
- Claim sources must be machine-checked across actual architecture and inventory
  docs.
- Selector routes must prove planned IDs exit through the planned-proof path and
  active IDs execute active tests.
- `INT-011` artifacts in the registry must match real assertions.
- The optimized plan must keep planned inventory separate from implemented proof.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Output: summary, blocking findings, non-blocking risks, required fixes,
  missing tests, missing logs/observability, and closure verdict.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 20 minutes | one bounded extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-adversary | Round 1 architecture blockers involved proof semantics, plan gates, and status boundaries. | architecture and maintainability |
| test-validity-adversary | Most accepted blockers relied on tests, selector behavior, and registry evidence. | test validity and evidence |
| implementation-adversary | Fixes touched Rust verifier logic, shell selectors, registry data, and SWE smoke assertions. | correctness and failure handling |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-adversary | `multi_agent_v1.spawn_agent` / architect | `019e92e2-7140-78c2-b93d-d814cefee535` | spawn_agent result, nickname `Cicero` | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |
| test-validity-adversary | `multi_agent_v1.spawn_agent` / test-engineer | `019e92e2-739a-7fa2-936d-1b5ad85aef1d` | spawn_agent result, nickname `Erdos` | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |
| implementation-adversary | `multi_agent_v1.spawn_agent` / code-reviewer | `019e92e2-7581-7463-9852-80de0bd5791d` | spawn_agent result, nickname `Hypatia` | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| architecture-round-2 | architecture-adversary | 1 | `019e92e2-7140-78c2-b93d-d814cefee535` | <=20 minutes | completed | Reviewer could not falsify Round 1 accepted-blocker closure from architecture perspective. | completed |
| test-validity-round-2 | test-validity-adversary | 1 | `019e92e2-739a-7fa2-936d-1b5ad85aef1d` | <=20 minutes | completed | Reviewer found `INT-011` artifact contract mismatch. | completed |
| implementation-round-2 | implementation-adversary | 1 | `019e92e2-7581-7463-9852-80de0bd5791d` | <=20 minutes | completed | Reviewer found active selector semantic gap and `INT-011` artifact contract mismatch. | completed |

### Reviewer Outputs

#### architecture-round-2

##### Summary

Closure passes for the accepted Round 1 blocker set from the architecture
perspective. Claimed adapter IDs are now machine-checked across both plan docs,
ranges are expanded, planned status is constrained to claimed adapter IDs,
promised traceability artifacts are generated, `ADAPT-DATA-005` covers
`create_task_plan`, and `INT-011` has counted execution plus concrete artifact
assertions.

##### Blocking Findings

- None.

##### Non-blocking Risks

- Claim-source discovery is still hard-coded to two docs.
- `INT-011` remains function-counted rather than scenario-counted.
- `docs/test-engineering.md` duplicated `META-008` for two purposes.

##### Required Fixes

- None required for Round 1 blocker closure.

##### Missing Tests

- Optional hardening: guard claim-source drift.

##### Missing Logs / Observability

- Optional hardening: include requirement status and selector route
  classification in `artifacts/adapter-proof-inventory.json`.

##### Closure Verdict

Pass.

#### test-validity-round-2

##### Summary

Closure fails from a test-validity perspective. Most blockers appear closed,
but `INT-011` still misdescribes some runtime artifact locations in
`tests/TEST_REGISTRY.toml`.

##### Blocking Findings

- `INT-011` does not fully align its declared runtime-artifact contract with
  what the test proves.
  - Broken assumption: the registry artifact list was updated to actual
    asserted artifacts.
  - Failure scenario: the registry claims root-level `git-diff.status.json`,
    `patch.diff`, `prediction.jsonl`, and `prediction.eval.json`, while the
    smoke test proves them under `tasks/<task-id>/attempts/1/`.
  - Trigger condition: traceability consumer, audit, or future artifact gate
    treats `tests/TEST_REGISTRY.toml` as the canonical proof contract.
  - Impact: the accepted `INT-011` runtime-artifact blocker is only partially
    closed.
  - Proof needed: make registry paths exactly match asserted attempt-level
    paths, or add a checker that compares `required_artifacts` to the asserted
    proof contract.

##### Non-blocking Risks

- Claim-source coverage is still a hard-coded two-file list.
- Some xtask tests are helper-level rather than full malformed-manifest tests.
- Planned ID list is duplicated in `scripts/verify-planned-adapter-selectors.sh`.

##### Required Fixes

- Fix `INT-011` `required_artifacts` so every attempt-level artifact uses
  `tasks/<task-id>/attempts/1/...`.
- Add validation that fails when registry-declared artifact paths diverge from
  the asserted grouped runtime proof.

##### Missing Tests

- Add a meta-test for `INT-011` artifact path alignment.
- Optional hardening: end-to-end negative planned-status and wrong-route tests.

##### Missing Logs / Observability

- Optional hardening: print claimed ID set or active/planned breakdown.

##### Closure Verdict

Fail.

#### implementation-round-2

##### Summary

Closure fails from an implementation perspective. Green commands are not enough
because active adapter selector behavior is not mechanically enforced, and
`INT-011` still misstates attempt-level artifacts.

##### Blocking Findings

- Active adapter selector behavior is still not mechanically enforced.
  - Broken assumption: planned-selector semantics closed the full textual
    selector validation blocker.
  - Failure scenario: `ADAPT-DATA-000` is rerouted to `planned_adapter_proof` or
    an unrelated passing test; `META-002` still passes because it only sees a
    case label, and `META-008` exercises only planned IDs.
  - Trigger condition: routine selector refactor or copy-paste edit.
  - Impact: the only active Phase 0 sentinel can stop proving the current gap
    while closure gates remain green.
  - Proof needed: add active claimed selector semantic validation or a meta-test
    that exercises active claimed selectors.
- `INT-011` machine-readable artifact contract still does not match the smoke
  test's actual proof.
  - Broken assumption: `assert_swe_runtime_artifacts()` made registry proof
    accurate.
  - Failure scenario: registry claims root-level patch artifacts while the test
    asserts attempt-level paths; the test also requires `git-diff.stderr.log`
    and `swe-bench-pro/eval/eval_results.json` not declared in the registry.
  - Trigger condition: audit, traceability consumer, or artifact-existence gate
    uses the registry as the contract.
  - Impact: the artifact generation mismatch/incomplete assertion blocker is
    not fully closed.
  - Proof needed: align `required_artifacts` with actual attempt-scoped files
    or reduce both to the same audited subset.

##### Non-blocking Risks

- Adapter claim scraping can be broadened by incidental future prose.
- `ADAPT-DATA-000` remains a temporary source-text sentinel.
- Planned ID list in the selector verifier must be kept in sync during later
  phases.

##### Required Fixes

- Add mechanical validation for active claimed adapter selectors.
- Align `INT-011` `required_artifacts` with actual attempt-scoped files.

##### Missing Tests

- Active claimed selector negative tests.
- Registry/artifact consistency test for `INT-011`.
- Optional: negative test for incidental doc mentions.

##### Missing Logs / Observability

- Optional: emit active/planned route classification in proof inventory.

##### Closure Verdict

Fail.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| test-validity | `INT-011` artifact contract paths mismatch tested attempt layout | Registry listed root-level patch artifacts while test asserted attempt-level paths | blocking | accept | Reviewer evidence matched `tests/TEST_REGISTRY.toml` and `external_smoke_contract.rs` | Updated `INT-011` `required_artifacts` to exact attempt-level paths, including `git-diff.stderr.log` and `swe-bench-pro/eval/eval_results.json`; added `xtask/src/runtime_artifacts.rs` meta-check with negative/positive unit tests | Round 3 closure review |
| implementation | Active adapter selector behavior not mechanically enforced | Active `ADAPT-DATA-000` could route to planned proof or unrelated passing test while case-label check passed | blocking | accept | Reviewer evidence matched `xtask/src/adapter_claims.rs` and `scripts/verify-planned-adapter-selectors.sh` | Added active route semantic validation to `xtask/src/adapter_claims.rs`, including negative tests for planned-proof and unrelated marker routes; updated `META-008` script to execute `ADAPT-DATA-000` plus all planned selectors | Round 3 closure review |
| implementation | `INT-011` artifact contract mismatch | Same failure as test-validity blocker | blocking | accept | Same evidence as test-validity finding | Same artifact contract fix and meta-check | Round 3 closure review |

Validation evidence after Round 2 fixes:

- `cargo fmt`: passed.
- `cargo test -p xtask -- --nocapture`: 13 passed.
- `scripts/test-after-change.sh --select META-002`: passed; generated traceability and adapter proof inventory.
- `scripts/test-after-change.sh --select META-008`: passed; active=1 planned=15.
- `scripts/test-after-change.sh --select INT-011`: ran 10 tests and passed.
- `cargo test -p harnesslab-adapters -- --nocapture`: 23 passed.
- `git diff --check`: passed.

## Round 3: Round 2 Blocker Closure Re-Review

### Review Input

#### Objective

Falsify whether the two accepted Round 2 blocking findings are now closed.

#### Review Target

The Round 2 fixes for active adapter selector semantics and `INT-011` runtime
artifact-contract alignment.

#### Target Locations

- `vs_review/2026-06-04-benchmark-adapter-phase-0-review.md`
- `xtask/src/main.rs`
- `xtask/src/adapter_claims.rs`
- `xtask/src/runtime_artifacts.rs`
- `scripts/test-after-change.sh`
- `scripts/verify-planned-adapter-selectors.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/test-engineering.md`
- `docs/plans/2026-06-04-benchmark-adapter-phase-0-inventory.md`
- `crates/harnesslab-cli/tests/external_smoke_contract.rs`
- `crates/harnesslab-adapters/src/registry.rs`

#### Accepted Round 2 Blockers Under Closure Review

1. `INT-011` `required_artifacts` did not match actual attempt-level artifacts
   asserted by the smoke test.
2. Active claimed adapter selector behavior was not mechanically enforced;
   `ADAPT-DATA-000` could be rerouted to planned proof or an unrelated passing
   test while gates stayed green.

#### Claimed Fixes

- `tests/TEST_REGISTRY.toml` now declares `INT-011` attempt-level artifacts
  under `tasks/<task-id>/attempts/1/`, including `git-diff.status.json`,
  `git-diff.stderr.log`, `patch.diff`, `prediction.jsonl`,
  `prediction.eval.json`, and `swe-bench-pro/eval/eval_results.json`.
- `xtask/src/runtime_artifacts.rs` adds an exact meta-check for the `INT-011`
  SWE-bench Pro runtime artifact contract, with negative and positive unit
  tests.
- `xtask/src/adapter_claims.rs` rejects active claimed adapter selectors that
  route to `planned_adapter_proof` and rejects active claimed routes that do not
  contain the snake-case marker for the claimed ID.
- `scripts/verify-planned-adapter-selectors.sh` now executes `ADAPT-DATA-000`
  plus the 15 planned adapter selectors.
- `docs/test-engineering.md` and the Phase 0 inventory doc describe the
  active+planned selector proof and `INT-011` artifact-contract validation.

#### Validation After Fixes

- `cargo fmt`: passed.
- `cargo test -p xtask -- --nocapture`: 13 passed.
- `scripts/test-after-change.sh --select META-002`: passed; registry ok, 16
  adapter proof claims from 2 sources, traceability generated.
- `scripts/test-after-change.sh --select META-008`: passed; active=1
  planned=15.
- `scripts/test-after-change.sh --select INT-011`: 10 tests passed.
- `cargo test -p harnesslab-adapters -- --nocapture`: 23 passed.
- `git diff --check`: passed.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Focus only on Round 2 blocker closure.
- Output: summary, blocking findings, non-blocking risks, required fixes,
  missing tests, missing logs/observability, and closure verdict.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 20 minutes | one bounded extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| test-validity-adversary | Round 2 blocker involved test/registry artifact proof mismatch. | test validity and evidence |
| implementation-adversary | Round 2 blocker involved selector semantics and verifier implementation. | correctness and failure handling |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| test-validity-adversary | `multi_agent_v1.spawn_agent` / test-engineer | `019e92ed-3e18-7430-9c53-7037410ca09c` | spawn_agent result, nickname `Bacon` | fork_context=false | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |
| implementation-adversary | `multi_agent_v1.spawn_agent` / code-reviewer | `019e92ed-3fc6-7fd0-8e77-fed8f1326cbd` | spawn_agent result, nickname `Mendel` | fork_context=false | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| test-validity-round-3 | test-validity-adversary | 1 | `019e92ed-3e18-7430-9c53-7037410ca09c` | <=20 minutes | completed | Reviewer found artifact-contract duplication and marker-spoof selector gaps. | completed |
| implementation-round-3 | implementation-adversary | 1 | `019e92ed-3fc6-7fd0-8e77-fed8f1326cbd` | <=20 minutes | completed | Reviewer could not falsify Round 2 closure from implementation perspective. | completed |

### Reviewer Outputs

#### test-validity-round-3

##### Summary

Closure fails from a test-validity perspective. The gates are green, but both
Round 2 closure claims still rely on duplicated constants or marker-string
heuristics rather than binding to the actual behavior they claim to protect.

##### Blocking Findings

- `INT-011` artifact-contract closure is still indirect.
  - Broken assumption: `xtask/src/runtime_artifacts.rs` verifies `INT-011`
    against the actual smoke-test assertions.
  - Failure scenario: `external_smoke_contract.rs` changes asserted artifacts,
    but `xtask/src/runtime_artifacts.rs` is not updated; `META-002` still passes
    because it compares registry to its own duplicated constant.
  - Trigger condition: edit to `assert_swe_runtime_artifacts` without matching
    manual edit to the xtask constant.
  - Impact: registry/test mismatch can recur while the gate remains green.
  - Proof needed: one shared artifact-contract source consumed by both the
    smoke test and registry verifier.
- Active selector enforcement is still spoofable.
  - Broken assumption: requiring the route line to contain the snake-case ID
    marker mechanically proves the intended active proof executes.
  - Failure scenario: route runs an unrelated passing test while preserving
    `adapt_data_000` in the line.
  - Trigger condition: selector refactor or copy-paste preserving the marker.
  - Impact: `ADAPT-DATA-000` can stop proving the intended current-gap sentinel.
  - Proof needed: exact dispatch validation for `package`, `test_name`, and
    `test_target`, plus a negative marker-spoof test.

##### Non-blocking Risks

- `META-008` hardcoded the planned selector list.
- New tests were meaningful but too narrow before exact dispatch validation.

##### Required Fixes

- Replace duplicated `INT-011` artifact lists with one shared source of truth.
- Tighten active selector validation from substring marker to exact dispatch
  validation.
- Make `META-008` derive active/planned IDs from the same claim/registry source.

##### Missing Tests

- Marker-spoof active-route negative test.
- Drift test proving artifact contract changes fail the registry verifier.

##### Missing Logs / Observability

- Include selector route/classification in proof inventory.
- Print the active selector being checked in `META-008`.

##### Closure Verdict

Fail.

#### implementation-round-3

##### Summary

Closure passes from an implementation perspective for the then-current Round 2
fixes. The reviewer could not falsify active selector execution or `INT-011`
artifact path alignment, but noted low-risk residual hardening around exact
active-route structure and proof inventory detail.

##### Blocking Findings

- None.

##### Non-blocking Risks

- Active-route binding was still substring-based in that reviewed version.
- Claim-source discovery remains hard-coded to two docs.
- `INT-011` contract enforcement was intentionally narrow to one grouped proof.

##### Required Fixes

- None for Round 2 closure in that reviewed version.

##### Missing Tests

- Optional hardening: exact active-route structure test.

##### Missing Logs / Observability

- Optional hardening: route classification in `adapter-proof-inventory.json`.

##### Closure Verdict

Pass.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| test-validity | `INT-011` artifact contract uses duplicated constants | Registry verifier compared to xtask constant instead of the smoke test's asserted contract | blocking | accept | The reviewer correctly identified drift between `external_smoke_contract.rs` and `xtask/src/runtime_artifacts.rs` as possible | Added shared `tests/artifact_contracts/int_011_swe_bench_pro_runtime_artifacts.txt`; smoke test and xtask verifier now both read it; `runtime_artifacts_002` proves contract drift fails | Round 4 closure review |
| test-validity | Active selector enforcement is marker-spoofable | Wrong test can preserve `adapt_data_000` marker and pass substring check | blocking | accept | The reviewer correctly identified marker containment as weaker than exact route validation | Replaced marker check with exact `package`/`test_name`/`test_target` validation for `ADAPT-DATA-000`; added marker-spoof negative test and exact-route positive test | Round 4 closure review |
| test-validity | `META-008` duplicates active/planned ID list | Hardcoded list can drift from claim/registry source | major | accept | The script maintained its own list | Added `xtask list-adapter-proof-selectors`; `META-008` derives active/planned IDs from the claim/registry source | Round 4 closure review |
| implementation | Optional exact active-route structure hardening | Same risk as test-validity marker-spoof blocker | major | accept | The subsequent test-validity review escalated the issue to blocking | Same exact route validation and tests | Round 4 closure review |

Validation evidence after Round 3 fixes:

- `cargo fmt`: passed.
- `cargo test -p xtask -- --nocapture`: 15 passed.
- `scripts/test-after-change.sh --select META-002`: passed; registry ok, 16
  adapter proof claims from 2 sources, traceability generated.
- `scripts/test-after-change.sh --select META-008`: passed; output includes
  `checking active adapter selector: ADAPT-DATA-000` and active=1 planned=15.
- `scripts/test-after-change.sh --select INT-011`: 10 tests passed.
- `cargo test -p harnesslab-adapters -- --nocapture`: 23 passed.
- `git diff --check`: passed.

## Round 4: Round 3 Blocker Closure Re-Review

### Review Input

#### Objective

Falsify whether the accepted Round 3 test-validity blockers are now closed.

#### Review Target

Shared `INT-011` artifact manifest, exact active selector route validation, and
dynamic active/planned adapter selector derivation.

#### Target Locations

- `tests/artifact_contracts/int_011_swe_bench_pro_runtime_artifacts.txt`
- `crates/harnesslab-cli/tests/external_smoke_contract.rs`
- `xtask/src/runtime_artifacts.rs`
- `xtask/src/adapter_claims.rs`
- `xtask/src/main.rs`
- `scripts/verify-planned-adapter-selectors.sh`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `docs/test-engineering.md`
- `docs/plans/2026-06-04-benchmark-adapter-phase-0-inventory.md`
- `vs_review/2026-06-04-benchmark-adapter-phase-0-review.md`

#### Accepted Round 3 Blockers Under Closure Review

1. `INT-011` artifact-contract closure was indirect because registry verifier
   and smoke test used duplicated artifact lists.
2. Active selector enforcement was marker-spoofable because `ADAPT-DATA-000`
   only needed a substring marker on the route line.
3. `META-008` duplicated active/planned adapter ID lists instead of deriving
   from the claim/registry source.

#### Claimed Fixes

- Added shared artifact manifest:
  `tests/artifact_contracts/int_011_swe_bench_pro_runtime_artifacts.txt`.
- `external_smoke_contract.rs` reads the shared manifest and asserts every path
  after substituting `<task-id>`.
- `xtask/src/runtime_artifacts.rs` reads the same shared manifest and compares
  `INT-011` `required_artifacts` exactly against it.
- Runtime artifact tests include bare attempt path rejection, contract drift
  rejection, and shared contract acceptance.
- `xtask/src/adapter_claims.rs` validates `ADAPT-DATA-000` by exact
  `package`/`test_name`/`test_target` assignments.
- `xtask/src/main.rs` adds `list-adapter-proof-selectors`;
  `scripts/verify-planned-adapter-selectors.sh` uses it to derive active and
  planned IDs dynamically and executes `ADAPT-DATA-000`.
- `adapter-proof-inventory.json` now includes selector class and selector route.

#### Validation After Fixes

- `cargo fmt`: passed.
- `cargo test -p xtask -- --nocapture`: 15 passed.
- `scripts/test-after-change.sh --select META-002`: passed.
- `scripts/test-after-change.sh --select META-008`: passed; output includes
  `checking active adapter selector: ADAPT-DATA-000` and active=1 planned=15.
- `scripts/test-after-change.sh --select INT-011`: 10 tests passed.
- `cargo test -p harnesslab-adapters -- --nocapture`: 23 passed.
- `git diff --check`: passed.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 20 minutes | one bounded extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| test-validity-adversary | Round 3 blockers were test/registry proof-surface issues. | test validity and evidence |
| implementation-adversary | Latest fixes changed Rust verifier logic and shell selector behavior. | correctness and failure handling |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| test-validity-adversary | `multi_agent_v1.spawn_agent` / test-engineer | `019e92f7-a4c8-7853-bd7a-701717b35a58` | spawn_agent result, nickname `Anscombe` | fork_context=false | Round 4 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |
| implementation-adversary | `multi_agent_v1.spawn_agent` / code-reviewer | `019e92f7-a7f2-7e73-b3e5-82f848a90b60` | spawn_agent result, nickname `Bernoulli` | fork_context=false | Round 4 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| test-validity-round-4 | test-validity-adversary | 1 | `019e92f7-a4c8-7853-bd7a-701717b35a58` | <=20 minutes | completed | Reviewer found duplicate assignment override gap in active route validation. | completed |
| implementation-round-4 | implementation-adversary | 1 | `019e92f7-a7f2-7e73-b3e5-82f848a90b60` | <=20 minutes | completed | Reviewer could not falsify latest implementation fixes. | completed |

### Reviewer Outputs

#### test-validity-round-4

##### Summary

Closure fails. The shared `INT-011` manifest and dynamic selector derivation are
materially closed, but active route validation still accepts the first matching
assignment while shell execution would use a later duplicate assignment.

##### Blocking Findings

- Active selector enforcement is spoofable through duplicate assignment
  override.
  - Broken assumption: exact `package` / `test_name` / `test_target`
    validation binds `ADAPT-DATA-000` to the intended proof.
  - Failure scenario: selector route keeps the expected `test_name` first but
    later overrides it with another passing test. The verifier reads the first
    occurrence while shell execution uses the last assignment.
  - Trigger condition: selector refactor or copy-paste introducing repeated
    assignments.
  - Impact: `ADAPT-DATA-000` can route to unrelated behavior while both
    `META-002` and `META-008` stay green.
  - Proof needed: reject duplicate `package=`, `test_name=`, and `test_target=`
    assignments, with a negative test for expected assignment first and
    overridden value later.

##### Non-blocking Risks

- `INT-011` closure is good for shared-list drift, but `META-002` does not prove
  the smoke test will keep consuming the manifest if that test is later
  rewritten.
- `list-adapter-proof-selectors` emits status and id only; enough for
  `META-008`, but not full provenance.

##### Required Fixes

- Reject duplicate route assignments or parse final effective assignments.
- Add duplicate assignment override negative test.

##### Missing Tests

- Duplicate assignment override test for active selector route.

##### Missing Logs / Observability

- Explicit verifier error for duplicate assignments.

##### Closure Verdict

Fail.

#### implementation-round-4

##### Summary

Closure passes from an implementation perspective. Shared `INT-011` artifact
contract, exact active selector validation, dynamic selector derivation, and
route classification are mechanically wired together in the reviewed version.

##### Blocking Findings

- None.

##### Non-blocking Risks

- Selector-route parsing is line-oriented and assumes current one-line case
  arms.
- `list-adapter-proof-selectors` is a lister, while `META-002` remains the
  semantic registry validator.

##### Required Fixes

- None for this closure round.

##### Missing Tests

- Optional hardening: multiline selector route assumption test.

##### Missing Logs / Observability

- None blocking.

##### Closure Verdict

Pass.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| test-validity | Active route assignment parser accepts first value while shell uses later duplicate override | Expected assignment can appear first and be overridden later on the same selector line | blocking | accept | Reviewer correctly identified duplicate `test_name=` as a spoof path not covered by exact-value checks | `assignment_values` now rejects duplicate `package`, `test_name`, and `test_target` assignments; added `registry_009_claimed_active_id_rejects_duplicate_assignment_override` negative test; duplicate errors are explicit | Round 5 closure review |

Validation evidence after Round 4 fix:

- `cargo fmt`: passed.
- `cargo test -p xtask -- --nocapture`: 16 passed.
- `scripts/test-after-change.sh --select META-002`: passed.
- `scripts/test-after-change.sh --select META-008`: passed.
- `git diff --check`: passed.

## Round 5: Duplicate Assignment Closure Re-Review

### Review Input

#### Objective

Falsify whether the accepted Round 4 duplicate-assignment blocker is closed.

#### Review Target

Active adapter selector validation for duplicate `package=`, `test_name=`, and
`test_target=` assignments.

#### Target Locations

- `xtask/src/adapter_claims.rs`
- `xtask/src/main.rs`
- `scripts/test-after-change.sh`
- `scripts/verify-planned-adapter-selectors.sh`
- `vs_review/2026-06-04-benchmark-adapter-phase-0-review.md`

#### Accepted Round 4 Blocker Under Closure Review

- Active selector enforcement was spoofable through duplicate assignment
  override: xtask read the first route assignment, while shell execution would
  use a later duplicate assignment.

#### Claimed Fix

- `xtask/src/adapter_claims.rs` now uses `assignment_values()` and
  `ensure_assignment()` to reject duplicate `package=`, `test_name=`, and
  `test_target=` assignments.
- Added `registry_009_claimed_active_id_rejects_duplicate_assignment_override`.
  It puts the expected `test_name` first and then overrides it with
  `some_other_passing_test`; verifier must fail with `duplicate test_name`.

#### Validation After Fix

- `cargo fmt`: passed.
- `cargo test -p xtask -- --nocapture`: 16 passed.
- `scripts/test-after-change.sh --select META-002`: passed.
- `scripts/test-after-change.sh --select META-008`: passed.
- `git diff --check`: passed.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 20 minutes | one bounded extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| test-validity-adversary | The remaining blocker is a selector-proof validity gap. | test validity and evidence |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| test-validity-adversary | `multi_agent_v1.spawn_agent` / test-engineer | `019e9301-ac08-74d1-b7ca-cbde44d20c36` | spawn_agent result, nickname `Helmholtz` | fork_context=false | Round 5 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| test-validity-round-5 | test-validity-adversary | 1 | `019e9301-ac08-74d1-b7ca-cbde44d20c36` | <=20 minutes | completed | Reviewer found mixed-form duplicate assignment override gap. | completed |

### Reviewer Outputs

#### test-validity-round-5

##### Summary

Closure fails. Duplicate double-quoted assignments are rejected, but mixed-form
duplicates using single-quoted or unquoted shell assignments can still override
the first value at runtime while the verifier sees only the double-quoted
assignment.

##### Blocking Findings

- Active selector enforcement is still spoofable through mixed-form duplicate
  assignment override.
  - Broken assumption: rejecting duplicate double-quoted assignments is enough
    to bind `ADAPT-DATA-000` to the intended proof.
  - Failure scenario: route keeps expected `test_name="..."` first, then
    overrides with `test_name='some_other_passing_test'` or
    `test_name=some_other_passing_test`.
  - Trigger condition: selector refactor or copy-paste using a later single
    quoted or unquoted assignment.
  - Impact: `ADAPT-DATA-000` can still route to unrelated behavior while
    `META-002` and `META-008` stay green.
  - Proof needed: parse all relevant shell assignment forms or reject repeated
    assignment tokens regardless of quote style; add single-quoted and unquoted
    override tests.

##### Non-blocking Risks

- Existing duplicate tests only covered the double-quoted `test_name` path.

##### Required Fixes

- Detect repeated `package=`, `test_name=`, and `test_target=` assignments
  across double-quoted, single-quoted, and unquoted forms.
- Add negative tests for single-quoted and unquoted overrides, and equivalent
  coverage for `package` / `test_target`.

##### Missing Tests

- Mixed-form duplicate assignment tests.

##### Missing Logs / Observability

- Explicit verifier error for mixed-form duplicate assignments.

##### Closure Verdict

Fail.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| test-validity | Mixed-form duplicate assignment override bypasses duplicate detection | Parser matched only `key=\"...\"`, while shell accepts later single-quoted or unquoted assignments | blocking | accept | Reviewer correctly identified that bash would execute the last assignment regardless of quote style | `assignment_values` now parses double-quoted, single-quoted, and unquoted assignment values; duplicate detection applies across all forms; added tests for single-quoted override, unquoted override, and duplicate `package` / `test_target` overrides | Round 6 closure review |

Validation evidence after Round 5 fix:

- `cargo fmt`: passed.
- `cargo test -p xtask -- --nocapture`: 19 passed.
- `scripts/test-after-change.sh --select META-002`: passed.
- `scripts/test-after-change.sh --select META-008`: passed.
- `git diff --check`: passed.

## Round 6: Mixed-Form Duplicate Assignment Closure Re-Review

### Review Input

#### Objective

Falsify whether the accepted Round 5 mixed-form duplicate-assignment blocker is
closed.

#### Review Target

Active selector assignment parsing and duplicate detection.

#### Target Locations

- `xtask/src/adapter_claims.rs`
- `scripts/test-after-change.sh`
- `scripts/verify-planned-adapter-selectors.sh`
- `vs_review/2026-06-04-benchmark-adapter-phase-0-review.md`

#### Accepted Round 5 Blocker Under Closure Review

- Active selector validation detected only duplicate double-quoted assignments.
  A selector could keep the expected `test_name="..."` first, then override with
  `test_name='some_other_passing_test'` or `test_name=some_other_passing_test`;
  xtask would pass while bash used the later assignment.

#### Claimed Fix

- `assignment_values()` now detects `key=` values for double-quoted,
  single-quoted, and unquoted shell assignment forms.
- `ensure_assignment()` rejects duplicate `package=`, `test_name=`, and
  `test_target=` assignments across all parsed forms.
- Added tests for double-quoted duplicate, single-quoted override, unquoted
  override, and duplicate `package` / `test_target` overrides.

#### Validation After Fix

- `cargo fmt`: passed.
- `cargo test -p xtask -- --nocapture`: 19 passed.
- `scripts/test-after-change.sh --select META-002`: passed.
- `scripts/test-after-change.sh --select META-008`: passed.
- `git diff --check`: passed.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 20 minutes | one bounded extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| test-validity-adversary | The remaining blocker is selector-proof validity. | test validity and evidence |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| test-validity-adversary | `multi_agent_v1.spawn_agent` / test-engineer | `019e9305-eff8-74f3-b941-2c1f89676252` | spawn_agent result, nickname `Carver` | fork_context=false | Round 6 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless independently inspected | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| test-validity-round-6 | test-validity-adversary | 1 | `019e9305-eff8-74f3-b941-2c1f89676252` | <=20 minutes | completed | Reviewer could not falsify mixed-form duplicate assignment closure. | completed |

### Reviewer Outputs

#### test-validity-round-6

##### Summary

Closure passes for the accepted Round 5 duplicate-assignment blocker. The core
spoof path is now rejected: `ensure_assignment()` fails on duplicate
`package=`, `test_name=`, or `test_target=` assignments, and
`assignment_values()` / `shell_assignment_value()` parse double-quoted,
single-quoted, and unquoted forms.

##### Blocking Findings

- None.

##### Non-blocking Risks

- `assignment_values()` uses substring search for `key=`, which is not a
  surviving bypass for the reviewed blocker but could create future false
  failures if unrelated tokens contain `package=` or `test_name=`.
- `verify-test-registry` still logs aggregate success counts rather than parsed
  assignment maps; acceptable for this closure.

##### Required Fixes

- None for the accepted Round 5 blocker.

##### Missing Tests

- None blocking.
- Optional hardening: parser test for prefixed/non-assignment substrings.

##### Missing Logs / Observability

- None blocking.

##### Closure Verdict

Pass.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| test-validity | No blocking findings | n/a | n/a | n/a | Round 6 passed and no required fixes remain for accepted blockers | No action needed | none |

Final validation evidence:

- `cargo fmt`: passed.
- `cargo test -p xtask -- --nocapture`: 19 passed.
- `scripts/test-after-change.sh --select META-002`: passed.
- `scripts/test-after-change.sh --select META-008`: passed.
- `scripts/test-after-change.sh --select INT-011`: 10 tests passed.
- `cargo test -p harnesslab-adapters -- --nocapture`: 23 passed.
- `git diff --check`: passed.

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 2 failed and produced accepted blockers.
  - Round 3 failed and produced accepted blockers.
  - Round 4 failed and produced accepted blockers.
  - Round 5 failed and produced accepted blockers.
  - Round 6 passed.
- Blocking re-review launch records:
  - Round 2: `019e92e2-7140-78c2-b93d-d814cefee535`, `019e92e2-739a-7fa2-936d-1b5ad85aef1d`, `019e92e2-7581-7463-9852-80de0bd5791d`
  - Round 3: `019e92ed-3e18-7430-9c53-7037410ca09c`, `019e92ed-3fc6-7fd0-8e77-fed8f1326cbd`
  - Round 4: `019e92f7-a4c8-7853-bd7a-701717b35a58`, `019e92f7-a7f2-7e73-b3e5-82f848a90b60`
  - Round 5: `019e9301-ac08-74d1-b7ca-cbde44d20c36`
  - Round 6: `019e9305-eff8-74f3-b941-2c1f89676252`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

Phase 0 proof-surface implementation is locally validated and adversarially
closed. Later adapter behavior remains planned until subsequent phases activate
the registered `ADAPT-DATA-*`, `ADAPT-RUNTIME-*`, and `SWEPRO-*` proof IDs.
