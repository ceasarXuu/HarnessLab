# Doctor Auth And Usage Review

## Round 1: Doctor Product Gap Review

### Review Input

#### Objective
Verify the final PRD gap closure for `harnesslab doctor`: doctor must directly report auth inheritance and usage parser configuration issues, not only Docker, command availability, and benchmark readiness.

#### Review Target
Recent changes split doctor logic into `crates/harnesslab-cli/src/doctor.rs`, kept CLI dispatch in `app.rs`, added auth include path/env/ssh socket diagnostics, added usage parser/source diagnostics, and registered `DOC-007`.

#### Target Locations
- `crates/harnesslab-cli/src/doctor.rs`
- `crates/harnesslab-cli/src/app.rs`
- `crates/harnesslab-cli/src/lib.rs`
- `crates/harnesslab-cli/tests/doctor_contract.rs`
- `tests/TEST_REGISTRY.toml`
- `scripts/test-after-change.sh`

#### Change Introduction
`doctor` now emits per-profile checks for auth include paths, inherited env configuration, optional SSH socket mount, and usage parser/source validity. A new integration test confirms missing auth paths, missing SSH socket, and unknown usage parser are visible in `doctor --json`. The old status-priority unit test moved from `app.rs` to `doctor.rs` to keep `app.rs` small.

#### Risk Focus
- Doctor becomes noisy or returns errors for valid default profiles.
- Auth path parsing mishandles `host:container:mode` entries or relative paths.
- Usage parser validation diverges from runtime usage collection.
- Moving doctor logic breaks CLI dispatch or registry selection.
- New file coverage and one-file-size constraints are violated.

#### Verification Status
Passed:
- Red before implementation: `cargo test -p harnesslab-cli --test doctor_contract doc_007_doctor_reports_auth_and_usage_configuration_problems` failed because checks were missing.
- Green after implementation: same test passed.
- `scripts/test-after-change.sh --select DOC-007` passed.
- `scripts/test-after-change.sh --select CLI-008` passed.
- `cargo test -p harnesslab-cli doctor::tests::doc_007_usage_source_validation_rejects_unsafe_file_paths` passed.

#### Reviewer Instructions
- Fresh internal subagent session.
- Do not inherit main-agent history.
- Read target files directly.
- Do not modify files.
- Focus on high-impact product correctness and test/registry adequacy.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| doctor-gap-closure | `multi_agent_v1.spawn_agent` using `code-reviewer` | `019e7cc3-9e89-7711-9f1a-74eda6821703` | spawn tool result | false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round1-doctor-gap | doctor-gap-closure | 1 | `019e7cc3-9e89-7711-9f1a-74eda6821703` | within timeout | completed | reviewer returned REQUEST CHANGES | accept blockers and re-review after fixes |

### Reviewer Outputs

#### round1-doctor-gap

Summary: REQUEST CHANGES. The refactor and wiring are correct, but the PRD gap is not closed yet.

Blocking findings:
- Doctor checks auth include path host existence but does not perform the PRD-required Docker dry-run mountability check.
- Relative auth include paths are resolved differently in doctor and runtime sandbox mount construction, so doctor can give false results.
- Usage parser/source issues are classified inconsistently with the product/runtime contract: missing parser is `ok`, invalid parser/source is `error`, while the spec treats usage instrumentation gaps as warning-only.

Non-blocking risks:
- This report was pending before the reviewer output was recorded.
- `tests/TEST_REGISTRY.toml` remains a pre-existing large file exception.

Missing tests:
- DOC-007 should parse JSON and assert status/severity/details, not just text presence.
- Need parity coverage for auth include path parsing.
- Need Docker dry-run mountability test path.

### Main Agent Response

| Finding | Decision | Response |
|---|---|---|
| Missing Docker auth dry-run | accept | Add Docker provider mount dry-run support and make doctor report mountability separately from host existence. |
| Doctor/runtime auth path divergence | accept | Extract shared auth mount parsing into core and use it from both doctor and sandbox. |
| Usage severity mismatch | accept | Align doctor with runtime by reporting usage parser absence and failures as warning-level checks, and add structured JSON assertions. |
| Report pending | accept | This report now records reviewer output and main-agent response. |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: pending
- Review completed: yes
- Review passed: no
- Allowed to proceed: no

## Round 2: Accepted Blocker Closure Review

### Review Input

#### Objective
Verify that the accepted Round 1 blockers are actually closed before committing the final doctor auth/usage diagnostics work.

#### Review Target
Round 2 changes add shared auth mount parsing in core, reuse that parser from runtime sandbox and doctor, add Docker auth mount dry-run checks, downgrade usage instrumentation gaps to warning-level doctor checks, strengthen DOC-007 with structured JSON assertions, and register the new sandbox dry-run test.

