# Universal Benchmark Adapter Protocol

- Created: 2026-06-08
- Updated: 2026-06-11
- Version: 0.1
- Status: Phase 3 contract foundations started; identity, registry, runtime authority, replay authority, data lifecycle contract, runtime lifecycle contract, readiness schema, and failure mapping schema gates implemented; live artifact/report/doctor/no-branch gates remain planned
- Source plan: `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`
- Source PRD: `prd/2026-06-07-universal-benchmark-adapter-protocol.md`

## 1. Purpose

This protocol defines the boundary between benchmark-specific adapter
implementation and generic HarnessLab surfaces. A benchmark is callable only
when its adapter implements the required protocol records and passes the
registered conformance gates for its declared capabilities.

Generic runner, replay, doctor, report, registry, and selector surfaces may
consume protocol identity, capabilities, schemas, and generic failure classes.
They must not branch on concrete benchmark ids such as `terminal-bench` or
`swe-bench-pro` outside adapter-owned modules, descriptor metadata, fixtures, or
the explicit legacy compatibility shim.

## 2. Identity Model

| Type | Required Shape | Semantics |
|---|---|---|
| `BenchmarkId` | stable lowercase string, for example `terminal-bench` | User-facing benchmark family id. It is not a dispatch branch key in generic code. |
| `AdapterId` | reverse-domain or stable namespaced string, for example `harnesslab.terminal-bench.runtime` | Runtime dispatch key resolved by the protocol registry. |
| `AdapterVersion` | semver-compatible string | Adapter implementation version used for replay drift detection. |
| `AdapterProtocolVersion` | integer or semver-compatible central protocol version | Version of this protocol understood by generic HarnessLab layers. |
| `SelectedMode` | stable string, for example `official-runner`, `patch-evaluator`, `deterministic-sample` | Adapter-selected mode used for capability validation and stable promotion. |
| `Stability` | `experimental`, `stable`, `legacy`, or `conditional-stable-blocked` | Governance status used by doctor, registry, and promotion gates. |

Identity strings must be deterministic, non-empty, and normalized before
registration. Aliases may exist only in registry metadata and must resolve to
one canonical id before runtime binding.

## 3. AdapterProtocolAuthority

`AdapterProtocolAuthority` is the canonical serialized authority for new
protocol paths.

| Field | Required | Meaning |
|---|---|---|
| `benchmark_id` | yes | Stable benchmark family id. |
| `adapter_id` | yes | Stable adapter implementation id and runtime dispatch key. |
| `protocol_version` | yes | Protocol version used by generic layers. |
| `adapter_version` | yes | Adapter implementation version used for drift checks. |
| `selected_mode` | yes | Mode selected by registry/user configuration. |
| `capabilities` | yes | Sorted capability ids declared by the adapter. |
| `stability` | yes | Governance status. |
| `legacy_runner_kind` | compatibility-only | Present only while reading or dual-writing old `ExternalRunnerKind` authority. |

Canonical read/write rules:

| Snapshot Set | Replay Behavior |
|---|---|
| New-only protocol fields in benchmark, task-runtime, preflight, and attempt snapshots | Use protocol authority; ignore legacy enum except diagnostic comparison. |
| Dual-written protocol plus legacy enum fields | Use protocol authority and verify legacy enum maps to the same adapter as consistency evidence. |
| Old-only legacy fields with no protocol authority | Use named legacy shim, emit replay warning, and fail if mapping is unknown. |
| Mixed protocol benchmark/task authority but legacy-only attempt snapshots | Fail closed with `protocol_authority_incomplete`. |
| Mixed old benchmark/task authority but protocol attempt snapshots | Fail closed with `protocol_authority_inconsistent`. |
| Protocol authority mismatch across public/private/runtime/task snapshots | Fail closed with `protocol_authority_mismatch`. |

## 4. TaskRuntimeBinding

`TaskRuntimeBinding` is the only runtime dispatch binding for new protocol
paths.

Required fields:

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

1. CLI/user input selects `benchmark_id`, split, and optional mode/adapter.
2. Registry resolves this to exactly one enabled binding.
3. Runtime dispatch uses `adapter_id`.
4. Generic code may inspect capabilities, stability, and protocol version.
5. Generic code must not switch on concrete benchmark id or adapter id.

Registry conflict checks must reject duplicate benchmark ids, duplicate adapter
ids, multiple defaults for one benchmark/mode, unsupported protocol versions,
capability sets that do not satisfy the selected mode, and unstable adapters
selected as stable.

