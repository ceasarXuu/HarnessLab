# Agent Registration Guide

本文面向第一次注册 agent 的 HarnessLab 用户。目标是完成一条稳定流程：

1. 初始化 HarnessLab home。
2. 选择要注册的 CLI agent 类型。
3. 创建或编辑一个 agent profile。
4. 用 `doctor` 做预检查。
5. 运行一次最小 benchmark，确认注册结果真的可用。

如果你只想查字段含义和完整取值范围，看 [Agent Profile Reference](agent-profile-reference.md)。

## 你正在注册什么

HarnessLab 注册的不是“一个模型名”，而是一个完整 CLI harness profile。一个 profile 包含：

- 启动命令，例如 `codex exec`、`claude -p`、`opencode run` 或自定义命令。
- 认证继承，例如允许进入运行环境的环境变量和配置目录。
- setup 规则，例如运行前需要安装或检查哪些命令。
- skills、tools、hooks 策略。
- usage parser 和报告标签。

如果同一个 CLI agent 有两个不同模型、不同 wrapper、不同权限或不同工具策略，应注册成两个 profile。这样每次实验的运行条件才可比较、可复现。

## 0. 准备 CLI

本文使用安装后的命令名：

```bash
harnesslab --help
```

如果你在仓库源码里直接运行，可以把下面命令里的 `harnesslab` 替换为：

```bash
target/debug/harnesslab
```

## 1. 初始化 HarnessLab home

HarnessLab home 保存配置、agent profiles、runs 和 benchmark 数据索引。普通用户建议使用默认 home：

```bash
export HARNESSLAB_HOME="$HOME/.harnesslab"
harnesslab --home "$HARNESSLAB_HOME" init
```

初始化后你会看到：

```text
$HARNESSLAB_HOME/
  config.toml
  agents/
  runs/
  benchmarks/
```

`agents/` 目录里会生成内置 agent 草稿和一个 README。先不要直接开始跑 benchmark，先完成下面的 profile 检查。

## 2. 查看注册字段

注册前先看 schema。它能告诉你哪些字段必填、有哪些取值，以及当前版本支持哪些语义段落：

```bash
harnesslab agent schema
harnesslab agent schema --json
```

普通用户可以看文本输出；如果你让另一个 agent 帮你生成 profile，优先给它 `--json` 输出。

当前 profile 主要字段包括：

- `name`：`--agent` 使用的名字。
- `kind`：agent 类型，例如 `codex`、`claude-code`、`opencode`、`pi-coding-agent`、`custom`。
- `command`：实际启动 agent 的命令。
- `[auth]`：允许继承哪些认证信息。
- `[setup]`：运行前如何准备 CLI 和 wrapper。
- `[skills]`、`[tools]`、`[hooks]`：能力策略。
- `[usage]`：token/cost 采集方式。
- `[labels]`：报告和 benchmark adapter 使用的标签。

完整字段表见 [Agent Profile Reference](agent-profile-reference.md)。

## 3. 选择注册方式

先按你的真实使用方式选 profile 类型：

| 你要注册的东西 | 推荐 `kind` | 说明 |
| --- | --- | --- |
| Codex CLI | `codex` | 用 Codex 默认 setup 和认证规则。 |
| Claude Code 或 Claude wrapper | `claude-code` | 适合 `claude`、`claude-ds` 一类命令。 |
| OpenCode CLI | `opencode` | 用 OpenCode 默认 setup 和认证规则。 |
| Pi Coding Agent | `pi-coding-agent` | 适合 Pi Coding Agent CLI。 |
| 其他命令行 agent | `custom` | 你自己声明 command、auth 和 setup。 |

选择原则：

- 如果你的 agent 属于已知 CLI，优先用对应 `kind`，不要直接用 `custom`。
- 如果你只是换了模型或 API wrapper，也应保留真实 `kind`，例如 `claude-ds` 仍然用 `claude-code`。
- 如果你不确定，先用最接近的 `kind`，再跑 `doctor` 看阻断信息。

## 4. 创建 profile

在 `$HARNESSLAB_HOME/agents/` 下创建一个以 profile 名命名的 TOML 文件：

```bash
$EDITOR "$HARNESSLAB_HOME/agents/claude-ds.toml"
```

一个最小 Claude wrapper 示例：

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
  "ANTHROPIC_MODEL",
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

