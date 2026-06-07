# Subagent VS Review: Universal Benchmark Adapter Protocol Phase 1

- Created: 2026-06-08
- Updated: 2026-06-08
- Report schema: adversarial-v1
- Task: Phase 1 protocol specification and planned gate registration review
- Report path: `vs_review/2026-06-08-universal-benchmark-adapter-protocol-phase-1-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: in progress; closure review pending

## Round 1: Protocol Spec And Planned Gate Review

### Review Input

#### Objective

Review Phase 1 of
`docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`.
The implementation slice adds `docs/adapter-protocol.md` and registers
`ADAPT-PROTOCOL-001..012` as planned protocol gates.

#### Review Target

- Protocol specification completeness.
- Phase 0 behavior mapping into protocol capabilities.
- Planned selector/requirement/test registry coverage.
- Whether Phase 1 exit criteria can honestly close.

#### Target Locations

- `docs/adapter-protocol.md`
- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`
- `prd/2026-06-07-universal-benchmark-adapter-protocol.md`
- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-phase-0-branch-inventory.md`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `scripts/test-after-change.sh`
- `scripts/verify-planned-adapter-selectors.sh`
- `xtask/src/adapter_claims.rs`

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-protocol-reviewer | `multi_agent_v1.spawn_agent` | `019ea32b-fabf-71e2-b795-eb221062a810` | spawn tool result | false | Round 1 protocol review packet plus target paths and verification evidence | main-agent history, reasoning, drafts, conclusions, full diff | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| architecture-protocol-reviewer | architecture-protocol-reviewer | 1 | `019ea32b-fabf-71e2-b795-eb221062a810` | completed | completed | returned blocking findings | accepted blockers fixed locally; fresh closure review required |

### Reviewer Outputs

#### architecture-protocol-reviewer

Verdict: `REQUEST CHANGES`.

Blocking findings:

- `docs/adapter-protocol.md` had the right skeleton, but did not map Phase 0's
  concrete Terminal-Bench and SWE-bench Pro behavior to protocol capabilities,
  required schema fields, artifacts, failure mapping, readiness probes, and
  future protocol selectors.
- Optional capability contracts were too generic. They named capabilities such
  as `docker.orchestration`, `host.agent_execution`,
  `cleanup.verdict_override`, and `patch.evaluator`, but did not define
  required operations, artifacts, readiness probes, failure subphases, or
  report metadata.
- The selected-mode/capability compatibility rule was not testable because no
  normative matrix defined what `official-runner`, `patch-evaluator`, or
  `deterministic-sample` require.

Non-blocking risks:

- Product questions remain open for later phases: toy vs real third adapter,
  out-of-tree packaging timing, public detail code policy, and old-only
  `ExternalRunnerKind` deprecation.
- Stable promotion evidence schema was present in the implementation plan but
  not fully included in the protocol spec.

### Main-agent Response To Round 1

| Finding | Response | Fix |
|---|---|---|
| Existing adapters were not mapped to protocol fields. | accept | Added `Existing Adapter Behavior Mapping` covering Terminal-Bench runtime/cleanup/log/Python bridge behavior, SWE-bench Pro metadata/workspace/evaluator/runtime snapshots, registry dispatch, doctor run-as, report, redaction, and official proof surfaces. |
| Optional capability contracts were not enforceable. | accept | Added per-capability contracts for `official.runner`, `patch.evaluator`, `cleanup.verdict_override`, `host.agent_execution`, `docker.orchestration`, `sandbox.runner`, `run_as.readiness`, and `custom.report_panel`. |
| Selected-mode compatibility was not testable. | accept | Added selected-mode compatibility matrix with required capabilities, invalid combinations, and `mode_capability_mismatch` failure. |
| Stable promotion evidence schema missing from protocol. | accept | Added stable promotion evidence fields to `docs/adapter-protocol.md`. |
| Dispatch observability was only in later plan notes. | accept | Added observability contract requiring benchmark id, adapter id/version, protocol version, selected mode, capability, stability, legacy shim usage, and failure phase fields. |

### Local Fix Evidence Before Round 2

Commands run after fixes:

- `cargo fmt --all --check`: passed.
- `cargo test -p xtask adapter_claims -- --nocapture`: 15 tests passed.
- `scripts/verify-test-registry.sh`: passed with `registry ok: 56
  requirements, 184 tests` and `adapter proof claims ok: 29 ids from 4
  sources`.
- `cargo run -p xtask -- verify-frozen-selector-manifest`: passed.
- `CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh`: passed
  with `adapter selectors ok: active=16 planned=13`.
- `cargo check -p xtask`: passed.
- `git diff --check`: passed.

## Round 2: Closure Review

### Review Input

#### Objective

Verify whether accepted Round 1 blockers are closed by the updated Phase 1
protocol specification and planned gate registration.

#### Review Target

- Existing adapter behavior mapping.
- Optional capability contracts.
- Selected-mode compatibility matrix.
- Stable promotion evidence schema.
- Dispatch observability contract.
- Planned `ADAPT-PROTOCOL-001..012` gate registration.

#### Target Locations

- `docs/adapter-protocol.md`
- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-phase-0-branch-inventory.md`
- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `scripts/test-after-change.sh`
- `scripts/verify-planned-adapter-selectors.sh`
- `xtask/src/adapter_claims.rs`

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| phase-1-closure-reviewer | `multi_agent_v1.spawn_agent` | `019ea330-77e8-78d0-b401-f8630f3a2fb0` | spawn tool result | false | Round 2 closure review packet plus target paths and verification evidence | main-agent history, reasoning, drafts, conclusions, full diff | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| phase-1-closure-reviewer | phase-1-closure-reviewer | 1 | `019ea330-77e8-78d0-b401-f8630f3a2fb0` | completed | completed | returned PASS | closure accepted |

### Reviewer Outputs

#### phase-1-closure-reviewer

Verdict: `PASS`.

Summary:

- Phase 1 blocker fixes are present and materially address the accepted
  findings.
- Existing behavior mapping covers Terminal-Bench, SWE-bench Pro, registry
  dispatch, reports, redaction, and official proof surfaces.
- Optional capability contracts define operations, artifacts, readiness,
  failures, and report metadata.
- Selected-mode compatibility matrix exists and is normative.
- Stable promotion evidence fields exist.
- Protocol observability contract requires benchmark id, adapter id/version,
  protocol version, selected mode, capability, stability, legacy shim usage,
  failure fields, and adapter phase/subphase.
- `ADAPT-PROTOCOL-001..012` planned gates are registered in requirements,
  test registry, selector routing, and planned selector inventory.

Blocking findings: none.

Non-blocking risks:

- Selected-mode negative fixtures remain planned for later activation through
  `ADAPT-PROTOCOL-002`, which is acceptable for Phase 1.

### Round 2 Closure Status

Accepted. All blocking findings from Round 1 have a recorded response,
implemented fix, local validation evidence, and fresh closure review. Phase 1
slice is ready to commit.
