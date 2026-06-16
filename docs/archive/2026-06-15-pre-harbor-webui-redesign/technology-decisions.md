# HarnessLab 关键技术选型

> 本文锁定 HarnessLab MVP 的开发与测试工程关键技术选型。目标是简洁、优雅、轻量、性能优先，同时不牺牲可测试性和可复现性。

## 1. 选型原则

| 原则 | 约束 |
|---|---|
| Single binary first | 用户优先通过一个 `harnesslab` CLI 二进制使用，不要求理解 runtime 依赖。 |
| File-first state | MVP 不引入数据库，run state、artifact、report 都落到文件系统。 |
| Process boundary | Benchmark、agent、Docker 都通过明确进程边界接入，避免把外部生态强耦合进 core。 |
| Minimal async | MVP 默认同步 worker pool；只有真正需要持续 IO multiplexing 时才引入 async runtime。 |
| Typed contracts | 配置、snapshot、result、report model 都先定义强类型结构，再序列化。 |
| Testability by design | 关键依赖必须可注入，支持 seeded failure、negative control、resume/replay 测试。 |
| Dependency budget | 每个生产依赖必须有明确职责；能用标准库稳定解决的，不加依赖。 |

## 2. 总体结论

| Area | Decision |
|---|---|
| 实现语言 | Rust stable，Edition 2024，精确版本由 `rust-toolchain.toml` 固定。 |
| CLI | `clap` derive。 |
| 配置格式 | TOML for user/editable manifests；JSON/JSONL for machine artifacts。 |
| 序列化 | `serde` + `serde_json` + `toml`。 |
| 进程执行 | 标准库 `std::process::Command` + 小型 timeout/kill 封装。 |
| PTY | `portable-pty`，仅用于 `input_mode = "tty"`。 |
| Docker | 调用 Docker CLI，不直接依赖 Docker SDK。 |
| 并发 | 固定容量 worker pool + channel；默认并发 4。 |
| Artifact store | 文件系统目录 + atomic write + append-only JSONL event log。 |
| Report | `askama` 生成单文件 HTML，内联最小 CSS/JS。 |
| Logging/events | 自研 typed event model，写 `events.jsonl`；console 输出独立渲染。 |
| Test runner | `cargo nextest` for fast tests；`cargo test --doc` for doctests。 |
| Coverage | `cargo llvm-cov`，输出 Cobertura、LCOV/JSON，门禁 line/branch/function。 |
| Mutation | `cargo-mutants`，M13 起覆盖 critical modules。 |
| Snapshots/golden | `insta` 用于 report/model golden，必须人工可读 diff。 |
| Packaging | Cargo binary first；后续再加 Homebrew/tap 和 install script。 |
| CI | GitHub Actions；required jobs 和 artifact retention 按 `docs/architecture/test-engineering.md`。 |

## 3. Language: Rust

### Decision

MVP 使用 Rust 实现一个 workspace：

```text
crates/
  harnesslab-cli/
  harnesslab-core/
  harnesslab-adapters/
  harnesslab-infra/
  harnesslab-report/
xtask/
```

### Why

- 性能足够好：进程调度、artifact 扫描、日志流、HTML 生成都能在低 overhead 下完成。
- 单二进制分发：个人开发者不需要额外 Python/Node runtime。
- 类型系统适合表达 run lifecycle、failure taxonomy、adapter contract。
- 测试工程匹配：LLVM coverage 可以导出 line/function/branch 数据；mutation testing 有 `cargo-mutants`。
- 依赖可控：标准库覆盖文件、进程、线程、路径、原子 rename 等核心需求。

### Rejected

| Option | Reason |
|---|---|
| Python | Benchmark 生态接入方便，但运行时依赖、分发、性能和类型边界不如 Rust；function/branch coverage 需要更多自定义胶水。 |
| Go | 单二进制和并发优秀，但 coverage 维度与既定 line/branch/function gate 不如 LLVM 方案直接。 |
| Node/TypeScript | CLI 开发快，但依赖树、单机性能和系统进程控制不如 Rust 简洁。 |
| Rust core + Python adapters | 过早复杂化，MVP 不做多语言插件 runtime。 |

## 4. CLI And Command Model

### Decision

CLI 使用 `clap` derive，命令结构固定：

```text
harnesslab init
harnesslab agent list
harnesslab doctor [--json]
harnesslab benchmark list
harnesslab benchmark info <benchmark>
harnesslab run --agent <name> --benchmark <name> --split <split>
harnesslab run resume <run-dir>
harnesslab run replay <run-dir>
harnesslab report open latest|<run-dir>
```

CLI 只做 parse、validation、human output dispatch，不直接写 artifact、不拼 Docker 命令、不生成 HTML。

### Pass Standard