## 5. Capability Model

Mandatory core capabilities:

| Capability | Required Behavior |
|---|---|
| `descriptor` | Exposes identity, protocol version, adapter version, stability, modes, splits, docs, and owner metadata. |
| `data.lifecycle` | Supports inspect, prepare, list tasks, snapshot task, and create task plan deterministically. |
| `readiness.basic` | Emits adapter-declared readiness checks with user-facing remediation for blockers. |
| `artifacts.basic` | Declares public/private artifacts before runtime writes or report exposure. |
| `failure.mapping` | Maps adapter failures to central failure class/code and adapter phase/subphase. |
| `replay.authority` | Declares required replay materials and adapter version drift policy. |
| `report.metadata` | Declares public score and artifact metadata for generic report rendering. |

Optional capability extensions:

| Capability | Meaning |
|---|---|
| `official.runner` | Adapter preserves an official runner/evaluator command shape. |
| `patch.evaluator` | Adapter consumes patch/diff predictions. |
| `cleanup.verdict_override` | Cleanup can affect final task verdict. |
| `host.agent_execution` | Adapter runs an agent on the host or bridge. |
| `docker.orchestration` | Adapter owns Docker setup/run/cleanup phases. |
| `sandbox.runner` | Adapter uses a sandboxed runner contract. |
| `custom.report_panel` | Adapter adds public report sections from declared public artifacts. |
| `remote.data_auth` | Adapter requires remote data authentication checks. |

Optional capabilities must have explicit readiness checks and conformance
fixtures before stable promotion.

## 6. Selected-mode Compatibility

The registry must validate selected mode against capabilities before binding.

| Selected Mode | Registry-required Capabilities | Registry-rejected Combinations | Deferred Conformance Gate |
|---|---|---|---|
| `deterministic-sample` | core capabilities only | optional execution/report extension capabilities on a deterministic-only binding | stable promotion evidence in `ADAPT-PROTOCOL-012` |
| `official-runner` | core capabilities plus `official.runner` and at least one execution capability: `host.agent_execution`, `docker.orchestration`, or `sandbox.runner` | no execution capability; stable without promotion evidence | official proof and artifact archive in `ADAPT-PROTOCOL-010` / `ADAPT-PROTOCOL-012` |
| `patch-evaluator` | core capabilities plus `patch.evaluator` and `host.agent_execution` | missing patch evaluator capability; missing host execution capability | patch artifact/redaction declarations in `ADAPT-PROTOCOL-005`; failure mapping in `ADAPT-PROTOCOL-006` |
| `cleanup-sensitive-runner` | core capabilities plus `cleanup.verdict_override` and one execution capability | cleanup override without an execution capability | cleanup report declaration in `ADAPT-PROTOCOL-005`; cleanup failure mapping in `ADAPT-PROTOCOL-006` |
| `custom-report` | core capabilities plus `custom.report_panel` | custom report mode without custom report capability | public report artifact metadata in `ADAPT-PROTOCOL-007` |

The registry-level rules above are the normative input for
`ADAPT-PROTOCOL-002`: duplicate adapter/default rejection, protocol version
rejection, capability/mode compatibility, legacy shim consistency, and stable
promotion evidence presence. Artifact schemas, redaction policy, detailed
failure mapping, readiness probe content, and report metadata are normative
protocol requirements, but they are enforced by later conformance gates listed
in the deferred column.

Adding a new mode requires a row here, registry conflict fixtures for the
registry-level constraints, and separate readiness/artifact/report coverage in
the relevant later gate.

## 7. Optional Capability Contracts

Optional capabilities are not free-form labels. Each capability below defines
required operations, artifacts, readiness probes, failure subphases, and report
metadata so current Terminal-Bench and SWE-bench Pro behavior can migrate
without generic-layer benchmark branches.

