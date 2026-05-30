# Subagent VS Review: Auth, Report, And Command Snapshot

- Created: 2026-05-30T15:17:40+0800
- Updated: 2026-05-30T15:58:40+0800
- Report schema: adversarial-v1
- Task: close PRD/MVP DoD gaps around real first-run agent setup, Docker auth inheritance, reproducible run evidence, and report visibility.
- Report path: `vs_review/2026-05-30-auth-report-command-snapshot-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: open

## Round 1: Initial Implementation Review

### Review Input

#### Objective
Close product gaps from the PRD/MVP DoD audit around real first-run agent setup and reproducible run evidence.

#### Review Target
Implementation, tests, report output, and test registry traceability for auth inheritance and run provenance.

#### Target Locations
- `crates/harnesslab-core/src/config.rs`
- `crates/harnesslab-cli/src/app.rs`
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-cli/src/runner/sandbox.rs`
- `crates/harnesslab-report/src/lib.rs`
- `crates/harnesslab-cli/tests/init_contract.rs`
- `crates/harnesslab-cli/tests/cli_contract.rs`
- `crates/harnesslab-cli/src/runner_tests.rs`
- `tests/TEST_REGISTRY.toml`

#### Change Introduction
Built-in `init` profiles expand default auth inheritance for Codex, Claude Code, opencode, and Pi Coding Agent. Pi detection uses `pi coding --version || pi --version`. Docker requests merge profile auth env and mounts with task sandbox data. Run root and attempt directories write command snapshots. Reports show agent config and command/provenance links.

#### Risk Focus
- Secret leakage in snapshots, command files, events, or report.
- Broken init flow or profile overwrite behavior.
- Docker auth mount/env behavior ignoring disable/exclude controls.
- Report claims that do not match actual run artifacts.
- Test registry drift after moving `INT-001` to `init_contract.rs`.
- Tests that prove only artifact existence, not useful content.

#### Assumptions To Attack
- Default auth paths are useful and do not leak values.
- `auth.inherit=false` suppresses all profile auth injection.
- `exclude_paths` matches host paths after expansion.
- Command snapshots are redacted and do not persist task instructions in argument mode.
- Report config summary is enough to satisfy PRD agent-config visibility.
- Tests would fail if command snapshots, replay commands, or agent config disappeared.

#### Adversarial Lenses
- security
- data
- testing
- observability
- maintenance

#### Verification Status
- Previous full gate before review passed: `scripts/test-after-change.sh` with 151 tests, line coverage 95.80%, branch coverage 80.42%.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 20 min | one bounded extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Multi-module correctness and provenance changes | auth flow, snapshot content, report/run state |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | `multi_agent_v1.spawn_agent` using `code-reviewer` | `019e77b2-95a5-7500-96d9-ed87d7f8222b` | subagent final response in session summary | fork_context=false | Round 1 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round1-implementation | implementation-adversary | 1 | `019e77b2-95a5-7500-96d9-ed87d7f8222b` | <20 min | completed | reviewer returned findings | completed |

### Reviewer Outputs

#### round1-implementation

##### Summary
REQUEST CHANGES. The implementation improved provenance and profile defaults, but three blocking findings invalidated the product claim before fixes.

##### Blocking Findings
- `auth.inherit=false` was ignored for Docker auth injection; profile env and mounts were still merged.
  - Broken assumption: disabling auth inheritance suppresses profile auth.
  - Failure scenario: a user disables inherited credentials but the container still receives API keys or host auth paths.
  - Trigger condition: profile with `auth.inherit=false` and non-empty `inherit_env` or `include_paths`.
  - Impact: privacy and credential boundary breach.
  - Proof needed: unit test covering disabled inheritance for env and mounts.
- `exclude_paths` compared against rendered mount strings instead of normalized host paths.
  - Broken assumption: `~/.codex` excludes a mount such as `~/.codex:/root/.codex:ro`.
  - Failure scenario: excluded auth directories still mount into Docker.
  - Trigger condition: include path has host:container:mode syntax and exclude path is host-only.
  - Impact: unwanted credential/config exposure.
  - Proof needed: unit test covering host path expansion and exclude matching.
- Run provenance was not fully reproducible for non-default `--home`.
  - Broken assumption: report and command snapshots reflect the exact effective command.
  - Failure scenario: report suggested `~/.harnesslab/runs/...` or synthesized commands instead of the actual home/run path.
  - Trigger condition: custom `--home` or replay from a non-default run location.
  - Impact: replay instructions can be wrong and experiment evidence becomes weak.
  - Proof needed: black-box report/content test asserting persisted original and replay commands.

##### Non-blocking Risks
- Test registry drift after moving `INT-001`.
- Snapshot/report tests were existence-only and would not catch command/config regressions.
- Argument-mode profiles could persist task instruction text in `agent/command.txt`.

