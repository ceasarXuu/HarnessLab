# Playbook: 用 claude-ds 跑一次 Terminal-Bench 实验

本文面向第一次使用 HarnessLab 的用户，目标是用最少步骤完成一次真实实验：

1. 注册本机 `claude-ds` CLI agent。
2. 通过 HarnessLab CLI 运行 `terminal-bench`。
3. 获取 `results.json` 和 HTML 报告。
4. 判断结果是 agent 能力问题，还是执行环境/适配层问题。

整个流程必须通过 HarnessLab 正式命令完成，不需要写临时脚本。

## 前置条件

本机需要先满足这些条件：

- Docker CLI 和 Docker daemon 可用。Mac 上推荐 Colima 或 Docker Desktop。
- 本机命令行可以直接运行 `claude-ds`。
- HarnessLab 已经构建或安装。本文示例使用仓库内二进制 `target/debug/harnesslab`；如果已经安装到 `PATH`，可以把命令里的 `target/debug/harnesslab` 替换成 `harnesslab`。
- Terminal-Bench 数据已经下载到 benchmark root 下。默认推荐放在 `.benchmarks/terminal-bench/terminal-bench-core-0.1.1`。

下载 Terminal-Bench 数据：

```bash
uvx --from terminal-bench tb datasets download \
  --dataset terminal-bench-core==0.1.1 \
  --output-dir .benchmarks/terminal-bench/terminal-bench-core-0.1.1 \
  --overwrite
```

校验数据：

```bash
find .benchmarks/terminal-bench/terminal-bench-core-0.1.1 -name task.yaml | wc -l
```

如果输出是 `80`，说明 full split 的任务数据已经在本机可用。

## 1. 初始化 HarnessLab home

HarnessLab 的 agent 配置和实验记录保存在 home 目录。普通使用建议放到 `~/.harnesslab`：

```bash
export HARNESSLAB_HOME="$HOME/.harnesslab"
target/debug/harnesslab --home "$HARNESSLAB_HOME" init
```

`init` 会创建：

- `$HARNESSLAB_HOME/config.toml`
- `$HARNESSLAB_HOME/agents/`
- `$HARNESSLAB_HOME/runs/`
- `$HARNESSLAB_HOME/benchmarks/`

它也会打印自动检测到的内置 agent，例如 `codex-default`、`claude-code-default`、`opencode-default`、`pi-coding-agent-default`。

`claude-ds` 通常是用户自己的 wrapper/alias，不一定会被内置检测自动创建，所以接下来手动添加一个 profile。

## 2. 注册 claude-ds agent

Agent 注册是 HarnessLab 的第一体验。用户或另一个辅助 agent 只需要写一份可读注册表：说明这个 harness 怎么启动、继承哪些认证、启用/禁用哪些 skills/tools/hooks。HarnessLab 负责校验、解释并展开为 sandbox 运行细节。

先确认 HarnessLab 仓库绝对路径：

```bash
pwd
```

然后创建 `$HARNESSLAB_HOME/agents/claude-ds.toml`。把示例里的 `<HARNESSLAB_REPO>` 替换成上一步看到的仓库绝对路径，例如 `<repo-root>`：

```toml
schema_version = 1
name = "claude-ds"
kind = "claude-code"
display_name = "Claude Code via DeepSeek API"
command = "claude-ds -p --bare --output-format text"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 300

[auth]
inherit = true
inherit_env = [
  "ANTHROPIC_BASE_URL",
  "ANTHROPIC_AUTH_TOKEN",
  "API_TIMEOUT_MS",
  "ANTHROPIC_MODEL",
  "ANTHROPIC_SMALL_FAST_MODEL",
  "ANTHROPIC_DEFAULT_OPUS_MODEL",
  "ANTHROPIC_DEFAULT_SONNET_MODEL",
  "ANTHROPIC_DEFAULT_HAIKU_MODEL",
  "CLAUDE_CODE_SUBAGENT_MODEL",
  "CLAUDE_CODE_DISABLE_NONESSENTIAL_TRAFFIC",
  "CLAUDE_CODE_DISABLE_NONSTREAMING_FALLBACK",
  "CLAUDE_CODE_EFFORT_LEVEL",
]
include_paths = []
exclude_paths = []
mount_ssh_socket = false
mount_docker_socket = false

[setup]
preset = "builtin"
required_commands = ["claude", "claude-ds"]
run_as = "harnesslab"
commands = []

[skills]
inherit = true
allow = []
deny = []
include_paths = []

[tools]
inherit = true
allow = []
deny = []

[hooks]
inherit = true
allow = []
deny = []

[usage]
parser = "none"
source = "agent_logs"
input_tokens_key = "input_tokens"
output_tokens_key = "output_tokens"
total_tokens_key = "total_tokens"
cost_usd_key = "cost_usd"

[labels]
terminal_bench_agent_import_path = "harnesslab_tb_agent:HarnessLabCommandAgent"
terminal_bench_agent_pythonpath = "<HARNESSLAB_REPO>/integrations/terminal_bench"
model = "deepseek"
```

