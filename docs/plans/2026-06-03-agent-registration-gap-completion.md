# Agent Registration Gap Completion Plan

> **For implementation agents:** this is a follow-up implementation plan for the gaps found after the first agent registration registry pass. Do not treat documentation-only presence as completion. Each slice must prove runtime behavior, doctor behavior, schema visibility, and traceability registration.

- Date: 2026-06-03
- Status: completed and verified on 2026-06-04
- Scope: agent registration capability policies, schema completeness, version probing, required command checks, auth/run-user runtime behavior, docs, tests, adversarial review
- Source: engineering audit of the current `d1408e6`/`20a7972` agent registration implementation

## 1. Goal

Finish the registration system so every user-facing profile parameter is either:

1. Fully materialized into runtime behavior.
2. Explicitly rejected before benchmark execution with precise field-level diagnostics.
3. Clearly marked as legacy/reserved and excluded from the normal user path.

The end state must be simple for users and maintainable for engineers:

- Users can reason from profile TOML to actual run behavior.
- `doctor --json` catches missing, unsupported, misspelled, or unenforceable declarations before run.
- `agent schema --json` exposes every supported field and known adapter label a helper agent needs.
- Runtime artifacts show resolved effective configuration, not just raw declarations.
- Unsupported behavior is not silently ignored.

## 2. Product Semantics To Lock

### 2.1 Capability Policy Semantics

For each capability domain (`skills`, `tools`, `hooks`), define a catalog for the selected agent kind:

```text
available(kind, domain)       = all known capabilities this kind can reference
default_enabled(kind, domain) = capabilities enabled by default
policy                         = { inherit, allow, deny, include_paths }
```

The effective set is:

```text
if allow is non-empty:
  target = allow
else if inherit = true:
  target = default_enabled(kind, domain)
else:
  target = {}

effective = target - deny
```

Validation rules:

- Every `allow` item must exist in `available`.
- Every `deny` item must exist in `available`; typoed deny entries are errors, not no-op warnings.
- `allow` and `deny` must be disjoint.
- `include_paths` is only meaningful for `skills`; it is not accepted for `tools` or `hooks` unless a future domain materializer explicitly supports it.
- If the selected `kind` has no enforceable materializer for a domain, any non-default policy for that domain is a blocking error.
- A policy is considered materialized only when HarnessLab changes actual runtime inputs, generated config, command arguments, or adapter environment in a way the target agent can obey.

This avoids the ambiguous model where `allow` is sometimes a filter and sometimes an enable list. `allow` is an explicit desired set. `inherit` only decides what happens when `allow` is empty.

### 2.2 Version Probe Semantics

`version_command` is a real operational check, not a decorative field.

- `doctor` executes it when present and reports stdout/stderr/exit status in redacted form.
- New runs store a redacted version snapshot.
- Replay compares the current probe result with the source run snapshot and reports a warning by default.
- Hard blocking replay on version mismatch is out of scope unless a future policy field is added.

### 2.3 Auth Semantics

`auth.inherit` applies to both Docker and host execution.

- `inherit = true`: pass only declared `inherit_env` values plus task-specified env vars.
- `inherit = false`: do not pass profile-declared env/path inheritance.
- Host execution must not accidentally inherit the whole parent process environment when the profile says not to inherit.
- A small execution baseline such as `PATH`, `HOME`, `TMPDIR`, and locale can be kept explicitly if required to launch commands; it must be documented as process baseline, not auth inheritance.

### 2.4 Run User Semantics

`setup.run_as` must be honest about where it is enforceable.

- Docker sandbox: `root`, `harnesslab`, and `current` may be materialized.
- Host execution: user switching is unsupported unless an explicit host user-switch mechanism is implemented.
- If a host task would require `run_as != current`, run precheck must block or downgrade only if the profile explicitly accepts current-user fallback. No silent fallback.

## 3. Known Gaps To Close

