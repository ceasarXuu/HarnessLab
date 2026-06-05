# Subagent VS Review: Benchmark Adapter Phase 2 Live Material Drift

- Created: 2026-06-05T10:32:25+0800
- Updated: 2026-06-05T12:24:49+0800
- Report schema: adversarial-v1
- Task: Complete the next Benchmark Adapter Layer Phase 2 slice by making SWE-bench Pro replay fail closed when live external runtime materials drift.
- Report path: `vs_review/2026-06-05-benchmark-adapter-phase-2-live-material-drift-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: open

## Round 1: Live Material Drift Implementation

### Review Input

#### Objective

Ensure SWE-bench Pro replay cannot silently reuse stale external runtime assumptions when live external materials recorded in `external-runtime.private.json` are missing or have drifted.

#### Review Target

Code implementation, test coverage, and documentation updates for the live material drift slice of Benchmark Adapter Phase 2.

#### Target Locations

- `crates/harnesslab-cli/src/runner/replay.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro/runtime_snapshot.rs`
- `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-05-benchmark-adapter-phase-2-inventory.md`
- `scripts/test-after-change.sh --select SWEPRO-005`
- `scripts/test-after-change.sh --select REPLAY-008`
- `scripts/test-after-change.sh --select INT-011`
- `scripts/test-after-change.sh --select INT-013`
- `scripts/test-after-change.sh --select META-002`
- `scripts/test-after-change.sh --select META-008`
- `cargo check --all-targets`
- `cargo test -p harnesslab-cli --all-features --lib`

#### Change Introduction

Replay validation now reads private `replay_materials` after public/private external runtime snapshot fingerprint validation. Materials whose `public_path` is absent or null are treated as live external dependencies, and replay blocks before creating a new run if those files are missing or their current checksum differs from the stored checksum. SWE-bench Pro records parquet, evaluator, and run-script materials as live external dependencies.

#### Risk Focus

- The live-material predicate might classify the wrong artifacts and either miss drift or create false blockers.
- Replay might still create a new run before the blocker fires.
- The checksum comparison might not match snapshot writer semantics.
- The test might only prove private snapshot mutation, not actual live dependency drift.
- Documentation might overclaim Phase 2 closure beyond SWE-bench Pro live materials.

#### Assumptions To Attack

- Private replay materials without `public_path` are the right boundary for live external dependencies.
- All live materials have usable absolute paths and stored checksums.
- Existing public/private fingerprint checks still catch snapshot tampering before live checks.
- Attempt-local archived artifacts should not be revalidated as live external dependencies in this slice.
- The new test restores mutated fixture files reliably and does not leave shared state dirty.
- Open items for Terminal-Bench, official runner identity drift, and legacy degraded replay remain explicit.

#### Adversarial Lenses

- implementation
- data
- failure
- testing
- maintenance
- observability

#### Verification Status

- Passed `scripts/test-after-change.sh --select SWEPRO-005`.
- Passed `scripts/test-after-change.sh --select REPLAY-008`.
- Passed `scripts/test-after-change.sh --select INT-011`.
- Passed `scripts/test-after-change.sh --select INT-013`.
- Passed `scripts/test-after-change.sh --select META-002`.
- Passed `scripts/test-after-change.sh --select META-008`.
- Passed `cargo check -p harnesslab-cli --tests`.
- Passed `cargo check --all-targets`.
- Passed `cargo test -p harnesslab-cli --all-features --lib`.
- Passed `cargo fmt --check`.
- Passed `git diff --check`.
- Line counts: `replay.rs` 328, `swe_runtime_snapshot_contract.rs` 363.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Try to falsify the implementation and validation claims; do not confirm the main-agent narrative.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| complex | 12 minutes | 6 minutes if the reviewer is alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | The slice changes replay fail-closed behavior and checksum/state validation. | correctness, data drift, false positives, replay sequencing |
| test-validity-adversary | The slice depends on a focused contract test and several meta selectors to prove a narrow Phase 2 claim. | test strength, self-deception, missing failure-path assertions |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | multi_agent_v1.spawn_agent code-reviewer | 019e95a0-e5b8-76a3-91fc-372874abe236 | spawn_agent result, nickname Fermat | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |
| test-validity-adversary | multi_agent_v1.spawn_agent test-engineer | 019e95a1-2aa7-7a63-b92f-07fe94f4ca01 | spawn_agent result, nickname Ampere | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| implementation-adversary-output | implementation-adversary | 1 | 019e95a0-e5b8-76a3-91fc-372874abe236 | 7 minutes | completed | returned blocking coordinated snapshot rewrite finding | accepted blocking fix required |
| test-validity-adversary-output | test-validity-adversary | 1 | 019e95a1-2aa7-7a63-b92f-07fe94f4ca01 | 8 minutes | completed | returned blocking parquet coverage finding | accepted blocking fix required |

### Reviewer Outputs

#### implementation-adversary-output

##### Summary

REQUEST CHANGES. The pre-run sequencing is correct, but the live-material guard still trusted a self-rewritten private/public snapshot pair.

##### Blocking Findings

- [HIGH] Coordinated edits of both snapshot files bypass the new live-material drift gate.
  - Broken assumption: public/private fingerprint validation catches snapshot tampering before live checks.
  - Failure scenario: after `parquet`, `evaluator`, or `run_script` drifts, a user/tool rewrites `external-runtime.private.json` with the new checksum, recomputes `runtime_fingerprint`/`public_fingerprint`, and rewrites `external-runtime.public.json` to match.
  - Trigger condition: any coordinated mutation of both snapshot files in the source attempt.
  - Impact: replay accepts drifted live external inputs as if they were authoritative.
  - Proof needed: add a contract test that mutates a live material and then updates both snapshot files/fingerprints in lockstep; replay should fail after the fix.

##### Non-blocking Risks

- [MEDIUM] `public_path == null` is an implicit proxy for live external dependency, not an explicit schema contract.
  - Broken assumption: materials without `public_path` are always exactly the live SWE dependencies that must be revalidated.
  - Failure scenario: future adapters may skip or over-apply live checks based on path layout rather than semantic scope.
  - Trigger condition: future material additions/refactors.
  - Impact: future false negatives or false positives.
  - Proof needed: add classification tests that assert exactly which materials are live-validated.

##### Required Fixes

- Add a parent attempt snapshot anchor so replay rejects coordinated rewrites of `external-runtime.private.json` and `external-runtime.public.json`.
- Reduce or correct the doc claim until the parent anchor exists.
- Make live-vs-archived material classification explicit instead of inferring it from `public_path`.

##### Missing Tests

- Coordinated private/public snapshot rewrite blocker.
- Live material classification test.

##### Missing Logs / Observability

- Replay validation emits stderr blockers only; no structured preflight record is written before aborting.

##### Evidence

- `crates/harnesslab-cli/src/runner/replay.rs:195` - self-consistent fingerprint validation.
- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs:323` - runtime fingerprint generation.
- `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs:89` - previous tests covered one-sided tampering.

