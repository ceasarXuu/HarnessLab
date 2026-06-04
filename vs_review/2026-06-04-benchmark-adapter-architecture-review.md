# Subagent VS Review: Benchmark Adapter Architecture

Status: closed

## Review Target

- Target artifact: `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- Review type: Round 0 design review before implementation
- Source session policy: no inherited main-agent context
- Requested by: user request to execute re-review

## Review Input Packet

### Objective

Falsify the Benchmark Adapter Layer Architecture Design before implementation.

### Shared Target

- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`

### Shared Instructions

- Fresh internal subagent session.
- Read repository files directly.
- Do not modify files.
- Do not inherit or assume the main agent's hidden context.
- Cite evidence paths and line numbers where possible.
- For each finding, include the broken assumption, counterexample/failure scenario, impact, and proof needed.
- If there are no blocking findings, state that explicitly.

### Architecture-Adversary Packet

Navigation targets:

- `docs/architecture.md` section 6
- `docs/mvp-development-spec.md` section 7
- `crates/harnesslab-adapters/src/registry.rs`
- `crates/harnesslab-cli/src/runner/external.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs`
- `tests/TEST_REGISTRY.toml`
- `scripts/test-after-change.sh`

Risk focus:

- adapter data/runtime boundaries
- dependency direction and whether runtime extraction belongs in CLI first
- migration sequence risks
- abstraction that hides real Terminal-Bench/SWE-bench behavior
- whether the plan prevents future benchmark-specific branching from leaking back into orchestrator
- replay/snapshot ownership boundaries

### Test-Validity-Adversary Packet

Navigation targets:

- `docs/test-engineering.md`
- `docs/mvp-development-spec.md` section 7 and section 13
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `scripts/test-after-change.sh`
- `crates/harnesslab-adapters/src/*.rs`
- `crates/harnesslab-cli/src/runner/external*.rs`
- `crates/harnesslab-cli/src/runner/external/*.rs`

Risk focus:

- whether proposed ADAPT-DATA / ADAPT-RUNTIME tests can catch real regressions
- selector routing and zero-test false pass risk
- fixture-only testing vs real official runner evidence
- Terminal-Bench and SWE-bench Pro preservation tests
- seeded failure coverage for timeouts, cleanup, replay, result parse, data corruption, and invalid patches
- whether acceptance matrix proof is concrete enough

### Observability-Adversary Packet

Navigation targets:

- `docs/development-operations.md`
- `docs/architecture.md` sections 5-6 and Terminal-Bench runtime details
- `docs/mvp-development-spec.md` section 7
- `crates/harnesslab-cli/src/runner/external.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_result.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_cleanup.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_runtime.rs`
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs`

Risk focus:

- whether event sequence and runtime snapshots are sufficient for incident diagnosis
- missing phase names, cleanup evidence, official runner versioning, progress/watchdog details
- replay diagnostics and mutable data risks
- result parse vs execution failure classification observability
- secret redaction and public artifact risks in command/log snapshots
- whether the plan gives future operators enough proof to distinguish benchmark verdict from HarnessLab runtime failure

## Reviewer Launch Records

| Round | Reviewer Role | Subagent Role | Agent ID | Nickname | Fresh Context | Context Excluded | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Round 0 | architecture-adversary | architect | `019e9260-db97-7f33-ab7e-087096dc4910` | Chandrasekhar | `fork_context=false` | full main-agent chat, hidden reasoning, previous conclusions | completed |
| Round 0 | test-validity-adversary | test-engineer | `019e9260-ddd8-7ca1-bee6-fbecf8a70377` | Sartre | `fork_context=false` | full main-agent chat, hidden reasoning, previous conclusions | completed |
| Round 0 | observability-adversary | code-reviewer | `019e9260-e071-7a83-85d2-2460cd6c0761` | Lovelace | `fork_context=false` | full main-agent chat, hidden reasoning, previous conclusions | completed |

## Timeout Policy

- Round type: complex design review
- Primary wait budget: 15 minutes
- Replacement policy: one fresh replacement attempt per role if the primary is lost or times out
- Timeout is not a pass

## Round 0 Reviewer Outputs

### Architecture-Adversary: Chandrasekhar

Summary: The plan was not implementation-safe because replay/snapshot ownership was contradictory and the runtime registry did not actually remove benchmark-specific CLI branching or raw-profile policy parsing.

Blocking findings:

1. Replay/snapshot ownership is undefined, and migration order leaves silent live-data fallback in place.
   - Broken assumption: runtime extraction can happen first while snapshot hardening waits for later.
   - Counterexample: current `TaskPlan.external_runner` persists mutable `dataset_path`/`source_path`, and replay can fall back to fresh adapter planning when `benchmark.snapshot.json` is missing.
   - Impact: replay can bind to changed datasets/source checkouts or have competing authorities between task snapshots and future runtime snapshots.
   - Required proof: define snapshot authority, retire silent replay re-plan for external benchmarks, and move snapshot hardening before or together with runtime registry extraction.
   - Evidence: `crates/harnesslab-core/src/benchmark.rs`, `crates/harnesslab-adapters/src/terminal_bench.rs`, `crates/harnesslab-adapters/src/swe_bench_pro.rs`, `crates/harnesslab-cli/src/runner/store.rs`, `crates/harnesslab-cli/src/runner/replay.rs`, `crates/harnesslab-cli/tests/replay_contract.rs`, `tests/TEST_REGISTRY.toml`.
2. Slice C does not cover benchmark-specific preflight and agent-policy branching already in CLI.
   - Broken assumption: replacing the execute-path `match ExternalRunnerKind` is enough.
   - Counterexample: `validate_profile_for_plan`, `host_agent_execution_reason`, and Terminal-Bench label/agent-kind parsing still live in CLI paths and use raw `AgentProfile`.
   - Impact: new benchmarks would still require CLI edits for validation, host/sandbox gating, and bridge rules.
   - Required proof: make `preflight` registry-dispatched and define a benchmark-facing materialized agent-runtime contract.
   - Evidence: `crates/harnesslab-cli/src/runner/external.rs`, `crates/harnesslab-cli/src/runner/external/terminal_bench.rs`, `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`.

Non-blocking risks:

- Event schema in the plan did not line up with current `external_runner_configured`, `terminal_bench_cleanup`, and SWE-bench Pro ad hoc events.
- Closed `ExternalRunnerKind` is acceptable for MVP but still means new benchmark families require core model edits.

### Test-Validity-Adversary: Sartre

Summary: The design's testing and validation claims were not falsifiable enough. The largest gaps were nonexistent proof surfaces, weak selector coverage, fixture-only official-runner proof, and acceptance claims for artifacts/events not yet produced.

Blocking findings:

1. `ADAPT-DATA-*`, `ADAPT-RUNTIME-*`, and `SWEPRO-*` were claimed as proof surfaces but did not exist.
   - Broken assumption: acceptance matrix can rely on nonexistent ID families.
   - Counterexample: runtime branching could remain in place while current `TB-*`/`INT-*` still pass.
   - Impact: generic dispatch and SWE preservation could regress silently.
   - Required proof: add requirements, registry entries, and selectors for concrete IDs.
2. `INT-011` selector routing is a false-comfort umbrella.
   - Broken assumption: `INT-011` proves external benchmark smoke contracts broadly.
   - Counterexample: `scripts/test-after-change.sh --select INT-011` runs exactly one SWE smoke test, while many `int_011_*` failure-path tests remain unselected.
   - Impact: documented selector can be green while no-diff, workspace failure, missing evaluator output, input modes, and sandbox-failure paths are unexercised.
   - Required proof: split cases into separate IDs or use counted grouped selector.
3. Runtime-proof contract is weaker than the repo's own test-engineering standard.
   - Broken assumption: current external smoke proof is concrete enough.
   - Counterexample: runtime proof standard expects run metadata, command snapshot, snapshots, events, attempt results, logs, and patch artifacts; `INT-011` only requires `results.json` and `report.html`.
   - Impact: green selector does not prove artifact durability, event emission, replay materials, or patch/evaluator artifacts.
   - Required proof: align required artifacts and assert contents.
4. Fixture shims do not prove preservation of real official-runner behavior.
   - Broken assumption: fake `uv`, `uvx`, and `docker` shims preserve official behavior.
   - Counterexample: parser drift, official CLI argument changes, evaluator schema changes, or side effects can break real runs while shim tests pass.
   - Impact: official runner compatibility can regress silently.
   - Required proof: add one real official-runner preservation proof per benchmark family.
5. Acceptance matrix claims runtime snapshots/events that are absent or renamed.
   - Broken assumption: runtime config observability is already grounded.
   - Counterexample: plan used `external-runtime.snapshot.json`, `external_runner_preflight`, and `external_runner_runtime_config`, while current code has `external_runner_configured` and no external runtime snapshot.
   - Impact: observability can be marked complete without durable snapshot or stable event proof.
   - Required proof: define exact artifact/event names and assert them.

Non-blocking risks:

- `external_benchmark_runtime` is too broad to localize Terminal-Bench versus SWE-bench Pro regressions.
- Existing SWE adapter tests are anchored to `descriptor/plan`, not the proposed prepare/list/snapshot split.
- Selector exact-match guards do not guarantee a broad registry title covers every claimed behavior.

### Observability-Adversary: Lovelace

Summary: The plan was not implementation-ready from an observability standpoint. The proposed event contract was weaker than current operations, replay snapshots did not prove immutable workload identity, and adapter runtime artifact redaction was underspecified.

Blocking findings:

1. Generic event sequence would regress incident diagnosis.
   - Broken assumption: a seven-event generic sequence is enough.
   - Counterexample: Terminal-Bench QEMU dataset prep, Docker build stall, no-progress, and cleanup override require existing detailed events to distinguish setup/build stall from official verdict and cleanup override.
   - Impact: operators could not prove whether the benchmark judged the agent or HarnessLab killed/overrode the run.
   - Required proof: define stable phase names and required fields, or preserve current event names/semantics.
2. Replay snapshot contract does not prove immutable workload identity.
   - Broken assumption: benchmark/task/external snapshots as described are enough.
   - Counterexample: Terminal-Bench mutates/copies QEMU datasets; SWE extracts parquet samples and uses evaluator source. The plan did not require hashes/manifests for these mutable inputs or official runner identity.
   - Impact: replay can silently execute different payloads or evaluators after data, `uvx`, or source drift.
   - Required proof: persist hashes/manifests and compare them before replay execution.
3. Adapter runtime artifact redaction boundary is underspecified.
   - Broken assumption: "secrets must be redacted in public artifacts" is specific enough.
   - Counterexample: `external-runtime.snapshot.json` could contain command, env policy, cleanup tokens, or replay diagnostics without a public/private schema.
   - Impact: command/env/setup secrets can leak into JSON/JSONL/report artifacts.
   - Required proof: define public/private snapshots and SEC tests scanning known fake secrets.

Non-blocking risks:

- Plan did not require preserving official benchmark verdict and final HarnessLab override provenance.
- `CleanupReport` was named but not shaped.
- Official runner versioning was optional rather than replay-visible.

## Main-Agent Triage

All blocking findings are accepted.

| ID | Reviewer | Finding | Decision | Action Taken |
| --- | --- | --- | --- | --- |
| A1 | architecture-adversary | replay/snapshot ownership undefined and live-data fallback left in place | accept | Added `Snapshot Authority`, moved snapshot/replay hardening before runtime registry, marked `TaskPlan.external_runner` as launch hint not replay authority, required retiring silent replay live replanning or explicit legacy degraded mode. |
| A2 | architecture-adversary | runtime registry does not own preflight/raw profile branching | accept | Added `BenchmarkAgentRuntimeConfig`, removed raw `AgentProfile` from normal runtime context, added registry-dispatched preflight ownership for validation, execution-mode gating, and bridge compatibility. |
| T1 | test-validity-adversary | claimed `ADAPT-*`/`SWEPRO-*` proof surfaces do not exist | accept | Added concrete initial `ADAPT-DATA-*`, `ADAPT-RUNTIME-*`, and `SWEPRO-*` IDs and requirement that they exist in requirements, registry, and selectors before proof claims. |
| T2 | test-validity-adversary | `INT-011` selector is an umbrella false pass | accept | Added requirement to split `INT-011` cases or route them through counted grouped selectors. |
| T3 | test-validity-adversary | runtime proof weaker than test-engineering standard | accept | Added required artifact standard for external runtime proofs. |
| T4 | test-validity-adversary | fixture shims do not prove real official runner behavior | accept | Added required official-runner/evaluator preservation proof per benchmark family. |
| T5 | test-validity-adversary | acceptance matrix claims absent/renamed snapshots/events | accept | Replaced generic event/snapshot claims with concrete event taxonomy, public/private snapshots, and assertions. |
| O1 | observability-adversary | generic event sequence regresses incident diagnosis | accept | Added stable event taxonomy, preserved existing Terminal-Bench operator-critical event names, and added stable SWE phase events. |
| O2 | observability-adversary | replay snapshots do not prove immutable workload identity | accept | Added immutable identity requirements for Terminal-Bench and SWE-bench Pro, including manifests/hashes and official runner/evaluator identity. |
| O3 | observability-adversary | adapter runtime artifact redaction boundary underspecified | accept | Added public/private runtime artifact contract and required fake secret scans. |

Validation evidence after plan fixes:

- `git diff --check` passed.
- Keyword validation confirmed the fixed plan now includes `BenchmarkAgentRuntimeConfig`, `Snapshot Authority`, `external-runtime.private.json`, `external-runtime.public.json`, concrete `ADAPT-DATA-*`, `ADAPT-RUNTIME-*`, `SWEPRO-*`, `INT-011` split/count routing, official-runner/evaluator preservation proof, preserved `external_runner_configured` and `terminal_bench_cleanup`, and stable SWE-bench Pro phase events.
- Focused fresh closure review launched.

## Round 1 Closure Review

### Closure Review Input

Scope: focused closure only. Reviewers were instructed not to restart broad
review unless the fixes created a new blocking issue.

Architecture closure target:

- Verify closure of A1 replay/snapshot authority and A2 runtime registry
  preflight/raw-profile boundary.

Test-validity closure target:

- Verify closure of T1 through T5 around proof-surface existence claims,
  `INT-011`, runtime-proof artifacts, official-runner preservation proof, and
  snapshot/event acceptance claims.

Observability closure target:

- Verify closure of O1 through O3 around event taxonomy, immutable replay
  identity, and public/private adapter runtime artifacts.

### Closure Launch Records

| Round | Reviewer Role | Subagent Role | Agent ID | Nickname | Fresh Context | Context Excluded | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Round 1 | architecture-adversary | architect | `019e9269-809f-7152-96d9-cf803dad8e99` | Russell | `fork_context=false` | full main-agent chat, hidden reasoning, previous conclusions | completed |
| Round 1 | test-validity-adversary | test-engineer | `019e9269-8284-7452-ae75-2d004e8e6b0e` | Copernicus | `fork_context=false` | full main-agent chat, hidden reasoning, previous conclusions | completed |
| Round 1 | observability-adversary | code-reviewer | `019e9269-84d9-78d1-b98c-17bbddf2edb6` | Locke | `fork_context=false` | full main-agent chat, hidden reasoning, previous conclusions | completed |

### Closure Reviewer Outputs

#### Architecture-Adversary Closure: Russell

Closure verdict: passed.

Remaining blocking findings: none.

Residual risks:

- Repo-wide architecture/spec docs are not yet synchronized to the stronger boundary. This is planned for the docs sync slice.
- Legacy degraded replay remains an open product decision.
- Current code still contains old behavior; closure was design-level only.

#### Test-Validity-Adversary Closure: Copernicus

Closure verdict: passed.

Remaining blocking findings: none.

Residual risks:

- Current repo does not yet have registered `ADAPT-DATA-*`, `ADAPT-RUNTIME-*`, or `SWEPRO-*` entries. This is acceptable for design closure because the plan now states those families do not count as proof until registered in requirements, registry, and selectors.
- Current `INT-011` selector remains misleading in implementation; the plan now explicitly forbids using it as umbrella proof and requires split/count routing.
- Future runtime proof artifacts/events are not implemented yet.

#### Observability-Adversary Closure: Locke

Closure verdict: failed.

Remaining blocking finding:

- `external_runner_timeout` and `external_runner_setup_failed` were still not preserved as concrete required/queryable events in the observability contract.

Decision: accept.

Action taken:

- Added `external_runner_timeout` and `external_runner_setup_failed` to the required common events table with required diagnostic fields.
- Added both event names to the Terminal-Bench compatibility event list.
- Extended `ADAPT-RUNTIME-005` to assert preservation of those event names.
- Extended Terminal-Bench runtime extraction acceptance to assert both events remain queryable with hard-timeout/setup-failure fields.

Validation evidence:

- `git diff --check` passed.
- Focused Round 2 observability closure review completed and passed.

## Round 2 Observability Closure Review

### Round 2 Closure Input

Scope: focused closure only for the remaining observability blocker. Reviewer
was instructed not to restart broad review unless the fix created a new
blocking observability issue.

Remaining blocker:

- `external_runner_timeout` and `external_runner_setup_failed` were not
  preserved as concrete required/queryable events with required diagnostic
  fields.

### Round 2 Launch Record

| Round | Reviewer Role | Subagent Role | Agent ID | Nickname | Fresh Context | Context Excluded | Status |
| --- | --- | --- | --- | --- | --- | --- | --- |
| Round 2 | observability-adversary | code-reviewer | `019e926e-ee30-7631-8c9f-b8412edb8385` | Bacon | `fork_context=false` | full main-agent chat, hidden reasoning, previous conclusions | completed |

### Round 2 Reviewer Output

Closure verdict: passed.

Remaining blocking findings: none.

Residual risks:

- Current implementation is still pre-migration. Required timeout/setup
  diagnostic fields are now a design-and-test obligation, not implemented
  first-class event fields yet.
- Current Terminal-Bench and SWE-bench Pro code still emit legacy/simple forms;
  this closure is design-level only.

Evidence:

- `external_runner_timeout` is now a required common event with hard-timeout
  diagnostics.
- `external_runner_setup_failed` is now a required common event with setup
  failure diagnostics.
- Terminal-Bench compatibility list preserves both names.
- `ADAPT-RUNTIME-005` requires preservation of both events.
- Slice E requires asserting both events stay queryable.

## Final Closure Status

Passed for design review.

All accepted Round 0 blocking findings received plan fixes and fresh focused
closure review:

- Architecture closure: passed.
- Test-validity closure: passed.
- Observability closure Round 1: failed on missing concrete
  `external_runner_timeout` and `external_runner_setup_failed` requirements.
- Observability closure Round 2: passed after those event requirements were
  added.

Unresolved blocking findings: none.

Residual implementation risks:

- Current code still contains pre-migration behavior. The review only closes
  the architecture design document, not implementation.
- `ADAPT-DATA-*`, `ADAPT-RUNTIME-*`, and `SWEPRO-*` are planned proof surfaces
  and must be registered before implementation can claim them.
- Current `INT-011` remains a misleading selector until the implementation
  slice splits or count-routes it.
- Legacy degraded replay remains an explicit open product decision.

## Closure Status

Round 0 found accepted blocking findings. Closure requires a fresh focused re-review after the plan fixes.
