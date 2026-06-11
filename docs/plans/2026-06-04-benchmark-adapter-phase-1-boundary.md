# Benchmark Adapter Phase 1 Boundary Contract

- Date: 2026-06-05
- Related plan: `docs/plans/2026-06-04-benchmark-adapter-architecture-design.md`
- Related selector: `ADAPT-DATA-001`
- Purpose: make the Phase 1 data/runtime boundary proof explicit and reviewable.

## Boundary Intent

Phase 1 data adapters may inspect benchmark data, prepare deterministic task
identity, create `TaskPlan` launch hints, and snapshot replay identity. They
must not execute processes, own attempt directories, write runtime events, or
depend on CLI/runtime infrastructure. Runtime execution remains Phase 3+
adapter scope.

The boundary test discovers covered production source files from the
non-`cfg(test)` file-module graph rooted at
`crates/harnesslab-adapters/src/lib.rs`. Inline production modules are
forbidden, and production `#[path]` module attributes are forbidden;
production helpers must be declared as normal file modules so they enter the
scanned graph and this artifact's covered source list.

## Allowed production dependencies

`crates/harnesslab-adapters` production dependency aliases and package names
are limited to:

| Dependency | Reason |
| --- | --- |
| `harnesslab-core` | Shared serializable benchmark, task, snapshot, and runner-hint types. |
| `serde` | Serialization support for core contract types. |
| `serde_json` | Stable task-plan hashing for `RuntimeTaskSnapshot`. |

Development-only test dependencies such as `tempfile` are allowed outside the
production dependency set.

## Forbidden imports

Production adapter source must not import:

| Import family | Reason |
| --- | --- |
| `std::env` | Ambient parent-process environment inspection belongs outside data adapters. |
| `std::process` | Process launch belongs to runtime adapters and shared execution infra. |
| `tokio::process` | Async process launch has the same runtime ownership boundary. |
| `harnesslab_cli` | Data adapters must not depend on CLI orchestration internals. |
| `harnesslab_infra` | Data adapters must not call shared runtime execution primitives directly. |

## Forbidden runtime symbols

The boundary test rejects direct references to these runtime symbols:

| Symbol | Runtime concern |
| --- | --- |
| `Command` | Host process construction. |
| `File` | Mutable file-handle access is outside the read-only data adapter surface. |
| `HostProcessExecutor` | Shared process execution primitive. |
| `ProcessExecutor` | Shared process execution primitive. |
| `ExecSpec` | Runtime execution command spec. |
| `DockerRunner` | Runtime/container execution. |
| `EventWriter` | Attempt/run event persistence. |
| `AttemptDir` | Attempt-directory ownership. |
| `ArtifactWriter` | Runtime artifact persistence. |
| `RunDir` | Run-directory ownership. |
| `OpenOptions` | Mutable file creation/update primitive. |
| `harnesslab_infra` | Runtime infrastructure dependency path or symbol. |

## Forbidden runtime calls

The boundary test rejects calls with these names after comments and string
literals are removed:

| Call | Runtime concern |
| --- | --- |
| `spawn` | Process launch. |
| `status` | Process execution status. |
| `output` | Process execution output capture. |
| `id` | Ambient process identity inspection. |
| `exit` | Ambient process termination. |
| `abort` | Ambient process termination. |
| `exec` | Runtime execution helper. |
| `exec_with` | Runtime execution helper. |
| `run_command` | Runtime execution helper. |
| `write_event` | Runtime event persistence. |
| `args` | Ambient process argument inspection. |
| `args_os` | Ambient process argument inspection. |
| `env` | Ambient environment/process-state helper. |
| `var` | Ambient environment inspection. |
| `var_os` | Ambient environment inspection. |
| `vars` | Ambient environment inspection. |
| `vars_os` | Ambient environment inspection. |
| `set_var` | Ambient environment mutation. |
| `remove_var` | Ambient environment mutation. |
| `current_dir` | Ambient process state inspection. |
| `set_current_dir` | Ambient process state mutation. |
| `current_exe` | Ambient process state inspection. |
| `temp_dir` | Ambient process/environment-derived path inspection. |
| `create` | Mutable file creation primitive. |
| `create_new` | Mutable file creation primitive. |
| `create_dir` | Directory ownership primitive. |
| `create_dir_all` | Directory ownership primitive. |
| `write` | Mutable file write primitive. |
| `write_all` | Mutable file write primitive. |
| `write_all_vectored` | Mutable file write primitive. |
| `copy` | Mutable file write primitive. |
| `hard_link` | Mutable filesystem ownership primitive. |
| `rename` | Mutable filesystem ownership primitive. |
| `remove_file` | Mutable filesystem ownership primitive. |
| `remove_dir` | Mutable filesystem ownership primitive. |
| `remove_dir_all` | Mutable filesystem ownership primitive. |
| `set_len` | Mutable file-handle resize primitive. |
| `set_permissions` | Mutable filesystem metadata primitive. |
| `set_times` | Mutable filesystem metadata primitive. |
| `symlink` | Mutable platform filesystem link primitive. |
| `symlink_file` | Mutable platform filesystem link primitive. |
| `symlink_dir` | Mutable platform filesystem link primitive. |