| Gap | Current State | Required End State |
| --- | --- | --- |
| Capability policies do not execute | `skills/tools/hooks` parse, validate names, summarize, then block all non-default policies. | Resolver computes effective capability sets and materializers either apply them or produce precise blocking errors. |
| Missing `allow` item is not checked | No catalog lookup exists. | `doctor` reports field-level errors such as `tools.allow[0] = "read_file" is unknown for kind codex`. |
| `version_command` does not run | Field appears in config/templates but no doctor/run/replay probe exists. | Doctor probes it, runs snapshot it, replay compares it. |
| Schema is incomplete | `agent schema --json` omits `version_command`, usage subfields, and known adapter labels. | Schema exposes all first-class fields and known labels with descriptions, values, examples, and required/default status. |
| `setup.required_commands` is not checked by doctor | Only first word of `command` is checked. | Doctor checks each required command and explains whether it is present, provided by builtin setup, custom setup, or unresolved until sandbox. |
| Host env inheritance is not isolated | Host process inherits parent environment through `Command` by default. | Host executor receives explicit env policy so `auth.inherit=false` and `inherit_env` are meaningful outside Docker too. |
| `setup.run_as` is not enforced for host tasks | Docker command wrapping honors it; host tasks ignore it. | Host-incompatible `run_as` is blocked or explicitly implemented; no silent mismatch. |
| Materialized snapshot only stores summaries for capabilities | Effective capability details are string summaries. | Snapshot stores structured resolved capability objects plus summaries for report display. |
| Known labels are only generic schema entries | Schema has `labels` as key/value; docs mention Terminal-Bench labels separately. | Schema includes known labels under `labels.<name>` while still allowing custom labels. |

## 4. Architecture

### 4.1 Core Types

Add small, explicit core types instead of ad hoc strings:

```rust
enum CapabilityDomain {
    Skills,
    Tools,
    Hooks,
}

struct CapabilityRef {
    domain: CapabilityDomain,
    name: String,
}

struct CapabilityCatalog {
    kind: AgentKind,
    domain: CapabilityDomain,
    available: BTreeSet<String>,
    default_enabled: BTreeSet<String>,
    enforcement: CapabilityEnforcement,
}

enum CapabilityEnforcement {
    Enforced,
    Unsupported { reason: String },
}

struct ResolvedCapabilityPolicy {
    domain: CapabilityDomain,
    inherit: bool,
    allow: Vec<String>,
    deny: Vec<String>,
    include_paths: Vec<String>,
    effective: Vec<String>,
    enforcement: CapabilityEnforcement,
    diagnostics: Vec<ProfileValidationError>,
}
```

Keep this in `harnesslab-core` when it is pure schema/validation logic. Keep actual per-agent command/config generation in `harnesslab-cli/src/agent_registry/` so core does not learn CLI filesystem behavior.

### 4.2 Materializer Boundary

Split materialization into three small responsibilities:

1. `CapabilityResolver`: pure calculation and validation.
2. `AgentKindCapabilityCatalog`: per-kind available/default capability metadata.
3. `AgentKindCapabilityMaterializer`: turns resolved policy into concrete runtime mutations.

The first implementation can use functions and enums rather than a heavy trait hierarchy, but the file boundary should leave room for future per-agent materializers:

```text
crates/harnesslab-cli/src/agent_registry/
  materializer.rs
  capabilities.rs
  capability_catalog.rs
  capability_runtime.rs
```

Do not add special-case branches inside Terminal-Bench or SWE-bench Pro for raw profile policy. Benchmark adapters consume already materialized runtime configuration.

### 4.3 Runtime Snapshot Shape

Extend `agent-runtime.materialized.json` from string summaries to structured data:

```json
{
  "setup_script": "...",
  "setup_summary": "...",
  "run_as": "harnesslab",
  "capabilities": {
    "skills": {
      "inherit": true,
      "allow": [],
      "deny": [],
      "include_paths": [],
      "effective": ["..."],
      "enforcement": "enforced"
    },
    "tools": {
      "inherit": false,
      "allow": ["bash"],
      "deny": [],
      "effective": ["bash"],
      "enforcement": "enforced"
    },
    "hooks": {
      "inherit": true,
      "allow": [],
      "deny": [],
      "effective": []
    }
  },
  "version": {
    "command": "claude --version",
    "status": "ok",
    "stdout": "Claude Code ...",
    "stderr": "",
    "exit_code": 0
  },
  "warnings": []
}
```

Retain old summary fields for backward-compatible reports until report rendering and replay are migrated.

### 4.4 Schema Source Of Truth

`agent schema --json` should be generated from a single field reference list that includes:

- serde fields in `AgentProfile`
- nested fields in `AuthConfig`, `SetupConfig`, `CapabilityPolicy`, `UsageConfig`
- known adapter labels
- legacy/reserved labels

Add a schema completeness test that fails when a first-class config field exists in structs/docs but is missing from schema output.

## 5. Implementation Slices

### Slice A: Schema Completeness

Files:

- `crates/harnesslab-core/src/agent_profile_reference.rs`
- `crates/harnesslab-cli/tests/agent_registry_contract.rs`
- `docs/agent-registration-guide.md`
- `docs/agent-profile-reference.md`

Tasks:

1. Add missing fields to `agent_profile_field_reference()`:
   - `version_command`
   - `usage.source`
   - `usage.input_tokens_key`
   - `usage.output_tokens_key`
   - `usage.total_tokens_key`
   - `usage.cost_usd_key`
   - `labels.model`
   - `labels.terminal_bench_agent`
   - `labels.terminal_bench_agent_import_path`
   - `labels.terminal_bench_agent_pythonpath`
   - `labels.terminal_bench_model`
   - `labels.sandbox_setup_command`
2. Add `default_value` and `status` metadata if needed so users can distinguish required, optional, legacy, and reserved fields.
3. Add a schema contract test that asserts every known profile parameter in docs appears in `agent schema --json`.
4. Update docs only after schema output is current.

Acceptance:

- `harnesslab agent schema --json | jq -r '.fields[].path'` includes every field above.
- `AGT-REG-003` covers schema completeness, not just a few paths.

### Slice B: Capability Policy Resolver

Files:

- `crates/harnesslab-core/src/agent_profile.rs`
- `crates/harnesslab-core/src/config.rs`
- new `crates/harnesslab-core/src/capability_policy.rs` if line count pressure appears
- `crates/harnesslab-core/src/config_tests.rs`

Tasks:

1. Implement pure resolver for `skills/tools/hooks` policies.
2. Encode the semantic rules from section 2.1.
3. Preserve existing conflict checks.
4. Add exact field paths for diagnostics:
   - `tools.allow[0]`
   - `skills.include_paths[1]`
   - `hooks.deny[0]`
5. Add unit tests:
   - `inherit=true, allow=[], deny=[]` uses defaults.
   - `inherit=true, allow=["bash"], deny=[]` resolves to `bash`.
   - `inherit=false, allow=["bash"], deny=[]` resolves to `bash`.
   - `inherit=false, allow=[], deny=[]` resolves to empty set.
   - `deny` removes entries from either default or explicit allow.
   - unknown allow item errors.
   - unknown deny item errors.
   - `tools.include_paths` and `hooks.include_paths` error unless supported.

Acceptance:

- Resolver tests prove the policy algebra independently from CLI materializers.
- Diagnostics include field paths and suggested fixes.

### Slice C: Capability Catalogs And Materializers

Files:

- `crates/harnesslab-cli/src/agent_registry/capability_catalog.rs`
- `crates/harnesslab-cli/src/agent_registry/capability_runtime.rs`
- `crates/harnesslab-cli/src/agent_registry/materializer.rs`
- `crates/harnesslab-cli/src/runner/sandbox.rs`
- `crates/harnesslab-cli/src/runner/external/terminal_bench_env.rs`

