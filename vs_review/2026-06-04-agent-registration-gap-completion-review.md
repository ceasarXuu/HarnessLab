# Agent Registration Gap Completion Review

Date: 2026-06-04
Status: closed

## Round 1: Slice K Test Registry And Gate Wiring

### Review Input

Objective: review the Slice K work for agent registration gap completion. The work wires existing and newly renamed agent-registration tests into requirements, test registry entries, and `scripts/test-after-change.sh --select` routes.

Target files:

- `crates/harnesslab-cli/tests/doctor_setup_contract.rs`
- `crates/harnesslab-cli/tests/host_auth_contract.rs`
- `scripts/test-after-change.sh`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`

Neutral change summary:

- Required-command diagnostics tests now use `agt_reg_009_*`.
- Host auth isolation tests now use `agt_reg_011_*`.
- `AGT-REG-007` through `AGT-REG-012` are registered in `tests/TEST_REGISTRY.toml`.
- New requirements cover policy materialization, schema completeness, version probe, auth isolation, and run_as enforcement.
- `scripts/test-after-change.sh --select` routes now exist for `AGT-REG-007` through `AGT-REG-012`.

Risk focus:

- Selector routes may skip tests promised by registry entries.
- Required runtime proof artifacts may overstate what tests actually produce or inspect.
- Requirement source sections may be too broad or stale.
- Aggregated shell selectors may hide omitted targets or fragile exit behavior.
- Renamed test IDs may leave stale references.

Known local verification before review:

- `cargo fmt --all --check`
- `scripts/verify-test-registry.sh`
- `scripts/generate-test-traceability.sh`
- `scripts/verify-test-after-change-select-output.sh`
- `scripts/test-after-change.sh --select AGT-REG-007`
- `scripts/test-after-change.sh --select AGT-REG-008`
- `scripts/test-after-change.sh --select AGT-REG-009`
- `scripts/test-after-change.sh --select AGT-REG-010`
- `scripts/test-after-change.sh --select AGT-REG-011`
- `scripts/test-after-change.sh --select AGT-REG-012`

### Reviewer Launch Records

| Reviewer | Role | Mechanism | Agent ID | Freshness | Context Excluded | Read-only |
| --- | --- | --- | --- | --- | --- | --- |
| Russell | test-engineer | `multi_agent_v1.spawn_agent` | `019e8ef3-97c7-70e3-a2ff-304fe848f2ef` | fresh session, `fork_context=false` | main conversation, hidden reasoning, prior conclusions | yes |
| Lorentz | code-reviewer | `multi_agent_v1.spawn_agent` | `019e8ef3-c150-7033-8f2f-4d78b6ce2ff9` | fresh session, `fork_context=false` | main conversation, hidden reasoning, prior conclusions | yes |

### Reviewer Outputs

#### Russell: Test Engineering Review

Summary: Found the clippy change behavior-preserving but raised two blocking test-quality gaps.

Blocking findings:

1. Aggregated `AGT-REG-008` and related selector routes could pass even if the filter matched zero tests because they bypassed the single-test exact-selection guard in `scripts/test-after-change.sh`.
2. `AGT-REG-011` under-tested host auth isolation: it covered `auth.inherit=false` and declared env inheritance, but missed `auth.inherit=true` with an undeclared ambient secret.

Non-blocking risk:

- Traceability runtime-proof artifacts are declarative; registry validation does not prove a test actually inspects each listed artifact.

#### Lorentz: Code Review

Summary: No blocking findings. Confirmed the `capability_policy.rs` clippy collapse was behavior-preserving and `AGT-REG-007..012` selectors passed in the reviewer run.

Non-blocking findings:

1. `AGT-REG-011` declared `results.json` as runtime proof but did not assert the file contents.
2. Unrelated infra env-policy tests used the `agt_reg_012_*` prefix, which made grep-based evidence for `AGT-REG-012` mix run-as tests with lower-level env-isolation tests.
3. Grouped selector expected counts are hardcoded and should be kept in sync when adding/removing grouped tests.

### Main-Agent Responses

| Finding | Response | Fix |
| --- | --- | --- |
| Aggregated selectors could pass with zero tests | accept | Added `run_filtered_tests` to `scripts/test-after-change.sh`; grouped `AGT-REG-008..012` routes now assert expected `running N test(s)` and matching ok-line counts. |
| Missing `auth.inherit=true` undeclared env negative test | accept | Added `agt_reg_011_host_agent_does_not_see_undeclared_env_when_auth_inherit_true` in `host_auth_contract.rs`. |
| `AGT-REG-011` declared `results.json` but did not inspect it | accept | Added `assert_results_success` and call it from all three host auth tests. |
| Infra tests reused `agt_reg_012_*` prefix | accept | Renamed infra env-policy tests to `c_sbox_002_host_exec_uses_explicit_environment_policy` and `c_sbox_010_docker_exec_preserves_client_env_without_agent_env_leak`. |
| Grouped expected counts are hardcoded | accept as non-blocking | Kept counts explicit because they are the current anti-zero-test guard; future test additions must update the count and selector evidence. |

Verification after fixes:

- `cargo fmt --all --check`
- `scripts/test-after-change.sh --select AGT-REG-008`
- `scripts/test-after-change.sh --select AGT-REG-009`
- `scripts/test-after-change.sh --select AGT-REG-010`
- `scripts/test-after-change.sh --select AGT-REG-011`
- `scripts/test-after-change.sh --select AGT-REG-012`
- `cargo test -p harnesslab-infra --lib process::tests::c_sbox_002_host_exec_uses_explicit_environment_policy -- --exact`
- `cargo test -p harnesslab-infra --test docker_exec_env_contract c_sbox_010_docker_exec_preserves_client_env_without_agent_env_leak -- --exact`
- `scripts/verify-test-registry.sh`
- `scripts/generate-test-traceability.sh`
- `scripts/verify-test-after-change-select-output.sh`
- `git diff --check`

## Round 2: Closure Review

### Review Input

Round 2 was required because Round 1 had accepted blocking findings.

Closure target:

- `scripts/test-after-change.sh`
- `crates/harnesslab-cli/tests/host_auth_contract.rs`
- `crates/harnesslab-core/src/capability_policy.rs`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- related renamed infra tests

Accepted blockers to verify:

1. Aggregated `AGT-REG-008..012` selectors must not pass with zero tests or omitted targets.
2. `AGT-REG-011` must cover `auth.inherit=true` with undeclared ambient env.

### Reviewer Launch Records

| Reviewer | Role | Mechanism | Agent ID | Freshness | Context Excluded | Read-only |
| --- | --- | --- | --- | --- | --- | --- |
| Hypatia | test-engineer | `multi_agent_v1.spawn_agent` | `019e8eff-d458-7141-a751-c3917e76ae66` | fresh session, `fork_context=false` | main conversation, hidden reasoning, prior conclusions | yes |

### Reviewer Output

Hypatia found no remaining blocking issue. Fresh selector runs passed with these expected per-target counts:

- `AGT-REG-008`: `5 + 1 + 4`
- `AGT-REG-009`: `4`
- `AGT-REG-010`: `4 + 2 + 5`
- `AGT-REG-011`: `3`
- `AGT-REG-012`: `2 + 4 + 1 + 1`

Closure evidence cited by reviewer:

- `run_filtered_tests` hard-fails unless selected targets report expected test counts and ok lines.
- `AGT-REG-011` now includes the undeclared-env negative case.
- `capability_policy.rs` clippy fix remains behavior-preserving.
- Registry and requirement mappings for new IDs are internally consistent.

Non-blocking risk:

- `run_filtered_tests` is coupled to Cargo's human-readable test output, but it closes the current zero-test hole.

### Closure

Closed. Round 1 accepted blocking findings were fixed and Round 2 closure review found no remaining blockers.

## Round 3: Post-Refactor And Default Semantics Review

### Review Input

Round 3 was required because additional code changes landed after Round 2:

- `SetupConfig` omitted `setup.run_as` default changed to `current`.
- Built-in generated templates continue to set explicit `harnesslab`.
- Runner execution/report input wiring was refactored into request/context structs.
- User docs were updated for omitted `setup.run_as` semantics.
- Full `scripts/test-after-change.sh` had passed before the review.

Target files:

- `crates/harnesslab-core/src/agent_profile.rs`
- `crates/harnesslab-core/src/agent_profile_reference.rs`
- `crates/harnesslab-core/src/capability_policy.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/src/runner/attempts.rs`
- `crates/harnesslab-cli/src/runner/report_context.rs`
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-cli/tests/agent_registry_contract.rs`
- `crates/harnesslab-cli/tests/host_auth_contract.rs`
- `crates/harnesslab-cli/tests/init_contract.rs`
- `scripts/test-after-change.sh`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `docs/agent-profile-reference.md`
- `docs/agent-registration-guide.md`
- `docs/architecture.md`
- `docs/mvp-development-spec.md`