关键字段说明：

| 字段 | 取值范围 | 示例 | 说明 |
| --- | --- | --- | --- |
| `schema_version` | 当前固定 `1` | `1` | profile schema 版本。 |
| `name` | `[a-zA-Z0-9][a-zA-Z0-9._-]*` | `claude-ds` | `--agent` 使用的 profile 名称。 |
| `kind` | `codex` / `claude-code` / `opencode` / `pi-coding-agent` / `custom` / `fake` | `claude-code` | 用于选择默认检测、认证继承、内置 setup 和 adapter 行为。 |
| `display_name` | 任意可读字符串 | `Claude Code via DeepSeek API` | 报告中展示的名称。 |
| `command` | shell 命令字符串 | `claude-ds -p --bare --output-format text` | HarnessLab 启动 agent 的命令模板。不要直接写密钥。 |
| `input_mode` | `stdin` / `argument` / `file` / `tty` | `stdin` | 任务说明如何传给 agent。 |
| `working_dir` | `workspace` / `run_dir` | `workspace` | agent 的启动目录。 |
| `timeout_sec` | 正整数秒 | `300` | profile 默认 agent 预算；Terminal-Bench task 自带更小预算时会被 task 配置收敛。 |
| `auth.inherit` | `true` / `false` | `true` | 是否继承 `inherit_env` 和 `include_paths` 中声明的认证/配置。 |
| `auth.inherit_env` | 环境变量名数组 | `["ANTHROPIC_AUTH_TOKEN"]` | 允许进入运行环境的变量名，只写变量名，不写值。 |
| `auth.include_paths` | 路径数组，支持 `host:container:mode` | `[]` | 显式挂载的认证/配置路径。 |
| `auth.exclude_paths` | 路径数组 | `[]` | 从继承路径中排除的路径。 |
| `mount_ssh_socket` | `true` / `false` | `false` | 是否挂载 SSH agent socket。 |
| `mount_docker_socket` | `true` / `false` | `false` | 是否挂载 Docker socket；默认禁止。 |
| `setup.preset` | `none` / `builtin` / `custom` | `builtin` | `builtin` 用 HarnessLab 内置逻辑安装/准备该 kind 的 CLI；`custom` 才允许 `setup.commands`。 |
| `setup.required_commands` | 命令名数组 | `["claude", "claude-ds"]` | setup 后必须存在的命令。包含 `claude-ds` 时，Claude Code adapter 会在容器内生成 wrapper。 |
| `setup.run_as` | `root` / `harnesslab` / `current` | `harnesslab` | agent 命令在 sandbox 内的运行用户。 |
| `setup.commands` | shell 命令数组 | `[]` | 高级 escape hatch，仅 `preset = "custom"` 时允许。 |
| `skills.inherit` | `true` / `false` | `true` | 是否继承该 agent kind 的默认 skills。 |
| `skills.allow` | 字符串数组 | `[]` | skills 白名单；空数组表示不额外限制。 |
| `skills.deny` | 字符串数组 | `[]` | skills 黑名单；不能和 `allow` 重复。 |
| `skills.include_paths` | 路径数组 | `[]` | 额外 skills 路径。 |
| `tools.inherit` | `true` / `false` | `true` | 是否继承默认 tools。 |
| `tools.allow` | 字符串数组 | `[]` | tools 白名单。 |
| `tools.deny` | 字符串数组 | `[]` | tools 黑名单；不能和 `allow` 重复。 |
| `hooks.inherit` | `true` / `false` | `true` | 是否继承默认 hooks。 |
| `hooks.allow` | 字符串数组 | `[]` | hooks 白名单。 |
| `hooks.deny` | 字符串数组 | `[]` | hooks 黑名单；不能和 `allow` 重复。 |
| `usage.parser` | `none` / `regex` / `json_path` | `none` | token/cost 采集方式。 |
| `terminal_bench_agent_import_path` | Python import path | `harnesslab_tb_agent:HarnessLabCommandAgent` | 接入官方 Terminal-Bench `--agent-import-path`。 |
| `terminal_bench_agent_pythonpath` | 绝对路径 | `<HARNESSLAB_REPO>/integrations/terminal_bench` | Terminal-Bench bridge 代码目录，不要保留 `<HARNESSLAB_REPO>` 字面量。 |
| `labels.model` | 字符串 | `deepseek` | 报告展示用模型/配置标签。 |