## Allowed std::fs read calls

Production adapters may call only this explicit `std::fs` read allowlist:

| Call | Data-adapter concern |
| --- | --- |
| `canonicalize` | Normalize discovered benchmark data/source paths. |
| `metadata` | Inspect file metadata without opening mutable handles. |
| `read` | Hash bounded fixture/source files for stable identity. |
| `read_dir` | Discover task directories and upstream source files. |
| `read_link` | Read symlink targets when future data discovery needs it. |
| `read_to_string` | Read small metadata files such as task YAML or README. |
| `symlink_metadata` | Inspect symlink metadata without following it. |

Any other qualified `std::fs::*` call is rejected by `ADAPT-DATA-001`, even if
it is not listed above in the generic call denylist.

## Forbidden runtime path literals

String literals in production adapter source must not contain these runtime
path tokens:

| Literal token | Runtime concern |
| --- | --- |
| `attempt` | Attempt-directory ownership. |
| `attempt_dir` | Attempt-directory ownership. |
| `run_dir` | Run-directory ownership. |
| `events.jsonl` | Runtime event persistence. |
| `external-runtime` | Runtime snapshot ownership. |

## Module graph coverage

The boundary assertion discovers files recursively from production `mod ...;`
declarations and rejects inline production `mod ... { ... }` declarations.
It also rejects production `#[path]` module attributes and `include!` source
inclusion, including macro syntax such as `include!(...)`, because redirected
or included files would otherwise bypass the normal file-module graph.
`#[cfg(test)]` modules are excluded because they are test scaffolding, not
production adapter behavior. Any new production helper module must appear in
this discovered file set and in the covered source list below.

## Forbidden module path attributes

Production source must not use `#[path = "..."] mod ...;`. The selector bans
this attribute instead of trying to resolve arbitrary redirected module files,
so the compiled helper set stays visible through normal module discovery.

## Covered source files

The boundary assertion covers:

- `crates/harnesslab-adapters/src/lib.rs`
- `crates/harnesslab-adapters/src/fake_patch.rs`
- `crates/harnesslab-adapters/src/fake_terminal.rs`
- `crates/harnesslab-adapters/src/protocol_contract_builtins.rs`
- `crates/harnesslab-adapters/src/protocol_contract.rs`
- `crates/harnesslab-adapters/src/protocol_registry.rs`
- `crates/harnesslab-adapters/src/registry.rs`
- `crates/harnesslab-adapters/src/swe_bench_pro.rs`
- `crates/harnesslab-adapters/src/swe_bench_pro_protocol.rs`
- `crates/harnesslab-adapters/src/terminal_bench.rs`
- `crates/harnesslab-adapters/src/terminal_bench_protocol.rs`

## Validation

`ADAPT-DATA-001` runs
`data_contract_tests::adapt_data_001_descriptor_and_inspect_data_do_not_mutate_cache`,
which calls `assert_data_adapter_boundary_contract()`. The assertion checks the
allowed production dependency alias/package set, this artifact, the discovered
non-test module graph, forbidden imports, forbidden runtime symbols, and
forbidden runtime calls, allowed `std::fs` read calls, module path-attribute
rules, and path literals.

The `ADAPT-DATA-001` registry row includes
`crates/harnesslab-adapters/Cargo.toml` because dependency alias/package purity
is an enforced input to this boundary proof. The dependency parser checks the
normal `[dependencies]` table and target-specific production dependency tables
such as `[target.'cfg(...)'.dependencies]`.