### Reviewer Launch Records

| Reviewer | Role | Mechanism | Agent ID | Freshness | Context Excluded | Read-only |
| --- | --- | --- | --- | --- | --- | --- |
| Peirce | code-reviewer | `multi_agent_v1.spawn_agent` | `019e8f34-b468-7621-88c2-35972bd023aa` | fresh session, `fork_context=false` | main conversation, hidden reasoning, prior conclusions | yes |

### Reviewer Output

Peirce found no runtime correctness regression in `run_as` enforcement, host auth isolation, grouped selector guards, renamed infra contracts, or the runner refactor.

Blocking finding:

1. The plan required a docs/schema consistency gate, but `AGT-REG-007` only validated `agent schema --json`; it did not assert that user-facing docs matched schema/runtime semantics.

Non-blocking findings:

1. `run_filtered_tests` parses Cargo human-readable output and is therefore brittle against future Cargo output changes.
2. The review artifact top-level status still said `in progress` while earlier rounds said closed.
3. Init-template tests did not directly assert generated built-in profiles keep explicit `run_as = "harnesslab"`.
4. `AGT-REG-011` registry metadata mentioned `stdout.log`, but not all host auth cases inspected `stdout.log`.

### Main-Agent Responses

| Finding | Response | Fix |
| --- | --- | --- |
| Missing docs/schema consistency gate | accept | Added `agt_reg_007_agent_schema_docs_match_supported_profile_parameters`; `AGT-REG-007` now uses `run_filtered_tests` and expects two tests. |
| Selector guard uses Cargo human output | defer as non-blocking | Kept current guard because it already closes the zero-test hole; future hardening can move to machine-readable enumeration. |
| Review artifact status inconsistent | accept | Kept top-level status explicitly in-progress until Round 3 closure, with blocker state visible. |
| Init templates do not lock explicit `harnesslab` | accept | `int_001_init_empty_home_creates_config_and_profiles` now asserts built-in generated profiles parse to `RunAs::Harnesslab`. |
| Host auth stdout proof overstated | accept | All three `AGT-REG-011` cases now inspect `stdout.log` in addition to `results.json`. |