如果你的 `claude-ds` wrapper 使用不同环境变量，只需要调整 `inherit_env`。不要把 API key 直接写进 `command`。

### 2.1 setup 和能力开关规则

| 段落 | 字段 | 取值范围 | 说明 |
| --- | --- | --- | --- |
| `setup` | `preset` | `none` / `builtin` / `custom` | `builtin` 使用 HarnessLab 内置 setup；`custom` 才允许高级命令。 |
| `setup` | `required_commands` | 命令名数组 | setup 后必须能找到的命令，预检查失败则阻断 run。 |
| `setup` | `run_as` | `root` / `harnesslab` / `current` | agent 在 sandbox 内的运行用户。 |
| `setup` | `commands` | shell 命令数组 | 仅 `preset = "custom"` 时允许。 |
| `skills` | `inherit` | `true` / `false` | 是否继承本机/agent 默认 skills。 |
| `skills` | `allow` | 字符串数组 | skills 白名单；非空时只启用这些 skills。 |
| `skills` | `deny` | 字符串数组 | skills 黑名单；不能和 `allow` 重复。 |
| `skills` | `include_paths` | 路径数组 | 额外 skills 路径。 |
| `tools` | `inherit` | `true` / `false` | 是否继承默认 tools。 |
| `tools` | `allow` | 字符串数组 | tools 白名单。 |
| `tools` | `deny` | 字符串数组 | tools 黑名单；不能和 `allow` 重复。 |
| `hooks` | `inherit` | `true` / `false` | 是否继承默认 hooks。 |
| `hooks` | `allow` | 字符串数组 | hooks 白名单。 |
| `hooks` | `deny` | 字符串数组 | hooks 黑名单；不能和 `allow` 重复。 |

当前版本会严格阻断无法 materialize 的非默认 `skills/tools/hooks` 策略。也就是说，声明了白名单/黑名单但对应 harness adapter 还不能证明可执行时，`doctor` 和 `run` 会提前报错，而不是静默忽略。

## 3. 运行预检查

先检查 agent profile、Docker、benchmark 数据和 usage 配置：

```bash
HARNESSLAB_BENCHMARKS_DIR=.benchmarks \
  target/debug/harnesslab --home "$HARNESSLAB_HOME" doctor
```

再确认 HarnessLab 能发现 Terminal-Bench 数据：

```bash
HARNESSLAB_BENCHMARKS_DIR=.benchmarks \
  target/debug/harnesslab --home "$HARNESSLAB_HOME" benchmark info terminal-bench
```

期望看到：

- `smoke` split 可运行。
- `full` split 可运行。
- full task count 为 `80`。

如果 `data_state=not_downloaded` 或 `data_state=corrupted`，先修数据路径，不要开始跑分。

## 4. 启动实验

第一次建议先跑 smoke，确认 Docker、认证、agent wrapper、Terminal-Bench adapter 都连通：

```bash
HARNESSLAB_BENCHMARKS_DIR=.benchmarks \
  target/debug/harnesslab --home "$HARNESSLAB_HOME" run \
  --agent claude-ds \
  --benchmark terminal-bench \
  --split smoke \
  --concurrency 1 \
  --timeout-sec 1800 \
  --json
```

真实完整实验使用 full split：

```bash
HARNESSLAB_BENCHMARKS_DIR=.benchmarks \
  target/debug/harnesslab --home "$HARNESSLAB_HOME" run \
  --agent claude-ds \
  --benchmark terminal-bench \
  --split full \
  --concurrency 4 \
  --timeout-sec 1800 \
  --json
```

参数说明：

- `--agent claude-ds` 选择刚注册的 profile。
- `--benchmark terminal-bench` 选择 Terminal-Bench adapter。
- `--split smoke` 只跑快速连通性任务。
- `--split full` 跑本地 Terminal-Bench 数据中的完整任务集。
- `--concurrency 4` 表示最多 4 个 task 并发。机器资源紧张时可以改成 `1` 或 `2`。
- `--timeout-sec 1800` 是用户给本次 run 的 agent 预算上限。Terminal-Bench task 自带更小的 `max_agent_timeout_sec` 时，HarnessLab 不会放大官方 task 的 agent 预算。
- `--json` 会输出机器可读的 run 路径、报告路径和 summary。

