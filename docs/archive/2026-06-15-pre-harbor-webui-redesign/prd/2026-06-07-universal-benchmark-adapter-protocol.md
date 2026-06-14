# PRD: Universal Benchmark Adapter Protocol

- Status: Draft
- Created: 2026-06-07
- Updated: 2026-06-07
- Owner / requester: maintainer
- Source request: Make the benchmark adapter layer a standard protocol so any benchmark that completes the adapter contract can be called seamlessly while non-adapter layers remain benchmark-agnostic.

## Requester Review Summary

- Key decisions:
  - The target is protocol isolation, not merely faster built-in benchmark wiring.
  - Each benchmark may have unavoidable integration cost, but that cost must be contained inside its adapter implementation.
  - Once an adapter satisfies the protocol and conformance gates, runner, replay, reporting, doctor, registry validation, and selector routing must not need benchmark-specific logic changes.
  - Benchmark identity above the adapter layer is an opaque id plus capabilities, not a reason for branching.
- Important exceptions:
  - A central registry may know which adapters are installed or enabled.
  - Governance metadata may classify adapters as experimental or stable.
  - Official benchmark preservation evidence may remain adapter-specific, but the proof shape must be standardized.
- Must-confirm before implementation:
  - Whether the first proof target should be a toy benchmark adapter, a real third benchmark, or both.
  - Whether out-of-tree adapters are required in the first release, or whether in-repo adapters with strict protocol isolation are enough.
- Status reason:
  - The product intent is clear, but launch slice and extension packaging model still need confirmation.

## 1. Background And Product Intent

HarnessLab currently has a maturing adapter layer for existing benchmark families, but the next product target is broader: a benchmark should become callable by implementing a standard adapter protocol rather than by teaching multiple system layers about that benchmark.

The adapter layer should become the only place where benchmark-specific behavior lives. All upstream and downstream product surfaces should work through a stable contract: planning, data readiness, execution, cleanup, scoring, snapshots, replay, diagnosis, redaction, and reporting.

## 2. Goals And Success Criteria

Goals:

- Define one benchmark adapter protocol that covers the full lifecycle from data discovery to replay.
- Allow a new benchmark to become callable after its adapter passes conformance checks.
- Keep benchmark-specific code out of runner orchestration, replay, report generation, doctor/readiness, and generic test routing.
- Make adapter quality visible through standardized metadata, capability declarations, and proof gates.
- Preserve official benchmark behavior without leaking benchmark-specific branches into non-adapter layers.

Success criteria:

- A new benchmark can be added by implementing adapter-owned files plus registry metadata and conformance tests.
- No non-adapter layer needs to match on the benchmark id to run, replay, diagnose, or report the benchmark.
- The system can reject incomplete adapters before runtime with actionable conformance failures.
- Stable adapters expose enough public/private artifacts for replay, audit, and redaction guarantees.
- Existing Terminal-Bench and SWE-bench Pro behavior can be expressed through the same protocol.

## 3. Users And Usage Context

Primary users:

- HarnessLab maintainers adding or upgrading benchmark support.
- Advanced internal or external engineers integrating a benchmark with known official runner or evaluator semantics.

Usage contexts:

- Add a new benchmark family.
- Upgrade an existing benchmark adapter version.
- Diagnose why a benchmark adapter is not runnable in the current environment.
- Replay a prior run without silent live replanning.
- Review whether an adapter is experimental or stable.

## 4. Scope

### In Scope

- Standard adapter lifecycle contract.
- Adapter capability metadata.
- Conformance test suite and selector registration rules.
- Artifact and snapshot contract.
- Error taxonomy and phase taxonomy.
- Redaction and public/private diagnostics contract.
- Replay material authority contract.
- Doctor/readiness integration through adapter-declared checks.
- Documentation template for adapter implementers.

### Out Of Scope

- Public marketplace or remote adapter distribution in the first slice.
- Arbitrary untrusted adapter execution sandboxing.
- Removing all central registry metadata.
- Guaranteeing official benchmark preservation without adapter-provided official-runner proof.

## 5. Core User Journey

1. Adapter author starts from the benchmark adapter template.
2. Adapter author fills descriptor, capabilities, data lifecycle, runtime lifecycle, evaluation mapping, snapshot contract, and readiness checks.
3. Adapter author runs the conformance suite.
4. The system reports missing protocol fields, missing artifacts, unsafe public data, unstable task ids, replay gaps, or unclassified failures.
5. Once conformance passes, the benchmark can be selected by id and called by generic run/replay/report/doctor flows.
6. A stable promotion review verifies official behavior preservation and closes any adapter-specific risks.

