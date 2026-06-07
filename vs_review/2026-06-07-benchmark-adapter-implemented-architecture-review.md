# Subagent VS Review: Benchmark Adapter Implemented Architecture

- Created: 2026-06-07T04:14:16+0800
- Updated: 2026-06-07T05:03:42+0800
- Report schema: adversarial-v1
- Task: independent three-party adversarial review of the implemented benchmark adapter architecture
- Report path: `vs_review/2026-06-07-benchmark-adapter-implemented-architecture-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: blocked

## Round 1: Implemented Architecture Three-Party Review

### Review Input

#### Objective

Review the current implemented benchmark adapter architecture, not only the
plan, and determine whether module boundaries, implemented behavior, tests,
docs, and review evidence support the claimed adapter-layer closure.

#### Review Target

- Implemented data/planning adapter layer.
- Implemented runtime/execution adapter layer.
- Test registry, selector, artifact, diagnostics, and review evidence backing
  the current closure claim.

#### Target Locations

- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-06-benchmark-adapter-phase-8-full-gate-closure.md`
- `vs_review/2026-06-06-benchmark-adapter-phase-8-final-review.md`
- `vs_review/2026-06-07-benchmark-adapter-remaining-closure-review.md`
- `crates/harnesslab-adapters/src/registry.rs`
- `crates/harnesslab-adapters/src/data_boundary_contract.rs`
- `crates/harnesslab-adapters/src/data_boundary_scan.rs`
- `crates/harnesslab-adapters/src/data_contract_tests.rs`
- `crates/harnesslab-adapters/src/terminal_bench.rs`
- `crates/harnesslab-adapters/src/swe_bench_pro.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_adapter.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_adapter_tests.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_adapter.rs`
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro_adapter.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_anchor.rs`
- `crates/harnesslab-cli/tests/external_runtime_snapshot_contract.rs`
- `crates/harnesslab-cli/tests/swe_runtime_phase_contract.rs`
- `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs`
- `crates/harnesslab-cli/tests/terminal_bench_runtime_event_contract.rs`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `tests/artifact_contracts/int_011_swe_bench_pro_runtime_artifacts.txt`
- `scripts/test-after-change.sh`
- `scripts/verify-planned-adapter-selectors.sh`
- `scripts/verify-test-registry.sh`
- `xtask/src/adapter_claims.rs`
- `xtask/src/runtime_artifacts.rs`

#### Change Introduction

The implemented architecture claims that data/planning adapters and
runtime/execution adapters are split: data adapters live in
`harnesslab-adapters`, runtime adapters live at the CLI external-runner
boundary, Terminal-Bench and SWE-bench Pro dispatch through a runtime registry,
and replay, snapshot, redaction, diagnostics, selector, and artifact contracts
are backed by registered tests and review evidence.

#### Risk Focus

- Architecture boundary claims are only documented but not enforced.
- CLI-local runtime traits create hidden coupling that blocks future adapter
  extension.
- Data adapters leak runtime responsibility, ambient state, or execution policy.
- Runtime dispatch can be bypassed by benchmark-specific branches.
- Replay and snapshot authority can silently fall back to mutable live planning.
- Tests and selector registries prove only route names, not real behavior.
- Closure docs overstate behavior or classify true blockers as post-closure
  enhancements.

#### Assumptions To Attack

- Data adapters only own data inspection, preparation, task listing, task
  snapshots, and task-plan creation.
- Runtime adapters own preflight, execution, cleanup metadata, runtime
  snapshots, event taxonomy, and external-runner failure semantics.
- `runtime_adapter_for` is the effective runtime dispatch authority.
- `ExternalRunnerKind` is acceptable as the MVP registry key.
- Passing selector and registry checks are sufficient evidence for the current
  closure claim.
- Public/private artifact boundaries remain valid across failure, retry,
  cleanup, and replay paths.

#### Adversarial Lenses

- architecture
- implementation correctness
- testing validity
- failure and retry behavior
- data and replay authority
- privacy and redaction
- observability
- maintenance and extension path

#### Verification Status

- Main thread reports `cargo test -p xtask -- --nocapture` passed with 26
  tests.
- Main thread reports `scripts/verify-test-registry.sh` passed with 43
  requirements, 171 tests, and 16 adapter proof claims.
- Main thread reports `CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh`
  passed with `active=15 planned=1`.
- Reviewers were instructed not to trust these claims blindly and to inspect
  files directly.

#### Reviewer Instructions

- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- For blocking or major findings, include the broken assumption, failure
  scenario, trigger condition, impact, and proof needed.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| complex | 20 minutes | one bounded 10 minute extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architect | Challenge implemented module boundaries, dependency direction, and extension path. | architecture, maintenance |
| code-reviewer | Challenge code-level dispatch, state, failure, and privacy behavior. | correctness, edge cases |
| test-engineer | Challenge registry, selector, artifact, and review evidence quality. | test validity, traceability |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architect | `multi_agent_v1.spawn_agent` | `019e9e92-4393-7660-9e6b-79d2ef7cea72` | spawn tool result | false | Round 1 architecture review input plus target paths | main-agent history, reasoning, drafts, conclusions, full diff | yes |
| code-reviewer | `multi_agent_v1.spawn_agent` | `019e9e92-460e-75e0-882d-d3e4075fadc9` | spawn tool result | false | Round 1 implementation review input plus target paths | main-agent history, reasoning, drafts, conclusions, full diff | yes |
| test-engineer | `multi_agent_v1.spawn_agent` | `019e9e92-4927-7e61-a60d-45bc648e5e35` | spawn tool result | false | Round 1 evidence review input plus target paths | main-agent history, reasoning, drafts, conclusions, full diff | yes |
| code-reviewer-replacement | `multi_agent_v1.spawn_agent` | `019e9eab-e0f3-7092-81a6-fa0fa111fbf8` | spawn tool result | false | Round 1 replacement implementation review input plus target paths | main-agent history, reasoning, drafts, conclusions, prior reviewer output | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| architect | architect | 1 | `019e9e92-4393-7660-9e6b-79d2ef7cea72` | under 20 minutes | completed | reviewer returned non-blocking architecture risks | completed |
| code-reviewer | code-reviewer | 1 | `019e9e92-460e-75e0-882d-d3e4075fadc9` | 20 minutes plus 10 minute extension | timed_out | no reviewer output after bounded extension | replacement spawned |
| code-reviewer-replacement | code-reviewer | 2 | `019e9eab-e0f3-7092-81a6-fa0fa111fbf8` | under 20 minutes | completed | replacement reviewer returned request-changes findings | completed |
| test-engineer | test-engineer | 1 | `019e9e92-4927-7e61-a60d-45bc648e5e35` | under 20 minutes | completed | reviewer returned blocking evidence-quality findings | completed |

### Reviewer Outputs

#### architect

##### Summary

The reviewer reran `CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh`
and found no blocking inconsistency between current code and scoped closure
claims. The data/runtime split is real, runtime dispatch is registry-owned at
the runner boundary, and replay fails closed on missing or drifted
external-runtime authority. Remaining concerns are maintainability and replay
semantic drift.

##### Blocking Findings

- none

##### Non-blocking Risks

- The extension path remains centralized. Adding a third external benchmark
  still requires coordinated edits to the core enum, data registry, runtime
  registry, selector inventory, and test registry.
  - Broken assumption: a new benchmark can be added without substantial central
    edits.
  - Failure scenario: a third benchmark compiles only after multiple central
    files are edited in lockstep.
  - Trigger condition: adding a new external benchmark family.
  - Impact: higher maintenance cost and merge-conflict pressure.
  - Proof needed: a third-benchmark stub exercise that counts required
    touchpoints.
- Replay authority does not cover Terminal-Bench env-derived timeout semantics.
  - Broken assumption: replay/snapshot authority prevents silent runtime
    semantic drift, not only data/material drift.
  - Failure scenario: source and replay runs use different
    `HARNESSLAB_TERMINAL_BENCH_PROCESS_TIMEOUT_SEC` or
    `HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC` values while validation
    still passes.
  - Trigger condition: replay on a host or CI job with different env overrides.
  - Impact: data authority is preserved, but execution semantics can drift
    silently.
  - Proof needed: replay contract that mutates those env vars and blocks or
    emits an explicit pre-execution mismatch event.
- The registry/selector proof system is honest but still not an independent
  artifact oracle.
  - Broken assumption: test registry and selector evidence are completely
    outside the self-described system.
  - Failure scenario: a registry row's `required_artifacts` intent drifts in a
    way not covered by the selected test.
  - Trigger condition: artifact-contract drift outside the exact `INT-011`
    hard-coded contract.
  - Impact: confidence gap in meta-evidence.
  - Proof needed: shared-location artifact harvesting or generic post-test
    artifact existence checks for selected adapter proofs.

##### Required Fixes

- If replay is intended to preserve runtime semantics, snapshot and validate
  Terminal-Bench env-derived timeout policy or emit an explicit replay drift
  event.
- Before a third benchmark lands, reduce central registration friction or make
  the required touchpoints explicit.
- Keep `required_artifacts` docs scoped to registry-shape validation plus the
  exact `INT-011` contract until shared artifact checks exist.

##### Missing Tests

- Replay contract for Terminal-Bench timeout env drift.
- Third-benchmark extension-path proof.
- Generic post-test artifact existence verification if `required_artifacts`
  becomes stronger than registry-shape validation.

##### Missing Logs / Observability

- No replay-time event was found that compares current Terminal-Bench runtime
  timeout policy against the source run's stored runtime snapshot.
- No generic emitted artifact currently proves `required_artifacts` publication
  outside selected tests' own assertions.

##### Evidence

- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md:8` - docs
  record the implemented/review-closed scope and CLI-local runtime-trait
  decision.