运行中 HarnessLab 会在控制台打印进度条、成功数量和失败数量。每个 task 执行后，HarnessLab 会清理对应的 Terminal-Bench compose container/network，避免一次实验污染下一次实验。

## 5. 读取命令输出

`--json` 输出大致如下：

```json
{
  "schema_version": 1,
  "command": "run",
  "status": "success",
  "exit_code": 0,
  "verdict": "benchmark_failure",
  "run_id": "claude-ds-terminal-bench-full-20260602T190745032129Z",
  "run_dir": "/path/to/.harnesslab/runs/claude-ds-terminal-bench-full-20260602T190745032129Z",
  "results_path": "/path/to/.harnesslab/runs/claude-ds-terminal-bench-full-20260602T190745032129Z/results.json",
  "report_path": "/path/to/.harnesslab/runs/claude-ds-terminal-bench-full-20260602T190745032129Z/report.html",
  "summary": {
    "total_tasks": 2,
    "success": 1,
    "partial_success": 0,
    "benchmark_failure": 1,
    "execution_failure": 0,
    "interrupted": 0,
    "total_duration_ms": 757006,
    "total_score": 1.0
  },
  "replay_source_run_id": null
}
```

注意两个概念：

- `status = "success"` 表示 HarnessLab CLI 成功完成并写出了实验产物，不表示所有 task 都答对。
- `verdict = "benchmark_failure"` 表示实验有效完成，但至少一个 task 被 benchmark 判错或超时。

如果 `execution_failure > 0`，说明有执行环境、Docker、adapter、cleanup 或 runner 问题。此时不要把结果当成 agent 能力分数，先看失败明细并修复执行链路。

## 6. 查看 HTML 报告

用 JSON 输出里的 `report_path` 打开报告：

```bash
open "/path/to/.harnesslab/runs/<run-id>/report.html"
```

也可以打印最近一次报告路径：

```bash
target/debug/harnesslab --home "$HARNESSLAB_HOME" report open latest
```

HTML 报告包含：

- 总体 summary。
- agent 配置摘要。
- run health 状态。
- 每个 task 的 state、outcome、failure class/code、score、duration 和 warnings。
- agent stdout/stderr、verifier stdout/stderr 链接。
- `command.txt`、`agent-profile.snapshot.json`、`benchmark.snapshot.json` 等复现材料。

## 7. 查看结构化结果

读取总体结果：

```bash
jq '.summary' "/path/to/.harnesslab/runs/<run-id>/results.json"
```

读取每个 task 明细：

```bash
jq '.tasks[] | {
  task_id,
  state,
  outcome,
  failure_class,
  failure_code,
  score: .benchmark_score,
  duration_ms,
  warnings
}' "/path/to/.harnesslab/runs/<run-id>/results.json"
```

常见失败分类：

| failure_class | failure_code | 含义 | 是否可计入 agent 能力评估 |
| --- | --- | --- | --- |
| `none` | `null` | task 成功 | 是 |
| `benchmark` | `agent_timeout` | 官方 benchmark 判定 agent 超时 | 是 |
| `benchmark` | `test_failed` | 官方 verifier 判定答案错误 | 是 |
| `benchmark` | `agent_output_parse_error` | agent 输出无法被 benchmark 解析 | 是 |
| `execution` | `external_runner_setup_failed` | Docker/setup/官方 runner 启动失败 | 否 |
| `execution` | `external_runner_no_progress` | 官方 runner 长时间无有效进展，被 HarnessLab watchdog 终止 | 否 |
| `execution` | `external_runner_timeout` | 外层 runner 超过硬超时 | 否 |
| `execution` | `agent_cleanup_failed` | agent 或 compose cleanup 没有收敛 | 否 |

评估原则：

- `benchmark/*` 是 benchmark 对 agent 的有效判定。
- `execution/*` 是实验系统或本地环境问题，必须先解决，再重新跑。
- `usage_unknown` 不影响 benchmark score，只表示当前 profile 没有配置 token/cost 解析器，成本不可比较。

## 8. 复现一次 run

HarnessLab 支持两种复现方式。

第一种是复制当时的原始命令：

```bash
cat "/path/to/.harnesslab/runs/<run-id>/command.txt"
```

其中 `original_command=` 是当时启动 run 的命令。重新执行它时，会使用当前 home 中最新的 agent 配置和当前 benchmark 数据。

第二种是 replay，使用 run 目录里的 runtime snapshot 尽量还原当时配置：