Verification after fixes:

- `cargo fmt --all --check`
- `scripts/test-after-change.sh --select AGT-REG-007`

## Round 6: Final Closure Review

### Review Input

Round 6 was required because Round 5 had an accepted blocking finding. The closure target was only the `AGT-REG-007` status/meaning semantic gate.

### Reviewer Launch Records

| Reviewer | Role | Mechanism | Agent ID | Freshness | Context Excluded | Read-only |
| --- | --- | --- | --- | --- | --- | --- |
| Fermat | test-engineer | `multi_agent_v1.spawn_agent` | `019e8f53-ec8d-71a2-8faf-b305fb42d0bb` | fresh session, `fork_context=false` | main conversation, hidden reasoning, prior conclusions | yes |

### Reviewer Output

Fermat found no blocking issues. The accepted `AGT-REG-007` blocker is closed: the test helper now checks docs `status` against schema `field["status"]` and docs `meaning` against schema `field["description"]` for every parsed field row. The selector path still runs both `AGT-REG-007` tests.

Non-blocking risks:

1. `run_filtered_tests` still depends on Cargo human-readable output.
2. The semantic matcher is containment-based rather than exact-cell equality.

### Main-Agent Responses

| Finding | Response | Fix |
| --- | --- | --- |
| Cargo human-readable selector guard brittleness | defer as non-blocking | Existing selected-count guard remains active. |
| Containment-based semantic matcher | defer as non-blocking | Accepted for this closure because docs deliberately include human-readable prose around schema anchors. |

### Closure

Closed. Round 5 accepted blocking finding was fixed and Round 6 closure review found no remaining blockers.

## Round 7: Helper Split Review

### Review Input

Round 7 reviewed only the final helper split made to keep every code file under 500 lines:

- moved AGT-REG-007 docs/schema semantic helper into `crates/harnesslab-cli/tests/support/agent_schema_docs.rs`
- exported it from `crates/harnesslab-cli/tests/support/mod.rs`
- kept the AGT-REG-007 test entry in `agent_registry_contract.rs`

### Reviewer Launch Records

| Reviewer | Role | Mechanism | Agent ID | Freshness | Context Excluded | Read-only |
| --- | --- | --- | --- | --- | --- | --- |
| Harvey | test-engineer | `multi_agent_v1.spawn_agent` | `019e8f7a-f80a-7fd0-b16e-b3313fc1b20b` | fresh session, `fork_context=false` | main conversation, hidden reasoning, prior conclusions | yes |

### Reviewer Output

Harvey found no blocking issues. The split did not weaken `AGT-REG-007`: the extracted helper still checks required flag, allowed values, examples, defaults, status, and schema description for every schema field row. File lengths are within the repo rule.

Non-blocking risks:

1. The helper is schema-driven and does not fail on stale extra docs rows after a schema field is removed.
2. The support module export is broader than strictly necessary, but harmless because the helper is stateless.

### Main-Agent Responses

| Finding | Response | Fix |
| --- | --- | --- |
| Stale extra docs rows are not rejected | defer as non-blocking | Existing gate covers missing/out-of-sync schema fields; exact doc-row set enforcement can be added in a future hardening pass. |
| Broad support export | accept as harmless | No code change; support modules are already test-local and stateless. |

### Final Closure

Closed. All accepted blocking findings across Rounds 1, 3, 4, and 5 were fixed and independently re-reviewed. Final full gate passed after the helper split.