- `docs/plans/2026-06-06-benchmark-adapter-phase-8-full-gate-closure.md:41` -
  closure evidence and scoped post-closure enhancements.
- `crates/harnesslab-adapters/src/data_boundary_contract.rs:154` - adapter crate
  dependency boundary and forbidden runtime imports/calls/path literals.
- `crates/harnesslab-cli/src/runner/external.rs:37` - runner boundary validates
  via runtime preflight and executes via runtime adapter dispatch.
- `crates/harnesslab-cli/src/runner/external/runtime_adapter.rs:10` - CLI-local
  runtime trait and closed-enum registry dispatch.
- `crates/harnesslab-cli/src/runner/external/terminal_bench_adapter.rs:56` -
  Terminal-Bench runtime ownership and env-derived timeout overrides.
- `crates/harnesslab-cli/src/runner/replay.rs:24` - fail-closed replay checks
  for stored runtime authority.
- `scripts/verify-planned-adapter-selectors.sh:4` - exact active/planned
  selector inventory is centrally enforced.

#### test-engineer

##### Summary

The reviewer reran `scripts/verify-test-registry.sh`,
`CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh`, and
`scripts/test-after-change.sh --select SWEPRO-001`; all passed. The reviewer
still returned `NEEDS ATTENTION` because the passes prove selector liveness and
metadata consistency more strongly than runtime artifact existence or official
SWE evaluator preservation.

