# Subagent VS Review: Universal Benchmark Adapter Protocol Phase 0

- Created: 2026-06-08T00:50:14+0800
- Updated: 2026-06-08T00:50:14+0800
- Report schema: adversarial-v1
- Task: Phase 0 implementation review for universal benchmark adapter protocol
- Report path: `vs_review/2026-06-08-universal-benchmark-adapter-protocol-phase-0-review.md`
- Review mode: fresh internal subagents
- Source session policy: no inherited main-agent context
- Status: in progress; Round 2 closure review pending

## Round 1: Phase 0 Review

### Review Input

#### Objective

Verify that Phase 0 implementation artifacts are sufficient to support the
universal benchmark adapter protocol migration before any runtime/replay/doctor
code changes begin.

#### Review Target

- Branch inventory completeness.
- Serialized compatibility risk classification.
- Frozen selector manifest strength.
- Baseline selector evidence.
- Whether Phase 0 exit criteria are honestly met.

#### Target Locations

- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`
- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-phase-0-branch-inventory.md`
- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-phase-0-frozen-selector-manifest.md`
- `tests/TEST_REGISTRY.toml`
- `tests/REQUIREMENTS.toml`
- `scripts/test-after-change.sh`
- `scripts/verify-planned-adapter-selectors.sh`
- `xtask/src/adapter_claims.rs`
- `crates/harnesslab-core/src/benchmark.rs`
- `crates/harnesslab-core/src/runtime.rs`
- `crates/harnesslab-cli/src/runner/external/runtime_adapter.rs`
- `crates/harnesslab-cli/src/runtime_compatibility.rs`
- `crates/harnesslab-cli/src/runner/replay.rs`

#### Change Introduction

Phase 0 adds two implementation artifacts:

- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-phase-0-branch-inventory.md`
- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-phase-0-frozen-selector-manifest.md`

The main implementation plan is updated to v0.3 / In Progress and links these
Phase 0 artifacts.

#### Risk Focus

- The branch inventory may claim all benchmark-specific matches are triaged
  while omitting important files or grouping too broadly.
- The frozen selector manifest may miss behavior selectors that should block
  protocol migration regressions.
- The baseline selector evidence may be too narrow to support Phase 0 closure.
- Generic-layer leaks may be misclassified as acceptable metadata.
- Test-only assertions may actually hide production coupling.
- The manifest may list commands but not enough expected pass/artifact evidence.

#### Assumptions To Attack

- The 72 matched files are all represented by the disposition categories.
- `scripts/verify-planned-adapter-selectors.sh` is enough baseline evidence for
  the current adapter proof surface.
- The selected `INT-*`, `TB-*`, `SEC-*`, `ADAPT-*`, and `SWEPRO-*` manifest rows
  cover the external runtime/replay/redaction/report/doctor behavior that must
  not regress.
- The plan should proceed to Phase 1 after this slice if review passes.

#### Verification Status

- `CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh`: passed with
  `adapter selectors ok: active=16 planned=1`.
- `git diff --check` on Phase 0 docs: passed.

#### Reviewer Instructions

- Fresh internal subagent session.
- Do not inherit main-agent context, chat history, reasoning, drafts, or
  conclusions.
- Read target files directly.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Report blocking findings, non-blocking risks, required fixes, missing tests,
  missing logs/observability, and concrete counterexamples for major findings.

### Reviewer Timeout Policy

| Complexity | Initial Wait | Extension | Max Attempts Per Role | Blocking Closure Behavior |
|---|---:|---:|---:|---|
| normal | 10 minutes | one bounded extension if alive | 2 | accepted blockers require fresh closure re-review |

### Reviewer Selection

| Reviewer | Reason Selected | Risk Area |
|---|---|---|
| architecture-reviewer | Check whether branch inventory correctly classifies generic-layer leaks and legacy compatibility surfaces. | architecture, migration safety |
| test-engineer | Check whether frozen selector manifest and baseline evidence are strong enough to prevent regression. | selector coverage, evidence quality |

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-reviewer | `multi_agent_v1.spawn_agent` | `019ea2fe-8a80-72c0-81e3-26b383b81c19` | spawn tool result | false | Round 1 Phase 0 review input plus target paths | main-agent history, reasoning, drafts, conclusions, full diff | yes |
| test-engineer | `multi_agent_v1.spawn_agent` | `019ea2fe-bb83-76c3-9679-4ad0a4a8ff96` | spawn tool result | false | Round 1 Phase 0 review input plus target paths | main-agent history, reasoning, drafts, conclusions, full diff | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| architecture-reviewer | architecture-reviewer | 1 | `019ea2fe-8a80-72c0-81e3-26b383b81c19` | completed | completed | returned blocking findings | accepted blockers fixed locally; fresh closure review launched |
| test-engineer | test-engineer | 1 | `019ea2fe-bb83-76c3-9679-4ad0a4a8ff96` | completed | completed | returned blocking findings | accepted blockers fixed locally; fresh closure review launched |

### Reviewer Outputs

#### architecture-reviewer

Verdict: `REQUEST CHANGES`.

Blocking findings:

- Inventory scope was too narrow because it scanned `crates`, `tests`,
  `scripts`, and `xtask`, but omitted active `integrations/terminal_bench/*`
  surfaces and related `PY-TB-*` / `AGT-REG-*` selectors.
- Frozen selector manifest was not enforceable enough: it did not include
  expected test counts, owning contracts, or benchmark-relevant `C-BENCH-*`,
  `PY-TB-001`, and `AGT-REG-*` rows.
- `crates/harnesslab-adapters/src/registry.rs` was misclassified as registry
  metadata even though it constructs descriptors and string-matches benchmark
  names to concrete adapters.
- Serialized authority risks were under-described: replay and runtime snapshots
  serialize benchmark, runner kind, adapter version, runtime policy,
  dataset/source paths, public/private snapshots, anchors, and fingerprints.

Non-blocking risk:

- `crates/harnesslab-core/src/agent_profile_reference.rs` was marked no-action,
  but its label allowlists feed readiness and compatibility behavior.

#### test-engineer

Verdict: `REQUEST CHANGES`.

Blocking findings:

- The only machine guard was `scripts/verify-planned-adapter-selectors.sh`,
  which covered only 16 active `ADAPT-*` / `SWEPRO-*` rows plus one planned
  sentinel and did not enforce `INT-*`, `TB-*`, `SEC-*`, report, replay,
  redaction, or doctor behavior.
- The frozen selector manifest lacked exact expected counts and owning
  contracts. It also incorrectly listed `INT-011` required artifacts as `[]`
  even though `tests/TEST_REGISTRY.toml` declares a full SWE-bench Pro runtime
  artifact set.
- Doctor/readiness coverage was missing because `DOC-004` was not frozen.

Non-blocking risks:

- `xtask/src/adapter_claims.rs` still knows only the older adapter proof
  prefixes.
- Weakening rules were prose-only and had no parser or lockfile.

### Main-agent Response To Round 1

| Finding | Response | Fix |
|---|---|---|
| Broader frozen selectors were not machine-enforced. | accept | Added `tests/FROZEN_SELECTOR_MANIFEST.toml` and `xtask/src/frozen_selectors.rs`; wired `cargo run -p xtask -- verify-frozen-selector-manifest` into `scripts/verify-test-registry.sh`. |
| Manifest missed expected counts, owning contracts, and correct `INT-011` artifacts. | accept | The lockfile now records `expected_test_count`, `expected_pass_threshold`, `file_patterns`, `required_artifacts`, and `owning_contracts` for 84 selectors; `INT-011` preserves its full runtime artifact list. |
| Doctor/readiness selector was missing. | accept | `DOC-004` is included in the frozen lockfile and coverage summary. |
| Inventory omitted `integrations/terminal_bench`. | accept | Branch inventory scan scope now includes `integrations`; Python bridge files are classified as adapter-owned runtime bridge support. |
| Registry was misclassified as metadata-only. | accept | Branch inventory now classifies `crates/harnesslab-adapters/src/registry.rs` behavior dispatch as a generic-layer leak to replace. |
| Serialized authority risks were under-described. | accept | Added a serialized authority field matrix covering benchmark plan, runtime preflight, task snapshots, public/private runtime snapshots, anchors, and fingerprints. |
| `agent_profile_reference.rs` had no owner. | accept | Reclassified as Phase 4 readiness/schema migration ownership. |
| `adapter_claims.rs` still lacks protocol rows. | defer | Phase 1/2 work must add `ADAPT-PROTOCOL-*` rows; Phase 0 now prevents weakening of existing rows. |

### Local Fix Evidence Before Round 2

Commands run after fixes:

- `cargo fmt --all --check`: passed.
- `cargo test -p xtask frozen_selectors -- --nocapture`: 3 tests passed.
- `cargo run -p xtask -- verify-frozen-selector-manifest`: passed with
  `frozen selector manifest ok: 84 selectors`.
- `scripts/verify-test-registry.sh`: passed; includes registry validation,
  frozen selector manifest validation, and traceability generation.
- `cargo check -p xtask`: passed.
- `git diff --check`: passed.
- `CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh`: passed
  with `adapter selectors ok: active=16 planned=1`.

## Round 2: Accepted-blocker Closure Review

### Review Input

#### Objective

Verify whether the accepted Round 1 blockers are closed by the current Phase 0
implementation slice.

#### Review Target

- Machine-enforced frozen selector lockfile and `xtask` guard.
- Updated test registry gate wiring.
- Updated Phase 0 inventory and frozen manifest docs.
- Whether any Round 1 blocking finding remains.

#### Target Locations

- `xtask/src/frozen_selectors.rs`
- `xtask/src/main.rs`
- `tests/FROZEN_SELECTOR_MANIFEST.toml`
- `tests/TEST_REGISTRY.toml`
- `scripts/verify-test-registry.sh`
- `scripts/test-after-change.sh`
- `scripts/verify-planned-adapter-selectors.sh`
- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-phase-0-branch-inventory.md`
- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-phase-0-frozen-selector-manifest.md`

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-closure-reviewer | `multi_agent_v1.spawn_agent` | `019ea30d-b4ef-7a51-9ba6-e82d13e7872d` | spawn tool result | false | Round 2 closure review packet plus target paths and verification evidence | main-agent history, reasoning, drafts, conclusions, full diff | yes |
| test-closure-reviewer | `multi_agent_v1.spawn_agent` | `019ea30d-eac5-74a2-91f8-c806e1115d7d` | spawn tool result | false | Round 2 closure review packet plus target paths and verification evidence | main-agent history, reasoning, drafts, conclusions, full diff | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| architecture-closure-reviewer | architecture-closure-reviewer | 1 | `019ea30d-b4ef-7a51-9ba6-e82d13e7872d` | completed | completed | returned blocking findings | accepted blockers fixed locally; fresh Round 3 closure review required |
| test-closure-reviewer | test-closure-reviewer | 1 | `019ea30d-eac5-74a2-91f8-c806e1115d7d` | completed | completed | returned blocking findings | accepted blockers fixed locally; fresh Round 3 closure review required |

### Reviewer Outputs

#### architecture-closure-reviewer

Verdict: `REQUEST CHANGES`.

Blocking findings:

- The frozen selector guard did not lock actual route targets. A selector could
  be remapped to a different same-count single test or an external shell proof
  could be replaced by another successful command while preserving registry
  command and inferred count.
- The branch inventory scan was not reproducibly exact after new files were
  added; the documented count needed correction and
  `integrations/terminal_bench/harnesslab_tb_ps.py` needed explicit
  disposition.

Non-blocking risks:

- `expected_pass_threshold` was only a non-empty string.
- The manifest print command can regenerate the lockfile from current routes,
  so review policy must still govern lockfile refreshes.
- `file_patterns` are literal path checks, not globs.

#### test-closure-reviewer

Verdict: `REQUEST CHANGES`.

Blocking findings:

- Frozen selectors could be removed or weakened if `tests/TEST_REGISTRY.toml`
  and `tests/FROZEN_SELECTOR_MANIFEST.toml` were changed together because the
  frozen universe was derived from the current registry.
- Router behavior could be weakened while preserving inferred counts because
  the guard did not compare route package/target/filter/script command.
- `META-002.file_patterns` omitted `tests/TEST_REGISTRY.toml` and
  `scripts/test-after-change.sh`, even though the guard reads both.

Non-blocking risks:

- The frozen selector manifest doc listed the wrong `INT-*` count.
- `expected_pass_threshold` was not machine-interpreted.

### Main-agent Response To Round 2

| Finding | Response | Fix |
|---|---|---|
| Registry/manifest co-deletion could pass. | accept | Added `xtask/src/frozen_selector_ids.rs` with an independent 84-id baseline. The guard now rejects missing or unexpected lockfile ids before relying on current registry-derived families. |
| Same-count route substitution could pass. | accept | Added `router_case` to `tests/FROZEN_SELECTOR_MANIFEST.toml`; `xtask/src/frozen_selectors.rs` now compares the exact router case line. |
| External script proof could be replaced by no-op. | accept | `router_case` freezes external script routes such as `PY-TB-001`, `AGT-REG-005`, `INT-029`, `INT-036`, `INT-037`, and `INT-038`; added a negative test for no-op substitution. |
| `META-002.file_patterns` missed guard inputs. | accept | Added `scripts/test-after-change.sh`, `tests/TEST_REGISTRY.toml`, and `xtask/src/frozen_selector_ids.rs` to `META-002.file_patterns`. |
| Branch inventory count and `harnesslab_tb_ps.py` were stale. | accept | Updated scan count to 80 and explicitly assigned `harnesslab_tb_ps.py` under Terminal-Bench adapter-owned runtime bridge support. |
| Guard observability was too coarse. | accept | Guard output now prints total and per-family counts. |
| `expected_pass_threshold` is not interpreted. | defer | It remains a human-readable field; route semantics are now enforced through `router_case` and expected count. |
| Lockfile can be regenerated from current routes. | defer | `print-frozen-selector-manifest` is retained for reviewed refreshes; guard protection now depends on independent ID baseline plus exact router case comparison. |

### Local Fix Evidence Before Round 3

Commands run after Round 2 fixes:

- `cargo fmt --all --check`: passed.
- `cargo test -p xtask frozen_selectors -- --nocapture`: 5 tests passed,
  including negative tests for same-count route substitution, external script
  no-op substitution, artifact weakening, and missing required frozen ids.
- `cargo run -p xtask -- verify-frozen-selector-manifest`: passed with
  `frozen selector manifest ok: total=84 ADAPT-DATA=6 ADAPT-RUNTIME=6 AGT-REG-005=1 AGT-REG-012=1 C-BENCH=10 DOC-004=1 INT=41 PY-TB-001=1 SEC=1 SWEPRO=5 TB=11`.
- `wc -l xtask/src/frozen_selectors.rs xtask/src/frozen_selector_ids.rs xtask/src/main.rs`: all code files remain under 500 lines.
- Branch scan command over `crates tests scripts xtask integrations`: 80 files.

## Round 3: Accepted-blocker Closure Review

### Review Input

#### Objective

Verify whether the accepted Round 2 blockers are closed by the current frozen
selector guard implementation.

#### Review Target

- Independent frozen selector id baseline.
- Exact router case locking.
- External shell proof no-op protection.
- Registry gate wiring and file pattern coverage.
- Updated branch inventory count and Terminal-Bench Python bridge assignment.

#### Target Locations

- `xtask/src/frozen_selectors.rs`
- `xtask/src/frozen_selector_ids.rs`
- `xtask/src/main.rs`
- `tests/FROZEN_SELECTOR_MANIFEST.toml`
- `tests/TEST_REGISTRY.toml`
- `scripts/verify-test-registry.sh`
- `scripts/test-after-change.sh`
- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-phase-0-branch-inventory.md`
- `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-phase-0-frozen-selector-manifest.md`

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| architecture-round-3-reviewer | `multi_agent_v1.spawn_agent` | `019ea319-32a8-7513-95c4-c2e5e8459059` | spawn tool result | false | Round 3 closure review packet plus target paths and verification evidence | main-agent history, reasoning, drafts, conclusions, full diff | yes |
| test-round-3-reviewer | `multi_agent_v1.spawn_agent` | `019ea319-6100-7bb2-ac5a-0e13dedcb632` | spawn tool result | false | Round 3 closure review packet plus target paths and verification evidence | main-agent history, reasoning, drafts, conclusions, full diff | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| architecture-round-3-reviewer | architecture-round-3-reviewer | 1 | `019ea319-32a8-7513-95c4-c2e5e8459059` | completed | completed | returned blocking finding | accepted blocker fixed locally; fresh Round 4 closure review required |
| test-round-3-reviewer | test-round-3-reviewer | 1 | `019ea319-6100-7bb2-ac5a-0e13dedcb632` | completed | completed | returned PASS | no further action |

### Reviewer Outputs

#### architecture-round-3-reviewer

Verdict: `REQUEST CHANGES`.

Blocking finding:

- The guard locked selector case lines but not the execution surface behind
  those cases. Keeping `PY-TB-001) exec
  scripts/verify-terminal-bench-python-adapter.sh ;;` unchanged while replacing
  the script body with a successful no-op would still pass. Changing the shared
  `run_filtered_tests` or common Cargo execution block could also weaken many
  selectors while preserving every exact `router_case`.

Non-blocking risks:

- Reviewed files were still uncommitted/untracked during review.
- `print-frozen-selector-manifest` can regenerate the lockfile from current
  routes, so reviewed refresh discipline remains required.

#### test-round-3-reviewer

Verdict: `PASS`.

Summary:

- Independent `REQUIRED_FROZEN_IDS` closes registry/manifest co-deletion.
- Exact `router_case` comparison closes same-count route substitution and
  selector-line no-op substitution.
- `META-002.file_patterns` includes registry/router/frozen guard inputs.
- Existing negative tests cover missing ids, artifact weakening, same-count
  route substitution, external route no-op substitution, and grouped count
  summing.

Non-blocking risk:

- The guard locked external script routes but did not inspect or execute script
  bodies. This was raised as non-blocking by the test reviewer and as blocking
  by the architecture reviewer; the main agent accepted the stricter finding.

### Main-agent Response To Round 3

| Finding | Response | Fix |
|---|---|---|
| External proof scripts could be replaced by no-op while route line remained unchanged. | accept | Added `xtask/src/frozen_execution_files.rs` and `execution_files` entries in `tests/FROZEN_SELECTOR_MANIFEST.toml`; hashes lock six external proof scripts plus `scripts/test-after-change.sh`. |
| Shared selector executor/common Cargo block could be weakened while route cases remain unchanged. | accept | `scripts/test-after-change.sh` is now content-hashed as an execution file. |
| Missing negative test for external script body no-op. | accept | Added `execution_file_hash_rejects_noop_replacement` in `xtask/src/frozen_execution_files.rs`. |
| Guard observability missing execution-surface integrity. | accept | Guard output now includes `execution_files=7`. |
| `print-frozen-selector-manifest` can regenerate lockfile from current state. | defer | Retained for reviewed lockfile refreshes; execution file hashes and independent ids require explicit reviewed changes. |

### Local Fix Evidence Before Round 4

Commands run after Round 3 fixes:

- `cargo fmt --all --check`: passed.
- `cargo test -p xtask frozen -- --nocapture`: 6 tests passed, including
  execution file no-op replacement rejection.
- `cargo run -p xtask -- verify-frozen-selector-manifest`: passed with
  `frozen selector manifest ok: total=84 execution_files=7 ADAPT-DATA=6 ADAPT-RUNTIME=6 AGT-REG-005=1 AGT-REG-012=1 C-BENCH=10 DOC-004=1 INT=41 PY-TB-001=1 SEC=1 SWEPRO=5 TB=11`.
- `scripts/verify-test-registry.sh`: passed; includes registry validation,
  frozen selector manifest validation, execution file hash validation, and
  traceability generation.
- Code file line counts: `xtask/src/frozen_execution_files.rs` 102,
  `xtask/src/frozen_selectors.rs` 443, `xtask/src/frozen_selector_ids.rs` 86,
  `xtask/src/main.rs` 473.

## Round 4: Execution-surface Closure Review

### Review Input

#### Objective

Verify whether Round 3 execution-surface weakening blocker is closed.

#### Review Target

- Execution file hash coverage for shared selector executor and external proof
  scripts.
- Guard wiring from frozen manifest validation to execution file validation.
- Negative test for external proof script body no-op.
- Registry file-pattern coverage for execution files.

#### Target Locations

- `xtask/src/frozen_execution_files.rs`
- `xtask/src/frozen_selectors.rs`
- `xtask/src/frozen_selector_ids.rs`
- `tests/FROZEN_SELECTOR_MANIFEST.toml`
- `tests/TEST_REGISTRY.toml`
- `scripts/verify-test-registry.sh`
- `scripts/test-after-change.sh`

### Reviewer Launch Records

| Reviewer | Internal Mechanism | Session / Job ID | Trace Source | Context Forked | Input Packet | Context Explicitly Excluded | Read-only |
|---|---|---|---|---|---|---|---|
| execution-surface-closure-reviewer | `multi_agent_v1.spawn_agent` | `019ea321-173d-7c81-96fd-f0fee659c249` | spawn tool result | false | Round 4 execution-surface closure review packet plus target paths and verification evidence | main-agent history, reasoning, drafts, conclusions, full diff | yes |

### Reviewer Timeout Records

| Reviewer Output Key | Reviewer Role | Attempt | Session / Job ID | Waited | Status | Reason | Action |
|---|---|---:|---|---:|---|---|---|
| execution-surface-closure-reviewer | execution-surface-closure-reviewer | 1 | `019ea321-173d-7c81-96fd-f0fee659c249` | completed | completed | returned PASS | closure accepted |

### Reviewer Outputs

#### execution-surface-closure-reviewer

Verdict: `PASS`.

Summary:

- Round 3 blocker is closed. The guard now locks both selector routing and
  execution surface.
- `xtask/src/frozen_execution_files.rs` defines the exact seven required
  execution files, validates manifest set and hashes, and tests external proof
  no-op body replacement rejection.
- `xtask/src/frozen_selectors.rs` wires execution file validation into frozen
  manifest verification.
- `tests/FROZEN_SELECTOR_MANIFEST.toml` includes top-level
  `[[execution_files]]`.
- Guard output includes `execution_files=7`.

Blocking findings: none.

Non-blocking risk:

- `print-frozen-selector-manifest` can still regenerate hashes from the current
  tree, so reviewed lockfile refresh discipline remains important. This is not
  a closure blocker because validation requires the fixed execution-file set.

### Round 4 Closure Status

Accepted. All blocking findings from Rounds 1-3 have a recorded response,
implemented fix, local validation evidence, and fresh closure review. Phase 0
slice is ready to commit.
