# HarnessLab 开发操作记录

本文记录容易踩坑的工程操作，避免后续从头排查。

## User Playbooks

- [Agent Registration Guide](agent-registration-guide.md)：面向第一次注册 agent 的用户，覆盖 init、schema、profile 编辑、doctor、最小 smoke run 和注册完成清单。
- [用 claude-ds 跑一次 Terminal-Bench 实验](playbooks/terminal-bench-claude-ds.md)：面向用户的最小使用路径，覆盖 agent 注册、预检查、run、结果查看、报告、replay/resume。
- [Agent Profile Reference](agent-profile-reference.md)：注册表字段、取值范围、materialization 支持矩阵和验证清单。
- [npm 包名和 CLI 命令占位发布](playbooks/npm-package-reservation.md)：记录 `@ceasarxuu/harnesslab`、`harnessrig`、`harnessyard` npm 包名和对应命令占位发布的预检、验证和发布复用步骤。

## M0 Rust Tooling

M0 本地 gate 依赖：

```bash
cargo install cargo-llvm-cov --version 0.8.7 --locked
rustup toolchain install nightly-2026-05-26 --component llvm-tools-preview
```

原因：

- `scripts/test-after-change.sh` 默认使用 `cargo test --workspace --all-features` 执行全量测试，避免本机 `cargo nextest` discovery 在 macOS pipe/list 阶段挂起时污染验收。
- 如需使用 nextest 快路径，显式运行 `HARNESSLAB_TEST_RUNNER=nextest scripts/test-after-change.sh`，并先安装 `cargo install cargo-nextest --version 0.9.136 --locked`。
- `cargo llvm-cov --branch` 会使用 nightly-only `-Z coverage-options=branch`，stable Rust 会失败；本项目固定使用 `nightly-2026-05-26`。
- 生产编译仍使用 `rust-toolchain.toml` 固定的 stable Rust；coverage 单独使用 `rust-toolchain.coverage.toml`。
- coverage gate 先执行一次带 instrumentation 的测试，再从同一份 profdata 导出 LCOV、Cobertura 和 JSON，避免三个报告来自不同测试轮次。

已验证命令：

```bash
scripts/test-after-change.sh
```

关键通过信号：