##### Required Fixes
- Gate profile env/mount injection on `profile.auth.inherit`.
- Normalize auth host paths before applying `exclude_paths`.
- Persist effective original and replay commands and have the report read those values.

##### Missing Tests
- Add negative auth inheritance/exclude-path tests.
- Add report and command snapshot content assertions.
- Add argument-mode command redaction test.

##### Missing Logs / Observability
- none

##### Evidence
- `crates/harnesslab-cli/src/runner/sandbox.rs`
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-cli/tests/cli_contract.rs`
- `tests/TEST_REGISTRY.toml`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | `auth.inherit=false` ignored | Disabled profile auth still injected | blocking | accept | `merged_env_vars` and `merged_mounts` now return task-only values when inheritance is disabled | Fixed in `crates/harnesslab-cli/src/runner/sandbox.rs`; added `docker_request_respects_auth_inherit_and_exclude_paths` | Round 2 closure review |
| implementation-adversary | `exclude_paths` compared against mount string | Host-only exclude failed for host:container:mode includes | blocking | accept | Auth mounts are parsed into structured host and mount values, host path is normalized before exclusion | Fixed in `crates/harnesslab-cli/src/runner/sandbox.rs`; same unit test covers exclusion | Round 2 closure review |
| implementation-adversary | Run provenance not reproducible | Report/command could point at wrong home/run location | blocking | accept | `command.txt` stores `original_command` and `replay_command`; report uses context values from stored run data | Fixed in `crates/harnesslab-cli/src/runner/store.rs`, `runner.rs`, and `crates/harnesslab-report/src/lib.rs`; black-box CLI test asserts content | Round 2 closure review |
| implementation-adversary | Test registry drift | Moved tests could be mapped to wrong files | non-blocking | accept | `CLI-002` now maps to `cli_contract.rs`; `INT-001` maps to `init_contract.rs` | Fixed in `tests/TEST_REGISTRY.toml` | Verify in full gate |
| implementation-adversary | Existence-only report tests | Tests would not catch weak provenance content | non-blocking | accept | CLI test now asserts report links, replay command text, run path, `--home`, and command snapshot fields | Fixed in `crates/harnesslab-cli/tests/cli_contract.rs` | Full gate |
| implementation-adversary | Argument-mode instruction leak | `agent/command.txt` could contain task instruction | non-blocking | accept | Argument-mode rendered command snapshots now use `[INSTRUCTION_ARGUMENT_REDACTED]` | Fixed in `crates/harnesslab-cli/src/runner/sandbox.rs`; added unit test | Full gate |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: no
- Blocking re-review passed: no
- Blocking re-review round links:
  - Round 2
- Blocking re-review launch records:
  - pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: accepted blocking findings require fresh closure review
- Allowed to proceed: no

## Round 2: Blocking Closure Review

### Review Input

#### Objective
Verify closure of Round 1 blockers before claiming the auth/provenance slice is complete.

#### Review Target
The final patched implementation and tests for auth inheritance, exclude-path matching, run command provenance, report content, and test registry mappings.

#### Target Locations
- `crates/harnesslab-cli/src/runner/sandbox.rs`
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-report/src/lib.rs`
- `crates/harnesslab-cli/tests/cli_contract.rs`
- `crates/harnesslab-cli/tests/init_contract.rs`
- `tests/TEST_REGISTRY.toml`

#### Change Introduction
Round 1 blockers were accepted and fixed. Docker auth injection is gated by `profile.auth.inherit`; auth include/exclude matching normalizes host paths; command snapshots persist redacted original/replay commands; report uses the stored command data; argument-mode command snapshots redact rendered instructions; registry mappings were corrected.

#### Risk Focus
- A Round 1 blocker is only partially fixed.
- Redaction or report provenance still leaks secrets or task instructions.
- Tests assert implementation details but miss real CLI/report output.
- Registry mappings still point to the wrong file.

#### Assumptions To Attack
- `profile.auth.inherit=false` prevents both env and mount inheritance.
- `~` expansion and host-only exclusions work for `host:container:mode`.
- Report replay/original commands are derived from stored run data, not hardcoded defaults.
- Command snapshots are useful for replay/debugging but do not persist instructions in argument mode.
- `CLI-002`, `DOC-001`, and `INT-001` registry file patterns are correct.

#### Adversarial Lenses
- security
- implementation
- testing
- observability

