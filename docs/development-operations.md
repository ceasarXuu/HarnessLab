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
brew install docker-compose docker-buildx
colima start --cpu 4 --memory 8 --disk 60 --runtime docker --vm-type vz --mount-type virtiofs --mount /Users/xuzhang:w --mount /Volumes/XU-1TB-NPM:w --save-config
```

存储约束：

- `~/.colima` 软链接到 `/Volumes/XU-1TB-NPM/devtools/containers/colima`。
- `~/.lima` 软链接到 `/Volumes/XU-1TB-NPM/devtools/containers/lima`。
- VM、Docker 镜像、容器层和 Colima 数据都应落在外置盘；Docker CLI 配置 `~/.docker` 保留在 home，体积很小。
- 必须显式挂载 `/Volumes/XU-1TB-NPM:w`，否则 Docker 容器里看不到外置盘项目目录，HarnessLab 的 bind mount 会变成空目录。
- Docker Compose/buildx 是官方 Terminal-Bench CLI 的实际依赖。Homebrew 插件安装后，`~/.docker/config.json` 需要包含 `/opt/homebrew/lib/docker/cli-plugins` 作为 `cliPluginsExtraDirs`。

已验证命令：

```bash
docker run --rm hello-world
docker run --rm -v /Volumes/XU-1TB-NPM/projects/HarnessLab:/workspace -w /workspace alpine:3.20 sh -lc 'test -f Cargo.toml'
target/debug/harnesslab --home <external-temp-home> run --agent fake --benchmark terminal-bench --split smoke --json
```

关键通过信号：

- `colima list` 显示 default profile 为 `Running`，`4` CPU、`8GiB` memory、`60GiB` disk、runtime 为 `docker`。
- `docker info` 能返回 server 信息。
- HarnessLab `doctor --json` 中 `docker.daemon` 为 `ok`。
- `terminal-bench` smoke run 返回 `status = success`，`results.json` 中 `success = 1`。

## Local Benchmark Data

项目内 benchmark 数据放在 `.benchmarks/`，该目录必须被 `.gitignore` 忽略，避免把大体积数据集、上游任务文件或下载缓存提交进仓库。

已下载数据：

- Terminal-Bench: `.benchmarks/terminal-bench/terminal-bench-core-0.1.1`
  - 下载命令：`uvx --from terminal-bench tb datasets download --dataset terminal-bench-core==0.1.1 --output-dir .benchmarks/terminal-bench/terminal-bench-core-0.1.1 --overwrite`
  - 校验信号：`find .benchmarks/terminal-bench/terminal-bench-core-0.1.1 -name task.yaml | wc -l` 输出 `80`。
- SWE-bench Pro: `.benchmarks/swe-bench-pro/ScaleAI__SWE-bench_Pro`
  - 下载命令：`huggingface-cli download ScaleAI/SWE-bench_Pro --repo-type dataset --local-dir .benchmarks/swe-bench-pro/ScaleAI__SWE-bench_Pro --max-workers 4`
  - 校验信号：`data/test-00000-of-00001.parquet` 有 `731` 行。

注意：

- Terminal-Bench 官方 registry 的 `terminal-bench-core==head` 当前下载会尝试复制临时 clone 下不存在的 `tasks/` 目录；本地使用固定版本 `0.1.1`，避免 head 漂移影响复现。
- 下载日志可以临时放在 `.benchmarks/_logs/`，同样不追踪。
- HarnessLab 已能发现 `.benchmarks/` 下的真实数据，并在 `benchmark info` 中报告本地 task count：Terminal-Bench `80`，SWE-bench Pro `731`。
- 当前真实数据已接入 HarnessLab external runner。`terminal-bench` 通过官方 `tb run` 执行，`swe-bench-pro` 通过官方 local Docker evaluator 执行；`smoke` 用于快速真实链路验证，`full` 会基于本地数据枚举完整 task 集。

## Official Terminal-Bench Run

官方 Terminal-Bench CLI 在 Colima 上运行时，Python Docker SDK 不会自动继承 Docker CLI context。执行 `tb run` 前需要显式设置 socket：

```bash
export DOCKER_HOST="unix://$HOME/.colima/default/docker.sock"
uvx --from terminal-bench tb run \
  --dataset-path .benchmarks/terminal-bench/terminal-bench-core-0.1.1 \
  --task-id hello-world \
  --agent oracle \
  --n-concurrent 1 \
  --n-attempts 1 \
  --output-path .benchmarks/_runs/terminal-bench-official \
  --run-id oracle-hello-world-compose-timeout600 \
  --no-upload-results \
  --global-agent-timeout-sec 120 \
  --global-test-timeout-sec 600 \
  --log-level info