[labels]
model = "deepseek"
```

关键规则：

- 不要把 API key 写进 `command`、`labels` 或 `setup.commands`。
- `auth.inherit_env` 只写环境变量名，不写变量值。
- `setup.commands` 只有在 `setup.preset = "custom"` 时才应该使用。
- `skills.allow` 和 `skills.deny` 不能包含同一个名字；`tools`、`hooks` 也是一样。

## 5. 运行预检查

每次新增或修改 profile 后都先跑：

```bash
harnesslab --home "$HARNESSLAB_HOME" doctor
```

如果你需要机器可读输出：

```bash
harnesslab --home "$HARNESSLAB_HOME" doctor --json
```

`doctor` 会检查：

- profile TOML 是否能解析。
- 字段值是否合法。
- `setup`、`skills`、`tools`、`hooks` 是否存在冲突。
- 当前 `kind` 是否能 materialize 你声明的策略。
- Docker、benchmark 数据和 usage 配置是否存在明显问题。

看到 blocking error 时不要直接开始 run。先根据 `field_path` 和 `suggested_fix` 修改 profile。

## 6. 处理能力策略

HarnessLab 不会静默忽略无法执行的能力策略。当前 MVP 中，非默认 `skills`、`tools`、`hooks` 策略只有在对应 `kind` 能证明可 materialize 时才允许进入 run。

这意味着：

- 默认继承通常可用：`inherit = true`，`allow = []`，`deny = []`。
- 白名单/黑名单是强声明，不是备注。
- 如果当前 adapter 还不能落实某个策略，`doctor` 和 `run` 会提前阻断。

如果你只是想先完成注册和首次实验，建议保持默认：

```toml
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
```

等首次 smoke run 通过后，再为不同能力策略创建新的 profile。

## 7. 跑一次最小实验

预检查通过后，先跑最小 split，不要一上来跑完整 benchmark：

```bash
harnesslab --home "$HARNESSLAB_HOME" run \
  --agent claude-ds \
  --benchmark terminal-bench \
  --split smoke \
  --concurrency 1 \
  --timeout-sec 1800 \
  --json
```

如果你还没有准备 Terminal-Bench 数据，先参考 [用 claude-ds 跑一次 Terminal-Bench 实验](playbooks/terminal-bench-claude-ds.md)。

首次 smoke run 的目标不是刷分，而是确认：

- HarnessLab 能找到你的 profile。
- 认证继承正确。
- setup 在 agent 命令前执行。
- benchmark adapter 能启动 agent。
- run artifacts 和报告能生成。

## 8. 检查注册是否真的生效

一次成功 run 之后，检查 run 目录中的快照。它证明 HarnessLab 使用的是 materialized runtime profile，而不是临时解释你的原始 TOML：

```bash
find "$HARNESSLAB_HOME/runs" -name agent-runtime.materialized.json | tail -1
```

重点查看：

- `setup_script`：实际执行的 setup。
- `setup_summary`：报告中展示的 setup 摘要。
- `skills_summary`、`tools_summary`、`hooks_summary`：最终能力策略摘要。
- `profile_name` 和 `display_name`：确认选择了正确 profile。

公共 artifact 会 redact 已知 secret；如果你发现密钥值出现在报告、`command.txt` 或公开日志中，应停止使用该 profile 并修正认证写法。

## 9. 常见问题

### `doctor` 说 profile load failed

通常是 TOML 语法错误、字段拼写错误或文件名/profile name 不一致。先用 `doctor --json` 看具体 `field_path`。

### `doctor` 说 policy cannot be materialized

你声明了非默认 `skills`、`tools` 或 `hooks` 策略，但当前 agent kind 还不能可靠执行它。首次注册时先恢复默认继承；如果要比较能力策略，创建单独 profile 并等待对应 adapter 支持。

### run 前就失败，没有创建 benchmark 结果

这是预期行为。HarnessLab 会在 run 前阻断无效 profile、缺失 benchmark 数据、无法 materialize 的策略和明显环境问题，避免留下误导性 benchmark verdict。

### Terminal-Bench 里 setup 失败

看 run artifacts 中的 setup 日志，例如 `agent_setup_stdout.log`、`agent_setup_stderr.log` 和 `agent_setup_command.sha256`。setup failure 是 HarnessLab 执行失败，不应当被当成 agent 解题失败。

### 报告里看不到我想比较的差异

把差异写进 profile：不同模型、不同 wrapper、不同 tools/skills/hooks 策略都应成为不同 profile 名称或 labels。不要只靠外部环境变量隐式改变实验条件。

## 10. 注册完成清单

注册完成前，逐项确认：

- `$HARNESSLAB_HOME/agents/<name>.toml` 存在。
- `name` 与你运行时的 `--agent <name>` 一致。
- `command` 不包含 secret。
- `auth.inherit_env` 只包含变量名。
- `setup.required_commands` 是裸命令名，不是 shell 管道或路径。
- `skills/tools/hooks` 没有无法 materialize 的非默认策略。
- `harnesslab doctor` 通过，或 warning 已被你明确接受。
- 最小 smoke run 通过，或失败分类能清楚区分 agent verdict 与执行环境问题。
- run 目录里存在 `agent-runtime.materialized.json`。

完成这些检查后，这个 profile 才算可以用于正式 benchmark 对比。