Tasks:

1. Add per-kind catalogs with explicit support state for each domain.
2. Start with minimal enforceable catalogs:
   - `fake`: full support for contract tests.
   - `custom`: unsupported unless explicit runtime env contract is added.
   - `codex`, `claude-code`, `opencode`, `pi-coding-agent`: only mark a domain `Enforced` after a black-box proof shows the generated config/CLI flags change actual behavior.
3. Materialize resolved effective capabilities into structured runtime data.
4. For unsupported domains, block non-default policies with precise field and reason.
5. For enforced domains, write runtime artifacts or env/config that the agent actually consumes.
6. Ensure Terminal-Bench bridge receives structured capability information, not only summary strings.

Acceptance:

- `doctor` distinguishes unknown capability, unsupported domain, and valid enforced policy.
- `run` does not start when a policy cannot be enforced.
- Enforced fake-kind policy changes observable runtime output in a contract test.
- No benchmark adapter directly reinterprets raw policy fields.

### Slice D: Doctor Diagnostics For Capabilities

Files:

- `crates/harnesslab-cli/src/doctor.rs`
- `crates/harnesslab-cli/tests/doctor_contract.rs`

Tasks:

1. Add one check per domain:
   - `agent.<name>.skills.policy`
   - `agent.<name>.tools.policy`
   - `agent.<name>.hooks.policy`
2. Include:
   - `available`
   - `default_enabled`
   - `effective`
   - `unsupported_reason`
   - `field_path`
   - `suggested_fix`
3. Make unknown allow/deny values `error`.
4. Make unsupported materializer `error` when policy is non-default.
5. Keep default unsupported domains `ok` only if no non-default policy is declared.

Acceptance:

- User can tell whether `"bash"` is unknown, unsupported, denied, or effectively enabled.
- Existing broad `capabilities.materialization` check can remain, but must not be the only diagnostic.

### Slice E: Version Command Probe

Files:

- `crates/harnesslab-cli/src/agent_registry/version_probe.rs`
- `crates/harnesslab-cli/src/doctor.rs`
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/tests/doctor_contract.rs`
- `crates/harnesslab-cli/tests/replay_contract.rs`

Tasks:

1. Add a bounded version probe runner:
   - timeout 5 seconds by default
   - stdout/stderr tail limit
   - redaction through known secret values
2. Doctor executes `version_command` when present.
3. New run stores `agent-version.snapshot.json` or embeds version in materialized runtime.
4. Replay compares source version snapshot to current probe.
5. Mismatch emits replay readiness warning and event.
6. Probe failure is warning in doctor unless the command is malformed; do not block normal use by default.

Acceptance:

- A nonexistent `version_command` produces a doctor warning mentioning the field.
- A successful command is stored in run artifacts.
- Replay mismatch is visible in JSON/event/report path.

### Slice F: Required Commands Doctor Check

Files:

- `crates/harnesslab-cli/src/doctor.rs`
- `crates/harnesslab-cli/src/agent_registry/materializer.rs`
- `crates/harnesslab-cli/tests/doctor_contract.rs`

Tasks:

1. Add `agent.<name>.setup.required_commands` doctor check.
2. For each required command, report:
   - command name
   - valid command-name syntax
   - host availability
   - whether builtin setup is expected to install/provide it
   - whether custom setup makes it unknown until sandbox
3. Treat invalid command syntax as error.
4. Treat missing command with no setup path as error.
5. Treat command satisfied by builtin setup as ok or info, not a false host error.

Acceptance:

- `required_commands = ["definitely-missing"]` with `preset = "none"` is an error.
- `required_commands = ["claude"]` with `kind = "claude-code"` and builtin setup explains that builtin setup can provide it.
- `required_commands = ["foo | bar"]` is an exact field error.

### Slice G: Host Auth Isolation

Files:

- `crates/harnesslab-infra/src/process.rs`
- `crates/harnesslab-cli/src/runner/sandbox.rs`
- `crates/harnesslab-cli/tests/external_smoke_contract.rs` or a new host execution contract

Tasks:

1. Extend `ExecSpec` with explicit environment policy:
   - `env_clear: bool`
   - `env_vars: BTreeMap<String, String>`
   - or a small `ResolvedEnvironment` struct.
2. Build resolved env from task env plus profile auth policy.
3. Host execution calls `Command::env_clear()` when profile policy requires isolation.
4. Always preserve minimal launch baseline explicitly.
5. Docker execution keeps using `docker -e`, but receives the same resolved env source.
6. Redact env values in snapshots and logs.

Acceptance:

- Host task with ambient `SECRET=...` and `auth.inherit=false` cannot read `SECRET`.
- Host task with `auth.inherit=true` and `inherit_env=["SECRET"]` can read it.
- Docker env behavior remains unchanged.

### Slice H: Run User Enforcement

Files:

- `crates/harnesslab-cli/src/runner/sandbox.rs`
- `crates/harnesslab-cli/src/agent_registry/materializer.rs`
- `crates/harnesslab-cli/tests/agent_registry_contract.rs`

Tasks:

1. Define where `RunAs` is enforceable:
   - Docker: enforceable.
   - Terminal-Bench bridge: enforceable through setup/bridge only if command runs inside sandbox path.
   - Host: unsupported except `current`.
2. Add run precheck:
   - host task plus `run_as = "harnesslab"` or `"root"` blocks before execution.
3. Add doctor warning if a profile uses non-current `run_as`, explaining it requires sandboxed tasks to be meaningful.
4. Do not silently ignore `run_as`.

Acceptance:

- Host benchmark profile with `run_as = "harnesslab"` fails before task execution.
- Docker task still wraps command with the requested user behavior.

### Slice I: Runtime Snapshot And Report Upgrade

Files:

- `crates/harnesslab-cli/src/agent_registry/materializer.rs`
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-report/src/lib.rs`
- `crates/harnesslab-cli/tests/run_output_contract.rs`

Tasks:

1. Add structured capability snapshots.
2. Keep text summaries for report compatibility.
3. Add version probe snapshot.
4. Make report show:
   - effective skills/tools/hooks
   - unsupported policy reason if run was blocked before report generation where applicable
   - version probe status when available
5. Ensure public artifacts redact known secret values.

Acceptance:

- `agent-runtime.materialized.json` contains structured effective policy data.
- `report.html` shows effective capability sets, not only raw declarations.
- Secret scan still passes.

### Slice J: Documentation And User Contract Cleanup

Files:

- `docs/agent-registration-guide.md`
- `docs/agent-profile-reference.md`
- `docs/mvp-development-spec.md`
- `docs/architecture.md`
- `docs/development-operations.md`

Tasks:

1. Update guide with final policy algebra.
2. Explain `inherit=false` and `allow` source explicitly.
3. Explain doctor behavior for missing allow/deny values.
4. Explain host auth isolation and run user limitations.
5. Move legacy `labels.sandbox_setup_command` into a legacy appendix.
6. Align field table with `agent schema --json`.
7. Record operational lessons from version probe, host env isolation, and capability materialization.

Acceptance:

- User-facing docs do not imply unsupported behavior is active.
- Every parameter table has meaning, range, example, and actual enforcement status.

### Slice K: Test Registry, Gates, And Review

Files:

- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `scripts/test-after-change.sh`
- `vs_review/YYYY-MM-DD-agent-registration-gap-completion-review.md`

Tasks:

1. Add or update requirements:
   - `agent_profile_policy_materialization`
   - `agent_profile_schema_complete`
   - `agent_profile_version_probe`
   - `agent_profile_auth_isolation`
   - `agent_profile_run_as_enforcement`
