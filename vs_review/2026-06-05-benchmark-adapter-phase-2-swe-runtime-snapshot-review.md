# Benchmark Adapter Phase 2 SWE Runtime Snapshot Review

- Date: 2026-06-05
- Target: SWE-bench Pro external runtime snapshots and replay readiness blockers
- Repository: `/Volumes/XU-1TB-NPM/projects/HarnessLab`
- Review type: post-implementation adversarial review
- Status: closed; Phase 2 SWE runtime snapshot closure review passed

## Round 1 Review Input

Objective:

Review the current uncommitted changes for the Benchmark Adapter Phase 2 slice:
SWE-bench Pro `external-runtime.private.json` /
`external-runtime.public.json` snapshots and external replay readiness blockers.

Target locations:

- `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro/runtime_snapshot.rs`
- `crates/harnesslab-cli/src/runner/external/swe_bench_pro.rs`
- `crates/harnesslab-cli/src/runner/replay.rs`
- `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs`
- `crates/harnesslab-cli/tests/external_smoke_contract.rs`
- `tests/artifact_contracts/int_011_swe_bench_pro_runtime_artifacts.txt`
- `scripts/test-after-change.sh`
- `tests/REQUIREMENTS.toml`
- `tests/TEST_REGISTRY.toml`
- `xtask/src/adapter_claims.rs`
- `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- `docs/plans/2026-06-05-benchmark-adapter-phase-2-inventory.md`

Neutral change summary:

- Added a generic external runtime snapshot writer.
- Added SWE-bench Pro runtime snapshot construction for public/private
  attempt-level runtime material snapshots.
- Shared SWE metadata/workspace/evaluator command builders between execution
  and snapshot generation.
- Extended replay readiness validation to require external runtime snapshots
  for external benchmark attempts and block missing/divergent snapshots before
  replay run creation.
- Activated `SWEPRO-005` with focused snapshot/replay blocker coverage.

Risk focus:

- Replay fail-closed semantics before run creation.
- JSON/path mismatch false positives or false negatives.
- Public/private artifact boundary and local path leakage.
- Multi-attempt and old-run compatibility side effects.
- Registry/selector false-pass risk after moving `SWEPRO-005` from planned to
  active.

Verification status before review:

- `scripts/test-after-change.sh --select SWEPRO-005`: 1 passed.
- `scripts/test-after-change.sh --select INT-011`: 10 passed.
- `scripts/test-after-change.sh --select REPLAY-008`: 1 passed.
- `scripts/test-after-change.sh --select REPLAY-007`: 1 passed.
- `scripts/test-after-change.sh --select INT-013`: 1 passed.
- `scripts/test-after-change.sh --select META-002`: passed with 42
  requirements, 170 tests, 16 adapter claims.
- `scripts/test-after-change.sh --select META-008`: passed with active=6,
  planned=10.
- `cargo check --all-targets`: passed.
- `cargo test -p harnesslab-cli --all-features --lib`: 119 passed.
- `cargo fmt --check`: passed.
- `git diff --check`: passed.

Reviewer instructions:

- Fresh session, read targets directly, do not modify files.
- Cite evidence paths and line numbers when possible.
- Challenge happy paths, old-run compatibility, replay readiness, path
  handling, and redaction assumptions.
- Provide blocking findings, non-blocking risks, required fixes, missing tests,
  and evidence.

## Round 1 Reviewer Launch Records

| Reviewer | Role | Agent ID | Freshness | Context | Instructions |
| --- | --- | --- | --- | --- | --- |
| Carson | `code-reviewer` | `019e9575-1d94-75d2-921f-4a15df1b550f` | Fresh internal subagent | `fork_context=false`; no main thread history inherited | Read-only, inspect target files directly |
| Lovelace | `test-engineer` | `019e9575-67b6-7d73-a3a1-af93954a9dac` | Fresh internal subagent | `fork_context=false`; no main thread history inherited | Read-only, inspect tests and public/private boundary |

## Round 1 Reviewer Outputs

### Lovelace (`test-engineer`)

Summary:

- Request changes.
- Two blocking findings:
  - `external-runtime.public.json` included `redaction_basis`, while the plan
    assigned redaction basis to private runtime material.
  - `SWEPRO-005` and replay validation proved artifact presence plus shallow
    identity checks, not stored runtime material authority.

Blocking findings:

1. Public/private boundary violation.
   - Counterexample: every public snapshot serialized `redaction_basis`.
   - Trigger: any SWE-bench Pro attempt writing `external-runtime.public.json`.
   - Impact: public schema violates the documented boundary and weakens future
     public artifact guarantees.
   - Evidence: `crates/harnesslab-cli/src/runner/external/runtime_snapshot.rs`.

2. Replay authority weaker than plan/test claim.
   - Counterexample: mutate `external-runtime.public.json` command/material
     metadata or private command/material metadata while keeping minimal
     identity fields unchanged; replay still passed.
   - Trigger: stale, edited, or partially regenerated runtime snapshots.
   - Impact: `SWEPRO-005` did not prove stored runtime materials were
     authoritative.
   - Evidence: `crates/harnesslab-cli/src/runner/replay.rs` and
     `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs`.

Non-blocking risks:

- `SWEPRO-005.file_patterns` pointed at `external_smoke_contract.rs` instead of
  the active `swe_runtime_snapshot_contract.rs`.
- The test did not prove secret-token/path redaction in snapshot context beyond
  checking `[REDACTED]`.
- Public material hashes may fingerprint private upstream materials; accepted
  as current schema policy, but should stay visible for Phase 6 security review.

### Carson (`code-reviewer`)

Summary:

- Request changes.
- The replay gate is in the right pre-run position, but was not fully
  fail-closed.

Blocking findings:

1. Replay accepted materially divergent external runtime snapshots.
   - Counterexample: mutate schema, visibility, command, or material checksum
     fields while preserving the old shallow identity fields.
   - Trigger: stale, tampered, or partially regenerated attempt snapshots.
   - Impact: stored runtime material authority claim was not enforced.

2. Attempt enumeration skipped unreadable entries.
   - Counterexample: unreadable attempt entry or `file_type()` error could be
     skipped by `filter_map(Result::ok)` / `is_ok_and`.
   - Trigger: corrupt source run, permissions issue, broken filesystem entry.
   - Impact: incomplete source authority could evade replay blockers.

3. Metadata extraction command snapshot could drift from the executed command.
   - Counterexample: metadata extraction used one parquet path, then a
     different first-sorted parquet appeared before snapshot writing.
   - Trigger: mutable dataset directory during a long SWE attempt.
   - Impact: shared command builders existed, but the recorded command could be
     re-derived from changed data.

Non-blocking risks:

- `SWEPRO-005.file_patterns` mismatch.
- Public path redaction missed shell-escaped variants for private paths
  containing apostrophes.

## Main-Agent Responses

- Accept Lovelace blocker 1. Fixed by removing `redaction_basis` from the
  public external runtime snapshot schema and adding explicit negative
  assertions in `SWEPRO-005`.
- Accept Lovelace blocker 2 and Carson blocker 1. Fixed by adding
  `runtime_fingerprint` and `public_fingerprint` authority checks. Replay now
  validates schema version, visibility, attempt number, adapter version,
  runtime policy, public/private artifact agreement, private dataset/source
  path binding, and recomputed fingerprints over private commands/materials and
  public redacted projections. `SWEPRO-005` now mutates public schema,
  public commands, private commands, and private material checksums and asserts
  replay blocks before a new run is created.
- Accept Carson blocker 2. Fixed by replacing lossy attempt enumeration with
  explicit `read_dir`, `DirEntry`, and `file_type()` error handling that turns
  unreadable attempt entries into replay blockers. `SWEPRO-005` now also adds
  an extra attempt directory without runtime snapshots and asserts replay
  blocks.
- Accept Carson blocker 3. Fixed by storing the resolved parquet path in
  `SweInstance` during metadata extraction and using that stored path for
  snapshot commands/materials. `SWEPRO-005` now creates a later
  lexicographically earlier parquet during the agent phase and asserts the
  snapshot still records the originally executed parquet.
- Accept non-blocking `SWEPRO-005.file_patterns` mismatch. Fixed
  `tests/TEST_REGISTRY.toml` to point at
  `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs`.
- Defer the `SWEPRO-001` file-pattern concern to Phase 5. It remains a planned
  selector, and current umbrella coverage remains under `INT-011`; dedicated
  `SWEPRO-001..004` activation is explicitly still planned.
- Accept shell-escaped path redaction risk. Fixed by adding raw,
  shell-escaped, and single-quoted shell-escaped private path variants to
  external runtime redaction refs. `SWEPRO-005` now uses a benchmark root path
  containing an apostrophe and asserts public snapshot output does not contain
  raw or shell-escaped private root paths.

## Closure Status

Round 1 blockers accepted and fixes implemented. Additional fresh closure
review is required before this slice can be closed.

## Round 2 Closure Review Input

Objective:

Verify that the accepted Round 1 blockers are closed, without reusing Round 1
agent context.

Fix summary:

- Removed `redaction_basis` from `external-runtime.public.json`.
- Added `runtime_fingerprint` and `public_fingerprint` checks so replay
  validates private raw runtime material and public redacted projection content,
  not just shallow identity fields.
- Replay validation now checks schema version, visibility, attempt number,
  adapter version, runtime policy, public/private artifact agreement,
  dataset/source binding, and fingerprint consistency.
- Attempt enumeration now treats `read_dir`, `DirEntry`, and `file_type()`
  errors as replay blockers instead of silently skipping them.
- SWE metadata extraction now stores the resolved parquet path in `SweInstance`
  and snapshot generation uses that executed path instead of re-scanning the
  mutable dataset directory after execution.
- External runtime redaction refs now include raw, shell-escaped, and
  single-quoted shell-escaped private path variants.
- `SWEPRO-005.file_patterns` now points to
  `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs`.

Verification after fixes:

- `scripts/test-after-change.sh --select SWEPRO-005`: 1 passed.
- `scripts/test-after-change.sh --select INT-011`: 10 passed.
- `scripts/test-after-change.sh --select REPLAY-008`: 1 passed.
- `scripts/test-after-change.sh --select REPLAY-007`: 1 passed.
- `scripts/test-after-change.sh --select INT-013`: 1 passed.
- `scripts/test-after-change.sh --select META-002`: passed with 42
  requirements, 170 tests, 16 adapter claims.
- `scripts/test-after-change.sh --select META-008`: passed with active=6,
  planned=10.
- `cargo check --all-targets`: passed.
- `cargo test -p harnesslab-cli --all-features --lib`: 119 passed.
- `cargo fmt --check && git diff --check`: passed.

Round 2 reviewer instructions:

- Fresh session, read current files directly, do not modify files.
- Focus on closure of Round 1 blockers only.
- Report any remaining blocker with counterexample, trigger, impact, and proof
  needed.

## Round 2 Reviewer Launch Records

| Reviewer | Role | Agent ID | Freshness | Context | Instructions |
| --- | --- | --- | --- | --- | --- |
| Aristotle | `code-reviewer` | `019e9589-3057-77d0-afd0-18e409ad3bd6` | Fresh internal subagent | `fork_context=false`; no Round 1 or main thread history inherited | Read-only closure review |
| Pasteur | `test-engineer` | `019e9589-883d-7233-a47f-37f85f1990af` | Fresh internal subagent | `fork_context=false`; no Round 1 or main thread history inherited | Read-only test closure review |

## Round 2 Reviewer Outputs

### Aristotle (`code-reviewer`)

Summary:

- Accepted Round 1 blockers appear closed.
- No remaining blocker found for public `redaction_basis` leakage, shallow
  replay acceptance, attempt enumeration skip behavior, post-execution parquet
  re-scan drift, or shell-escaped private path redaction.

Blocking findings:

- None.

Non-blocking risks:

- `SWEPRO-005.file_patterns` still pointed to `external_smoke_contract.rs` at
  the time of review. This was not a runtime snapshot blocker but made closure
  metadata inaccurate.
- No direct test forces an OS-level `DirEntry` or `file_type()` error branch.
  The extra attempt directory test covers full enumeration and missing
  snapshots, not filesystem error injection.
- No direct public `runtime_materials[*].checksum` mutation test. The
  `public_fingerprint` code covers it by inspection, while the test mutates a
  public command.

### Pasteur (`test-engineer`)

Summary:

- One blocking finding remained: `SWEPRO-005.file_patterns` registry metadata
  was still stale even though selector routing was correct.
- Selector/helper coverage was otherwise sound: the selector runs the exact
  `swepro_005_replay_requires_stored_swe_runtime_materials` test in
  `swe_runtime_snapshot_contract.rs`, and the redaction/parquet/mutation checks
  are invoked by that test.

Blocking findings:

1. `SWEPRO-005` registry metadata stale.
   - Counterexample: `tests/TEST_REGISTRY.toml` still pointed
     `SWEPRO-005.file_patterns` at
     `crates/harnesslab-cli/tests/external_smoke_contract.rs`.
   - Trigger: human traceability or audit tooling following registry metadata
     instead of selector route.
   - Impact: `META-002` and `META-008` could pass while registry metadata
     remained wrong.

Non-blocking risks:

- Bundling helper checks into one top-level test reduces diagnostic precision
  but does not create false passes.
- Direct public material checksum mutation and OS-level unreadable entry tests
  remain possible future hardening.

## Round 2 Main-Agent Responses

- Accept Pasteur blocker. Fixed `SWEPRO-005.file_patterns` in
  `tests/TEST_REGISTRY.toml` to
  `crates/harnesslab-cli/tests/swe_runtime_snapshot_contract.rs`.
- Accept meta false-pass risk. Added an `xtask` active adapter proof guard so
  active route specs can require the selected test file in registry
  `file_patterns`; added `registry_013_claimed_active_id_requires_registry_file_pattern_for_selector_target`.
- Defer direct OS-level `DirEntry`/`file_type()` error injection and direct
  public material checksum mutation tests. Current production code is
  fail-closed and existing `SWEPRO-005` exercises extra-attempt enumeration,
  public command mutation, private command mutation, and private material
  checksum mutation. These additional direct branches are future hardening, not
  closure blockers.

## Round 2 Closure Verification

- `cargo test -p xtask adapter_claims -- --nocapture`: 13 passed.
- `scripts/test-after-change.sh --select META-002`: passed with 42
  requirements, 170 tests, 16 adapter claims.
- `scripts/test-after-change.sh --select META-008`: passed with active=6,
  planned=10.

Round 2 blockers are closed.

## Final Closure Status

- Blocking findings found: yes
- Accepted blocking findings fixed: yes
- Blocking re-review completed: yes
- Blocking re-review passed: yes
- Rejected findings backed by evidence: n/a
- Deferred findings documented: yes; direct OS-level enumeration fault
  injection and direct public material checksum mutation remain future
  hardening, not Phase 2 closure blockers.
- Blocked reason: n/a
- Allowed to proceed: yes
