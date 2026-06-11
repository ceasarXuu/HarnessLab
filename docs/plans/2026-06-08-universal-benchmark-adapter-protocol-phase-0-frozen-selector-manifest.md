# Universal Benchmark Adapter Protocol Phase 0 Frozen Selector Manifest

- Created: 2026-06-08
- Updated: 2026-06-08
- Source plan: `docs/plans/2026-06-08-universal-benchmark-adapter-protocol-implementation-plan.md`
- Phase: 0, Frozen Regression Selector Manifest
- Status: Implemented, pending closure review

## 1. Purpose

This manifest freezes the current selector evidence that must not be weakened
during the universal adapter protocol migration. Any migration phase that
removes, renames, weakens, or changes the expected behavior of these selectors
must either fail or record an equivalent-or-stronger replacement before
continuing.

## 2. Baseline Evidence

| Check | Command | Result |
|---|---|---|
| Adapter proof selector inventory | `CARGO_INCREMENTAL=0 scripts/verify-planned-adapter-selectors.sh` | passed; `adapter selectors ok: active=16 planned=1` |
| Frozen registry source | `tests/TEST_REGISTRY.toml` | current id/title/command/artifact/contract rows are locked by `tests/FROZEN_SELECTOR_MANIFEST.toml` |
| Frozen command router | `scripts/test-after-change.sh --select <id>` | every frozen selector must remain routed or be replaced by an equivalent-or-stronger registered selector |
| Machine guard | `cargo run -p xtask -- verify-frozen-selector-manifest` | passed; `frozen selector manifest ok: total=86 execution_files=7 ADAPT-DATA=6 ADAPT-PROTOCOL-001=1 ADAPT-PROTOCOL-002=1 ADAPT-RUNTIME=6 AGT-REG-005=1 AGT-REG-012=1 C-BENCH=10 DOC-004=1 INT=41 PY-TB-001=1 SEC=1 SWEPRO=5 TB=11` |

## 3. Weakening Rules

A migration has weakened this manifest if any frozen selector:

- disappears from `tests/TEST_REGISTRY.toml`
- changes from `active` to `planned`, `deprecated`, or unregistered
- loses required artifacts without a documented stronger replacement
- changes command to a narrower test without review
- drops expected failure-mode coverage
- no longer routes through `scripts/test-after-change.sh --select <id>`
- is removed from an adapter/protocol selector guard without equivalent
  replacement
- changes `file_patterns`, owning contracts, or expected selected-test count
  without updating this reviewed manifest

## 4. Machine-enforced Manifest Schema

The authoritative frozen selector lockfile is
`tests/FROZEN_SELECTOR_MANIFEST.toml`. Every row records:

| Field | Meaning | Guard Behavior |
|---|---|---|
| `execution_files` | Exact content hashes for `scripts/test-after-change.sh` and external proof scripts. | Common selector executor changes and external proof no-op replacements must fail until the reviewed lockfile is updated. |
| `id` | Exact selector id. | Must exist in `tests/TEST_REGISTRY.toml` and `scripts/test-after-change.sh`. |
| `status` | Current registry status. | Must not change from active/planned without review. |
| `command` | Exact selector command. | Must match registry command exactly. |
| `router_case` | Exact `scripts/test-after-change.sh` case body for the selector. | Must match the router exactly, so same-count test substitutions and shell proof no-ops fail. |
| `expected_test_count` | Selected Cargo test count when the route exposes one. | Must continue to match the router-inferred count. External shell proofs use no count and must exit successfully. |
| `expected_pass_threshold` | Human-readable pass condition. | Must be non-empty. |
| `file_patterns` | Registry files/contracts that define the proof surface. | Must match the registry and exist on disk. |
| `required_artifacts` | Runtime artifacts the selector proves. | Must match the registry exactly. |
| `owning_contracts` | Registry contract names that own the proof. | Must match `tests.verifies.contracts` exactly. |

## 5. Frozen Coverage Summary

| Family / Explicit ID | Frozen Count | Notes |
|---|---:|---|
| `ADAPT-DATA-*` | 6 | Includes planned sentinel `ADAPT-DATA-000` and active data adapter proofs. |
| `ADAPT-RUNTIME-*` | 6 | Runtime preflight, snapshots, cleanup, events, and internal error proofs. |
| `C-BENCH-*` | 10 | Built-in benchmark descriptor, discovery, split, and timeout metadata contracts. |
| `DOC-004` | 1 | Doctor benchmark readiness. |
| `INT-*` | 41 | External runtime, replay, report, redaction, resume, Terminal-Bench behavior, and malformed artifact behavior. |
| `SEC-*` | 1 | Public secret redaction. |
| `SWEPRO-*` | 5 | SWE-bench Pro phase failure and replay material proofs. |
| `TB-*` | 11 | Terminal-Bench runtime behavior contracts. |
| `PY-TB-001` | 1 | Terminal-Bench Python bridge and process cleanup. |
| `AGT-REG-005`, `AGT-REG-012` | 2 | Terminal-Bench registered setup and run-as/readiness behavior. |
| Total | 84 | Verified by `cargo run -p xtask -- verify-frozen-selector-manifest`. |

## 6. Notable Corrected Rows

| ID | Correction |
|---|---|
| `INT-011` | Required artifacts are no longer `[]`; the lockfile preserves all current SWE-bench Pro runtime artifacts including `run.json`, `command.txt`, `results.json`, `events.jsonl`, `report.html`, profile snapshots, task result, public/private runtime snapshots, agent/verifier logs, diff/eval artifacts, and SWE eval output. |
| `DOC-004` | Doctor readiness is explicitly frozen instead of being implied by broad `INT-*` coverage. |
| `PY-TB-001` | Terminal-Bench Python adapter support is explicitly frozen. |
| `AGT-REG-005`, `AGT-REG-012` | Registered setup and run-as readiness are explicitly frozen because they feed adapter readiness and compatibility behavior. |
| `C-BENCH-*` | Built-in benchmark descriptor/discovery contracts are frozen because registry behavior is part of the adapter migration boundary. |
| execution files | `scripts/test-after-change.sh` plus six Terminal-Bench external proof scripts are content-hashed so helper rewrites and proof-script no-ops cannot bypass the route lock. |

## 8. Phase 0 Exit Status

| Exit Criterion | Status | Evidence |
|---|---|---|
| Frozen ids listed with exact commands. | Met | `tests/FROZEN_SELECTOR_MANIFEST.toml` lists every frozen command. |
| Required artifacts listed where registry declares them. | Met | `tests/FROZEN_SELECTOR_MANIFEST.toml` locks current `required_artifacts` values and the guard compares them to registry rows. |
| Adapter selector baseline is green. | Met | `scripts/verify-planned-adapter-selectors.sh` passed with `active=16 planned=1`. |
| Expected counts and owning contracts are frozen. | Met | `tests/FROZEN_SELECTOR_MANIFEST.toml` records `expected_test_count`, `router_case`, execution file hashes, and `owning_contracts`; `xtask/src/frozen_selectors.rs` validates them. |
| Weakening rule documented and enforced. | Met | Section 3 defines weakening; `scripts/verify-test-registry.sh` runs the frozen selector guard. |

## 9. Next Required Work

- Extend this manifest with `ADAPT-PROTOCOL-*` rows as soon as Phase 1 creates
  planned registry entries.
- Treat any selector replacement as a reviewed plan update, not an incidental
  test registry edit.