| Capability | Required Operations | Required Artifact Declarations | Required Readiness Probes | Required Failure Subphases | Report Metadata |
|---|---|---|---|---|---|
| `official.runner` | prepare official command, execute official runner/evaluator, parse official result, preserve official stderr/stdout refs when public-safe | `run.json`, `results.json`, `events.jsonl`, official stdout/stderr or evaluator result when produced | official tool/version present, dataset compatible, selected mode allowed | `official_setup`, `official_execute`, `official_parse`, `official_timeout` | official verdict, official warning labels, tool/version when public-safe |
| `patch.evaluator` | capture diff, write prediction, invoke evaluator, parse evaluator output, distinguish empty patch from diff capture failure | `patch.diff`, `prediction.jsonl`, evaluator stdout/stderr, evaluator result JSON when produced | workspace writable, repo checkout available, evaluator tool available | `metadata_extract`, `workspace_prepare`, `diff_capture`, `patch_empty`, `evaluator_execute`, `evaluator_parse` | score field, patch artifact ref, evaluator warning labels |
| `cleanup.verdict_override` | run pre/post cleanup, classify cleanup outcome, decide final verdict override or warning | `cleanup-report.json`, cleanup public/private diagnostics when produced | cleanup tool available, cleanup scope known, stale resources discoverable | `cleanup_discovery`, `cleanup_execute`, `cleanup_parse`, `cleanup_timeout` | cleanup warning/override summary |
| `host.agent_execution` | materialize command, enforce run-as/auth policy, execute agent, capture stdout/stderr, map agent timeout/parse failures | agent stdout/stderr, materialized profile snapshot, command snapshot | run-as supported by host, inherited auth policy valid, required commands available | `agent_materialize`, `agent_precheck`, `agent_execute`, `agent_timeout`, `agent_output_parse` | agent runtime summary and safe command text |
| `docker.orchestration` | prepare Docker platform/env, run setup/build/test phases, watch activity/no-progress, cleanup containers/processes | Docker setup/build/run logs, activity events, run-health when produced | Docker available, platform compatible, network/sandbox policy allowed | `docker_setup`, `docker_build`, `docker_execute`, `docker_no_progress`, `docker_cleanup` | Docker/platform warning labels and health summary |
| `sandbox.runner` | create sandbox, copy inputs, execute command, copy outputs, destroy sandbox | sandbox lifecycle logs, copied output manifest | sandbox backend available, mounts/network allowed | `sandbox_create`, `sandbox_copy_in`, `sandbox_execute`, `sandbox_copy_out`, `sandbox_destroy` | sandbox health summary |
| `run_as.readiness` | resolve requested run user, compare against host capability, block before task execution if incompatible | doctor/readiness JSON refs, run precheck event when produced | requested run-as supported, profile policy valid | `run_as_resolve`, `run_as_precheck` | public remediation text |
| `custom.report_panel` | declare public section id, title, source artifact refs, and rendering shape | report public artifacts for every section | public artifact declarations exist and are redaction-safe | `report_metadata`, `report_render` | section title and public artifact refs |

Generic code may use these fields to sort, render, validate, and block. It must
not implement special behavior for a concrete benchmark id.

## 8. Existing Adapter Behavior Mapping

This table maps Phase 0 coupling to protocol-owned capability contracts. It is
the migration checklist for `ADAPT-PROTOCOL-010`.