##### Blocking Findings

- `required_artifacts` remains mostly declarative metadata rather than
  executable runtime proof.
  - Broken assumption: registry meta-tests prevent over-claiming.
  - Failure scenario: a selector passes while the actual test no longer
    produces the declared artifact, or the artifact path/content drifts.
  - Trigger condition: any `required_runtime_proof = true` requirement is linked
    to an active test with a non-empty `required_artifacts` list.
  - Impact: `ADAPT-RUNTIME-*`, `SWEPRO-*`, and closure docs can self-certify
    runtime proof without artifact existence or semantic checks.
  - Proof needed: convert registry proof from non-empty list validation to
    post-test artifact existence and key-field validation, at least for
    `ADAPT-RUNTIME-003/004/005` and `SWEPRO-001..005`.
- Several adapter proofs are source/document-shape assertions rather than
  independent black-box behavior proof.
  - Broken assumption: passing selector equals architecture behavior.
  - Failure scenario: dispatch or ownership drifts through helpers, aliases, or
    unscanned files while string-based tests still pass.
  - Trigger condition: architecture regression not expressed with the exact
    forbidden strings.
  - Impact: `ADAPT-DATA-001` and `ADAPT-RUNTIME-001` can prove shape without
    proving real runtime exclusivity.
  - Proof needed: black-box execution proof from real run events/artifacts
    showing selected runtime adapter provenance and adapter-owned cleanup.