#### Verification Status
- Passed targeted tests after fixes:
  - `cargo test -p harnesslab-cli --test cli_contract int_003_fake_terminal_success_creates_report_and_results`
  - `cargo test -p harnesslab-cli --test init_contract`
  - `cargo test -p harnesslab-cli docker_request_respects_auth_inherit_and_exclude_paths`
  - `cargo test -p harnesslab-cli command_snapshot_redacts_argument_instruction`

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Focus on whether the accepted blocking findings are truly closed.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 20 min | one bounded extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Required closure review for accepted blocking implementation findings | auth/provenance correctness and tests |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | `multi_agent_v1.spawn_agent` using `code-reviewer` | `019e77c0-61f2-7d02-84b8-338f217c6c8e` | spawn tool result | fork_context=false | Round 2 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round2-implementation | implementation-adversary | 1 | `019e77c0-61f2-7d02-84b8-338f217c6c8e` | <20 min | completed | reviewer returned findings | completed |

### Reviewer Outputs

#### round2-implementation

##### Summary
REQUEST CHANGES. The earlier auth/provenance fixes mostly closed, but persisted profile redaction introduced an execution-semantic regression for resume/replay.

##### Blocking Findings
- Replay/resume can execute a mutated command because `agent-profile.snapshot.json` is redacted before persistence and then loaded as the executable profile.
  - Broken assumption: command/provenance redaction affects only debug/report artifacts, not runtime replay inputs.
  - Failure scenario: a profile command contains a literal value matching an env value listed in `auth.inherit_env`; the original run succeeds, but replay/resume reads a redacted command and behaves differently.
  - Trigger condition: a command such as `printf ok > result.txt` plus inherited env value `ok`.
  - Impact: replay/debug provenance is not faithful and resume/replay can fail or produce different outputs.
  - Proof needed: replay and resume contracts that preserve behavior when the report snapshot redacts command text.

##### Non-blocking Risks
- Original-command provenance silently degraded when `command.txt` was missing or malformed.
  - Broken assumption: report commands are always derived from stored run data.
  - Failure scenario: legacy or damaged run dirs substitute replay command as original command.
  - Trigger condition: missing `command.txt`.
  - Impact: report can present misleading reproduction history.
  - Proof needed: explicit unavailable marker when original command cannot be read.

##### Required Fixes
- Split execution-state persistence from redacted report artifacts.
- Do not load redacted report snapshots as executable resume/replay input.
- Stop silently substituting replay as original command when `command.txt` is missing.

##### Missing Tests
- Add replay contract proving redacted public snapshot does not mutate replay behavior.
- Add resume contract proving redacted public snapshot does not mutate resume behavior.
- Add fallback test for missing original command provenance.

##### Missing Logs / Observability
- Add an artifact distinction between runtime and redacted profile snapshots.

##### Evidence
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/tests/replay_contract.rs`
- `crates/harnesslab-cli/tests/resume_contract.rs`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | Redacted profile snapshot loaded for resume/replay | Public redaction mutated runtime command | blocking | accept | `store::write_run_inputs` now writes unredacted `agent-profile.runtime.json` and redacted `agent-profile.snapshot.json`; `resume_run` and `replay_run` call `store::load_run_profile` | Fixed in `crates/harnesslab-cli/src/runner/store.rs` and `crates/harnesslab-cli/src/runner.rs`; added replay and resume contracts with env value `ok` redaction reproduction | Round 3 closure review |
| implementation-adversary | Missing command snapshot falls back to replay as original | Report can mislabel missing original provenance | non-blocking | accept | `original_command_from_snapshot` now returns `[ORIGINAL_COMMAND_UNAVAILABLE]` when `command.txt` has no original command | Fixed in `crates/harnesslab-cli/src/runner/store.rs`; added store unit test | Round 3 closure review |
| implementation-adversary | Runtime/report snapshot distinction not observable | Operator cannot tell which snapshot is executable | non-blocking | accept | `command.txt` now records `agent_runtime_snapshot=agent-profile.runtime.json` and `agent_report_snapshot=agent-profile.snapshot.json` | Fixed in `crates/harnesslab-cli/src/runner/store.rs` | Full gate |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: no
- Blocking re-review passed: no
- Blocking re-review round links:
  - Round 3
- Blocking re-review launch records:
  - Pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: accepted Round 2 blocking finding requires fresh closure review
- Allowed to proceed: no

## Round 3: Runtime Snapshot Closure Review

### Review Input

#### Objective
Verify closure of the Round 2 blocking regression: redacted report snapshots must not mutate resume/replay behavior.

#### Review Target
Runtime profile snapshot persistence, resume/replay profile loading, report/provenance snapshot behavior, and regression tests.

#### Target Locations
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/tests/replay_contract.rs`
- `crates/harnesslab-cli/tests/resume_contract.rs`
- `crates/harnesslab-cli/tests/cli_contract.rs`
- `crates/harnesslab-report/src/lib.rs`