#### Target Locations
- `crates/harnesslab-core/src/config.rs`
- `crates/harnesslab-cli/src/runner/sandbox.rs`
- `crates/harnesslab-cli/src/doctor.rs`
- `crates/harnesslab-cli/tests/doctor_contract.rs`
- `crates/harnesslab-infra/src/docker.rs`
- `crates/harnesslab-infra/src/docker_tests.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `vs_review/2026-05-31-doctor-auth-usage-review.md`

#### Change Introduction
Auth include path parsing now flows through `parse_auth_mount` and `normalized_auth_host_path` so doctor and runtime share host path and Docker `-v` mount semantics. Doctor reports `auth.include_paths`, `auth.docker_mount`, `auth.env`, and optional `auth.ssh_socket`. Existing auth mounts are dry-run checked through Docker. Usage parser absence, unsupported parser, and unsupported source are warning-level diagnostics because token/cost collection is optional.

#### Risk Focus
- Shared auth parsing fails for realistic `host`, `host:container`, `host:container:mode`, `~`, `~/path`, absolute, or relative host entries.
- Doctor claims mountability without invoking the same Docker mount string that runtime will use.
- DOC-007 proves only strings instead of structured status/severity/details.
- New C-SBOX-012 test is not wired into the test registry and selected test runner.
- Usage warnings accidentally affect run ranking or make valid default profiles fail doctor.
- File-size and coverage constraints regress.

#### Verification Status
Passed:
- `cargo test -p harnesslab-core cfg_004_auth_mount_parser_matches_runtime_mount_contract`
- `cargo test -p harnesslab-infra c_sbox_012_mount_check_reports_dry_run_status`
- `cargo test -p harnesslab-cli doctor::tests::doc_007`
- `cargo test -p harnesslab-cli --test doctor_contract doc_007_doctor_reports_auth_and_usage_configuration_problems`
- `scripts/test-after-change.sh --select DOC-007`
- `scripts/test-after-change.sh --select CLI-008`
- `scripts/test-after-change.sh --select C-SBOX-012`
- `scripts/test-after-change.sh` passed with 180 tests, registry ok with 78 tests, secret scan ok, line coverage 95.47%, branch coverage 80.58%.

#### Reviewer Instructions
- Fresh internal subagent session.
- Do not inherit main-agent history.
- Read target files directly.
- Do not modify files.
- Try to disprove closure of each accepted Round 1 blocker.
- Cite concrete files and lines for any finding.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| blocker-closure | `multi_agent_v1.spawn_agent` using `code-reviewer` | `019e7cd0-4296-7bd2-a8b5-360b862d501a` | spawn tool result | false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round2-blocker-closure | blocker-closure | 1 | `019e7cd0-4296-7bd2-a8b5-360b862d501a` | within timeout | completed | reviewer returned REQUEST CHANGES | accept blocker and re-review after fix |

### Reviewer Outputs

#### round2-blocker-closure

Summary: REQUEST CHANGES.

Blocking findings:
- Doctor still did not validate the same auth mount set runtime uses. `append_auth_checks()` derived checks from raw `profile.auth.include_paths`, while runtime skipped mounts when `inherit == false` and filtered `exclude_paths`. This could produce `auth.docker_mount` errors for mounts runtime would never attempt.

Non-blocking risks:
- none

Missing tests/evidence:
- The core auth mount parity test existed but was not selectable from the registry.
- Some previously written targeted command names were ambiguous without full names.
- This report still showed Round 2 pending before the reviewer output was recorded.

Round 1 blocker closure verdict:
- Missing Docker auth dry-run: not closed because the dry-run was computed from non-effective mounts.
- Doctor/runtime auth path divergence: not closed.
- Usage severity mismatch: closed.
- Report pending: not closed until this report is updated.

### Main Agent Response

| Finding | Decision | Response |
|---|---|---|
| Doctor/runtime effective auth mount divergence | accept | Move effective auth mount selection into `harnesslab-core::effective_auth_mount_specs`, including `inherit`, `exclude_paths`, invalid entries, and deduplication; use it from both sandbox runtime and doctor. |
| Auth dry-run computed from non-effective mounts | accept | Doctor now builds `auth.docker_mount` from effective mounts only, and only dry-runs existing effective host paths. |
| Core auth parity test not selectable | accept | Add `CFG-006` to `scripts/test-after-change.sh` and `tests/TEST_REGISTRY.toml`. |
| Report pending | accept | Record reviewer output, responses, and closure state in this artifact. |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes, pending Round 3 confirmation
- Review completed: yes
- Review passed: no
- Allowed to proceed: no

## Round 3: Effective Auth Mount Closure Review

### Review Input

#### Objective
Verify closure of the Round 2 blocker: doctor must validate the same effective auth mount set as runtime.

#### Review Target
Effective auth mount selection has been moved to core and reused by both runtime sandbox and doctor.

#### Target Locations
- `crates/harnesslab-core/src/config.rs`
- `crates/harnesslab-cli/src/runner/sandbox.rs`
- `crates/harnesslab-cli/src/doctor.rs`
- `crates/harnesslab-cli/tests/doctor_contract.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `vs_review/2026-05-31-doctor-auth-usage-review.md`