2. Add tests:
   - `AGT-REG-007`: schema completeness.
   - `AGT-REG-008`: capability policy resolver and exact doctor errors.
   - `AGT-REG-009`: required command diagnostics.
   - `AGT-REG-010`: version command probe and replay warning.
   - `AGT-REG-011`: host auth isolation.
   - `AGT-REG-012`: run_as precheck.
3. Add `scripts/test-after-change.sh --select` routes for the new AGT-REG IDs.
4. Run full local gate.
5. Run `subagent-vs-review` after code changes.
6. Fix all accepted blocking findings and re-review before closure.

Acceptance:

- `scripts/verify-test-registry.sh` passes.
- `scripts/generate-test-traceability.sh` passes.
- `scripts/test-after-change.sh` passes.
- Review artifact is closed with accepted blockers fixed.

## 6. Suggested Commit Slices

1. `test: expose agent registration gap contracts`
2. `feat: complete agent schema field reference`
3. `feat: resolve capability policies from catalogs`
4. `feat: materialize enforced agent capabilities`
5. `feat: probe agent version commands`
6. `feat: isolate host auth environment`
7. `feat: enforce run-as support boundaries`
8. `feat: persist structured materialized agent runtime`
9. `docs: align agent registration contract`
10. `test: register agent registration completion gates`
11. `review: close agent registration gap review`

Follow the repository rule of committing and pushing each small completed topic.

## 7. Acceptance Matrix

| Requirement | Proof |
| --- | --- |
| Every profile parameter is visible in schema | `AGT-REG-007`, `harnesslab agent schema --json` field list |
| `inherit=false + allow=[x]` has defined source and behavior | `AGT-REG-008` resolver tests |
| Missing allow item fails doctor | `AGT-REG-008` doctor integration |
| Unsupported policy blocks before run | Existing `AGT-REG-006` plus upgraded exact diagnostics |
| `version_command` is real | `AGT-REG-010` doctor/run/replay tests |
| `required_commands` are checked | `AGT-REG-009` |
| Host auth inheritance is enforceable | `AGT-REG-011` |
| Host run_as mismatch is not silent | `AGT-REG-012` |
| Runtime artifacts show effective configuration | upgraded `AGT-REG-004` |
| Docs match runtime behavior | docs diff plus schema/docs consistency test |
| Full system remains healthy | `scripts/test-after-change.sh` |

## 8. Risks And Constraints

- Some real CLIs may not expose a stable way to enforce tools/hooks. Do not fake enforcement. Keep those domains unsupported until a black-box proof exists.
- Host env isolation can break commands that depend on ambient environment. Preserve only an explicit minimal baseline and document it.
- Version probing must be bounded to avoid hanging doctor.
- Capability catalogs can become stale. Keep catalogs small, explicit, and validated by tests instead of broad guessed lists.
- Avoid turning the materializer into benchmark-adapter logic. Benchmarks consume materialized runtime; they do not decide profile semantics.

## 9. Done Definition

This plan is complete only when:

1. All gaps in section 3 are either implemented or explicitly represented as blocking unsupported behavior with exact diagnostics.
2. No user-facing field is merely decorative.
3. `doctor --json` explains every unsupported or impossible declaration before run.
4. `agent schema --json` and docs agree.
5. Full local gate passes.
6. Fresh adversarial review is closed with all accepted blockers fixed.

## 10. Completion Evidence

- Implementation commits through `4e2a5cb test: wire agent registration gap gates` are pushed to `origin/main`.
- `AGT-REG-007` through `AGT-REG-012` are registered and routed through `scripts/test-after-change.sh --select`.
- The full `scripts/test-after-change.sh` gate passed after the final helper split, including Rust tests, Python bridge tests, real Terminal-Bench verifier smoke checks, test registry validation, secret scan, and coverage checks.
- `agent schema --json` exposes active/legacy status, allowed values, examples, and defaults for profile fields; docs/schema consistency is covered by `AGT-REG-007`.
- Fresh adversarial review is closed in `vs_review/2026-06-04-agent-registration-gap-completion-review.md` with accepted blockers fixed and re-reviewed.