#### test-validity-adversary-output

##### Summary

One blocking validity gap remains. Implementation validates before replay creates a new run directory, and selectors are wired correctly, but `SWEPRO-005` did not black-box prove the parquet drift branch that docs claimed.

##### Blocking Findings

- `SWEPRO-005` over-claims live external drift coverage for parquet.
  - Broken assumption: updated `SWEPRO-005` proves live external dependency drift for all SWE live materials.
  - Failure scenario: a regression stops treating `parquet` as a live dependency while evaluator and run-script checks still work.
  - Trigger condition: `swe_runtime_materials` declares `parquet` as live, but replay no longer blocks parquet drift.
  - Impact: stale dataset content could be silently reused in replay while selector/meta/docs still pass.
  - Proof needed: mutate the recorded parquet file after the initial run, replay, and assert the blocker is specifically the live-material drift path for `material=parquet`.

##### Non-blocking Risks

- The before-new-run proof is partly implementation-backed rather than fully black-box-backed.
  - Broken assumption: tests alone prove no replay run directory is ever created.
  - Failure scenario: replay briefly creates and cleans up a run directory before failing.
  - Trigger condition: future reorder moves run directory creation earlier and adds cleanup.
  - Impact: behavior contract weakens without test failure.
  - Proof needed: assert no new run-id path remains after blockers; transient creation still requires implementation review.
- `SWEPRO-005` does not pin live-material classification directly.
  - Broken assumption: the right SWE materials are live because drift tests pass.
  - Failure scenario: future refactor gives `parquet` or `evaluator` a non-null public path and removes it from live validation.
  - Trigger condition: material snapshot path filling changes.
  - Impact: replay could stop treating a material as live without targeted failure.
  - Proof needed: assert `parquet`, `evaluator`, and `run_script` are recorded with live scope, while attempt-local artifacts are archived.

##### Required Fixes

- Extend `SWEPRO-005` to mutate the parquet file and assert `material=parquet`.
- Keep selector and registry entries unchanged; this strengthens the current proof family.
- Trim docs or land the parquet assertion before keeping the doc claim.

##### Missing Tests

- Parquet-drift replay blocker case.
- Direct live/archived material classification assertion.
- Stronger no-new-run-id assertion for replay blockers.

##### Missing Logs / Observability

- Broad substring assertions should pin material diagnostics more tightly.
- No structured pre-run replay-block artifact exists; stderr is the current contract.

##### Evidence

- `crates/harnesslab-cli/src/runner/external/swe_bench_pro/runtime_snapshot.rs:149` - live material declarations.
- `crates/harnesslab-cli/src/runner/replay.rs:210` - live material validation.
- `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs:133` - previous evaluator/run-script coverage only.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | Coordinated private/public snapshot rewrites bypass drift gate | Self-consistent attempt snapshots were treated as sufficient authority | blocking | accept | A coordinated rewrite can recompute both fingerprints and checksums inside the attempt files | Added `ExternalRuntimeAttemptSnapshot`, wrote attempt anchors into `task-runtime.snapshot.json`, and replay now validates public/private file checksums, paths, and fingerprints against the parent anchor | Fresh closure review required |
| implementation-adversary | `public_path == null` is implicit live-dependency classification | Future adapters could misclassify live vs archived materials | non-blocking | accept | Reviewer identified a real maintainability weakness | Added explicit `validation_scope` with `live_external` and `archived_attempt`; replay now fails closed when scope is missing or unknown | Covered by closure review and `SWEPRO-005` |
| test-validity-adversary | `SWEPRO-005` over-claims parquet drift coverage | Dataset parquet drift could regress while evaluator/run-script tests pass | blocking | accept | Docs claimed parquet drift but the test did not mutate parquet | Extended `SWEPRO-005` to mutate the recorded parquet file and assert `material=parquet` live drift blocker | Fresh closure review required |
| test-validity-adversary | No direct live/archived classification assertion | Scope contract could change silently | non-blocking | accept | Classification is now a first-class schema field | Added assertions for live `parquet`, `evaluator`, `run_script` and archived `raw_sample`, `prediction_eval_json` | Covered by `SWEPRO-005` |
| test-validity-adversary | No-new-run proof was count-based | Future cleanup could hide transient run creation | non-blocking | accept | Count-only check is weaker than run-id set comparison | Replaced count-only replay blocker helper with sorted run-id set equality | Transient create-and-clean remains implementation-reviewed |

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
- Blocked reason: accepted blocking findings require fresh closure review
- Allowed to proceed: no

## Round 4: Concurrent Projection Closure

### Review Input

#### Objective

Verify the Round 3 accepted blocker is closed by serializing external runtime
attempt anchor projection under a run-scoped lock, and by adding concurrency
coverage for same-task multi-attempt overlap and multi-task parallel
completion.

#### Review Target

Closure review for the concurrent anchor projection fix and its evidence.

#### Target Locations