- SWE-bench Pro lacks the real official evaluator preservation proof required
  by the plan.
  - Broken assumption: Terminal-Bench/SWE tests cover real external benchmark
    failure paths.
  - Failure scenario: upstream evaluator CLI, output schema, docker interaction,
    or prediction/eval contract drifts while fake-tool selectors remain green.
  - Trigger condition: `fake_swe_tools()` continues to simulate success while
    real `uv`, evaluator, or docker behavior changes.
  - Impact: `SWEPRO-001..005` and `INT-011` do not fully support official
    evaluator preservation closure claims.
  - Proof needed: at least one real SWE evaluator path or equivalent verifier
    script in closure evidence, or docs must be downgraded.

##### Non-blocking Risks

- `scripts/verify-planned-adapter-selectors.sh` proves the hard-coded 15+1
  inventory runs, but not that those selectors are sufficient behavior proof.
- Some `required_artifacts` entries use mixed path granularity that will need
  normalization before a real shared-location existence gate can work.
- Fake Docker/uv helpers isolate host instability but narrow coverage of real
  CLI drift, real stderr shape, and real cleanup side effects.

##### Required Fixes

- Upgrade registry runtime proof to executable artifact existence plus minimal
  JSON/log semantic checks.
- Add a black-box dispatch proof for `ADAPT-RUNTIME-001` from run events and
  attempt artifacts.
- Add real SWE official evaluator preservation proof or downgrade closure
  wording around that claim.

##### Missing Tests

- Generic registry artifact existence gate for `required_runtime_proof = true`.
- Black-box registry dispatch test through CLI execution.
- Real SWE official evaluator/verifier test for invocation, prediction schema,
  and `eval_results.json` parse.
- Failure-path semantic tests for malformed artifact content, not only
  existence/absence.

##### Missing Logs / Observability

- Stable structured `adapter dispatch provenance` should be emitted rather than
  inferred from message text.
- Official tool identity/version/source hash should be exposed in auditable
  artifacts, especially for SWE evaluator paths.
- `adapter-proof-inventory.json` should record proof strength and checked
  artifacts/fields, not only claim/status/selector route.

##### Evidence

- `xtask/src/main.rs:246` - `required_runtime_proof` checks only whether an
  active test has a non-empty `required_artifacts` list.
- `xtask/src/runtime_artifacts.rs:10` - registry artifact validation is path
  normalization plus exact `INT-011` list matching.
- `xtask/src/adapter_claims.rs:33` - adapter claim proof is registration and
  selector-route consistency.
- `crates/harnesslab-cli/src/runner/external/runtime_adapter_tests.rs:19` -
  `ADAPT-RUNTIME-001` uses source-string assertions.
- `crates/harnesslab-adapters/src/data_contract_tests.rs:310` - data adapter
  proof includes coverage/boundary document assertions.
- `crates/harnesslab-cli/tests/support/swe.rs:190` - SWE tests rely on fake
  toolchain helpers.
- `docs/plans/2026-06-06-benchmark-adapter-phase-8-full-gate-closure.md:99` -
  shared-location post-test artifact existence checks are still future work.

#### code-reviewer-replacement

##### Summary

The replacement reviewer returned `REQUEST CHANGES`. It did not find a direct
preflight, execute, or run-level cleanup bypass of `runtime_adapter_for(...)`;
the current entrypoints route through the registry, and exact `ADAPT-RUNTIME-001`
and `ADAPT-RUNTIME-002` unit contracts passed locally. The blocking issue is
that several post-start external-runtime error paths still return raw `Err`, so
the outer scheduler can treat them as worker failures instead of classified
attempt results.

