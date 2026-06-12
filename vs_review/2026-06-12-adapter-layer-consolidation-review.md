# Adapter Layer Consolidation Adversarial Review

## Review Round 1

### Review Input

**Objective:**
Comprehensive adversarial review of the adapter layer construction code across
the entire `harnesslab-adapters` crate and its integration with the CLI runtime
adapter layer. The adapter layer has been built up incrementally over 8+ phases
and 20+ adversarial reviews. This review focuses on cross-cutting concerns,
architectural consistency, edge cases, and risks that may have been missed in
the incremental reviews.

**Review Target:**
Full adapter layer: data adapters, protocol contracts, registry, data boundary
contracts, runtime adapters, and their integration.

**Target Locations:**
- `crates/harnesslab-adapters/src/registry.rs` — `BenchmarkAdapter` trait + factory
- `crates/harnesslab-adapters/src/protocol_registry.rs` — `AdapterRegistry` + bindings
- `crates/harnesslab-adapters/src/protocol_contract.rs` — lifecycle contract validation
- `crates/harnesslab-adapters/src/protocol_artifact_contract.rs` — artifact contract validation
- `crates/harnesslab-adapters/src/data_boundary_contract.rs` — static boundary scanning
- `crates/harnesslab-adapters/src/data_boundary_rule_sets.rs` — boundary rule sets
- `crates/harnesslab-adapters/src/data_boundary_scan.rs` — source code scanner
- `crates/harnesslab-adapters/src/scaffold_golden_adapter.rs` — golden adapter
- `crates/harnesslab-adapters/src/protocol_contract_tests.rs` — protocol tests
- `crates/harnesslab-adapters/src/data_contract_tests.rs` — data contract tests
- `crates/harnesslab-core/src/adapter_protocol.rs` — core protocol types
- `crates/harnesslab-cli/src/runner/external/runtime_adapter.rs` — runtime adapter trait + dispatch

**Change Introduction (Neutral):**
The adapter layer has been built through multiple phases:
- Phase 0-1: Data adapter trait, boundary contracts, data lifecycle
- Phase 2-3: Runtime adapter trait, runtime registry, snapshots
- Phase 4-8: Runtime extraction, cleanup, docs, diagnostics, closure
- Protocol phases: Identity model, registry, lifecycle contracts, artifact
  contracts, no-branch guard, replay authority, compatibility profiles,
  scaffold golden adapter, migration preservation, extension proof, promotion.

**Risk Focus:**
1. Are there inconsistencies between the data adapter layer and runtime adapter
   layer that could cause runtime failures?
2. Does the data boundary contract scanner have blind spots?
3. Are there concurrency or state-consistency issues in the registry?
4. Is the `protocol_descriptor()` method optional but effectively required for
   new adapters — and what happens when it returns `None`?
5. Are there error handling gaps where errors are silently swallowed?
6. Is the `fnv64` checksum appropriate for its use cases, or could collisions
   cause replay failures?
7. Does the `runtime_adapter_for_task` function handle all edge cases correctly?
8. Are there test gaps in error paths, especially for the protocol contract
   validators?
9. Is the `attempt_scope()` obfuscation (`concat!("att", "empt")`) robust?
10. Are there any hardcoded assumptions that will break when a 4th adapter is added?

**Assumptions to Attack:**
- The data boundary scanner catches all forbidden patterns.
- `fnv64` checksums are collision-resistant enough for task identity.
- The `protocol_descriptor()` returning `None` is safe for legacy adapters.
- Registry validation at construction time catches all invalid states.
- The `runtime_adapter_for_task` function correctly handles both protocol-bound
  and legacy tasks.
- All validation functions are idempotent and side-effect-free.
- The source scanner correctly handles all Rust syntax edge cases.

**Adversarial Lenses:**
- Architecture: consistency between data and runtime layers, protocol evolution
- State: registry immutability, validation idempotency
- Input: edge cases in protocol IDs, artifact paths, capability sets
- Concurrency: thread safety of static adapter instances
- Failure: error propagation, panic safety, missing error paths
- Data: boundary contract completeness, checksum collision risk
- Security: path traversal in artifact paths, injection in protocol IDs
- Maintenance: cost of adding a 4th adapter, template drift
- Testing: error path coverage, negative test completeness
- Observability: logging gaps in validation failures