#### Change Introduction
The run store now writes `agent-profile.runtime.json` unredacted for exact local resume/replay and keeps `agent-profile.snapshot.json` redacted for report-facing provenance. `resume_run` and `replay_run` load the runtime snapshot when present. Missing original command provenance now renders `[ORIGINAL_COMMAND_UNAVAILABLE]` instead of silently substituting replay command text. New replay/resume tests reproduce the redaction case using env value `ok`.

#### Risk Focus
- Runtime snapshot is still not used in every resume/replay path.
- The unredacted runtime snapshot is accidentally linked from report HTML.
- The regression tests pass without proving behavior preservation.
- Legacy fallback behavior is broken.

#### Assumptions To Attack
- Current runs write both runtime and report profile snapshots.
- Resume and replay prefer `agent-profile.runtime.json`.
- Report-visible snapshot remains redacted.
- Replay/resume still succeed when the report snapshot command is redacted.
- Missing original command is explicit.

#### Adversarial Lenses
- security
- implementation
- testing
- observability

#### Verification Status
- Passed targeted tests:
  - `cargo test -p harnesslab-cli --test replay_contract`
  - `cargo test -p harnesslab-cli --test resume_contract`
  - `cargo test -p harnesslab-cli store::tests::missing_original_command_is_explicit`
  - `cargo test -p harnesslab-cli --test cli_contract int_003_fake_terminal_success_creates_report_and_results`

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Focus on whether the Round 2 blocking finding is truly closed.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 20 min | one bounded extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Required closure review for accepted runtime snapshot blocker | resume/replay correctness and provenance safety |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | `multi_agent_v1.spawn_agent` using `code-reviewer` | `019e77ca-56dd-7d12-84b9-60151fa8704c` | spawn tool result | fork_context=false | Round 3 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round3-implementation | implementation-adversary | 1 | `019e77ca-56dd-7d12-84b9-60151fa8704c` | <20 min | completed | reviewer returned PASS with non-blocking risks | completed |

### Reviewer Outputs

#### round3-implementation

##### Summary
PASS. No blocking regression remains for new runs written by this change. Dual snapshots are written, resume/replay prefer the unredacted runtime snapshot, report HTML links only the redacted snapshot plus `command.txt`, and missing original command provenance resolves to `[ORIGINAL_COMMAND_UNAVAILABLE]`.

##### Blocking Findings
- none

##### Non-blocking Risks
- Historical runs without `agent-profile.runtime.json` can still degrade silently if their only profile snapshot is already redacted.
  - Broken assumption: resume/replay preserve behavior even when the runtime snapshot is missing.
  - Failure scenario: a pre-fix or copied run lacking `agent-profile.runtime.json` executes a redacted report snapshot.
  - Trigger condition: missing runtime snapshot plus redacted report snapshot.
  - Impact: replay/resume fidelity for legacy run dirs can mutate silently.
  - Proof needed: test that deletes `agent-profile.runtime.json` and expects explicit degraded-mode failure.
- Report non-exposure of `agent-profile.runtime.json` was not protected by a negative test.
  - Broken assumption: future report template changes will not link the unredacted runtime snapshot.
  - Failure scenario: template adds the runtime snapshot link and tests still pass.
  - Trigger condition: report template change.
  - Impact: future secret-surface regression.
  - Proof needed: explicit report assertion that runtime snapshot is absent.

##### Required Fixes
- none required for blocking closure

##### Missing Tests
- Add end-to-end report rendering coverage for missing original command provenance.
- Add legacy fallback coverage for missing `agent-profile.runtime.json`.
- Update test registry artifacts for the runtime snapshot invariant.

##### Missing Logs / Observability
- Add an event/log indicating whether resume/replay loaded runtime or legacy report snapshot.
- Add an event/log or artifact signal when original command provenance is unavailable.