##### Blocking Findings

- Raw adapter `Err` paths can abort the whole run without a structured external
  task result.
  - Broken assumption: external runtime failures always close into one
    adapter-authoritative `TaskAttemptResult`, snapshot pair, and cleanup
    verdict.
  - Failure scenario: Terminal-Bench hits a post-start `?` path in pre-cleanup,
    process launch, event write, cleanup-report write, snapshot write, or final
    result write; SWE hits `?` in sandbox launch, `git diff`, evaluator exec,
    snapshot write, or final `result.json` write.
  - Trigger condition: any I/O, process-launch, or runtime-write failure after
    the attempt starts.
  - Impact: `execute_task` returns `Err`, `execute_attempts_with` records
    `first_error`, the run may abort, and the task can end without
    `result.json` or final snapshot/cleanup closure.
  - Proof needed: fault-injection contracts that force post-start `Err` branches
    and assert classified `TaskAttemptResult`, final external-runtime snapshots,
    and deterministic run health.

##### Non-blocking Risks

- SWE workspace-prep failure writes the happy-path runtime snapshot shape.
  - Broken assumption: snapshots stay phase-accurate on partial failure.
  - Failure scenario: `prepare_workspace(...)` fails, but
    `write_swe_runtime_snapshots(...)` still records agent/evaluation commands
    and advertises prediction/eval/patch artifacts.
  - Trigger condition: workspace-preparation failure in `SWEPRO-002`.
  - Impact: public/runtime artifacts overclaim phases and files that never ran.
  - Proof needed: use setup-failure snapshots for this branch or make snapshots
    phase-aware.
- Replay keeps its own adapter-version map outside the runtime adapter registry.
  - Broken assumption: the registry is the single runtime authority.
  - Failure scenario: adapter version changes in implementation while replay
    validation uses its own hard-coded match.
  - Trigger condition: adapter-version bump or new external runner.
  - Impact: execute/preflight/cleanup can stay correct while replay drift is
    validated against stale authority.
  - Proof needed: expose adapter version from the same runtime adapter registry
    object and have replay consume it.

##### Required Fixes

- Add a normalization boundary around external runtime execution so
  adapter/internal `Err` becomes a structured `TaskAttemptResult` with final
  snapshot/report closure where possible.
- Make SWE workspace-prep failure emit failure-phase snapshots only.
- Remove replay's hard-coded adapter-version match and derive current adapter
  version from runtime adapter authority.

##### Missing Tests

- Fault-injection contract for Terminal-Bench post-start launch/cleanup/snapshot
  or result-write failure.
- Fault-injection contract for SWE `sandbox::run_agent`, `git diff`, evaluator
  exec, final snapshot, and result write failures.
- Workspace-prep-failure snapshot contract asserting no agent/evaluation command
  or patch/public-artifact claims when those phases never ran.

##### Missing Logs / Observability

- No guaranteed `external_runner_internal_error` event with `adapter_id`,
  `phase`, and `attempt` exists for unexpected adapter `Err` paths.
- SWE snapshots do not encode explicit phase diagnostics on failure, so
  operators cannot distinguish "phase never ran" from "phase ran but produced
  missing artifacts" from snapshots alone.

##### Evidence

- `crates/harnesslab-cli/src/runner/external.rs:184` - direct execution is
  routed through `runtime_adapter_for(runner.kind).execute(ctx)`.