- `CLI-*` tests cover command parse, invalid arguments, exit code, `--json` stability.
- `clap` generated help is snapshotted with stable normalization.
- CLI layer imports application services, never imports Docker/report renderer internals.

## 5. Config And File Formats

### Decision

Use only two config/data families:

| Use | Format | Examples |
|---|---|---|
| User editable config | TOML | `~/.harnesslab/config.toml`, `~/.harnesslab/agents/*.toml` |
| Test engineering manifests | TOML | `tests/TEST_REGISTRY.toml`, `tests/REQUIREMENTS.toml`, `coverage-critical.toml` |
| Machine snapshots/results | JSON | `run.json`, `task.snapshot.json`, `result.json`, `manifest.json` |
| Event streams and predictions | JSONL | `events.jsonl`, `prediction.jsonl` |
| Human report | HTML | `report.html` |

YAML is intentionally not used in HarnessLab-owned files. It is pleasant for examples, but Rust YAML support is fragmented and adds avoidable parser risk. HarnessLab may read upstream benchmark YAML if a benchmark requires it, but that parsing stays inside the adapter boundary.

### Pass Standard

- All user config structs derive `Serialize`, `Deserialize`, and schema/example generation helpers.
- All machine artifacts are canonical JSON with deterministic key ordering where applicable.
- Atomic writes use temp file in the same directory, fsync where practical, then rename.
- No production code depends on `serde_yaml` or YAML forks for HarnessLab-owned config.

## 6. Process, PTY, And Docker

### Process Execution

MVP uses `std::process::Command` behind `ProcessExecutor`.

Required behavior:

- stream stdout/stderr to files without loading full logs into memory.
- enforce timeout with child-group kill.
- record command, cwd, env allowlist, exit status, duration.
- expose fake executor for deterministic unit/contract tests.

### PTY

`portable-pty` is used only when an agent profile requires terminal semantics. Non-PTY execution remains the default because it is faster, simpler, and easier to test.

### Docker

MVP calls Docker CLI through `ProcessExecutor`.

Why not Docker SDK:

- User requirement is "Docker installed", which implies Docker CLI availability.
- CLI calls are easy to snapshot, log and reproduce.
- Docker SDK introduces heavier dependency surface and daemon API version handling.
- Core still depends only on `SandboxProvider`, so a future SDK/cloud provider remains possible.

Pass standard:

- every Docker command is built by typed command builders, not ad hoc strings.
- `doctor` runs Docker dry-run checks before `run`.
- orphan cleanup is idempotent and covered by fake/real smoke tests.

## 7. Concurrency And Performance

### Decision

Use a bounded synchronous worker pool for task execution.

MVP does not use Tokio by default. Docker, agent CLI and verifier execution are external process dominated; default concurrency is 4, so thread-based workers are simpler and performant enough.

Rules:

- scheduler owns queueing and backpressure.
- worker count is configurable and capped by profile/benchmark limits.
- stdout/stderr streaming never buffers whole logs in memory.
- artifact collection uses streaming copy and size limits.
- report generation reads `results.json` and task summaries, not full raw logs.

Performance baselines remain those in `docs/archive/stubs/mvp-development-spec.md`.

## 8. Artifact Store

### Decision

Use filesystem as the only MVP store:

```text
runs/<run-id>/
  run.json
  command.txt
  snapshots/
    config.snapshot.json
    agent-profile.snapshot.json
    benchmark.snapshot.json
    environment.snapshot.json
  results.json
  events.jsonl
  report.html
  tasks/
    <task-id>/
      task.snapshot.json
      attempts/
        <n>/
          instruction.md
          agent/
            stdout.log
            stderr.log
            result.json
          verifier/
            stdout.log
            stderr.log
            result.json
          artifacts/
            manifest.json
          diff.patch
          prediction.jsonl
          result.json
```

No SQLite in MVP.

Why:

- run directories are easy to inspect, archive, diff, replay and share.
- one run is one directory, matching the product's single-run first scope.
- database schema migration is unnecessary before cross-run analytics exists.

SQLite may be introduced later only for cross-run index/search, never as the source of truth for a run.

## 9. Report Rendering

### Decision

Use `askama` compile-time templates for `report.html`.

Rules:

- report model is pure data and independent of HTML.
- HTML is single-file for summary/table/filter UI.
- raw logs and large artifacts stay linked by relative path, not inlined.
- template golden tests normalize timestamps, run IDs and paths.

No React/Vue/Svelte for MVP report. It would add build tooling and dependency weight without solving the core experiment-record use case.

## 10. Test Engineering Stack

### Required Tools

| Tool | Purpose |
|---|---|
| `cargo nextest` | fast unit/contract/integration test execution. |
| `cargo llvm-cov` | line/function/branch coverage reports. |
| `cargo-mutants` | mutation testing for critical modules. |
| `insta` | report/model snapshot and golden diff. |
| `assert_cmd` | CLI black-box tests. |
| `tempfile` | isolated HarnessLab home and run dirs. |
| `proptest` | targeted property tests for config merge, redaction and state transitions. |