##### Evidence
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/tests/replay_contract.rs`
- `crates/harnesslab-cli/tests/resume_contract.rs`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | No blocking runtime snapshot regression remains | n/a | blocking-check | accept | Closure review PASS | No blocking action needed | n/a |
| implementation-adversary | Redacted legacy report snapshot can still mutate fallback behavior | Missing runtime snapshot could execute redacted profile | non-blocking | accept | `load_run_profile` now rejects redacted report snapshots when runtime snapshot is missing | Fixed in `crates/harnesslab-cli/src/runner/store.rs`; added `redacted_report_snapshot_without_runtime_is_rejected` and replay integration test | Round 4 hardening review |
| implementation-adversary | Report lacks negative test for runtime snapshot exposure | Future report can expose unredacted runtime snapshot | non-blocking | accept | Replay integration test now asserts report HTML does not contain `agent-profile.runtime.json` | Fixed in `crates/harnesslab-cli/tests/replay_contract.rs` | Round 4 hardening review |
| implementation-adversary | Missing original command only unit-covered | End-to-end report may not show sentinel | non-blocking | accept | Replay contract removes `command.txt`, resumes the run, and asserts `[ORIGINAL_COMMAND_UNAVAILABLE]` in report HTML | Fixed in `crates/harnesslab-cli/tests/replay_contract.rs` | Round 4 hardening review |
| implementation-adversary | Snapshot source and unavailable provenance lack observability | Runtime/report snapshot source was silent | non-blocking | accept | Resume/replay now emit `profile_snapshot_loaded`; `command.txt` already records runtime/report snapshot artifact names | Fixed in `crates/harnesslab-cli/src/runner/store.rs` and `runner.rs` | Round 4 hardening review |
| implementation-adversary | Registry artifact list stale | Runtime snapshot invariant not declared | non-blocking | accept | `INT-012` and `INT-014` required artifacts now include runtime and report snapshots | Fixed in `tests/TEST_REGISTRY.toml` | Full gate |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 3
- Blocking re-review launch records:
  - Round 3 Reviewer Launch Records
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: n/a
- Allowed to proceed: yes

## Round 4: Runtime Snapshot Hardening Review

### Review Input

#### Objective
Verify the Round 3 non-blocking hardening changes did not introduce regressions and adequately protect the runtime/report snapshot boundary.

#### Review Target
Legacy fallback behavior, report non-exposure of runtime snapshot, missing original command report rendering, profile snapshot load observability, and registry artifacts.

#### Target Locations
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/tests/replay_contract.rs`
- `crates/harnesslab-cli/tests/resume_contract.rs`
- `tests/TEST_REGISTRY.toml`

#### Change Introduction
`load_run_profile` now rejects a redacted report snapshot when `agent-profile.runtime.json` is missing. Replay tests delete `agent-profile.runtime.json` and expect explicit failure, assert the report does not expose the runtime snapshot, and assert report HTML shows `[ORIGINAL_COMMAND_UNAVAILABLE]` after deleting `command.txt`. Resume/replay emit `profile_snapshot_loaded` events. Registry artifacts now list both runtime and report snapshots for replay/resume contracts.

#### Risk Focus
- The explicit legacy failure is too broad or too narrow.
- Report still exposes the runtime snapshot indirectly.
- The new tests are not selected by registry or can pass without proving the invariant.
- Added events break line limits or event redaction assumptions.

#### Assumptions To Attack
- Redacted legacy profile fallback fails clearly.
- Unredacted runtime profile remains private to local artifacts and is not report-linked.
- Missing original command is user-visible in generated report.
- Registry artifacts reflect the new snapshot contract.

#### Adversarial Lenses
- security
- testing
- observability
- maintenance

#### Verification Status
- Passed targeted tests:
  - `cargo test -p harnesslab-cli --test replay_contract`
  - `cargo test -p harnesslab-cli --test resume_contract`
  - `cargo test -p harnesslab-cli store::tests::redacted_report_snapshot_without_runtime_is_rejected`
  - `scripts/test-after-change.sh --select INT-012`
  - `scripts/test-after-change.sh --select INT-014`
- File length check: key files are at or below 500 lines after moving event helper to `store`.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Focus on whether the hardening changes are correct and adequately tested.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 12 min | one bounded extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Code changed after Round 3 PASS and touches security/provenance boundary | snapshot loading and tests |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | `multi_agent_v1.spawn_agent` using `code-reviewer` | `019e77d2-1874-7802-a9be-2660ae3ad67a` | spawn tool result | fork_context=false | Round 4 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round4-implementation | implementation-adversary | 1 | `019e77d2-1874-7802-a9be-2660ae3ad67a` | <12 min | completed | reviewer returned REQUEST CHANGES | completed |

### Reviewer Outputs

#### round4-implementation

##### Summary
REQUEST CHANGES. Two blocking issues remained: replay/resume could re-expose secrets if public redaction was recomputed from the current environment, and new hardening tests were not selected by registry IDs.

##### Blocking Findings
- Replay/resume can re-expose secrets in `report.html` and `command.txt` when runtime snapshot is reused without original env vars present.
  - Broken assumption: unredacted runtime profile never leaks through report/debug artifact paths.
  - Failure scenario: initial run writes `agent-profile.runtime.json`; replay/resume loads it with no current secret env, recomputes redaction from an empty source, and renders the raw command into report and command snapshots.
  - Trigger condition: profile command contains an env-secret literal and replay/resume runs without that env var.
  - Impact: violates report and artifact redaction requirements.
  - Proof needed: replay/resume contracts scanning public artifacts for a fake secret after current env is absent.