- `crates/harnesslab-cli/src/runner/external/runtime_adapter.rs:75` - preflight
  dispatch uses the runtime adapter registry.
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs:23` - pre-cleanup
  `?` can return raw `Err`.
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs:45` - process
  launch `?` can return raw `Err`.
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs:181` - cleanup
  report write `?` can return raw `Err`.
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs:97` - agent run
  `?` can return raw `Err`.
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs:127` - patch
  capture `?` can return raw `Err`.
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs:159` - evaluator
  run `?` can return raw `Err`.
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs:209` - final SWE
  runtime snapshot write `?` can return raw `Err`.
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs:217` - final
  result write `?` can return raw `Err`.
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs:72` - workspace
  prep failure branch uses the full runtime snapshot writer.
- `crates/harnesslab-cli/src/runner/replay.rs:199` - replay uses an independent
  `current_adapter_version` lookup.

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| architect | Third benchmark extension remains centralized. | New external benchmark requires core enum, data registry, runtime registry, selector inventory, and registry edits. | medium | accept | Current MVP intentionally keeps `ExternalRunnerKind` closed, but the touchpoint cost is real. | Recorded as non-blocking architecture debt. | Add extension-path checklist or stub benchmark proof before adding a third benchmark. |
| architect | Terminal-Bench replay does not validate env-derived timeout semantics. | Replay can preserve data/material authority while process/no-output timeout semantics drift through environment changes. | medium | accept | Runtime timeout values are env-derived in adapter code and not compared by replay authority. | Recorded as non-blocking replay-semantic drift risk. | Add replay contract for timeout env drift or snapshot timeout policy. |
| architect / test-engineer | `required_artifacts` is not a generic artifact oracle. | Registry validates shape/non-empty artifacts, but not generic post-test existence and semantic content. | blocking | accept | `xtask/src/main.rs:246-260` checks non-empty artifact lists; Phase 8 docs keep shared-location existence checks as future work. | Recorded as accepted blocker against stronger runtime-proof closure claims. | Implement shared artifact proof report/existence checks or keep closure wording downgraded. |
| test-engineer | Adapter proof has source/doc-shape self-proof loops. | Source-string and doc coverage assertions can pass without independent black-box behavior proof. | blocking | accept | `ADAPT-RUNTIME-001` source-string checks are useful regression guards but not full black-box dispatch proof. | Recorded as accepted evidence-quality blocker. | Add black-box dispatch provenance proof from CLI run events/artifacts. |
| test-engineer | SWE-bench Pro lacks real official evaluator preservation proof. | Fake toolchain selectors can stay green while real evaluator CLI/schema/drift breaks. | blocking | accept | SWE tests use fake helpers; plan language asks for official evaluator preservation evidence. | Recorded as accepted evidence-quality blocker. | Add real SWE verifier/evaluator proof or downgrade docs that imply this is complete. |
| code-reviewer-replacement | Raw adapter `Err` paths can abort run without structured attempt closure. | Post-start I/O/process/write failures can bubble as scheduler worker errors instead of classified external task results. | blocking | accept | Local source inspection confirms multiple `?` paths in Terminal-Bench and SWE execution after attempt start. | Recorded as accepted implementation blocker. | Add external-runtime error normalization boundary plus fault-injection tests. |
| code-reviewer-replacement | SWE workspace-prep failure writes happy-path snapshot shape. | Failure before agent/evaluator phases can still advertise commands/artifacts for phases that never ran. | medium | accept | Workspace prep failure branch calls `write_swe_runtime_snapshots`, which is not phase-aware. | Recorded as non-blocking state-consistency risk, but should be fixed with the error-normalization work. | Use setup-failure snapshots or phase-aware snapshot writer. |
| code-reviewer-replacement | Replay adapter-version authority is split from runtime registry. | Replay can validate against a hard-coded version map that drifts from runtime adapter authority. | medium | accept | `runtime_adapter_for` owns execution dispatch, while replay has independent `current_adapter_version`. | Recorded as non-blocking authority split. | Expose adapter version through runtime adapter registry and consume it in replay. |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: no
- Blocking re-review completed: no
- Blocking re-review passed: no
- Blocking re-review round links:
  - n/a; implementation fixes have not been made in this review-only task
- Blocking re-review launch records:
  - n/a
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: accepted blocking findings remain unresolved
- Allowed to proceed: no

## Final Conclusion

Blocked / request changes. The implemented adapter architecture has real
data/runtime separation and no direct `runtime_adapter_for` dispatch bypass was
found, but the three-party review did not pass. Accepted blockers remain around
post-start adapter error normalization, generic executable runtime artifact
proof, black-box dispatch provenance, and real SWE official evaluator
preservation evidence.