| Existing Behavior / Selector Surface | Protocol Capability | Required Protocol Fields / Artifacts | Failure / Readiness Mapping | Future Proof |
|---|---|---|---|---|
| Terminal-Bench runtime, timeout, env, result, cleanup, and snapshots (`ADAPT-RUNTIME-*`, `TB-*`, `INT-021..046`) | `official.runner`, `docker.orchestration`, `cleanup.verdict_override`, `artifacts.basic`, `replay.authority` | `AdapterProtocolAuthority`, `TaskRuntimeBinding`, runtime public/private snapshots, `run-health.json`, `cleanup-report.json`, declared Docker/activity artifacts | `official_timeout`, `docker_no_progress`, `cleanup_execute`, `cleanup_timeout`, readiness for Docker/platform/network | `ADAPT-PROTOCOL-004`, `ADAPT-PROTOCOL-005`, `ADAPT-PROTOCOL-006`, `ADAPT-PROTOCOL-010` |
| Terminal-Bench Python bridge and process cleanup (`PY-TB-001`, `AGT-REG-005`, `AGT-REG-012`) | `host.agent_execution`, `run_as.readiness`, `official.runner` | materialized agent profile, setup command hash/logs, agent stdout/stderr, run-as readiness probes | `agent_materialize`, `agent_precheck`, `run_as_precheck`, `agent_execute` | `ADAPT-PROTOCOL-004`, `ADAPT-PROTOCOL-007`, `ADAPT-PROTOCOL-010` |
| Terminal-Bench infra log scanning (`TB-007`, `TB-011`) | `docker.orchestration`, `failure.mapping` | declared log artifacts and parser scope metadata | `docker_setup`, `docker_execute`; parser must ignore verifier-only logs unless declared | `ADAPT-PROTOCOL-004`, `ADAPT-PROTOCOL-010` |
| SWE-bench Pro metadata/workspace/agent/evaluator/runtime snapshots (`INT-011`, `SWEPRO-*`) | `patch.evaluator`, `host.agent_execution`, `artifacts.basic`, `replay.authority` | `patch.diff`, `prediction.jsonl`, `prediction.eval.json`, `swe-bench-pro/eval/eval_results.json`, public/private runtime snapshots, replay materials | `metadata_extract`, `workspace_prepare`, `diff_capture`, `patch_empty`, `evaluator_execute`, `evaluator_parse` | `ADAPT-PROTOCOL-004`, `ADAPT-PROTOCOL-005`, `ADAPT-PROTOCOL-006`, `ADAPT-PROTOCOL-010` |
| Runtime compatibility branches in `runtime_compatibility.rs` | `readiness.basic`, `host.agent_execution`, `docker.orchestration`, `run_as.readiness` | readiness probes with capability, phase, severity, remediation, required tools | `run_as_precheck`, `agent_precheck`, `docker_setup` | `ADAPT-PROTOCOL-007`, `ADAPT-PROTOCOL-008` |
| Doctor run-as behavior in `doctor_run_as.rs` | `run_as.readiness` | public blocker/remediation probe and private detail ref when needed | `run_as_resolve`, `run_as_precheck` | `ADAPT-PROTOCOL-007`, `ADAPT-PROTOCOL-010` |
| Current registry descriptor and adapter string dispatch | `descriptor`, `TaskRuntimeBinding`, registry metadata | benchmark id, adapter id, selected mode, default binding, stability, capabilities | duplicate/default/mode mismatch registry failures | `ADAPT-PROTOCOL-001`, `ADAPT-PROTOCOL-002`, `ADAPT-PROTOCOL-008` |
| Report and public artifact rendering (`INT-003`, `INT-009`, `INT-017`, `INT-020`, `INT-034`) | `report.metadata`, `artifacts.basic`, `custom.report_panel` | `score_fields`, `public_artifacts`, `summary_fields`, declared detail sections | `report_metadata`, `report_render`, malformed public event log failure | `ADAPT-PROTOCOL-005`, `ADAPT-PROTOCOL-007` |
| Public/private redaction (`SEC-001`, `ADAPT-RUNTIME-003`, replay/resume redaction selectors) | `artifacts.basic`, `replay.authority` | visibility, redaction policy, public/private fingerprints, private detail refs | unsafe public artifact and redaction drift failures | `ADAPT-PROTOCOL-005`, `ADAPT-PROTOCOL-006` |
| Official preservation scripts and archives | `official.runner`, stable promotion evidence | official tool name/version, official command, environment, artifact archive, result comparison | `official_execute`, `official_parse`, conditional stable blockers | `ADAPT-PROTOCOL-010`, `ADAPT-PROTOCOL-012` |

## 9. Versioned Schemas

### 9.1 ReadinessProbe

| Field | Required | Allowed Values / Notes |
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
| `required_tools` | optional | Tool names and versions. |
| `privacy_scope` | yes | `public` or `private`. |

### 9.2 ArtifactDeclaration

| Field | Required | Allowed Values / Notes |
|---|---|---|
| `artifact_id` | yes | Stable id. |
| `path` | yes | Relative attempt path. |
| `artifact_type` | yes | `runtime_snapshot`, `event_log`, `result`, `report_public`, `diagnostic_public`, `diagnostic_private`, or `adapter_custom`. |
| `visibility` | yes | `public` or `private`. |
| `producer_phase` | yes | Adapter phase/subphase. |
| `required_for_replay` | yes | Boolean. |
| `redaction_policy` | yes | `none`, `scan`, `structured`, or `private_only`. |
| `schema_version` | yes | Artifact schema version. |

### 9.3 FailureMapping

| Field | Required | Allowed Values / Notes |
|---|---|---|
| `failure_class` | yes | Central class. |
| `failure_code` | yes | Central public code. |
| `adapter_phase` | yes | Protocol phase. |
| `adapter_subphase` | yes | Adapter-local stable subphase. |
| `adapter_detail_code` | optional | Adapter-local detail mapped to central code. |
| `public_message` | yes | Redacted user-facing message. |
| `private_diagnostics` | optional | Private diagnostics ref. |
| `health_impact` | yes | Central health impact mapping. |

### 9.4 ReportMetadata