- New hardening tests are not wired into registry selectors.
  - Broken assumption: `scripts/test-after-change.sh --select INT-012/INT-014` proves the new invariants.
  - Failure scenario: new tests exist but selectors run only one older exact test.
  - Trigger condition: selected registry gate.
  - Impact: hardening regressions can merge undetected.
  - Proof needed: distinct registry IDs/selectors for new hardening tests.

##### Non-blocking Risks
- Legacy redacted-snapshot detection relied on magic substring `[REDACTED]`.
  - Broken assumption: explicit legacy failure is neither too broad nor too narrow.
  - Failure scenario: a legitimate command containing `[REDACTED]` is rejected, or future redaction changes are missed.
  - Trigger condition: missing runtime snapshot plus sentinel matching.
  - Impact: false rejection or false acceptance.
  - Proof needed: remove heuristic and use a clear runtime-snapshot-required rule.

##### Required Fixes
- Persist and reuse report-facing redaction data instead of recomputing from current env during replay/resume.
- Add public-artifact secret scans for replay and resume.
- Register each hardening test under a selected test ID.
- Remove sentinel-based legacy redaction detection.

##### Missing Tests
- Replay/resume regression tests where the initial run has a fake secret, replay/resume runs without that env var, and public artifacts are scanned.
- Registry-backed selectors for the new replay/resume hardening tests.

##### Missing Logs / Observability
- Add or verify a contract around `profile_snapshot_loaded`.

##### Evidence
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/tests/replay_contract.rs`
- `crates/harnesslab-cli/tests/resume_contract.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | Replay/resume can re-expose secrets in report/debug artifacts | Public redaction recomputed from current env | blocking | accept | Report-facing profile is now loaded from persisted `agent-profile.snapshot.json`; `agent_config_summary`, root `command.txt`, and agent `command.txt` use the report profile, while execution uses `agent-profile.runtime.json` | Fixed in `store.rs`, `runner.rs`, `sandbox.rs`, `external.rs`, and `external/swe_bench_pro.rs`; added `INT-017` replay and `INT-020` resume public-artifact secret scans | Round 5 closure review |
| implementation-adversary | New hardening tests not selected by registry IDs | Selected gates skipped new invariants | blocking | accept | Added distinct selectors `INT-017` to `INT-020` in `scripts/test-after-change.sh` and registry entries in `tests/TEST_REGISTRY.toml` | Targeted selector runs for `INT-017`, `INT-018`, `INT-019`, and `INT-020` pass exactly one test each | Round 5 closure review |
| implementation-adversary | Sentinel-based legacy redaction detection brittle | `[REDACTED]` literal could false-positive | non-blocking | accept | Runtime snapshot is now required for resume/replay; missing runtime snapshot fails regardless of report snapshot contents | Fixed in `store::load_run_profile`; added `missing_runtime_snapshot_is_rejected_without_heuristics` | Round 5 closure review |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: no
- Blocking re-review passed: no
- Blocking re-review round links:
  - Round 5
- Blocking re-review launch records:
  - Pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: accepted Round 4 blocking findings require closure review
- Allowed to proceed: no

## Round 5: Public Artifact Redaction Closure Review

### Review Input

#### Objective
Verify closure of the Round 4 blockers: replay/resume must not leak runtime-profile secrets into public artifacts, and new hardening tests must be registry-selectable.

#### Review Target
Execution profile vs report profile separation, command/report artifact redaction, registry selectors, and line limits.

#### Target Locations
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/src/runner/sandbox.rs`
- `crates/harnesslab-cli/src/runner/external.rs`
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs`
- `crates/harnesslab-cli/src/runner/attempts.rs`
- `crates/harnesslab-cli/tests/replay_contract.rs`
- `crates/harnesslab-cli/tests/resume_contract.rs`
- `scripts/test-after-change.sh`
- `tests/TEST_REGISTRY.toml`

#### Change Introduction
Runs now keep two profile roles through execution: runtime profile for actual agent execution and persisted report profile for all public artifacts. Replay/resume load `agent-profile.runtime.json` for execution and `agent-profile.snapshot.json` for report/debug output. Missing runtime snapshot is a hard error. Internal, Terminal-Bench, and SWE-bench Pro command snapshots use the report profile. New `INT-017` to `INT-020` selectors cover public-artifact redaction, missing runtime snapshot, missing original command report rendering, and resume public-artifact redaction.

#### Risk Focus
- Any report, command, event, JSON, or text artifact still leaks fake secrets after replay/resume without current env.
- New selectors do not run the intended tests exactly once.
- External runner command snapshots still use runtime profile.
- New `runner/attempts.rs` split creates coverage or line-limit issues.

#### Assumptions To Attack
- Runtime profile is never used for public command/report snapshots.
- Persisted report profile redaction survives replay/resume without env vars.
- Registry-selectable hardening tests prove the invariants.
- Missing runtime snapshot fails clearly.

