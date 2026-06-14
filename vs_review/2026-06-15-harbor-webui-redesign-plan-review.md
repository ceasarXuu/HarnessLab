# Subagent VS Review: Harbor WebUI Redesign Plan

- Created: 2026-06-15T03:26:33+0800
- Updated: 2026-06-15T03:58:00+0800
- Report schema: adversarial-v1
- Task: Evaluate and harden the HarnessLab Harbor WebUI redesign plan before implementation.
- Report path: `vs_review/2026-06-15-harbor-webui-redesign-plan-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: passed

## Round 1: Architecture And Product Plan Review

### Review Input

#### Objective

Adversarially review the new HarnessLab Harbor WebUI redesign plan and related
PRD/archive handling before it is treated as implementation-ready.

#### Review Target

Architecture, product, engineering plan, test strategy, observability strategy,
and documentation archival.

#### Target Locations

- `docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`
- `prd/2026-06-15-harnesslab-webui-prd.md`
- `docs/archive/2026-06-15-pre-harbor-webui-redesign/README.md`
- `docs/architecture.md`
- `docs/technology-decisions.md`
- `docs/adapter-protocol.md`

#### Change Introduction

The plan abandons a self-owned Rust runtime and Rust-Python bridge. It chooses
Python/FastAPI + Vue 3 + TypeScript, local SQLite metadata plus file artifacts,
SSE logs, direct Harbor 0.13.x API integration, and a declarative AgentProfile
compiler that maps built-in Harbor agents directly and materializes
custom-command agents only when needed. Legacy CLI-first/self-runtime documents
were moved to `docs/archive` with original-path stubs.

#### Risk Focus

- Harbor API stability or capability assumptions.
- Generated custom-agent security and maintenance risk.
- SQLite/file artifact drift and recovery gaps.
- Cancellation, queueing, logs, and backend restart recovery concreteness.
- Testable phase gates versus narrative gates.
- Archive/stub effectiveness and traceability risk.
- PRD and engineering plan consistency.

#### User-Perspective Review Focus

- Whether a realistic user can understand what HarnessLab adds over Harbor.
- Whether a first-time user can complete agent registration and run an experiment.
- Whether failed Docker/Harbor jobs are understandable and recoverable.
- Whether reports and leaderboards remain understandable while preserving raw evidence.

#### Assumptions To Attack

- Direct Harbor Python API is safer than invoking Harbor CLI.
- Vue/FastAPI/SQLite is the best local-first implementation shape.
- Single active job plus queue is enough for MVP.
- SSE is sufficient for live logs/status.
- Original-path stubs prevent stale docs from being misread.
- Existing Rust test discipline can be reused in a Python/Web rewrite.

#### Adversarial Lenses

- architecture
- product logic
- usability
- maintenance
- testing
- observability
- security
- release/migration

#### Verification Status

- Author inspected local Harbor 0.13.2 CLI/API and existing job artifacts.
- No implementation tests exist yet because this is a planning/documentation change.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| complex | 15 minutes | one bounded 10 minute extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-adversary | Highest risk is whether the redesign creates durable boundaries, avoids reimplementing Harbor, and has concrete migration/testing gates. | architecture, maintenance, testing, observability |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-adversary | `multi_agent_v1.spawn_agent` with `agent_type=critic` | `019ec799-813a-7672-a7fd-10b71b0b4a02` | spawn_agent tool result in current thread | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round1-architecture-adversary | architecture-adversary | 1 | `019ec799-813a-7672-a7fd-10b71b0b4a02` | < 15 minutes | completed | Reviewer returned `REJECT` with blocking findings. | completed |

### Reviewer Outputs

#### round1-architecture-adversary

##### Summary

Reviewer verdict: `REJECT`. The direction is clear, but the plan is not
implementation-ready because four high-risk contracts were not closed:
Harbor lifecycle control, experiment/leaderboard semantics, SQLite/file
recovery authority, and archive/stub coverage.

##### Blocking Findings

- Harbor lifecycle contract is not concrete enough.
  - Broken assumption: Direct Harbor Python API is safer than CLI and already sufficient for cancel, restart recovery, and queue serialization.
  - Failure scenario: Backend cancels or restarts while `Job.run()` continues or leaves Docker resources behind; SQLite and actual Harbor job state drift.
  - Trigger condition: Long benchmark, Docker hang, user cancellation, backend restart.
  - Impact: Queue blockage, orphan containers, wrong terminal status, unreliable recovery.
  - Proof needed: Harbor 0.13.2 API spike covering async call shape, durable job identity, cancellation escalation, restart recovery, and real cancel/restart tests.
- Experiment product semantics are not converged.
  - Broken assumption: `Experiment` can safely mean single run, comparison, batch, report, queue item, and leaderboard sample.
  - Failure scenario: Implementers invent conflicting meanings for multi-agent/multi-benchmark fan-out and Harbor job mapping.
  - Trigger condition: PRD multi-agent or multi-benchmark creation flow.
  - Impact: API, storage, queue, report, and leaderboard drift.
  - Proof needed: Canonical `experiment`, `run`, `batch`, `comparison`, and `template` contract.
- SQLite plus file artifact recovery authority is undefined.
  - Broken assumption: SQLite index plus file artifacts plus startup reconcile is enough.
  - Failure scenario: Crash between DB writes, file writes, Harbor result, event mirroring, or report generation creates unrecoverable drift.
  - Trigger condition: `kill -9`, disk full, report interruption, service upgrade.
  - Impact: Lost queue order, state drift, inconsistent logs, unclear repair authority.
  - Proof needed: Queue schema, event authority, write order, recovery priority, manual repair, and kill-at-transition tests.
- Archive/stub strategy misses the repo root.
  - Broken assumption: Stubbing old docs under `docs/` prevents stale product understanding.
  - Failure scenario: A user or fresh agent starts from root `README.md` and reads the old Rust CLI benchmark harness narrative.
  - Trigger condition: Normal repository onboarding.
  - Impact: Misunderstands HarnessLab vs Harbor and active implementation path.
  - Proof needed: Update root README and docs index to point to the Harbor WebUI plan.

##### Non-blocking Risks

- Custom-command trust boundary is under-specified.
  - Broken assumption: Limiting to `{{instruction}}` and env allowlists is enough.
  - Failure scenario: Imported/shared command profiles leak credentials or perform unsafe host/path side effects.
  - Trigger condition: Custom command with install/run scripts and `include_paths`.
  - Impact: Local security and reproducibility failures.
  - Proof needed: Trusted-local-operator rule, command preview, mount diagnostics, and warnings.
- SSE disconnect recovery is too weak.
  - Broken assumption: One-way stream is automatically enough.
  - Failure scenario: Browser refresh or network blip loses status/log events.
  - Trigger condition: Long logs, page refresh, tab switch, network interruption.
  - Impact: Users see discontinuous progress and cannot audit missed events.
  - Proof needed: `Last-Event-ID`, offset replay, and unavailable replay behavior.
- Rust test discipline migration is underspecified.
  - Broken assumption: Existing registry/gates can be reused with small naming changes.
  - Failure scenario: `WEB-*` IDs exist but gates remain Cargo/Rust centered.
  - Trigger condition: Python/frontend implementation begins.
  - Impact: Test discipline appears preserved but no longer protects the actual product.
  - Proof needed: Python/Web registry and gate migration plan.

##### User-Perspective Checks

- HarnessLab vs Harbor: risk, because root README still described the old product.
- First-run flow: risk, because Harbor/Docker/credential preflight was not clearly first.
- Docker/Harbor failures: target is good, but cancel/restart recovery contract was not executable.
- Reports/leaderboards: generally pass on preserving raw evidence through summary shell plus Harbor artifact links.

##### Required Fixes

- Add Harbor lifecycle ADR/spike covering real API, identity, cancel escalation, restart recovery, and fallback to CLI/worker execution.
- Define canonical `Experiment`/`Run`/`Batch`/`Comparison`/`Template` semantics and sync PRD/plan.
- Add crash-consistency design for SQLite plus files.
- Add leaderboard eligibility/comparability rules.
- Update root README and docs index.

##### Missing Tests

- Real Harbor cancel + backend restart + orphan cleanup.
- Kill-after-transition crash-consistency tests.
- Leaderboard comparability/exclusion tests, including smoke exclusion.
- SSE reconnect/resume tests.
- Custom-command quoting, mount support, and diagnostics tests.
- First-start doctor/preflight e2e.

##### Missing Logs / Observability

- `experiment.cancel_requested`, `experiment.cancel_escalated`, `experiment.reconciled`, `experiment.reconcile_decision`
- `queue.position_assigned`, `queue.dequeued`, `queue.rebuilt_from_store`
- `state.authority_selected`, `sqlite_file_drift_detected`
- `leaderboard.entry_included`, `leaderboard.entry_excluded`
- Harbor API capability/version snapshot event

##### Evidence

- `docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`
- `prd/2026-06-15-harnesslab-webui-prd.md`
- `docs/archive/2026-06-15-pre-harbor-webui-redesign/README.md`
- `docs/architecture.md`
- `docs/technology-decisions.md`
- `docs/adapter-protocol.md`
- `README.md`
- `tests/TEST_REGISTRY.toml`
- `scripts/test-after-change.sh`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| architecture-adversary | Harbor lifecycle contract is not concrete enough. | Direct Harbor Python API can already support cancel/restart/queue semantics. | blocking | accept | Harbor `0.13.2` has async `Job.create`/`Job.run` but no stable top-level `Job.cancel()` contract. | Added `4.1.1 Harbor Lifecycle Contract` with spike requirements, best-effort cancellation sequence, cleanup escalation, and fallback decision gate. | Closure re-review Round 2. |
| architecture-adversary | Experiment product semantics are not converged. | One `Experiment` object can cover single run, batch, comparison, queue, report, and leaderboard sample. | blocking | accept | PRD allowed multi-agent and multi-benchmark flows while plan schema had one benchmark per experiment. | Added `4.5 Entity Semantics`, `runs` table, run-level API endpoints, fan-out rules, and PRD terminology/flow updates. | Closure re-review Round 2. |
| architecture-adversary | SQLite plus file recovery authority is undefined. | SQLite index plus files plus reconcile is enough without write-order and authority rules. | blocking | accept | Plan had both SQLite events and JSONL events without source-of-truth definition. | Added `4.6 Crash Consistency And Recovery`, `queue_items`, event mirror offsets, write order, startup recovery, and kill-at-transition test requirements. | Closure re-review Round 2. |
| architecture-adversary | Archive/stub misses repo root. | Stubbing old docs under `docs/` is enough. | blocking | accept | Root `README.md` still described old Rust CLI product. | Rewrote root `README.md`, added `docs/README.md`, updated archive README and Phase 0 criteria. | Closure re-review Round 2. |
| architecture-adversary | Custom-command trust boundary is under-specified. | `{{instruction}}` and env allowlist are sufficient. | major | accept | Custom command profiles can run arbitrary local/operator-provided commands. | Added trusted-local-operator rule, command preview, mount/back-end support checks, and first-run warning requirements. | Validate in Phase 2 tests. |
| architecture-adversary | SSE reconnect contract is too weak. | One-way stream is enough without event ids and offsets. | major | accept | Long-running browser sessions need reconnect/replay semantics. | Added SSE recovery contract using monotonic event id, `Last-Event-ID`, `after`, log offsets, and `log.replay_unavailable`. | Validate in Phase 3 tests. |
| architecture-adversary | Test discipline migration is under-specified. | Existing Rust registry/gates can be reused directly. | major | accept | Current test registry and scripts are Rust/Cargo centered. | Added `8.4 Test Engineering Migration`, web registry/gate tasks, and Phase 1 gate skeleton requirement. | Validate in Phase 1 docs/tests. |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: pending
- Blocking re-review passed: pending
- Blocking re-review round links:
  - Round 2 pending
- Blocking re-review launch records:
  - pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: accepted blocking findings require closure review
- Allowed to proceed: no

## Final Conclusion

Round 1 found accepted blocking issues. Main-agent fixes were applied to the
plan, PRD, README, and docs index. A fresh closure re-review is required before
the plan may be marked passed.

## Round 2: Accepted Blocking Closure Review

### Review Input

#### Objective

Verify closure of accepted blocking findings from Round 1 for the HarnessLab
Harbor WebUI redesign plan.

#### Review Target

Closure review for plan/PRD/README/doc-index updates responding to Round 1
blocking findings and major risks.

#### Target Locations

- `docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`
- `prd/2026-06-15-harnesslab-webui-prd.md`
- `README.md`
- `docs/README.md`
- `docs/archive/2026-06-15-pre-harbor-webui-redesign/README.md`
- `vs_review/2026-06-15-harbor-webui-redesign-plan-review.md`

#### Change Introduction

Round 1 accepted blocking findings were addressed by adding a Harbor lifecycle
contract/spike, explicit experiment/run/batch/comparison semantics, SQLite/file
crash consistency rules, leaderboard comparability rules, SSE replay semantics,
custom-command trust-boundary requirements, Python/Web test migration guidance,
and root README/docs entry updates.

#### Risk Focus

- Whether the accepted blocking findings are truly closed at planning level.
- Whether closure introduced new contradictions.
- Whether PRD and plan now agree on experiment/run/report/leaderboard semantics.
- Whether root entrypoints no longer advertise the old Rust runtime as current.

#### User-Perspective Review Focus

- Whether a fresh user can start from `README.md` or `docs/README.md` and find
  the active Harbor WebUI direction.
- Whether first-run, failure recovery, and leaderboard semantics are
  understandable.

#### Assumptions To Attack

- The added lifecycle spike is concrete enough to block unsafe Phase 3 work.
- Run-level fan-out solves multi-agent/multi-benchmark ambiguity.
- Crash consistency rules are testable.
- README/docs index closure prevents stale onboarding.

#### Adversarial Lenses

- architecture
- product logic
- recovery
- testing
- documentation/onboarding

#### Verification Status

- Round 1 findings accepted and plan/PRD/README docs edited.
- No implementation code exists yet for the new architecture.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| complex | 15 minutes | one bounded 10 minute extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-adversary | Closure target is architecture/product contract adequacy after accepted blocking fixes. | architecture, product semantics, recovery, documentation |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-adversary | `multi_agent_v1.spawn_agent` with `agent_type=critic` | `019ec7a3-a593-7491-a470-8791957c0c3a` | spawn_agent tool result in current thread | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless needed | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round2-closure-architecture-adversary | architecture-adversary | 1 | `019ec7a3-a593-7491-a470-8791957c0c3a` | < 15 minutes | completed | Reviewer returned `OKAY`; accepted blocking findings are closed for planning stage. | completed |

### Reviewer Outputs

#### round2-closure-architecture-adversary

##### Summary

Reviewer verdict: `OKAY`. The four accepted Round 1 blocking findings are
adequately closed for a plan-stage artifact. The reviewer cited concrete
additions for Harbor lifecycle fallback gates, experiment/run semantics,
crash-consistency authority, and root/docs/archive onboarding.

##### Blocking Findings

- none

##### Non-blocking Risks

- Real Harbor cancel/orphan-cleanup proof is still more implicit than explicit.
  - Broken assumption: lifecycle spike plus fake-cancel coverage is enough to prove real cleanup behavior.
  - Failure scenario: a cancel request marks a run terminal while Harbor work or Docker resources continue.
  - Trigger condition: mid-run cancel or backend death during cancel grace.
  - Impact: orphan containers, blocked queue, false terminal state.
  - Proof needed: explicit real Harbor cancel + post-restart cleanup assertion, or state that Phase 1 spike artifact is authoritative proof.
- Custom-command trust-boundary tests are still thin.
  - Broken assumption: preview/warning plus env allowlist fully proves safe compilation behavior.
  - Failure scenario: quoting or unsupported mount cases compile into unsafe or backend-broken agents.
  - Trigger condition: complex shell strings or `include_paths` on non-Docker backends.
  - Impact: unsafe local execution or non-reproducible runs.
  - Proof needed: explicit quoting-normalization and mount-denial tests.

##### User-Perspective Checks

- Onboarding passes: root `README.md` and `docs/README.md` point to the Harbor WebUI source of truth.
- First-run flow passes at plan stage: PRD now starts with Harbor/Docker/disk/data-dir preflight.
- Failure recovery is understandable enough for planning: `cancelled` versus `interrupted` and restart reconciliation are explicit.
- Leaderboard/report semantics pass: runs, not experiments, are ranked; exclusion reasons are surfaced.

##### Required Fixes

- none required for accepted Round 1 blocker closure

##### Missing Tests

- No blocker-level missing tests remain.
- Recommended additions: real Harbor cancel/restart/orphan-cleanup test; custom-command quoting and unsupported-mount denial tests.

##### Missing Logs / Observability

- No blocker-level gap remains.
- Optional strengthening: add explicit `experiment.interrupted` event if `interrupted` remains a first-class terminal state.

##### Evidence

- `docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`
- `prd/2026-06-15-harnesslab-webui-prd.md`
- `README.md`
- `docs/README.md`
- `docs/archive/2026-06-15-pre-harbor-webui-redesign/README.md`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| architecture-adversary | Accepted Round 1 blockers are closed. | n/a | n/a | accept | Closure reviewer returned `OKAY` with no remaining blocking findings. | Marked report passed. | n/a |
| architecture-adversary | Real Harbor cancel/orphan-cleanup proof should be explicitly named. | Lifecycle spike plus fake-cancel coverage may be read too broadly. | non-blocking | accept | Named test makes Phase 3 proof concrete. | Added `tests/python/test_real_harbor_cancel_recovery.py` to Docker-marked Phase 3 test command. | Phase 3 implementation. |
| architecture-adversary | Custom-command quoting and mount-denial tests should be explicit. | Preview/warning requirements do not by themselves prove compiler safety. | non-blocking | accept | Named tests make the trust-boundary checks executable. | Added quoting normalization and unsupported `include_paths` mount denial tests to Phase 2. | Phase 2 implementation. |
| architecture-adversary | Add explicit `experiment.interrupted` event. | `interrupted` is a first-class terminal state. | non-blocking | accept | Event family should match terminal states. | Added `experiment.interrupted` to required event families. | Phase 3 implementation. |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 1
- Blocking re-review launch records:
  - `019ec7a3-a593-7491-a470-8791957c0c3a`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

Passed. Round 1 accepted blocking findings were fixed in the plan, PRD, README,
and docs entrypoints. Round 2 fresh closure review found no remaining blocking
findings for the planning-stage artifact.