```bash
target/debug/harnesslab --home "$HARNESSLAB_HOME" run replay \
  "/path/to/.harnesslab/runs/<run-id>" \
  --json
```

Replay 会读取：

- `run.json`
- `agent-profile.runtime.json`
- `benchmark.snapshot.json`
- `command.txt`

如果缺少 runtime profile snapshot，HarnessLab 会拒绝 replay，避免用已经被脱敏的 public snapshot 假装复现。

## 9. 示例真实结果

下面是一轮本机真实验证 run 的摘要。它使用 `claude-ds`，通过 HarnessLab 正式 CLI 跑 Terminal-Bench 的两个 QEMU task 子集，不是 mock，也不是临时脚本。

启动命令：

```bash
HARNESSLAB_BENCHMARKS_DIR=.benchmarks/_terminal-bench-two-qemu-root-20260602T164020Z \
  target/debug/harnesslab --home .benchmarks/_harnesslab-home-terminal-real \
  run --agent claude-ds --benchmark terminal-bench --split full \
  --concurrency 1 --timeout-sec 1800 --json
```

结果：

| task_id | state | failure_class | failure_code | score |
| --- | --- | --- | --- | --- |
| `build-initramfs-qemu` | `success` | `none` | `null` | `1` |
| `build-tcc-qemu` | `failure` | `benchmark` | `agent_timeout` | `0` |

总体：

- `total_tasks = 2`
- `success = 1`
- `benchmark_failure = 1`
- `execution_failure = 0`
- `total_score = 1.0`

这个结果说明 HarnessLab 的 Terminal-Bench 执行链路没有工程失败；`build-tcc-qemu` 是官方 benchmark 给出的 agent timeout，属于有效 benchmark failure。

## 10. 常见问题

### doctor 提示 Docker 不可用

先让 Docker daemon 运行起来，再执行：

```bash
docker info
```

`docker info` 不能成功时，不要启动完整 benchmark。

### benchmark info 显示 not_downloaded

确认 `HARNESSLAB_BENCHMARKS_DIR` 指向的是 benchmark root，而不是直接指向 `terminal-bench-core-0.1.1` 目录。

正确结构：

```text
.benchmarks/
  terminal-bench/
    terminal-bench-core-0.1.1/
      hello-world/
        task.yaml
```

### claude-ds not found

先在宿主机确认：

```bash
which claude-ds
claude-ds -p --bare --output-format text
```

如果宿主机可用但 benchmark 容器里不可用，先看 run 目录里的 `agent-runtime.materialized.json` 和 task 级 `agent_setup_*.log`，确认 `[setup] preset = "builtin"` 已安装 `claude` 并写入 `/usr/local/bin/claude-ds`。

### API key 没有传入

确认 `claude-ds.toml` 的 `auth.inherit_env` 包含你的实际认证环境变量，并在启动 HarnessLab 的 shell 里已经导出这些变量。

不要把密钥直接写进 profile 的 `command` 字段。

### 报告里有 execution failure

优先打开 task 明细里的 stdout/stderr 和 verifier 日志，再看 `events.jsonl`、`run-health.json`。只要 `execution_failure > 0`，这轮实验就不应该用于 agent 排名。

### 运行中断后继续

如果 run 目录已经生成，可以 resume：

```bash
target/debug/harnesslab --home "$HARNESSLAB_HOME" run resume \
  "/path/to/.harnesslab/runs/<run-id>" \
  --json
```

Resume 会复用 run 目录里的 runtime snapshot，继续补齐 pending/interrupted task，并重新生成报告。

## 最小路径清单

按最短路径完成一次实验：

```bash
export HARNESSLAB_HOME="$HOME/.harnesslab"

target/debug/harnesslab --home "$HARNESSLAB_HOME" init

# 编辑 $HARNESSLAB_HOME/agents/claude-ds.toml，填入本文 profile。

HARNESSLAB_BENCHMARKS_DIR=.benchmarks \
  target/debug/harnesslab --home "$HARNESSLAB_HOME" doctor

HARNESSLAB_BENCHMARKS_DIR=.benchmarks \
  target/debug/harnesslab --home "$HARNESSLAB_HOME" run \
  --agent claude-ds \
  --benchmark terminal-bench \
  --split smoke \
  --concurrency 1 \
  --timeout-sec 1800 \
  --json

open "$HARNESSLAB_HOME/runs/<run-id>/report.html"
```

Smoke 跑通后，把 `--split smoke` 改成 `--split full`，再根据机器资源把 `--concurrency` 调到 `2` 或 `4`。