## 6. Interaction And Information Design

Generic system surfaces should display benchmark information from adapter metadata:

- benchmark id
- display name
- adapter version
- stability level
- supported modes
- required local tools
- required dataset shape
- supported splits
- known environment blockers
- public artifact list
- private artifact list
- replay readiness status

Users should not need to know which internal code path handles a benchmark.

## 7. Product Rules And State Logic

- A benchmark adapter is protocol-complete only when it passes all required conformance gates for its declared capabilities.
- Non-adapter layers may branch on generic capability flags, but must not branch on concrete benchmark ids.
- Adapter runtime failures must map into standardized failure class, failure code, adapter phase, and adapter subphase.
- Adapter artifacts must declare public/private boundaries before they are written or exposed.
- Replay must use stored adapter authority and fail closed when required runtime materials are missing or drifted.
- Doctor/readiness must consume adapter-declared checks rather than hard-coded benchmark checks outside the adapter layer.
- Experimental adapters may run with warnings; stable adapters require full conformance, official behavior proof, docs, and adversarial review.

## 8. Edge Cases, Errors, And Recovery

- Incomplete adapter metadata: fail before run with missing-contract diagnostics.
- Unsupported local environment: report adapter readiness blocker and remediation.
- Runtime artifact write failure: fail fast and preserve private diagnostics when possible.
- Partial public/private snapshot pair: report partial provenance and do not claim full replay authority.
- Unknown evaluator output: classify as evaluator parse or evaluator error, not generic adapter crash.
- Missing replay material: block replay before execution.
- Adapter version drift: warn or block according to replay policy.
- Unsupported benchmark capability: hide or disable incompatible generic actions.

## 9. Content And Terminology

- Adapter protocol: the required contract an adapter must implement to make a benchmark callable.
- Protocol-complete: an adapter passes all required conformance checks for its declared capabilities.
- Capability: a generic feature such as data preparation, official runtime, patch evaluation, cleanup verdict override, replay, or public report artifacts.
- Stable adapter: an adapter with conformance, official behavior evidence, docs, and review closure.
- Experimental adapter: an adapter that passes basic protocol checks but lacks full stability proof.

## 10. Acceptance Criteria

- `AC-001`: A new benchmark can be added without changing generic runner, replay, report, doctor, or selector orchestration logic beyond registry metadata.
- `AC-002`: The conformance suite rejects adapters with missing descriptor, unstable task ids, incomplete data lifecycle, unclassified failures, missing runtime snapshots, unsafe public diagnostics, or replay material gaps.
- `AC-003`: Generic run flow can execute any protocol-complete adapter by benchmark id.
- `AC-004`: Generic replay flow can validate adapter version and runtime material authority without benchmark-specific branches.
- `AC-005`: Generic doctor/readiness output can explain adapter blockers from adapter-declared checks.
- `AC-006`: Generic reporting can render adapter-declared public artifacts without benchmark-specific report code.
- `AC-007`: Terminal-Bench and SWE-bench Pro can be represented as instances of the same protocol.
- `AC-008`: A sample third benchmark adapter proves horizontal extension by passing conformance with no generic-layer code changes.
- `AC-009`: Stable promotion requires official behavior preservation proof and `/vs_review/` closure.

## 11. Review Checklist And Sign-off Questions

- Should the first proof adapter be a toy benchmark, a real benchmark, or both?
- Should first release require out-of-tree adapter packaging, or is strict in-repo protocol isolation enough?
- Which generic capabilities are mandatory for all adapters versus optional?
- What is the minimum evidence needed to promote an adapter from experimental to stable?
- Should adapter authors be allowed to declare custom failure codes, or must all failures map to a central taxonomy?

## 12. Clarification Decision Log

| Topic | Decision | Rationale | Source Round |
|---|---|---|---|
| Product target | Adapter protocol isolation | User clarified that adapter cost is expected, but all benchmark-specific adaptation must be contained in the adapter layer. | 2026-06-07 clarification |
| Upper-layer behavior | Benchmark-agnostic after protocol completion | Runner, replay, report, doctor, and selector surfaces should not care which benchmark is behind the protocol. | 2026-06-07 clarification |
| Extension claim | Protocol-complete adapters should be immediately callable | The value is horizontal expansion with predictable integration cost. | 2026-06-07 clarification |