#### Adversarial Lenses
- security
- testing
- observability
- maintenance

#### Verification Status
- Passed targeted tests:
  - `cargo test -p harnesslab-cli --test replay_contract`
  - `cargo test -p harnesslab-cli --test resume_contract`
  - `cargo test -p harnesslab-cli runner::tests::run_008_panic_message_preserves_string_payloads`
  - `cargo test -p harnesslab-cli runner::store::tests::missing_runtime_snapshot_is_rejected_without_heuristics`
  - `scripts/test-after-change.sh --select INT-017`
  - `scripts/test-after-change.sh --select INT-018`
  - `scripts/test-after-change.sh --select INT-019`
  - `scripts/test-after-change.sh --select INT-020`
- File length check: `runner.rs` 481 lines, `runner/attempts.rs` 70 lines, `swe_bench_pro.rs` 495 lines, `cli_contract.rs` 500 lines.

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Focus on closure of Round 4 blockers and any new high-impact regressions.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| high-risk | 20 min | one bounded extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| implementation-adversary | Required closure review after accepted security/test blockers | redaction, replay/resume, registry selectors |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| implementation-adversary | `multi_agent_v1.spawn_agent` using `code-reviewer` | `019e77df-394a-7e70-9587-a28289cca73f` | spawn tool result | fork_context=false | Round 5 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round5-implementation | implementation-adversary | 1 | `019e77df-394a-7e70-9587-a28289cca73f` | <20 min | completed | reviewer returned REQUEST CHANGES | completed |

### Reviewer Outputs

#### round5-implementation

##### Summary
REQUEST CHANGES. Runtime/report profile separation looked correct, but registry validation failed because new hardening tests referenced an undefined requirement id. The reviewer also noted two status-note commands needed fully qualified test names.

##### Blocking Findings
- New hardening tests were not registry-valid because they referenced undefined requirement id `redaction`.
  - Broken assumption: `INT-017` and `INT-020` were fully registry-selectable.
  - Failure scenario: `scripts/verify-test-registry.sh` and the full after-change gate fail with `unknown requirement redaction`.
  - Trigger condition: registry validation.
  - Impact: Round 4 cannot close under the source-of-truth test registry.
  - Proof needed: replace `redaction` with existing `secret_redaction` or define a matching requirement, then rerun registry validation.

##### Non-blocking Risks
- Two verification commands in the review input used unqualified names that could run zero tests if copied with `--exact`.
  - Broken assumption: the quoted verification commands always prove coverage.
  - Failure scenario: Cargo reports zero tests for copied unqualified exact-name commands.
  - Trigger condition: copied manual verification command.
  - Impact: false confidence in validation evidence.
  - Proof needed: use fully qualified test names in the report.

##### Required Fixes
- Update `INT-017` and `INT-020` to reference `secret_redaction`.
- Rerun `scripts/verify-test-registry.sh`.
- Correct report verification commands to fully qualified test names.

##### Missing Tests
- none beyond registry meta gate after requirement fix

##### Missing Logs / Observability
- none blocking

##### Evidence
- `tests/TEST_REGISTRY.toml`
- `tests/REQUIREMENTS.toml`
- `scripts/verify-test-registry.sh`
- `crates/harnesslab-cli/src/runner_tests.rs`
- `crates/harnesslab-cli/src/runner/store.rs`

### Main Agent Response

| Reviewer | Finding | Broken Assumption / Failure Scenario | Severity | Decision | Evidence / Reason | Action Taken | Follow-up |
|---|---|---|---|---|---|---|---|
| implementation-adversary | Undefined requirement id `redaction` in `INT-017` and `INT-020` | Registry validation fails despite selector tests passing | blocking | accept | `tests/REQUIREMENTS.toml` defines `secret_redaction`, not `redaction`; registry now validates 15 requirements / 76 tests | Fixed `tests/TEST_REGISTRY.toml`; reran `scripts/verify-test-registry.sh` and selector tests `INT-017` / `INT-020` | Round 6 closure review |
| implementation-adversary | Verification commands in report could run zero tests if copied with `--exact` | Unqualified unit-test names are weak evidence | non-blocking | accept | Report now uses fully qualified names `runner::tests::run_008_panic_message_preserves_string_payloads` and `runner::store::tests::missing_runtime_snapshot_is_rejected_without_heuristics`; both rerun successfully | Updated report and reran fully qualified commands | Round 6 closure review |

### Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: no
- Blocking re-review passed: no
- Blocking re-review round links:
  - Round 6
- Blocking re-review launch records:
  - Pending
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: accepted Round 5 blocking finding requires closure review
- Allowed to proceed: no

## Round 6: Registry Closure Review

### Review Input