- `crates/harnesslab-infra/src/file_lock.rs`
- `crates/harnesslab-infra/src/artifact.rs`
- `crates/harnesslab-infra/src/lib.rs`
- `crates/harnesslab-cli/src/runner/external.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_anchor.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/replay.rs`
- `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs`
- `crates/harnesslab-cli/tests/support/runtime_snapshot.rs`
- `tests/TEST_REGISTRY.toml`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-05-benchmark-adapter-phase-2-inventory.md`

#### Change Introduction

External runtime attempt anchor projection now lives in
`runtime_anchor.rs`. The projection acquires
`.harnesslab-locks/external-runtime-anchor.lock` at the run directory before
reading and rewriting both `task-runtime.snapshot.json` and
`benchmark.snapshot.json`. The writer emits
`external_runtime_anchor_projected` after the authority files are updated.
`harnesslab_infra::with_exclusive_file_lock` provides the shared lock helper.
Unit tests cover same-task multi-attempt concurrent projection, multi-task
parallel completion, projection events, and serialized file mutation.

#### Risk Focus

- The lock may not cover the whole authority read/modify/write critical
  section.
- Parallel attempts might still lose benchmark or task-runtime anchors.
- The lock file may create unintended artifact or replay side effects.
- Event emission may fail after authority files were updated or introduce a
  deadlock.
- Checksum/fingerprint semantics may have changed during the module split.
- Tests may be too synthetic to prove the production lost-update class.
- Docs may overclaim beyond source-run artifact authority.

#### Assumptions To Attack

- A stable run-scoped lock path is sufficient for all concurrent projection
  writers in a run.
- Same-task and multi-task concurrency tests cover the accepted lost-update
  failure modes.
- `atomic_write_json` remains acceptable inside the projection lock.
- Event logging after authority update is diagnostically useful and does not
  create private path leaks.
- `SWEPRO-005.required_artifacts` now reflects the authority surface.

#### Adversarial Lenses

- implementation
- concurrency
- testing
- observability
- documentation

#### Verification Status

- Passed `cargo test -p harnesslab-infra --lib lock_001_serializes_file_mutation`.
- Passed `cargo test -p harnesslab-cli --lib runtime_anchor::tests`.
- Passed `cargo check -p harnesslab-cli --tests`.
- Passed `scripts/test-after-change.sh --select SWEPRO-005`.
- Passed `scripts/test-after-change.sh --select REPLAY-007`.
- Passed `scripts/test-after-change.sh --select REPLAY-008`.
- Passed `scripts/test-after-change.sh --select INT-011`.
- Passed `scripts/test-after-change.sh --select INT-013`.
- Passed `scripts/test-after-change.sh --select META-002`.
- Passed `scripts/test-after-change.sh --select META-008`.
- Passed `cargo check --all-targets`.
- Passed `cargo test -p harnesslab-cli --all-features --lib`.
- Passed `cargo test -p harnesslab-infra --all-features`.
- Passed `cargo test -p harnesslab-core --all-features`.
- Passed `cargo test -p harnesslab-adapters --all-features`.
- Passed `cargo fmt --check`.
- Passed `git diff --check`.
- Touched code files remain below 500 lines.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Focus on closure of the Round 3 concurrent projection blocker and whether
  any new blocking regression exists.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 15 minutes | 8 minutes if reviewer is alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Round 3 blocker was an implementation concurrency bug. | lock scope, authority writes, side effects |
| test-validity-adversary | Closure depends on new concurrency tests and validation evidence. | test strength, selector adequacy, registry/docs traceability |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | multi_agent_v1.spawn_agent code-reviewer | 019e95f7-5917-7a60-b08b-5eb1b5a2e5a1 | spawn_agent result, nickname Hegel | fork_context=false | Round 4 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |
| test-validity-adversary | multi_agent_v1.spawn_agent test-engineer | 019e95f7-c3c2-7a32-a067-f46dfa68a8f1 | spawn_agent result, nickname Plato | fork_context=false | Round 4 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| pending | implementation-adversary | 1 | 019e95f7-5917-7a60-b08b-5eb1b5a2e5a1 | pending | pending | waiting for fresh Round 4 reviewer result | pending |
| pending | test-validity-adversary | 1 | 019e95f7-c3c2-7a32-a067-f46dfa68a8f1 | pending | pending | waiting for fresh Round 4 reviewer result | pending |

### Reviewer Outputs

Pending fresh Round 4 reviewer outputs.

### Main Agent Response

Pending Round 4 reviewer outputs.

### Closure Status

- Blocking findings found: pending
- Accepted blocking findings fixed: pending
- Blocking re-review completed: pending
- Blocking re-review passed: pending
- Blocking re-review round links:
  - pending
- Blocking re-review launch records:
  - listed above
- Rejected findings backed by evidence: pending
- Deferred findings documented: pending
- Blocked reason: pending
- Allowed to proceed: pending

## Final Conclusion

Pending fresh reviewer outputs.

## Round 2: Blocking Fix Closure

### Review Input

#### Objective

Verify that accepted Round 1 blocking findings are actually closed after adding task-runtime attempt anchors, explicit material validation scopes, parquet drift coverage, and coordinated snapshot rewrite tests.

#### Review Target

Closure review of the implementation and tests after accepted blocking fixes.

#### Target Locations

- `crates/harnesslab-core/src/benchmark.rs`
- `crates/harnesslab-adapters/src/registry.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/replay.rs`
- `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs`
- `crates/harnesslab-cli/tests/support/runtime_snapshot.rs`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-05-benchmark-adapter-phase-2-inventory.md`

#### Change Introduction

The fix adds `ExternalRuntimeAttemptSnapshot` entries to `task-runtime.snapshot.json` after external runtime snapshots are written. Replay now compares task-runtime core fields to `benchmark.snapshot.json`, then validates attempt snapshot paths, public/private file checksums, and runtime/public fingerprints against the task-runtime anchor before checking live external materials. Runtime materials now carry explicit `validation_scope` values, and replay fails closed when the scope is missing or unknown. `SWEPRO-005` now mutates parquet, asserts `material=parquet`, rewrites public/private snapshots in coordination, and verifies the task-runtime anchor blocks the rewrite.

#### Risk Focus

- Coordinated snapshot rewrite may still bypass replay if the anchor is incomplete or not validated.
- Scope validation may still silently skip old/bad material records.
- The task-runtime anchor may not be written consistently or may break `REPLAY-007`.
- Parquet coverage may be too implementation-specific or may not hit the intended branch.
- Docs may still overclaim beyond SWE-bench Pro.

#### Assumptions To Attack

