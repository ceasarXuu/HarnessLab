# Benchmark Adapter Phase 1 Selector Coverage Matrix

- Date: 2026-06-04
- Related plan: `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- Related inventory: `docs/plans/2026-06-04-benchmark-adapter-phase-1-inventory.md`
- Purpose: record which adapters, methods, and failure modes each active
  `ADAPT-DATA-*` selector exercises so route exactness is not mistaken for
  behavioral proof completeness.

| Selector | Methods Covered | Adapters Covered | Failure / Drift Coverage |
| --- | --- | --- | --- |
| `ADAPT-DATA-001` | `descriptor`, `inspect_data` | `terminal-bench`, `swe-bench-pro` | no cache mutation, content-level fingerprint, structured data/runtime boundary guard with module-graph coverage, dependency alias/package assertions, import/symbol/call/path assertions, ambient-env/process-state ban, `std::fs` read allowlist plus mutable filesystem denylist, production `#[path]` and `include!` bans, and runtime path-literal assertions |
| `ADAPT-DATA-002` | `prepare` | `terminal-bench`, `swe-bench-pro` | idempotent ready prepare; rejects `corrupted`, `partial`, and auth/missing data states |
| `ADAPT-DATA-003` | `list_tasks` | `terminal-bench`, `swe-bench-pro` | deterministic task ids and source refs; SWE prepared-data drift is rejected |
| `ADAPT-DATA-004` | `snapshot_task` | `terminal-bench`, `fake-patch`, `swe-bench-pro` | serializable identity; mutation-sensitive patch-style snapshot hashes; SWE upstream metadata hash changes when evaluator content changes |
| `ADAPT-DATA-005` | `create_task_plan`, `plan(split)` | `fake-terminal`, `fake-patch`, `terminal-bench`, `swe-bench-pro` | stable plan creation; wrapper equivalence; SWE wrapper fails on source/data skew after drift |

## Boundary Assertions

- The data adapter crate may construct `ExternalRunnerSpec` as a launch hint,
  but it must not execute processes, own attempt directories, write event logs,
  or depend on CLI/infra runtime execution modules.
- `docs/plans/2026-06-04-benchmark-adapter-phase-1-boundary.md` records the
  allowed production dependency set, forbidden import families, forbidden
  runtime symbols, forbidden runtime calls, allowed `std::fs` read calls,
  forbidden module path attributes, and covered source files for the
  `ADAPT-DATA-001` boundary assertion.
- The boundary assertion discovers production files from the non-test module
  graph rooted at `crates/harnesslab-adapters/src/lib.rs`, so newly added
  production helper modules are covered before Phase 1 can close.
- `ADAPT-DATA-001` registry file patterns include
  `crates/harnesslab-adapters/Cargo.toml`, fake adapter source files,
  production adapter source files, the boundary helper, the scan helper, and
  boundary/coverage docs because all of them are part of the proof surface.
- SWE-bench Pro planning must use prepared task ids and prepared source/data
  identity. Post-prepare filesystem drift must be detected explicitly instead
  of silently rebinding to live data.
- README/parquet task count and evaluator `run_scripts/<instance_id>` count must
  agree before SWE-bench Pro reports data as `ready`.
