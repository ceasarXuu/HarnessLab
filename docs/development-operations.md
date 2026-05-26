# HarnessLab 开发操作记录

本文记录容易踩坑的工程操作，避免后续从头排查。

## M0 Rust Tooling

M0 本地 gate 依赖：

```bash
cargo install cargo-nextest --version 0.9.136 --locked
cargo install cargo-llvm-cov --version 0.8.7 --locked
rustup toolchain install nightly-2026-05-26 --component llvm-tools-preview
```

原因：

- `cargo nextest` 用于快速测试执行；未安装时 gate 曾降级到 `cargo test`，但这不能作为最终验收。
- `cargo llvm-cov --branch` 会使用 nightly-only `-Z coverage-options=branch`，stable Rust 会失败；本项目固定使用 `nightly-2026-05-26`。
- 生产编译仍使用 `rust-toolchain.toml` 固定的 stable Rust；coverage 单独使用 `rust-toolchain.coverage.toml`。
- coverage gate 先执行一次带 instrumentation 的测试，再从同一份 profdata 导出 LCOV、Cobertura 和 JSON，避免三个报告来自不同测试轮次。

已验证命令：

```bash
scripts/test-after-change.sh
```

关键通过信号：

- nextest summary 显示所有测试通过且 `0 skipped`。
- `xtask check-coverage` 输出 line coverage `>= 97%`、branch coverage `>= 97%`。
- `xtask check-coverage` 会同时检查 `coverage-critical.toml` 中的 critical module 阈值。
- `scripts/check-new-file-coverage.sh` 会检查新增生产 Rust 文件是否进入 LCOV 报告。
- `coverage/cobertura.xml` 和 `coverage/coverage.json` 都生成。

注意：

- 当前 function coverage 在 M0 被编码化 waiver 替代，因为 cargo-llvm-cov 会把 CLI binary integration test 中未执行的重复 monomorphization 计入函数总数。
- 该 waiver 记录在 `coverage-critical.toml`，不是人工跳过。
- `crates/harnesslab-core/src` 在 M0 没有 LLVM branch counter，因此 critical config 只对它启用 line threshold；一旦该模块出现分支逻辑，必须恢复 branch threshold。