- `task-runtime.snapshot.json` is a sufficient parent anchor for this Phase 2 slice.
- Replay validates anchor paths/checksums/fingerprints before live material checks.
- `validation_scope=live_external` is the only live-material path and unknown scope fails closed.
- `SWEPRO-005` proves parquet drift and coordinated rewrite blockers before new run creation.
- Existing selectors and registry entries still cover the strengthened contract.

#### Adversarial Lenses

- implementation
- data
- failure
- testing
- maintenance

#### Verification Status

- Passed `scripts/test-after-change.sh --select SWEPRO-005`.
- Passed `scripts/test-after-change.sh --select REPLAY-007`.
- Passed `scripts/test-after-change.sh --select REPLAY-008`.
- Passed `scripts/test-after-change.sh --select INT-011`.
- Passed `scripts/test-after-change.sh --select INT-013`.
- Passed `scripts/test-after-change.sh --select META-002`.
- Passed `scripts/test-after-change.sh --select META-008`.
- Passed `cargo check -p harnesslab-cli --tests`.
- Passed `cargo check -p harnesslab-core --tests`.
- Passed `cargo check --all-targets`.
- Passed `cargo test -p harnesslab-core --all-features`.
- Passed `cargo test -p harnesslab-adapters --all-features`.
- Passed `cargo test -p harnesslab-cli --all-features --lib`.
- Passed `cargo fmt --check`.
- Passed `git diff --check`.
- Line counts remain below 500 for touched code files.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Confirm only whether the accepted blocking findings are closed; still report any new blocking regression found in the closure target.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 15 minutes | 8 minutes if the reviewer is alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Round 1 implementation blocker accepted and fixed through parent anchoring. | coordinated rewrite bypass, replay sequencing, fail-closed behavior |
| test-validity-adversary | Round 1 test blocker accepted and fixed through parquet/scope/coordinated rewrite assertions. | proof strength, selector adequacy, false confidence |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | multi_agent_v1.spawn_agent code-reviewer | 019e95bc-026b-7813-a0fd-2d0d3cf8218e | spawn_agent result, nickname Epicurus | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |
| test-validity-adversary | multi_agent_v1.spawn_agent test-engineer | 019e95bc-4c43-7a13-8327-5b4104bc29b5 | spawn_agent result, nickname Mill | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| implementation-adversary-output | implementation-adversary | 1 | 019e95bc-026b-7813-a0fd-2d0d3cf8218e | 15 minutes | completed | returned blocking higher-level anchor finding | accepted blocking fix required |
| superseded-test-validity-adversary | test-validity-adversary | 1 | 019e95bc-4c43-7a13-8327-5b4104bc29b5 | 15 minutes | superseded | implementation target changed after accepted closure blocker | closed without output; Round 3 required |

### Reviewer Outputs

#### implementation-adversary-output

##### Summary

REQUEST CHANGES. Parquet coverage and explicit `validation_scope` closure landed, but `task-runtime.snapshot.json` alone is a mutable parent anchor. Replay ignored `external_runtime_attempts` while comparing task runtime authority to `benchmark.snapshot.json`, so rewriting child snapshots plus task-runtime anchor could still pass.

##### Blocking Findings

- [HIGH] `task-runtime.snapshot.json` is not a sufficient parent anchor for the Phase 2 slice.
  - Broken assumption: `task-runtime.snapshot.json` provides sufficient parent authority for attempt snapshot anchors.
  - Failure scenario: after live parquet drift, a coordinated rewrite updates `external-runtime.private.json`, `external-runtime.public.json`, and `task-runtime.snapshot.json.external_runtime_attempts[*]`; replay succeeds.
  - Trigger condition: any coordinated rewrite that includes the parent task-runtime artifact.
  - Impact: Round 1 coordinated-rewrite blocker is only partially closed.
  - Proof needed: add a black-box contract test that mutates parquet, rewrites child snapshots and task-runtime anchor, and asserts replay still blocks.

##### Non-blocking Risks

- `validation_scope` fail-closed behavior was in code but not black-box pinned.
  - Broken assumption: unknown or missing scopes will remain fail-closed as the schema evolves.
  - Failure scenario: a future adapter omits/renames `validation_scope`.
  - Trigger condition: schema edits or future adapter generalization.
  - Impact: regression could pass until runtime.
  - Proof needed: mutate `validation_scope` to missing and unknown in replay contract tests.
- Docs overstated what the task-runtime anchor protected.
  - Broken assumption: anchored attempt entries broadly reject coordinated rewrites.
  - Failure scenario: docs imply full coordinated-rewrite closure while only child-snapshot-only rewrites were blocked.
  - Trigger condition: using docs as closure evidence.
  - Impact: Phase 2 status overclaimed.
  - Proof needed: either narrow docs or land higher-level anchor.

##### Required Fixes

- Anchor `external_runtime_attempts` from immutable replay authority, practically `benchmark.snapshot.json`.
- Add replay contract for child-plus-task-runtime coordinated rewrite.
- Add missing/unknown `validation_scope` replay tests.
- Correct docs to benchmark authority.

##### Missing Tests

- Coordinated rewrite including `task-runtime.snapshot.json`.
- Missing `validation_scope` replay blocker.
- Unknown `validation_scope` replay blocker.

##### Missing Logs / Observability

- Replay preflight still reports only through stderr; no structured event records authority layer pass/fail.

##### Evidence