**Verification Status:**
- `cargo test -p harnesslab-adapters` — comprehensive test suite
- `cargo test -p harnesslab-core` — protocol type tests
- `cargo test -p harnesslab-cli` — runtime adapter tests
- `cargo run -p xtask -- verify-forbidden-diff` — passes
- `scripts/verify-planned-adapter-selectors.sh` — passes
- Known unverified: no fuzz testing of protocol IDs, no concurrency stress tests,
  no adversarial path traversal tests beyond the basic ones in tests.

**Reviewer Instructions:**
- Fresh session. Do not inherit main-agent context or reasoning.
- Read targets directly from the repository.
- Do not modify files.
- Cite evidence paths and line numbers when possible.
- Try to disprove the implementation. Look for counterexamples, hidden risks,
  and failure scenarios.
- Focus on high-impact failure modes, not style preferences.
- Pay special attention to cross-cutting concerns that span multiple files.

### Reviewer Launch Record

| Role | Code reviewer / architecture challenger |
| Mechanism | internal subagent (Task tool, general_purpose_task) |
| Context forked? | No (fresh session with navigation packet only) |
| Input packet | Above |
| Read-only instructions | Yes |

### Reviewer Output

**Summary:**
Four blocking findings, five non-blocking risks, seven missing test categories,
and four observability gaps were identified. The most impactful findings are:
(1) boundary scanner misses `#[cfg(all(test, ...))]` attributes, (2)
`runtime_cleanup_targets` silently swallows adapter resolution errors,
(3) `runtime_adapters()` uses a hardcoded array size that breaks on new
adapters, and (4) `built_in_protocol_adapter_descriptors()` only returns 2 of
the 3 registered adapters.

**Blocking Findings:**

1. **BLOCKER-1: Boundary scanner misses `#[cfg(all(test, ...))]`**
   - Broken assumption: The scanner catches all cfg-test-gated code.
   - Failure scenario: `#[cfg(all(test, feature = "x"))]` modules are not
     stripped, so their forbidden imports would be scanned as production code.
   - Trigger: Any test module gated with `#[cfg(all(test, ...))]`.
   - Impact: False-positive build failures or, worse, false negatives if the
     scanner is relied upon for boundary enforcement.
   - Evidence: `data_boundary_scan.rs:383`, `data_boundary_contract.rs:107`
   - **Status: accepted and fixed.**

2. **BLOCKER-2: `runtime_cleanup_targets` silently swallows errors**
   - Broken assumption: Errors are always propagated.
   - Failure scenario: `runtime_adapter_for_task` fails for a task → cleanup
     targets are silently skipped with no log, warning, or event.
   - Trigger: Any task with malformed `runtime_binding` or missing
     `external_runner` when `runtime_binding` is also `None`.
   - Impact: Resource leaks (Docker containers, temp files, tokens).
   - Evidence: `runtime_adapter.rs:144`
   - **Status: accepted and fixed.**

3. **BLOCKER-3: `runtime_adapters()` hardcoded array size `[_; 2]`**
   - Broken assumption: Adding a 4th adapter is safe.
   - Failure scenario: A new runtime adapter is added but the array size is not
     updated. `runtime_adapter_for_adapter_id` cannot find the new adapter.
   - Trigger: Adding any new benchmark that needs a runtime adapter.
   - Impact: Runtime failure with "unknown runtime adapter_id".
   - Evidence: `runtime_adapter.rs:94`
   - **Status: accepted and fixed.**

4. **BLOCKER-4: `built_in_protocol_adapter_descriptors()` only returns 2 of 3**
   - Broken assumption: Registry validation covers all adapters.
   - Failure scenario: `DeterministicSampleAdapter` is in the registry but not
     in `built_in_protocol_adapter_descriptors()`. Protocol contract tests
     (`adapt_protocol_003/004/005`) only validate 2 adapters.
   - Trigger: Adding a 4th adapter and forgetting to add it here.
   - Impact: New adapter's protocol contracts are not validated in the test
     suite.
   - Evidence: `protocol_contract_builtins.rs:5-13`,
     `protocol_contract_tests.rs:12`
   - **Status: accepted and fixed.**

**Non-blocking Risks:**