| Field | Required | Notes |
|---|---|---|
| `score_fields` | yes | Public score fields and units. |
| `public_artifacts` | yes | Artifact ids safe for report rendering. |
| `summary_fields` | yes | Generic summary labels. |
| `warnings` | optional | Public warning labels. |
| `detail_sections` | optional | Public sections backed only by declared public artifact refs. |

## 10. Lifecycle Contract

Data lifecycle:

- `inspect` must not mutate local cache.
- `prepare` must be idempotent and reject partial/corrupt data.
- `list_tasks` must return stable task ids and source refs.
- `snapshot_task` must capture replay-sufficient task identity.
- `create_task_plan` must be stable from prepared data and task descriptor.

Runtime lifecycle:

- `preflight` returns readiness checks and blocks required missing capabilities.
- `execute` writes only declared artifacts.
- `cleanup` is adapter-owned when cleanup can affect verdict.
- `failure_mapping` must classify every adapter-visible failure before report.
- `score_extraction` must return public score fields declared in report
  metadata.

Replay lifecycle:

- Replay reads `AdapterProtocolAuthority`, task runtime binding, public/private
  runtime snapshots, anchors, and declared replay materials.
- Missing, mixed, or drifted authority fails closed with a protocol failure
  code.
- Live replanning is forbidden unless the adapter declares an explicit replay
  compatibility policy and the stored authority permits it.

## 11. Public And Private Boundaries

Adapters must declare artifacts before producing them. Public artifacts may be
rendered in reports only when their declaration marks `visibility = public` and
their redaction policy is satisfied. Private diagnostics must remain private
and may be referenced by `private_detail_ref` rather than copied into public
output.

Public/private runtime snapshots must agree on authority, task id, attempt id,
adapter version, runtime policy, public artifacts, and fingerprints. Public
snapshots must not expose private dataset/source paths or redaction basis.

## 12. Doctor And Readiness

Doctor must consume adapter-declared readiness probes. Generic doctor code may
sort, aggregate, redact, and display probes by severity and phase, but it must
not know concrete benchmark readiness rules.

Blocked and fatal checks require remediation. Environment-specific probes may
be private, but the public output must still contain a safe explanation and
next action.

## 13. Reporting

Reports render adapter-declared public artifacts, score fields, warnings, and
detail sections. Generic report code must not branch on concrete benchmark ids.
Adapter custom sections are allowed only through declared public artifacts and
public metadata.

## 14. Observability

Every protocol runtime dispatch and replay validation path must emit or expose
the following fields in events, snapshots, or diagnostics:

| Field | Required In | Purpose |
|---|---|---|
| `benchmark_id` | dispatch, snapshots, replay blockers | User-facing family identity. |
| `adapter_id` | dispatch, snapshots, replay blockers | Runtime dispatch authority. |
| `protocol_version` | dispatch, snapshots | Protocol compatibility. |
| `adapter_version` | snapshots, replay blockers | Drift detection. |
| `selected_mode` | dispatch, readiness, report | Mode/capability validation. |
| `capability` | readiness, conformance failure, report warnings | Generic gating and diagnostics. |
| `stability` | doctor, registry, report | Governance state. |
| `legacy_shim_used` | replay and migration diagnostics | Audit legacy authority use. |
| `failure_class` / `failure_code` | result, report, health | Central failure classification. |
| `adapter_phase` / `adapter_subphase` | events and private diagnostics | Adapter-owned failure localization. |

Generic layers may add correlation ids and timing data. They must not add
benchmark-specific event fields outside adapter-declared metadata.

## 15. Stability And Promotion

Experimental adapters may run with warnings when core conformance passes.
Stable adapters additionally require:

- active protocol conformance selectors
- stable promotion evidence record
- docs for adapter users/authors
- `/vs_review/` closure report
- no environment-gated proof gaps

If official proof is environment-gated, status must remain `experimental` or
`conditional-stable-blocked`, not `stable`.

Stable promotion evidence fields:

| Field | Required | Notes |
|---|---|---|
| `adapter_id` | yes | Adapter under review. |
| `adapter_version` | yes | Version being promoted. |
| `protocol_version` | yes | Protocol version. |
| `conformance_command` | yes | Exact command and output path. |
| `official_tool_name` | conditional | Required when official runner/evaluator exists. |
| `official_tool_version` | conditional | Required when official tool exists. |
| `official_command` | conditional | Exact preservation command. |
| `environment` | yes | OS/tools/Docker/network assumptions. |
| `artifact_archive` | yes | Archived proof artifacts. |
| `result_comparison` | conditional | Official vs HarnessLab comparison. |
| `known_conditions` | optional | Environment-gated caveats. |
| `review_report` | yes | `/vs_review/` path. |
| `status` | yes | `experimental`, `stable`, or `conditional-stable-blocked`. |