- `crates/harnesslab-cli/src/runner/replay.rs` previously dropped `external_runtime_attempts` before comparing task runtime to benchmark authority.
- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs` previously appended attempt anchors only into task-runtime artifact.

#### superseded-test-validity-adversary

No output recorded. This reviewer was closed because the implementation target changed after the accepted implementation blocker. This is not counted as a passing review.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | `task-runtime.snapshot.json` alone is not sufficient parent anchor | Child snapshots plus task-runtime anchor could be rewritten together | blocking | accept | Reviewer provided a concrete bypass and black-box repro | External runtime writer now projects `external_runtime_attempts` into `benchmark.snapshot.json` and `task-runtime.snapshot.json`; replay requires task-runtime snapshot to equal benchmark authority exactly | Round 3 fresh closure required |
| implementation-adversary | Scope fail-closed not black-box pinned | Missing/unknown `validation_scope` could regress | non-blocking | accept | The previous test asserted valid scopes but did not mutate invalid scopes | `SWEPRO-005` now rewrites public/private snapshots, task-runtime, and benchmark authority with missing/unknown scope and asserts replay blocker | Round 3 fresh closure required |
| implementation-adversary | Docs overstated task-runtime anchor protection | Docs implied full coordinated rewrite closure | non-blocking | accept | Docs named task-runtime as anchor root | Plan docs now describe benchmark snapshot as replay authority and task-runtime as matching mirror | Round 3 fresh closure required |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: pending
- Blocking re-review passed: pending
- Blocking re-review round links:
  - Round 3 pending
- Blocking re-review launch records:
  - pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: accepted Round 2 blocking finding requires fresh Round 3 closure review
- Allowed to proceed: no

## Round 3: Benchmark Authority Closure

### Review Input

#### Objective

Verify the Round 2 accepted blocker is closed by projecting external runtime attempt anchors into `benchmark.snapshot.json` and requiring `task-runtime.snapshot.json` to match benchmark authority exactly.

#### Review Target

Final closure review after benchmark-authority anchoring and expanded replay tests.

#### Target Locations

- `crates/harnesslab-core/src/benchmark.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/replay.rs`
- `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs`
- `crates/harnesslab-cli/tests/support/runtime_snapshot.rs`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-05-benchmark-adapter-phase-2-inventory.md`

#### Change Introduction

External runtime snapshot writing now updates both `task-runtime.snapshot.json` and the corresponding `BenchmarkPlan.task_runtime_snapshots` entry inside `benchmark.snapshot.json` with identical `external_runtime_attempts`. Replay reads `benchmark.snapshot.json` as the root authority and requires the per-task `task-runtime.snapshot.json` artifact to equal that snapshot exactly, including `external_runtime_attempts`. The test now proves child-only coordinated rewrites fail on attempt anchor mismatch, child-plus-task-runtime rewrites fail on task-runtime mismatch against benchmark authority, and missing/unknown `validation_scope` fails closed even when public/private snapshots, task-runtime, and benchmark authority are recomputed consistently.

#### Risk Focus

- A full coordinated rewrite may still bypass replay if benchmark authority can be rewritten without detection.
- The Phase 2 goal may now be limited to source-run artifact authority rather than cryptographic immutability.
- Replay may break existing task runtime snapshot contract tests because benchmark snapshot mutates after attempt completion.
- Tests may not actually rewrite enough authority layers to prove the intended branch.
- Docs must accurately state what is closed and what remains open.

#### Assumptions To Attack

- `benchmark.snapshot.json` is the intended root replay authority for this phase.
- `task-runtime.snapshot.json` exact equality to benchmark authority closes task-runtime-only anchor rewrites.
- `SWEPRO-005` now hits parquet drift, child-only rewrite, child-plus-task rewrite, missing scope, and unknown scope paths.
- Current validation remains below 500-line code file limit.
- Remaining open items are not hidden as complete.

#### Adversarial Lenses

- implementation
- data
- testing
- maintenance
- documentation

#### Verification Status

- Passed `scripts/test-after-change.sh --select SWEPRO-005`.
- Passed `scripts/test-after-change.sh --select REPLAY-007`.
- Passed `scripts/test-after-change.sh --select REPLAY-008`.
- Passed `scripts/test-after-change.sh --select INT-011`.
- Passed `scripts/test-after-change.sh --select META-008`.
- Passed `cargo check -p harnesslab-cli --tests`.
- Passed `cargo check --all-targets`.
- Passed `cargo test -p harnesslab-cli --all-features --lib`.
- Passed `cargo fmt --check`.
- Passed `git diff --check`.
- Line counts: main SWE runtime snapshot test 490, support runtime snapshot helper 232, runtime snapshot writer 486, replay 411.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Focus on closure of Round 2 blocker and whether any new blocking regression exists.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 15 minutes | 8 minutes if reviewer is alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Round 2 blocker was implementation authority boundary. | benchmark authority, replay equality, coordinated rewrite bypass |
| test-validity-adversary | Final proof depends on strengthened `SWEPRO-005`. | black-box branch coverage, helper correctness, selector adequacy |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | multi_agent_v1.spawn_agent code-reviewer | 019e95d0-6d4b-7571-b2eb-0db53ad38295 | spawn_agent result, nickname Feynman | fork_context=false | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |
| test-validity-adversary | multi_agent_v1.spawn_agent test-engineer | 019e95d0-bee6-7ee1-98c7-275e7d4e54fc | spawn_agent result, nickname Euclid | fork_context=false | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| implementation-adversary-output | implementation-adversary | 1 | 019e95d0-6d4b-7571-b2eb-0db53ad38295 | 13 minutes | completed | returned blocking concurrent projection lost-update finding | accepted blocking fix required |
| test-validity-adversary-output | test-validity-adversary | 1 | 019e95d0-bee6-7ee1-98c7-275e7d4e54fc | 11 minutes | completed | returned no blocking findings and two non-blocking registry/meta-scope risks | non-blocking follow-up accepted |

### Reviewer Outputs

#### implementation-adversary-output

##### Summary

REQUEST CHANGES. Benchmark authority closes the prior coordinated rewrite path,
but the projection writer can lose valid anchors when multiple external
attempts finish concurrently.

##### Blocking Findings

- [HIGH] Lost-update race in anchor projection can corrupt replay authority for
  valid concurrent runs.
  - Broken assumption: `atomic_write_json` is enough to make authority updates
    safe under concurrent external attempt completion.
  - Failure scenario: attempt A and attempt B both read the same
    `task-runtime.snapshot.json` or `benchmark.snapshot.json`, each appends one
    anchor, and the later whole-file write overwrites the earlier anchor.
  - Trigger condition: same-task multi-attempt overlap or multi-task parallel
    completion when both need to update the shared benchmark authority file.
  - Impact: a valid source run can lose an external runtime attempt anchor,
    causing replay to reject a real completed attempt or trust incomplete
    authority state.
  - Proof needed: serialize authority projection updates per run and add
    concurrency coverage for same-task multi-attempt overlap plus multi-task
    parallel completion.

##### Non-blocking Risks

- [MEDIUM] Full root-level benchmark rewrite remains a trust-boundary limit.
  - This is acceptable for the current Phase 2 local artifact authority model,
    as long as docs do not claim cryptographic immutability or tamper-proof
    root authority.