### Coverage

The default coverage gate:

```text
cargo +nightly-2026-05-26 llvm-cov test --workspace --all-features --exclude xtask --branch --no-report
cargo +nightly-2026-05-26 llvm-cov report --lcov --output-path coverage/lcov.info
cargo run -p xtask -- check-coverage --lcov coverage/lcov.info --min-line 95 --min-branch 70
cargo +nightly-2026-05-26 llvm-cov report --cobertura --output-path coverage/cobertura.xml
cargo +nightly-2026-05-26 llvm-cov report --json --output-path coverage/coverage.json
```

Branch coverage may require a pinned coverage toolchain until stable Rust exposes the required instrumentation cleanly. Production builds stay on stable Rust; the coverage toolchain is test-only and recorded in `rust-toolchain.coverage.toml` if needed.

Required implementation gate:

- line coverage `>= 95%`
- branch coverage `>= 70%` until the pinned Rust coverage path reports stable branch counters for this code shape.
- function coverage is covered by the encoded waiver in `coverage-critical.toml`; line coverage remains the hard 95% minimum.
- critical modules as defined in `coverage-critical.toml`: core `>= 98%` line and CLI `>= 95%` line / `>= 70%` branch.

M0 activates the documented function-coverage waiver in `coverage-critical.toml`. This is temporary until the Rust coverage path can report function coverage without duplicate unexecuted binary-test monomorphizations.

### Mutation

M13 hardening runs:

```text
cargo mutants --package harnesslab-core --timeout 120
```

Critical modules must hit `>= 80%` mutation kill rate or document a concrete waiver.

## 11. Dependency Budget

Initial production dependencies should fit this budget:

| Category | Allowed |
|---|---|
| CLI | `clap` |
| Serialization | `serde`, `serde_json`, `toml` |
| Errors | `thiserror`; `anyhow` only at CLI/application boundary |
| Time | `time` |
| Templates | `askama` |
| PTY | `portable-pty` |
| Glob/walk | `globset`, `walkdir` or `ignore` |
| Checksums | `blake3` for internal artifacts |
| Progress | `indicatif` |
| YAML | Adapter boundary only, only when an upstream benchmark requires YAML parsing. Requires a short ADR naming the benchmark and crate. |

Not allowed in MVP without a new ADR:

- async runtime as a global default.
- embedded database.
- Docker SDK.
- frontend framework for report.
- YAML parser for HarnessLab-owned config.
- dynamic plugin runtime.

## 12. CI Selection

Use GitHub Actions for MVP.

Shell scripts under `scripts/` are stable user/CI entrypoints. Any non-trivial parsing, TOML validation, traceability generation or coverage checking must live in `xtask`, so shell remains glue rather than business logic.

Required jobs:

```text
fmt
clippy
unit-contract-nextest
integration-fast
coverage
new-file-coverage
registry-check
traceability-check
report-golden
security-redaction
docs-link-check
```

Nightly/manual jobs:

```text
docker-full
terminal-bench-smoke
swe-bench-pro-smoke
mutation-critical
resume-replay-e2e
```

Artifacts:

- `coverage/cobertura.xml`
- `coverage/coverage.json`
- `artifacts/test-traceability.json`
- report golden diffs
- local gate log

Retention target: at least 90 days for required CI artifacts.

## 13. M0 Deliverables

M0 must commit:

```text
rust-toolchain.toml
rust-toolchain.coverage.toml
tools.versions.toml
Cargo.toml
crates/harnesslab-*/Cargo.toml
xtask/
scripts/test-after-change.sh
tests/REQUIREMENTS.toml
tests/TEST_REGISTRY.toml
coverage-critical.toml
.github/workflows/ci.yml
```

M0 pass standard:

- `cargo fmt --check` passes.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passes.
- `cargo nextest run --workspace --all-features` passes.
- coverage reports are generated in required formats.
- registry and traceability checks fail on seeded missing entries.
- fake secret scan fails on seeded leaked secret.
- repository has no YAML dependency for HarnessLab-owned config.

## 14. Reference Links

- `clap` derive docs: <https://docs.rs/clap/latest/clap/_derive/>
- Serde derive docs: <https://serde.rs/derive.html>
- TOML crate docs: <https://docs.rs/toml/latest/toml/>
- `portable-pty` docs: <https://docs.rs/portable-pty/>
- `askama` docs: <https://docs.rs/askama/latest/askama/>
- cargo-nextest docs: <https://www.nexte.st/>
- cargo-llvm-cov repository: <https://github.com/taiki-e/cargo-llvm-cov>
- cargo-mutants docs: <https://mutants.rs/>