## 16. Acceptance Matrix

| PRD AC | Protocol Proof Selector | Phase | Proof Standard |
|---|---|---:|---|
| `AC-001` | `ADAPT-PROTOCOL-011` | 7 | Third adapter adds no generic runner/replay/report/doctor/selector behavior changes. |
| `AC-002` | `ADAPT-PROTOCOL-003`, `ADAPT-PROTOCOL-004`, `ADAPT-PROTOCOL-005`, `ADAPT-PROTOCOL-007` | 3-5 | Conformance rejects missing lifecycle, unclassified failure, unsafe public artifact, and missing readiness contract. |
| `AC-003` | `ADAPT-PROTOCOL-002`, `ADAPT-PROTOCOL-004`, `ADAPT-PROTOCOL-006`, `ADAPT-PROTOCOL-011` | 2-7 | Generic run resolves binding by ids; runtime lifecycle contract is declared in Phase 3 and live protocol execution proof remains required before final closure. |
| `AC-004` | `ADAPT-PROTOCOL-006` | 4 | Replay validates adapter authority and material drift without concrete benchmark branch. |
| `AC-005` | `ADAPT-PROTOCOL-007` | 4 | Doctor output is generated from adapter readiness probes. |
| `AC-006` | `ADAPT-PROTOCOL-005`, `ADAPT-PROTOCOL-007` | 3-4 | Report renders only declared public artifacts and report metadata. |
| `AC-007` | `ADAPT-PROTOCOL-010` | 6 | Terminal-Bench and SWE-bench Pro pass protocol gates with frozen behavior selectors green. |
| `AC-008` | `ADAPT-PROTOCOL-011` | 7 | Sample third adapter proves horizontal extension. |
| `AC-009` | `ADAPT-PROTOCOL-012` | 8 | Stable promotion evidence archive and review closure validate stable status. |

## 17. Protocol Selector Plan

| Selector | Status At Phase 1 | Activation Phase | Required Proof |
|---|---|---:|---|
| `ADAPT-PROTOCOL-001` | active | 2 | Descriptor, identity, and protocol authority schema validation. |
| `ADAPT-PROTOCOL-002` | active | 2 | Registry conflict and binding resolution validation. |
| `ADAPT-PROTOCOL-003` | active | 3 | Data lifecycle protocol contract foundation. |
| `ADAPT-PROTOCOL-004` | active | 3 | Runtime lifecycle, readiness, and failure taxonomy contract foundation. |
| `ADAPT-PROTOCOL-005` | planned | 3 | Artifact declaration, public/private, and redaction conformance. |
| `ADAPT-PROTOCOL-006` | planned | 4 | Replay authority old/new/mixed fixture conformance. |
| `ADAPT-PROTOCOL-007` | planned | 4 | Generic doctor/readiness/report metadata conformance. |
| `ADAPT-PROTOCOL-008` | planned | before Phase 4 exit | Static no-branch guard with bypass fixtures. |
| `ADAPT-PROTOCOL-009` | planned | 5 | Scaffold golden path and generated adapter conformance. |
| `ADAPT-PROTOCOL-010` | planned | 6 | Existing adapter migration preservation manifest. |
| `ADAPT-PROTOCOL-011` | planned | 7 | Third-adapter horizontal extension proof and forbidden-diff guard. |
| `ADAPT-PROTOCOL-012` | planned | 8 | Stable promotion evidence archive validation. |

## 18. Decision Log

| Decision | Status | Rationale | Revisit Trigger |
|---|---|---|---|
| In-repo protocol isolation first | accepted for this implementation plan | It proves genericity and conformance without adding marketplace/plugin distribution risk. | User requires out-of-tree third-party adapter packaging in the first release. |
| Out-of-tree adapter packaging deferred | accepted for this implementation plan | Packaging, trust, and untrusted execution are separate product concerns. | Stable adapters need third-party distribution before Phase 8. |
| Concrete benchmark branches forbidden in generic layers | accepted | Horizontal extension only works when adapter-specific costs stay inside adapter-owned files. | Static guard blocks legitimate metadata; then allowlist metadata, not behavior. |
| Official behavior proof standardized, not genericized | accepted | Official proof can remain adapter-specific, but the evidence schema must be standard. | Stable promotion cannot compare official and HarnessLab outputs. |