##### Required Fixes

- Add a run-scoped lock or single-writer coordinator around the read/modify/write
  of both `task-runtime.snapshot.json` and `benchmark.snapshot.json`.
- Add concurrency tests for same-task multi-attempt overlap and multi-task
  parallel completion.
- Rerun `SWEPRO-005`, `REPLAY-007`, `REPLAY-008`, and `INT-011`.
- Keep docs scoped to source-run artifact authority unless a higher-level
  immutable manifest or signature is added.

##### Missing Tests

- Same-task concurrent attempt anchor projection.
- Multi-task concurrent benchmark authority projection.

##### Missing Logs / Observability

- No event records when external runtime anchors are projected into replay
  authority.

##### Evidence

- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs` used
  whole-file read/modify/write for both task-runtime and benchmark authority.
- `harnesslab_infra::atomic_write_json` only makes individual file replacement
  atomic; it does not serialize concurrent read/modify/write cycles.

#### test-validity-adversary-output

##### Summary

PASS for Round 2 closure. `SWEPRO-005` now exercises parquet drift,
child-snapshot rewrite, child-plus-task-runtime rewrite, missing scope, and
unknown scope branches. No blocking test-validity findings remain for the
benchmark-authority closure target.

##### Blocking Findings

None.

##### Non-blocking Risks

- [LOW] `SWEPRO-005` registry `required_artifacts` under-described the authority
  surface by listing only `external-runtime.private.json` and
  `external-runtime.public.json`.
  - Suggested follow-up: include `benchmark.snapshot.json` and
    `tasks/<task-id>/task-runtime.snapshot.json`, or document the split.
- [LOW] `META-008` covers adapter-proof selectors, not every `REPLAY-*` or
  `INT-*` selector.
  - Suggested follow-up: do not cite `META-008` as broad replay/integration
    selector coverage.

##### Required Fixes

None blocking.

##### Missing Tests

None blocking.

##### Missing Logs / Observability

No blocking observability gaps identified in the test-validity scope.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | Lost-update race in anchor projection | Concurrent attempts can overwrite each other's benchmark or task-runtime authority anchors | blocking | accept | The previous writer performed whole-file read/modify/write without serializing the critical section | Added `harnesslab_infra::with_exclusive_file_lock`, moved projection to `runtime_anchor`, locked `.harnesslab-locks/external-runtime-anchor.lock`, and added same-task plus multi-task concurrency tests | Round 4 fresh closure required |
| implementation-adversary | Projection lacked an explicit event | Operators could not see when authority anchoring happened | non-blocking | accept | Logging is useful for this new authority mutation | Added `external_runtime_anchor_projected` after successful projection and documented the event | Covered by validation and Round 4 |
| test-validity-adversary | `SWEPRO-005.required_artifacts` under-described authority surface | Registry made the proof look narrower than the actual authority chain | non-blocking | accept | The authority chain now includes benchmark and task-runtime snapshots | Added `benchmark.snapshot.json` and `tasks/<task-id>/task-runtime.snapshot.json` to `SWEPRO-005.required_artifacts` | Rerun `META-002` and `META-008` |
| test-validity-adversary | `META-008` can be over-cited | Adapter-proof selector meta-test is not a broad replay/integration proof | non-blocking | accept | The review correctly scoped the selector | Docs and final evidence will cite `META-008` only for adapter selector registry health, not for replay coverage | n/a |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: pending
- Blocking re-review passed: pending
- Blocking re-review round links:
  - Round 4 pending
- Blocking re-review launch records:
  - pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: accepted Round 3 blocking finding requires fresh Round 4 closure review
- Allowed to proceed: no

## Round 4: Concurrent Projection Closure

### Review Input

#### Objective

Verify the Round 3 accepted blocker is closed by serializing external runtime
attempt anchor projection under a run-scoped lock, and by adding concurrency
coverage for same-task multi-attempt overlap and multi-task parallel
completion.

#### Review Target

Closure review for the concurrent anchor projection fix and its evidence.

#### Target Locations

- `crates/harnesslab-infra/src/file_lock.rs`
- `crates/harnesslab-infra/src/artifact.rs`
- `crates/harnesslab-infra/src/lib.rs`
- `crates/harnesslab-cli/src/runner/external.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_anchor.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/replay.rs`
- `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs`
- `crates/harnesslab-cli/tests/support/runtime_snapshot.rs`
- `tests/TEST_REGISTRY.toml`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-05-benchmark-adapter-phase-2-inventory.md`

#### Change Introduction

External runtime attempt anchor projection now lives in `runtime_anchor.rs`.
The projection acquires `.harnesslab-locks/external-runtime-anchor.lock` at the
run directory before reading and rewriting both `task-runtime.snapshot.json` and
`benchmark.snapshot.json`. The writer emits `external_runtime_anchor_projected`
after the authority files are updated. `harnesslab_infra::with_exclusive_file_lock`
provides the shared lock helper. Unit tests cover same-task multi-attempt
concurrent projection, multi-task parallel completion, projection events, and
serialized file mutation.

#### Risk Focus

- The lock may not cover the whole authority read/modify/write critical section.
- Parallel attempts might still lose benchmark or task-runtime anchors.
- The lock file may create unintended artifact or replay side effects.
- Event emission may fail after authority files were updated or introduce a deadlock.
- Checksum/fingerprint semantics may have changed during the module split.
- Tests may be too synthetic to prove the production lost-update class.
- Docs may overclaim beyond source-run artifact authority.

#### Verification Status