```

已验证信号：

- Docker/Compose/buildx 可用后，`hello-world` + `oracle` 能完整跑完，`results.json` 中 `accuracy = 1.0`、`n_resolved = 1`。
- HarnessLab external runner 已能用 `tb-oracle` profile 跑通真实 `terminal-bench/smoke`，并在 run 目录同时保存官方 `results.json`、agent/test 日志、HarnessLab `results.json` 和 `report.html`。
- 首次运行时容器内测试阶段可能卡在 Debian apt 下载；120 秒曾触发 `test_timeout`，600 秒完成。不要把这个误判为 agent 或 HarnessLab runner 失败。
- 结果文件位于 `.benchmarks/_runs/terminal-bench-official/<run-id>/results.json`，容器内 agent/test 日志在同级任务目录下。
- `run-id` 必须全小写；Docker Compose 会把它拼进 project name，包含大写 `T/Z` 的时间戳会触发 `invalid project name`。

### HarnessLab Terminal-Bench Real-Run Checks

真实验证 Terminal-Bench runner 时，优先使用 `.benchmarks/` 下的小子集目录，保持流程经过 HarnessLab CLI，而不是临时脚本绕过：

```bash
HARNESSLAB_BENCHMARKS_DIR=.benchmarks/_terminal-bench-subset-20260601T031542 \
  target/debug/harnesslab \
  --home .benchmarks/_harnesslab-home-terminal-real \
  run --agent claude-ds --benchmark terminal-bench --split full \
  --concurrency 1 --timeout-sec 180
```

检查顺序：

- 先查 `results.json`，确认 `failure_class/failure_code` 和官方 `results[].failure_mode` 一致。官方 `agent_timeout` 必须映射为 HarnessLab `execution/agent_timeout`，不能误报成 `benchmark/test_failed`。
- 再查 `run-health.json`，确认 `agent_timeouts`、`docker_network_failures`、`completed` 与结果一致。少量真实任务中有 agent timeout 但未到阈值时，`status` 仍应为 `ok`。
- 再查 `report.html`，确认任务明细展示 `execution/agent_timeout` 等 snake_case 分类，避免报告层把 JSON 修复掩盖掉。
- 最后查 Docker 残留：`docker network ls --filter label=com.docker.compose.project` 和 `docker ps -a --filter label=com.docker.compose.project` 不应留下本次 run id 对应的资源。

真实运行中看到 `terminal-bench cleanup post_task ... projects=none removed containers=0 networks=0` 不代表 HarnessLab 没有保护；通常是官方 Terminal-Bench 已先执行 `docker compose down`，HarnessLab fallback 只是在确认并清理遗留资源。

## Official SWE-bench Pro Run

SWE-bench Pro 官方仓库和运行产物放在 `.benchmarks/` 下，避免进入 git 跟踪：

```bash
git clone --depth 1 https://github.com/scaleapi/SWE-bench_Pro-os.git .benchmarks/_src/SWE-bench_Pro-os
```

本机已验证 local Docker evaluator 能跑通 public 数据第一条 instance 的 gold patch：

```bash
cd .benchmarks/_src/SWE-bench_Pro-os
DOCKER_HOST="unix://$HOME/.colima/default/docker.sock" uv run \
  --with docker --with pandas --with tqdm --with pyarrow --with modal --with datasets \
  python swe_bench_pro_eval.py \
  --raw_sample_path ../../_runs/swe-bench-pro-official/gold-first/raw_sample.csv \
  --patch_path ../../_runs/swe-bench-pro-official/gold-first/gold_patches.json \
  --output_dir ../../_runs/swe-bench-pro-official/gold-first/eval \
  --scripts_dir run_scripts \
  --dockerhub_username jefzda \
  --use_local_docker \
  --docker_platform linux/amd64 \
  --num_workers 1 \
  --redo
```

已验证信号：

- 本地数据 `.benchmarks/swe-bench-pro/ScaleAI__SWE-bench_Pro/data/test-00000-of-00001.parquet` 有 `731` 行。
- 官方 evaluator 使用 `jefzda/sweap-images:<dockerhub_tag>` 预构建镜像，不需要本机构建全部镜像。
- Apple Silicon 上必须显式使用 `--docker_platform linux/amd64`，否则可能拉取不到匹配镜像。
- gold patch 首条 instance 输出 `Overall accuracy: 1.0`，结果在 `.benchmarks/_runs/swe-bench-pro-official/gold-first/`。
- HarnessLab external runner 已能用 `swe-gold` profile 跑通真实 `swe-bench-pro/smoke`，从 parquet 抽取 instance metadata，准备官方 Docker image `/app` workspace，捕获 `patch.diff`/`prediction.jsonl`，调用官方 evaluator，并生成 HarnessLab `results.json` 和 `report.html`。
- `uv run` 过程中如果宿主 Python 环境泄漏出 NumPy 1.x 编译扩展 warning，只要 evaluator 继续运行且最终 accuracy 输出正常，不要误判为官方 evaluator 失败；后续可通过更干净的 uv venv 固化。