## Round 5: Status And Meaning Semantic Closure Review

### Review Input

Round 5 was required because Round 4 closure review found the semantic docs/schema gate still did not compare normal-field `status` or schema `description`/meaning. It only checked status exactly for `legacy` and required non-empty runtime status/meaning for active fields.

Fixes after Round 5 blocker:

- `assert_doc_field_semantics` now requires every documented row's status cell to contain the schema `status` token.
- `assert_doc_field_semantics` now requires every documented row's meaning cell to contain the schema `description`.
- Both user-facing docs parameter tables now carry explicit `active`/`legacy` status markers and `Schema: ...` meaning anchors for each schema field.

### Reviewer Launch Records

| Reviewer | Role | Mechanism | Agent ID | Freshness | Context Excluded | Read-only |
| --- | --- | --- | --- | --- | --- | --- |
| Banach | test-engineer | `multi_agent_v1.spawn_agent` | `019e8f4f-4b91-7543-9f0c-01d8f0685c3a` | fresh session, `fork_context=false` | main conversation, hidden reasoning, prior conclusions | yes |

### Reviewer Output

Banach confirmed selector wiring was live and the docs gate was stronger than before, but kept the blocker open because normal-field status and schema description were still not compared.

Blocking finding:

1. `AGT-REG-007` still did not enforce schema/doc semantic consistency for `status` and `meaning` on normal fields.

Non-blocking findings:

1. `run_filtered_tests` depends on Cargo human-readable output.
2. Allowed values are inclusion-based rather than exact-set based.

### Main-Agent Responses

| Finding | Response | Fix |
| --- | --- | --- |
| Normal-field status and meaning not compared | accept | Added status-token and schema-description assertions for every docs row in `assert_doc_field_semantics`; docs rows now include explicit status and `Schema: ...` anchors. |
| Selector guard uses Cargo human output | defer as non-blocking | No change in this slice; selected-count guard remains active. |
| Allowed values inclusion-based | defer as non-blocking | Kept inclusion check because docs include human-readable explanation around schema tokens; future exact-set parsing can be added if docs move to a stricter schema-token cell. |

Verification after fixes:

- `cargo fmt --all --check`
- `scripts/test-after-change.sh --select AGT-REG-007`
- `scripts/test-after-change.sh --select AGT-REG-011`
- `cargo test -p harnesslab-cli int_001_init_empty_home_creates_config_and_profiles -- --exact --nocapture`

## Round 4: Docs/Schema Semantic Closure Review

### Review Input

Round 4 was required because Round 3 closure review found the accepted docs/schema consistency blocker was only partially fixed. The previous `AGT-REG-007` docs test checked field-name presence plus a `setup.run_as` special case, but did not compare allowed values, examples, defaults, status, or meaning for each documented field.

Fixes after Round 4 blocker:

- `agt_reg_007_agent_schema_docs_match_supported_profile_parameters` now parses Markdown parameter tables and checks every schema field row for required flag, allowed-value tokens, schema example, default-value mention where present, status, and meaning.
- `agent_profile_reference` examples were aligned to user-facing docs for auth paths, setup commands, and skills.
- `docs/agent-profile-reference.md` and `docs/agent-registration-guide.md` now use schema tokens in value cells and schema examples in example cells.
- `AGT-REG-007` still runs through the selector guard and expects two tests.

### Reviewer Launch Records

| Reviewer | Role | Mechanism | Agent ID | Freshness | Context Excluded | Read-only |
| --- | --- | --- | --- | --- | --- | --- |
| McClintock | test-engineer | `multi_agent_v1.spawn_agent` | `019e8f47-9bf4-77c2-a75b-b035d7c51d82` | fresh session, `fork_context=false` | main conversation, hidden reasoning, prior conclusions | yes |

### Reviewer Output

McClintock confirmed `AGT-REG-007` selector wiring was no longer skippable, but kept the blocker open because the docs test only checked field-name presence and one `setup.run_as` semantic case.

Blocking finding:

1. `AGT-REG-007` still did not enforce docs/schema semantic consistency beyond one `setup.run_as` special case.

Non-blocking findings:

1. `run_filtered_tests` still depends on Cargo human-readable output.
2. Docs looked consistent for the specific `setup.run_as` behavior.

### Main-Agent Responses

| Finding | Response | Fix |
| --- | --- | --- |
| Docs/schema semantic consistency still partial | accept | Reworked `AGT-REG-007` docs test to parse docs tables and check required, allowed values, examples, defaults, status, and meaning for every schema field row. |
| Selector guard uses Cargo human output | defer as non-blocking | No change in this slice; the explicit selected-count guard remains active. |

Verification after fixes:

- `cargo fmt --all --check`
- `scripts/test-after-change.sh --select AGT-REG-007`