#### Change Introduction
`effective_auth_mount_specs(profile)` returns only active auth mounts after applying `auth.inherit`, `auth.exclude_paths`, parse validation, and deduplication. Runtime `merged_mounts` appends these effective mounts to task sandbox mounts. Doctor reports all configured include paths with an `active` flag, warns only on missing active paths, and dry-runs only existing effective mounts. DOC-007 now verifies an excluded existing mount is not dry-run checked. `CFG-006` makes the effective auth mount core parity test registry-selectable.

#### Risk Focus
- Runtime and doctor still diverge for `inherit=false`, excluded paths, duplicate mounts, or invalid mount entries.
- Doctor hides configured auth path problems that should remain visible.
- Registry/test runner still cannot select the effective auth mount test.
- The review artifact does not close prior findings.

#### Verification Status
Passed:
- `cargo test -p harnesslab-core cfg_004_effective_auth_mount_specs_match_runtime_rules`
- `cargo test -p harnesslab-cli --test doctor_contract doc_007_doctor_reports_auth_and_usage_configuration_problems`
- `scripts/test-after-change.sh --select CFG-006`
- `scripts/test-after-change.sh --select DOC-007`
- `scripts/test-after-change.sh --select C-SBOX-012`

#### Reviewer Instructions
- Fresh internal subagent session.
- Do not inherit main-agent history.
- Read target files directly.
- Do not modify files.
- Focus only on whether the Round 2 accepted blocker and associated evidence gaps are closed.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| effective-auth-closure | `multi_agent_v1.spawn_agent` using `code-reviewer` | `019e7cd8-9088-7a03-a582-374f7a6eb03d` | spawn tool result | false | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round3-effective-auth-closure | effective-auth-closure | 1 | `019e7cd8-9088-7a03-a582-374f7a6eb03d` | within timeout | completed | reviewer returned PASS | update report and proceed |

### Reviewer Outputs

#### round3-effective-auth-closure

Summary: PASS.

Blocking findings:
- none

Non-blocking risks:
- The Round 3 report section still showed pending before this update.

Missing tests/evidence:
- No standalone LSP diagnostics tool was available to the reviewer; the reviewer used compilation-backed targeted tests instead.

Round 2 blocker closure:
- Closed. Effective auth mount selection is centralized in core, runtime and doctor call the same function, doctor warns only on missing active entries, and doctor dry-runs only existing effective mounts. DOC-007 proves an excluded mount is not checked, and CFG-006 is selectable from both the script and registry.

### Main Agent Response

| Finding | Decision | Response |
|---|---|---|
| Round 3 report pending | accept | Update reviewer output, response, and closure status in this artifact. |
| No standalone LSP diagnostics | accept | Treat compilation-backed targeted tests and full gate as sufficient for this Rust CLI repo. |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Review completed: yes
- Review passed: yes
- Allowed to proceed: yes

## Round 4: Line Count Split Review

### Review Input

#### Objective
Verify that splitting `config.rs` tests into `config_tests.rs` to satisfy the 500-line file limit did not break test selection or coverage.

#### Review Target
After Round 3 passed, line-count audit found `crates/harnesslab-core/src/config.rs` exceeded 500 lines. The config tests were moved into `crates/harnesslab-core/src/config_tests.rs`, and `config.rs` now includes them via `#[cfg(test)] #[path = "config_tests.rs"] mod tests;`.

#### Target Locations
- `crates/harnesslab-core/src/config.rs`
- `crates/harnesslab-core/src/config_tests.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`
- `vs_review/2026-05-31-doctor-auth-usage-review.md`

#### Verification Status
Passed:
- `scripts/test-after-change.sh` passed with 181 tests, registry ok with 79 tests, secret scan ok, line coverage 95.39%, branch coverage 80.94%.
- Line-count audit shows all Rust files are now <= 500 lines; `config.rs` is 358 lines.

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| line-count-split | `multi_agent_v1.spawn_agent` using `code-reviewer` | `019e7cde-8841-7ee0-bc78-0fe4be4d68db` | spawn tool result | false | Round 4 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round4-line-count-split | line-count-split | 1 | `019e7cde-8841-7ee0-bc78-0fe4be4d68db` | within timeout | completed | reviewer returned PASS | update metadata and proceed |

### Reviewer Outputs

#### round4-line-count-split

Summary: PASS.

Blocking findings:
- none

Non-blocking risks:
- `TEST_REGISTRY.toml` file patterns for config-related tests pointed only to `config.rs`, while moved test bodies now live in `config_tests.rs`.
- This Round 4 report section still showed pending before this update.

Missing tests/evidence:
- none

Line-count/test-split change:
- Acceptable. The split preserved the `config::tests::*` namespace, selected test mappings ran, registry verification passed, and repo-wide Rust file sizes are <= 500 lines.

### Main Agent Response

| Finding | Decision | Response |
|---|---|---|
| Config test file patterns incomplete | accept | Add `crates/harnesslab-core/src/config_tests.rs` to config-related registry file patterns. |
| Round 4 report pending | accept | Update reviewer output, response, and closure status in this artifact. |

### Closure Status

- Blocking findings found: no
- Review completed: yes
- Review passed: yes
- Allowed to proceed: yes