#### Objective
Verify closure of the Round 5 registry blocker and confirm no new high-impact regression remains in the runtime/report snapshot slice.

#### Review Target
Registry requirement IDs, selector wiring, report verification commands, and the runtime/report redaction boundary.

#### Target Locations
- `tests/TEST_REGISTRY.toml`
- `tests/REQUIREMENTS.toml`
- `scripts/test-after-change.sh`
- `vs_review/2026-05-30-auth-report-command-snapshot-review.md`
- `crates/harnesslab-cli/src/runner/store.rs`
- `crates/harnesslab-cli/src/runner.rs`
- `crates/harnesslab-cli/src/runner/sandbox.rs`
- `crates/harnesslab-cli/tests/replay_contract.rs`
- `crates/harnesslab-cli/tests/resume_contract.rs`

#### Change Introduction
`INT-017` and `INT-020` now reference existing requirement `secret_redaction`; the registry validates successfully. Review report verification commands now use fully qualified unit test names. Selector tests for `INT-017` and `INT-020` still execute exactly one test and pass.

#### Risk Focus
- Registry still fails under full validation.
- Hardening selectors are still not traceable to requirements.
- Report claims validation that did not actually run.
- Runtime/report redaction boundary regressed while fixing registry metadata.

#### Assumptions To Attack
- `scripts/verify-test-registry.sh` passes.
- `INT-017` and `INT-020` are registry-valid and selected correctly.
- Fully qualified unit-test commands run exactly the intended tests.
- No new public artifact redaction leak was introduced.

#### Adversarial Lenses
- testing
- evidence
- security
- maintenance

#### Verification Status
- Passed:
  - `scripts/verify-test-registry.sh` -> registry ok: 15 requirements, 76 tests
  - `cargo test -p harnesslab-cli runner::tests::run_008_panic_message_preserves_string_payloads`
  - `cargo test -p harnesslab-cli runner::store::tests::missing_runtime_snapshot_is_rejected_without_heuristics`
  - `scripts/test-after-change.sh --select INT-017`
  - `scripts/test-after-change.sh --select INT-020`

#### Reviewer Instructions
- Fresh internal subagent session.
- No inherited main-agent context.
- Read target files directly.
- Do not modify files.
- Focus on closure of Round 5 blocker and validation-evidence quality.
- Cite evidence paths and line numbers when possible.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 12 min | one bounded extension if alive | 2 | cannot pass if review is unavailable |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| test-validity-adversary | Required closure review after accepted registry/test-evidence blocker | registry, selectors, validation proof |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| test-validity-adversary | `multi_agent_v1.spawn_agent` using `code-reviewer` | `019e77e5-5fa0-71b0-97ab-e00fd9c6d446` | spawn tool result | fork_context=false | Round 6 Review Input | main-agent history, reasoning, drafts, conclusions | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| round6-test-validity | test-validity-adversary | 1 | `019e77e5-5fa0-71b0-97ab-e00fd9c6d446` | within initial window | completed | reviewer returned PASS | record result and close review |

### Reviewer Outputs

#### round6-test-validity

Summary: PASS.

Blocking findings: none.

Non-blocking risk: the repo-tracked review artifact itself was still pending before this update. Accepted and fixed by recording the Round 6 PASS result here.

Required fixes: update this review report to replace the pending Round 6 section with the completed PASS result.

Missing tests/logs: none blocking.

Evidence reviewed:
- `scripts/verify-test-registry.sh` passes with `registry ok: 15 requirements, 76 tests`.
- `scripts/test-after-change.sh --select INT-017` and `--select INT-020` pass.
- Fully qualified unit-test commands target the intended tests:
  - `cargo test -p harnesslab-cli runner::tests::run_008_panic_message_preserves_string_payloads`
  - `cargo test -p harnesslab-cli runner::store::tests::missing_runtime_snapshot_is_rejected_without_heuristics`
- Runtime and report profile paths remain separated in the reviewed runner/store/sandbox code.

### Main Agent Response

| Finding | Decision | Response |
|---|---|---|
| No blocking findings | Accepted | No code change required. |
| Non-blocking report artifact still pending | Accepted | This section now records the completed Round 6 result and closes the review. |

### Closure Status

- Blocking findings found: no
- Accepted blocking findings fixed: n/a
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Blocking re-review round links:
  - Round 6
- Blocking re-review launch records:
  - `019e77e5-5fa0-71b0-97ab-e00fd9c6d446`
- Rejected findings backed by evidence: n/a
- Deferred findings documented: n/a
- Blocked reason: n/a
- Allowed to proceed: yes

## Final Conclusion

PASS. Round 6 closure review found no blocking issues. The runtime/report snapshot boundary, registry selectors, and validation evidence are allowed to proceed.
