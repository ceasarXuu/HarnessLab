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
- `xtask check-coverage` 输出 line coverage `>= 95%`、branch coverage `>= 70%`，并额外执行 critical module 阈值。
- `xtask check-coverage` 会同时检查 `coverage-critical.toml` 中的 critical module 阈值。
- `scripts/check-new-file-coverage.sh` 会检查新增生产 Rust 文件是否进入 LCOV 报告。
- `coverage/cobertura.xml` 和 `coverage/coverage.json` 都生成。

注意：

- 当前 function coverage 在 M0 被编码化 waiver 替代，因为 cargo-llvm-cov 会把 CLI binary integration test 中未执行的重复 monomorphization 计入函数总数。
- 该 waiver 记录在 `coverage-critical.toml`，不是人工跳过。
- 新增生产 Rust 文件必须出现在 LCOV 中；纯 DTO / type-only 文件允许没有 executable line 记录。

## Docker / External Benchmark Smoke

当前开发机可能没有安装 Docker CLI。遇到这种环境时不要伪造真实 Docker smoke 成功，应验证两类信号：

- `harnesslab doctor --json` 明确报告 `docker.daemon` 为 `error`，message 为 Docker CLI/daemon 的真实状态。
- `terminal-bench` / `swe-bench-pro` 的 smoke adapter 可以生成 run plan；若运行时 Docker 不存在，run 应落到结构化 `sandbox_create_failed`，并写入 `results.json` 与 HTML 报告。

测试策略：

- Docker 命令构造和生命周期用可注入 fake runner 覆盖，不依赖本机 Docker。
- 外部 benchmark smoke 的本机负路径用 CLI 黑盒测试覆盖，确保缺 Docker 时用户看到配置/环境问题，而不是进程崩溃。
- run 执行前后会按 `harnesslab.run_id` 做 best-effort Docker orphan cleanup，并把结果写入 `events.jsonl` 的 `docker_cleanup` 事件；Docker 缺失时 cleanup warning 不应阻断报告生成。
- `input_mode = "file"` 的 instruction 文件必须写入 workspace。容器任务传入 `/workspace/instruction.txt`，host 任务传入宿主机 workspace 内路径，避免 agent 看到未挂载的宿主机路径。
- 真 Docker 正向 smoke 只应在 Docker CLI 和 daemon 可用时执行，不能作为本地 coverage gate 的必要条件。

## Local Colima Setup

本机使用轻量 Docker 方案：

```bash
brew install colima docker
colima start --cpu 4 --memory 8 --disk 60 --runtime docker --vm-type vz --mount-type virtiofs --mount <home>:w --mount <external-volume>:w --save-config
```

存储约束：

- `~/.colima` 软链接到 `<external-volume>/devtools/containers/colima`。
- `~/.lima` 软链接到 `<external-volume>/devtools/containers/lima`。
- VM、Docker 镜像、容器层和 Colima 数据都应落在外置盘；Docker CLI 配置 `~/.docker` 保留在 home，体积很小。
- 必须显式挂载 `<external-volume>:w`，否则 Docker 容器里看不到外置盘项目目录，HarnessLab 的 bind mount 会变成空目录。

已验证命令：

```bash
docker run --rm hello-world
docker run --rm -v <repo-root>:/workspace -w /workspace alpine:3.20 sh -lc 'test -f Cargo.toml'
target/debug/harnesslab --home <external-temp-home> run --agent fake --benchmark terminal-bench --split smoke --json
```

关键通过信号：

- `colima list` 显示 default profile 为 `Running`，`4` CPU、`8GiB` memory、`60GiB` disk、runtime 为 `docker`。
- `docker info` 能返回 server 信息。
- HarnessLab `doctor --json` 中 `docker.daemon` 为 `ok`。
- `terminal-bench` smoke run 返回 `status = success`，`results.json` 中 `success = 1`。