- Passed `cargo test -p harnesslab-infra --lib lock_001_serializes_file_mutation`.
- Passed `cargo test -p harnesslab-cli --lib runtime_anchor::tests`.
- Passed `cargo check -p harnesslab-cli --tests`.
- Passed `scripts/test-after-change.sh --select SWEPRO-005`.
- Passed `scripts/test-after-change.sh --select REPLAY-007`.
- Passed `scripts/test-after-change.sh --select REPLAY-008`.
- Passed `scripts/test-after-change.sh --select INT-011`.
- Passed `scripts/test-after-change.sh --select INT-013`.
- Passed `scripts/test-after-change.sh --select META-002`.
- Passed `scripts/test-after-change.sh --select META-008`.
- Passed `cargo check --all-targets`.
- Passed `cargo test -p harnesslab-cli --all-features --lib`.
- Passed `cargo test -p harnesslab-infra --all-features`.
- Passed `cargo test -p harnesslab-core --all-features`.
- Passed `cargo test -p harnesslab-adapters --all-features`.
- Passed `cargo fmt --check`.
- Passed `git diff --check`.

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Round 3 blocker was an implementation concurrency bug. | lock scope, authority writes, side effects |
| test-validity-adversary | Closure depends on new concurrency tests and validation evidence. | test strength, selector adequacy, registry/docs traceability |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | multi_agent_v1.spawn_agent code-reviewer | 019e95f7-5917-7a60-b08b-5eb1b5a2e5a1 | spawn_agent result, nickname Hegel | fork_context=false | Round 4 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |
| test-validity-adversary | multi_agent_v1.spawn_agent test-engineer | 019e95f7-c3c2-7a32-a067-f46dfa68a8f1 | spawn_agent result, nickname Plato | fork_context=false | Round 4 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| implementation-adversary-output | implementation-adversary | 1 | 019e95f7-5917-7a60-b08b-5eb1b5a2e5a1 | 15 minutes | completed | returned no blocking findings and two non-blocking risks | non-blocking follow-up accepted |
| test-validity-adversary-output | test-validity-adversary | 1 | 019e95f7-c3c2-7a32-a067-f46dfa68a8f1 | 15 minutes | completed | returned blocking registry-backed proof gap | accepted blocking fix required |

### Reviewer Outputs

#### implementation-adversary-output

##### Summary

No blocking findings. The accepted lost-update blocker is closed on the reviewed
Unix path: projection takes a single run-scoped lock before reading or mutating
either authority file, then updates `task-runtime.snapshot.json` and
`benchmark.snapshot.json` inside that critical section, and replay verifies the
anchored attempt metadata before allowing replay.

##### Blocking Findings

None.

##### Non-blocking Risks

- [MEDIUM] Partial-commit failure mode still exists: the code writes
  `task-runtime.snapshot.json`, then `benchmark.snapshot.json`, then appends
  `external_runtime_anchor_projected`. Replay will fail closed later if the
  second write or event append fails, so this is availability-only and not the
  original lost-update bug.
- [LOW] The previous docs overstated lock guarantees for non-Unix because the
  non-Unix lock implementation was a no-op.

##### Required Fixes

None for Round 4 closure of the accepted lost-update blocker.

##### Missing Tests

- Optional fault-injection test for benchmark-write or event-append failure.
- Optional non-Unix platform contract if non-Unix execution becomes supported.

##### Missing Logs / Observability

No blocking gap. The new `external_runtime_anchor_projected` event is emitted
after authority writes.

##### Evidence

- Lock acquisition and critical section in `crates/harnesslab-cli/src/runner/external/runtime_anchor.rs`.
- Replay anchor validation in `crates/harnesslab-cli/src/runner/replay.rs`.
- Artifact collection does not copy `.harnesslab-locks` from run root into task artifacts.
- Fresh verification included `cargo check -p harnesslab-cli --tests`, `cargo test -p harnesslab-infra --lib lock_001_serializes_file_mutation`, `cargo test -p harnesslab-cli --lib runtime_anchor::tests`, and exact `SWEPRO-005`.

#### test-validity-adversary-output

##### Summary

The code-path race appears covered, but one blocking closure gap remained: the
proof was only in ad hoc cargo commands, while the plan requires every proof
claim to have a requirement row, registry row, and selector route.

##### Blocking Findings

- [HIGH] Phase-closure proof for the accepted Round 3 race fix was not
  registry-backed.
  - Broken assumption: ad hoc validation commands were enough to close a Phase 2
    proof claim.
  - Failure scenario: the run-scoped lock or projection event regresses while
    active selector rows still pass because no selector routes the new tests.
  - Impact: phase closure violates the plan's traceability invariant.
  - Required proof: add an active requirement, test registry row, and selector
    route covering serialized projection and `external_runtime_anchor_projected`.

##### Non-blocking Risks

- The concurrency tests are synthetic but correctly target the read/modify/write
  race boundary.
- The projection event assertion was initially weak because it did not parse
  JSONL or pin task id and attempt-relative paths.
- `SWEPRO-005.required_artifacts` still lists bare external runtime filenames;
  precise attempt-relative paths can be tightened later.

##### Required Fixes

- Add an active requirement/test-registry/selector route for the race-fix proof.
- Make that active proof cover serialized same-task and multi-task projection
  behavior plus `external_runtime_anchor_projected` payload.
- Align Phase 2 docs with the active selector row.

##### Missing Tests

- Active selector-backed test row for the projection race proof.
- Active selector-backed assertion for projection event payload fields.

##### Missing Logs / Observability

No active selector initially treated `external_runtime_anchor_projected` as a
required contract.

##### Evidence

- New unit tests were present in `runtime_anchor.rs`, but no active selector row
  routed them before this fix.
- The closure invariant is documented in the architecture plan's cross-phase
  invariants.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | Partial commit between authority files or event append | A failure after the first authority write can strand a run in fail-closed inconsistent state | non-blocking | accept | Replay later rejects inconsistent authority, so the risk is availability-only, not silent incorrect replay | Documented as residual risk; no code change in this slice | Future transactional authority writer or fault-injection test |
| implementation-adversary | Non-Unix lock path was no-op | Docs described the lock as preventing lost updates without platform qualifier | non-blocking | accept | A no-op lock is unsafe for replay authority | Changed non-Unix `with_exclusive_file_lock` to fail closed instead of silently proceeding | Covered by compile/tests on Unix; future non-Unix support needs platform lock |
| test-validity-adversary | Race-fix proof was not registry-backed | Active selectors could pass while the lock/projection tests regressed | blocking | accept | The plan explicitly requires requirement row, registry row, and selector route for proof claims | Added `external_runtime_anchor_projection` requirement, active `REPLAY-009` registry row, selector route, stronger event payload assertions, and docs pointing to `REPLAY-009` | Round 5 fresh closure required |
| test-validity-adversary | Projection event assertion was weak | Test only searched strings and did not pin task id/paths | non-blocking | accept | Docs promise task id and attempt-relative public/private paths | Event test now parses JSONL `Event` values and asserts run id, task id, event name, attempt, and attempt-relative public/private paths | Covered by `REPLAY-009` |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: pending
- Blocking re-review passed: pending
- Blocking re-review round links:
  - Round 5 pending