1. **RISK-1: `stable_file_checksum` silently falls back on read failure**
   - `registry.rs:184-189`: When `std::fs::read` fails, the function returns a
     `"missing:{path}"` checksum. The error is swallowed — callers cannot
     distinguish between a real checksum and a missing-file fallback.
   - **Status: acknowledged. Low risk; the fallback is deterministic and
     distinct from real checksums.**

2. **RISK-2: Boundary scanner does not handle raw string literals**
   - `data_boundary_scan.rs:259-302`: Raw strings (`r#"..."#`) are not stripped,
     so forbidden literals inside them would trigger false positives.
   - **Status: acknowledged. No raw strings currently in production code.**

3. **RISK-3: Character literal stripping is incomplete**
   - `data_boundary_scan.rs:351-354`: Escape sequences like `'\n'`, `'\t'`,
     `'\\'` are not handled.
   - **Status: acknowledged. No such literals currently in production code.**

4. **RISK-4: `attempt_scope()` obfuscation is defensive but unnecessary**
   - `protocol_artifact_contract.rs:133-135`: `concat!("att", "empt")` avoids
     the scanner catching the literal `"attempt"`. The scanner already strips
     artifact declarations before checking path literals.
   - **Status: acknowledged. Harmless but indicates incomplete trust in the
     scanner.**

5. **RISK-5: Adapter count inconsistency across the codebase**
   - 3 registry bindings, 2 runtime adapters, 2 in `built_in_protocol_adapter_descriptors()`
   - Adding a 4th adapter requires manual updates in 7+ locations.
   - **Status: acknowledged. Partially addressed by BLOCKER-3 and BLOCKER-4 fixes.**

**Missing Tests (identified, not all fixed in this round):**

1. `runtime_adapter_for_task` when both `runtime_binding` and `external_runner` are `None`
2. `runtime_adapter_for_task` when `runtime_binding` is `Some` but `external_runner` is `None`
3. `runtime_cleanup_targets` error-swallowing behavior (now logged)
4. Boundary scanner with `#[cfg(all(test, ...))]` (now tested)
5. Boundary scanner with raw string literals
6. `protocol_descriptor()` returning `None` on a production adapter
7. `stable_file_checksum` fallback behavior

**Missing Logs/Observability (identified, partially addressed):**

1. `runtime_cleanup_targets` now logs when skipping tasks (fixed)
2. `stable_file_checksum` fallback still unlogged
3. `adapter_for_with_root` returning `None` still unlogged
4. Protocol validation failures only appear in panic messages, not structured logs

### Main Agent Response

All four blocking findings are **accepted** and fixed:

1. **BLOCKER-1 fix**: Added `#[cfg(all(test` to the cfg-test stripping logic in
   both `data_boundary_scan.rs:383` and `data_boundary_contract.rs:107`. Added
   a regression test in `assert_boundary_scanner_regressions`.

2. **BLOCKER-2 fix**: Changed `runtime_cleanup_targets` from `let Ok(adapter) =
   ... else { continue; }` to a proper `match` with `eprintln!` warning on the
   error branch (`runtime_adapter.rs:143-157`).

3. **BLOCKER-3 fix**: Changed `runtime_adapters()` return type from
   `[&'static dyn BenchmarkRuntimeAdapter; 2]` to `Vec<&'static dyn
   BenchmarkRuntimeAdapter>` (`runtime_adapter.rs:94`).

4. **BLOCKER-4 fix**: Added `DeterministicSampleAdapter` to
   `built_in_protocol_adapter_descriptors()` in `protocol_contract_builtins.rs`.
   Updated the `assert_eq!(descriptors.len(), 2)` assertion to `3` in
   `protocol_contract_tests.rs:12`.

### Validation

- `cargo test -p harnesslab-adapters`: 36 passed, 0 failed
- `cargo test -p harnesslab-cli --lib`: 127 passed, 0 failed
- `cargo test -p harnesslab-core`: all passed
- Pre-existing xtask test failures (`registry_008`, `registry_013`) confirmed
  unrelated to these changes (fail on clean `main` too)

### Closure Status

All four accepted blocking findings have been implemented and validated. The
non-blocking risks are acknowledged and tracked. The missing tests for error
paths (items 1-3, 5-7) and missing logs (items 2-4) are deferred to future
work.

**Status: closed.**