- `== tests ==` 阶段显示 `cargo test --workspace --all-features` 全部通过；如果显式选择 nextest，则 nextest summary 必须显示所有测试通过且 `0 skipped`。
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
colima start --cpu 4 --memory 8 --disk 60 --runtime docker --vm-type vz --mount-type virtiofs --mount <home>:w --mount <external-volume>:w --save-config
```

存储约束：

- `~/.colima` 软链接到 `<external-volume>/devtools/containers/colima`。
- `~/.lima` 软链接到 `<external-volume>/devtools/containers/lima`。
- VM、Docker 镜像、容器层和 Colima 数据都应落在外置盘；Docker CLI 配置 `~/.docker` 保留在 home，体积很小。
- Colima/Lima 的 home 目录必须位于支持 Unix socket 的文件系统上。若大盘不支持 socket，可把 `~/.colima` 保留在 NPM 盘，只把 `_lima/colima/disk` 和 `_lima/_disks/colima/datadisk` 软链接到大盘；不要把整个 Colima home 迁到不支持 socket 的盘，否则 `usernet` 会因 `bind: operation not supported` 无法启动。
- 必须显式挂载 `<external-volume>:w`，否则 Docker 容器里看不到外置盘项目目录，HarnessLab 的 bind mount 会变成空目录。
- Docker Compose/buildx 是官方 Terminal-Bench CLI 的实际依赖。Homebrew 插件安装后，`~/.docker/config.json` 需要包含 `/opt/homebrew/lib/docker/cli-plugins` 作为 `cliPluginsExtraDirs`。

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

验证 agent 注册和 materialized setup 时，使用正式黑盒脚本：

```bash
scripts/test-after-change.sh --select AGT-REG-005
```

该脚本会执行 `harnesslab init`、写入临时注册 profile、运行 `doctor --json`，再通过 `harnesslab run --agent registered-setup --benchmark terminal-bench --split smoke --json` 启动真实 Terminal-Bench import-agent 流程。验收证据必须包括 run 根目录的 `agent-runtime.materialized.json`、`command.txt`，以及官方 task log 目录里的 `agent_setup_command.sha256`、`agent_setup_stdout.log` 和 `agent_setup_stderr.log`；其中 `agent_setup_command.sha256` 要与 `agent-runtime.materialized.json.setup_script` 的 sha256 一致，`agent_setup_stdout.log` 要能证明 setup 在 registered agent 命令前运行。

Agent 注册相关变更的运维检查要同时覆盖这些 artifacts：

- `agent-profile.runtime.json`：private runtime snapshot，只用于 resume/replay，不作为可分享 artifact。
- `agent-profile.snapshot.json`：公开 redacted profile snapshot；`command`、`version_command`、`setup.commands`、known labels 中的 secret 必须被 redact。
- `agent-runtime.materialized.json`：公开 materialized runtime snapshot；必须包含结构化 `capabilities.*`，报告中的 effective capability 摘要应来自这里。
- `agent-version.snapshot.json`：存在 `version_command` 时生成；doctor/run/replay 都必须使用 bounded probe，stdout/stderr 只保留 redacted tail。
- `report.html`：必须链接 profile/runtime/version snapshots，并展示 effective capability sets 和 version probe status。
- `tasks/**/agent/command.txt`：公开 command snapshot；host、Docker、Terminal-Bench、SWE-bench Pro 路径都必须使用同一 public redaction 语义。

Replay 排查时不要只看当前 shell 环境。Replay 会从 source run 的 runtime profile 与 redacted report profile 差异中恢复 source-known redaction basis，用于继续 redact version probe、materialized setup、events 和 report。如果 replay 在当前环境变量缺失时泄漏 source run 已知 secret，优先检查 `runner/redaction.rs` 的 substring recovery 和 `runner/version.rs` 的 replay warning redaction。

Host auth 隔离排查时，用黑盒测试证明 ambient env 没有进入 host agent 进程。`auth.inherit=false` 不应传入父进程完整环境；`auth.inherit=true` 也只传入声明的 `inherit_env` 加 task env 和最小 launch baseline。Host run 的 `setup.run_as` 只能是 `current`；如果看到 host path 使用 `root` 或 `harnesslab` 没有被 precheck 阻断，这是运行语义 bug，不是 warning。

检查顺序：

- 先查 `results.json`，确认 `failure_class/failure_code` 和官方 `results[].failure_mode` 一致。官方结果里的 `agent_timeout` 是 benchmark verdict，必须映射为 HarnessLab `benchmark/agent_timeout`，不能误报成 `benchmark/test_failed` 或 HarnessLab 执行层超时。
- 再查 `events.jsonl`，确认每个 task 都写入了 `external_runner_configured`，其中包含有效的 `process_timeout_sec`、`no_output_timeout_sec` 和 `activity_grace_sec`；默认无日志 watchdog 应显示为具体秒数，不应是 `disabled`。
- 再查 `run-health.json`，确认 `agent_timeouts`、`external_runner_no_progress`、`external_runner_timeouts`、`execution_stalls`、`docker_network_failures`、`completed` 与结果一致。官方 `agent_timeout` 不计入执行层 `agent_timeouts`；只有 HarnessLab 自己杀掉进程的 agent 执行超时、runner hard timeout 或 no-progress 才会增加 `execution_stalls` 并可能触发 run-health abort。
- 再查 `report.html`，确认任务明细展示 `benchmark/agent_timeout`、`execution/external_runner_no_progress`、`execution/external_runner_timeout`、`execution/agent_cleanup_failed` 等 snake_case 分类；如果官方结果同时给成功任务带了 `failure_mode=agent_timeout`，报告应在 `Warnings` 列展示 `agent_timeout`。
- 最后查 Docker 残留：`docker network ls --filter label=com.docker.compose.project` 和 `docker ps -a --filter label=com.docker.compose.project` 不应留下本次 run id 对应的资源。

Terminal-Bench runner 有三层超时：

- 官方 agent/test 超时：来自 Terminal-Bench `task.yaml` 的 `max_agent_timeout_sec` 和 `max_test_timeout_sec`。HarnessLab 传给 `tb run` 的 `--global-agent-timeout-sec` 会在 agent timeout 基础上给 import-agent cleanup 增加 30 秒余量，传给 `tb run` 的 `--global-test-timeout-sec` 必须保持 benchmark verifier 自己的测试超时。用户 `--timeout-sec` 不能放大官方 verifier timeout。
- HarnessLab agent env 超时：传给 Python bridge 的 `HARNESSLAB_AGENT_TIMEOUT_SEC` 必须是 task agent timeout cap 之后的值。例如 QEMU task 的 `max_agent_timeout_sec=360.0` 时，即使命令行传 `--timeout-sec 1800`，bridge 也应看到 `HARNESSLAB_AGENT_TIMEOUT_SEC=360`。
- HarnessLab 进程守护超时：外层进程硬超时默认为 `agent_timeout + test_timeout + 1800`，为官方 setup、首次 Docker build、agent、verifier、cleanup 留出完整窗口。无日志输出 watchdog 默认为 `max(agent_timeout, test_timeout) + 120`，下限 `1800` 秒，并会被外层硬超时截断。Terminal-Bench 的 no-output watchdog 同时检查官方 `run.log` 是否增长，以及受管进程组内的 Docker setup/build 活动；首次构建镜像时即使 stdout/stderr 静默，只要 `run.log` 仍在推进，或短时间内仍有 `docker compose`、`docker-buildx`、`docker build` 或 `docker pull` 等活动进程，就不应触发 `external_runner_no_progress`。纯进程活动最多只能延期一个额外 watchdog 窗口；`docker exec` 不属于默认 setup/build 活动信号。

Terminal-Bench runtime 对普通 task 默认导出 `DOCKER_DEFAULT_PLATFORM=linux/amd64` 和 `BUILDKIT_PROGRESS=plain` 后再执行 `tb run`。对 `build-initramfs-qemu` 和 `build-tcc-qemu`，Apple Silicon 默认改用 `linux/arm64` 容器并交叉编译 x86_64 kernel/rootfs；真实报告里的 `external_runner_configured` 事件必须记录最终平台值。诊断其他平台时可以临时设置 `HARNESSLAB_TERMINAL_BENCH_DOCKER_PLATFORM=<platform>` 覆盖。
在 Apple Silicon 上，`build-initramfs-qemu` 和 `build-tcc-qemu` 的 amd64 setup build 可能因为 QEMU/binfmt 下的 GCC 崩溃而在 agent 启动前失败。HarnessLab 对这两个 task 做 attempt-local dataset 兼容准备：复制该 task 目录到当前 attempt；native arm64 模式给 Dockerfile 注入 `gcc-x86-64-linux-gnu` 并把 kernel build 改为 `make ARCH=x86_64 CROSS_COMPILE=x86_64-linux-gnu- -j$(nproc)`；强制 amd64 emulation 模式仅降级为 `make -j1`。该处理必须写入 `terminal_bench_dataset_prepared` 事件，且不得修改 `.benchmarks/terminal-bench/...` 原始数据。

如果官方 runner 长时间卡住，且既没有继续输出日志，也没有可接受的受控 Docker setup/build 活动 grace，HarnessLab 会杀掉整个进程组，写入 `external_runner_no_progress` 事件，并把任务标记为 `execution/external_runner_no_progress`。如果官方 runner 持续有活动但超过外层 hard timeout，HarnessLab 写入 `external_runner_timeout` 事件，并把任务标记为 `execution/external_runner_timeout`。这两类失败说明 benchmark runner 或本地 Docker 阶段有执行层问题，不应算作 agent 解题能力失败；`external_runner_timeout` 会触发 run-health abort，避免继续污染完整 bench。
如果官方 runner 已经写出 `results.json` 但随后被 HarnessLab hard timeout 或 no-progress watchdog 杀掉，执行层失败必须压过官方结果；官方结果只作为 verifier 日志或 `warnings[]` 辅助排查。
如果官方 Terminal-Bench 在 agent 启动前的 `docker compose build/up` 阶段失败，官方结果可能只给 `failure_mode=unknown_agent_error`。HarnessLab 必须扫描 `run.log/stdout/stderr` 中的 Docker compose setup failure，把结果覆盖为 `execution/external_runner_setup_failed`，并让 run-health 中止剩余 pending task；这类错误不计入 agent 解题能力。
当 no-output watchdog 因匹配到 Docker setup/build 活动或官方 `run.log` 进度而延期时，HarnessLab 会限流写入 `external_runner_activity` 事件。Docker 活动事件包含匹配的 pid、命令名和模式；进度文件事件包含 `run.log` 路径。进度文件会在进程运行期间持续采样，早期写入不会被延迟到 watchdog 边界才计时。活动消失后不会重新获得完整 watchdog 窗口；纯活动持续存在也只能延期一个额外 watchdog 窗口。只有官方日志文件真实增长才会刷新无日志窗口，下一次短周期复查若仍无进展会进入 no-progress 判定。最终 `external_runner_no_progress` 事件会包含 `activity_grace_exhausted`、`current_activity`、`last_activity` 和 `last_progress`，用于判断是没有任何活动、活动 grace 过期，还是进度文件增长后再次卡住。

调试真实卡死场景时可以临时设置 `HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC=<seconds>` 缩短 watchdog 等待；该值只用于本次进程，必须大于 `0`，并会被限制在外层硬超时之前。确实要允许长时间静默时，可显式设置为 `0`、`off`、`disabled` 或 `none` 关闭。
开发诊断或契约测试需要覆盖 hard-timeout 路径时，可以临时设置 `HARNESSLAB_TERMINAL_BENCH_PROCESS_TIMEOUT_SEC=<seconds>` 缩短 HarnessLab 外层进程守护时间；不设置时仍使用默认 `agent_timeout + test_timeout + 1800`。

如果官方结果出现 `failure_mode=parse_error`，HarnessLab 必须映射为 `benchmark/agent_output_parse_error`。常见原因是 agent 在 shell 脚本前输出自然语言前言；Terminal-Bench Python adapter 会剥离纯前言并从第一行 shell-looking 命令开始执行，但不得丢弃包含 shell 语义的内容。

使用 `terminal_bench_agent_import_path = "harnesslab_tb_agent:HarnessLabCommandAgent"` 接入本机 CLI agent 时，HarnessLab 传给适配层的 `HARNESSLAB_AGENT_TIMEOUT_SEC` 必须保持原始 agent 预算，传给官方 `tb run` 的 `--global-agent-timeout-sec` 会额外增加清理余量，避免官方外层 timeout 先中断 `perform_task`。排查真实 run 时，如果看到 `agent_timeout` 且宿主机仍有对应 agent 子进程，优先修适配层进程树清理，而不是继续跑完整 bench。

真实 full run 监控时，宿主机不得出现 PPID 为 `1` 的残留 agent 子进程，例如 `claude --dangerously-skip-permissions ...`。Terminal-Bench Python adapter 在 agent timeout 和正常 agent 命令退出后都会按运行期 ancestry 快照与 `HARNESSLAB_AGENT_RUN_TOKEN` 扫描并清理残留子进程，并在官方 task log 目录写出 `agent_cleanup.log`。如果 agent 主动清除 token 并快速 daemonize，adapter 不能安全强杀无法归属的新进程；需要诊断这类风险时可启用 strict global process scan，把 agent 窗口内新出现且无法归属的 live pid 作为 `execution/agent_cleanup_failed` 证据。如果仍出现 orphan，或结果中出现 `execution/agent_cleanup_failed`，必须提前终止 run 并修复清理边界，不能把后续结果当成有效 benchmark 分数。

Terminal-Bench post-task compose cleanup 失败同样必须映射为 `execution/agent_cleanup_failed`，即使官方 `results.json` 已经给出 success 或 benchmark failure。cleanup failure 表示运行环境没有收敛，可能继续占用 Docker network/address pool；不能只写入 `events.jsonl` warning 后把任务计入有效 benchmark 得分。

不要为了节省时间把 `HARNESSLAB_TERMINAL_BENCH_NO_OUTPUT_TIMEOUT_SEC` 设得小于或接近 agent 预算。真实 Terminal-Bench 中部分 agent 生成脚本后会在容器内长时间无官方 `run.log` 增量，过短 watchdog 会把本应由官方 runner 给出的 `benchmark/agent_timeout` 或成功结果误判为 `execution/external_runner_no_progress`。

真实运行中看到 `terminal-bench cleanup post_task ... projects=none removed containers=0 networks=0` 不代表 HarnessLab 没有保护；通常是官方 Terminal-Bench 已先执行 `docker compose down`，HarnessLab fallback 只是在确认并清理遗留资源。

run 级 cleanup 会同时清理已经记录在 `terminal-bench-compose-projects.json` 的项目，并按 Terminal-Bench 官方 run id 归一化规则生成 scan token，扫描仍带有对应 run token 的 compose project。pre-run stale cleanup 会读取旧 sibling run 的 `run.json`，用旧 run 自己的 `run_id` 扫描；即使旧 run 没有 project snapshot，只要目录名能识别为 Terminal-Bench run，也会尝试兜底扫描。这个机制覆盖 active task 尚未进入 `post_task`、但 run 因内部错误或健康 abort 退出的场景。

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