- Blocking re-review launch records:
  - pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: accepted Round 4 blocking finding requires fresh Round 5 closure review
- Allowed to proceed: no

## Round 5: Registry-Backed Projection Proof Closure

### Review Input

#### Objective

Verify the Round 4 accepted blocker is closed. The blocker was that the
concurrent projection proof was not backed by an active requirement, registry
row, and selector route.

#### Review Target

Narrow test-validity closure review for the proof traceability fix.

#### Target Locations

- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `scripts/test-after-change.sh`
- `crates/harnesslab-cli/src/runner/external/runtime_anchor.rs`
- `crates/harnesslab-infra/src/file_lock.rs`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-05-benchmark-adapter-phase-2-inventory.md`
- `vs_review/2026-06-05-benchmark-adapter-phase-2-live-material-drift-review.md`

#### Change Introduction

The fix adds active requirement `external_runtime_anchor_projection`, active
registry row `REPLAY-009`, and selector route
`scripts/test-after-change.sh --select REPLAY-009`. The selector runs the CLI
runtime anchor concurrency tests and the infra file-lock serialization test.
The event assertion now parses JSONL `Event` values and checks `run_id`,
`task_id`, event name, attempt number, and attempt-relative public/private
paths. Phase 2 docs now cite `REPLAY-009` as the active proof surface.

#### Verification Status

- Passed `scripts/test-after-change.sh --select REPLAY-009`.
- Passed `scripts/test-after-change.sh --select META-002`.
- Passed `scripts/test-after-change.sh --select META-008`.
- Passed `scripts/test-after-change.sh --select SWEPRO-005`.
- Passed `cargo fmt --check`.
- Passed `git diff --check`.

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| test-validity-adversary | Round 4 blocker was a test traceability and selector proof gap. | requirement/registry/selector closure, event proof strength |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| test-validity-adversary | multi_agent_v1.spawn_agent test-engineer | 019e9604-d356-7521-afd1-b50f66db946a | spawn_agent result, nickname Parfit | fork_context=false | Round 5 Review Input | main-agent history, reasoning, drafts, conclusions, full diff unless inspected directly | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| test-validity-adversary-output | test-validity-adversary | 1 | 019e9604-d356-7521-afd1-b50f66db946a | 8 minutes | completed | returned no blocking findings | closure accepted |

### Reviewer Outputs

#### test-validity-adversary-output

##### Summary

Round 4's accepted blocker is closed. No new blocking findings were found.
The requirement, registry row, and executable selector are aligned and active:
`external_runtime_anchor_projection` exists in `tests/REQUIREMENTS.toml`,
`REPLAY-009` exists in `tests/TEST_REGISTRY.toml`, and
`scripts/test-after-change.sh --select REPLAY-009` routes to both the runtime
anchor tests and file-lock test with exact-count guards. Phase 2 docs now point
to `REPLAY-009` for this proof and scope `META-008` as selector-health
evidence rather than replay proof.

##### Blocking Findings

None.

##### Non-blocking Risks

- `REPLAY-009` event-payload assertions now parse JSONL `Event` records and
  pin `run_id`, `task_id`, event name, attempt marker, and attempt-relative
  paths, but currently assert payload details for the same-task overlap case
  rather than the parallel multi-task case.
- The non-Unix lock path now fails closed and the production call site
  propagates that failure, but there is no platform-specific black-box test for
  non-Unix fail-closed behavior.

##### Required Fixes

None for closure of the Round 4 blocker.

##### Missing Tests

- Optional `REPLAY-009` assertion for the multi-task parallel event payload.
- Optional platform-gated test for non-Unix fail-closed lock behavior if
  non-Unix support becomes part of the runtime surface.

##### Missing Logs / Observability

No blocking observability gap for this closure target. The
`external_runtime_anchor_projected` event is emitted after both authority files
are updated, matching the documented Phase 2 observability claim.

##### Evidence

- Active requirement: `tests/REQUIREMENTS.toml`.
- Active registry row: `tests/TEST_REGISTRY.toml`.
- Selector route and exact-count guards: `scripts/test-after-change.sh`.
- Runtime anchor projection and event assertion:
  `crates/harnesslab-cli/src/runner/external/runtime_anchor.rs`.
- Non-Unix lock fails closed:
  `crates/harnesslab-infra/src/file_lock.rs`.
- Docs cite `REPLAY-009` directly:
  `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md` and
  `docs/plans/2026-06-05-benchmark-adapter-phase-2-inventory.md`.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| test-validity-adversary | Registry-backed projection proof closure | Round 4 blocker required active requirement, registry row, and selector route | blocking closure | accept passed | Reviewer found `external_runtime_anchor_projection`, `REPLAY-009`, and selector routing aligned and active | No further blocking action required | n/a |
| test-validity-adversary | Multi-task event payload not pinned | Event payload is strongly checked for same-task overlap but not multi-task case | non-blocking | accept | Same-task event assertion already pins the event contract; multi-task payload would strengthen symmetry | Recorded as optional follow-up; no scope expansion in this closure round | Future `REPLAY-009` enhancement |
| test-validity-adversary | Non-Unix fail-closed path lacks platform test | Non-Unix lock behavior cannot be black-box verified on current Unix environment | non-blocking | accept | Implementation now fails closed instead of no-op; current runtime is Unix | Recorded as optional platform-specific follow-up | Future non-Unix support gate |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 5: Registry-Backed Projection Proof Closure
- Blocking re-review launch records:
  - listed above
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

The accepted blocking findings from Rounds 1 through 4 are fixed and received
fresh closure review. Round 5 passed with no blocking findings. The current
SWE-bench Pro Phase 2 snapshot authority slice is allowed to proceed to commit,
with residual non-blocking follow-ups documented for transactional multi-file
authority writes, multi-task projection event payload symmetry, and future
non-Unix lock testing.
