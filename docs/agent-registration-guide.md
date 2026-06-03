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

### 4.1 完整参数说明

注册流程里的每个参数都应该能回答四件事：它是什么意思、可以填什么、应该怎么填、当前是否真的会被运行时落实。下面按 TOML 结构列出完整说明。

#### 顶层参数

| 参数 | 必填 | 取值范围 | 示例 | 生效状态 | 解释 |
| --- | --- | --- | --- | --- | --- |
| `schema_version` | 是 | 当前固定 `1` | `1` | loader 强校验 | Agent profile schema 版本。除非 HarnessLab 升级 schema，否则不要改。 |
| `name` | 是 | `[a-zA-Z0-9][a-zA-Z0-9._-]*` | `"claude-ds"` | loader 强校验，并用于 `--agent` 查找 | profile 唯一名称，也是运行时 `--agent claude-ds` 使用的值。必须以 ASCII 字母或数字开头。 |
| `kind` | 是 | `codex`、`claude-code`、`opencode`、`pi-coding-agent`、`custom`、`fake` | `"claude-code"` | 选择 setup 默认值、能力 catalog 和 materializer 支持边界 | CLI harness 类型。HarnessLab 用它选择默认 auth、setup 和 materialization 规则。`fake` 仅用于契约测试。 |
| `display_name` | 是 | 任意可读字符串 | `"Claude Code via DeepSeek API"` | 写入 snapshot/report | 报告中展示给用户看的名称。可以比 `name` 更长、更易读。 |
| `command` | 是 | shell command 字符串 | `"claude-ds -p --bare --output-format text"` | setup 后由 runner 执行；公开 artifacts 会 redact | HarnessLab 启动 agent 的命令模板。不要写 API key。`input_mode = "argument"` 时必须包含 `{{instruction}}`；`input_mode = "file"` 时必须包含 `{{instruction_file}}` 或 `{{instruction}}`。 |
| `input_mode` | 是 | `stdin`、`argument`、`file`、`tty` | `"stdin"` | command renderer 强校验 | HarnessLab 如何把任务说明交给 agent。`stdin` 通过标准输入传入；`argument` 放入命令参数；`file` 写入文件后传路径；`tty` 用终端语义运行。 |
| `working_dir` | 是 | `workspace`、`run_dir` | `"workspace"` | process runner 执行 | agent 进程的启动目录。`workspace` 表示 benchmark 工作区；`run_dir` 表示 HarnessLab run 目录。 |
| `timeout_sec` | 是 | 正整数秒数 | `300` | 用作单 task 默认进程超时，benchmark/run 配置可进一步收紧 | 单个 task 的默认 agent 超时。 |
| `version_command` | 否 | shell command 字符串或省略 | `"claude --version"` | doctor/run/replay bounded probe；输出 redacted | 可选版本探测命令，用于记录/诊断 agent CLI 版本。失败不应被当作解题失败。 |

#### `[auth]` 参数

| 参数 | 必填 | 取值范围 | 示例 | 生效状态 | 解释 |
| --- | --- | --- | --- | --- | --- |
| `auth.inherit` | 是 | `true`、`false` | `true` | host/Docker 都按显式 auth source 执行 | 是否启用声明式认证继承。设为 `false` 时，不继承 `inherit_env` 和 `include_paths` 中的认证信息。 |
| `auth.inherit_env` | 是 | 环境变量名数组 | `["ANTHROPIC_AUTH_TOKEN"]` | 仅在 `auth.inherit = true` 时传入；host 使用显式 env map | 允许传入运行环境的环境变量名。这里只写变量名，不写变量值。 |
| `auth.include_paths` | 是 | 路径数组，或 `host:container:mode` 数组 | `["~/.claude:/root/.claude:ro"]` | 仅在 `auth.inherit = true` 时参与挂载；doctor 做可读/可挂载检查 | 显式挂载到运行环境的认证/配置路径。`mode` 常用 `ro` 或 `rw`。 |
| `auth.exclude_paths` | 是 | 路径数组 | `["~/.claude/logs"]` | 解析 inherited auth paths 时生效 | 从继承路径中排除的路径，用于避免挂载不需要或敏感的目录。 |
| `auth.mount_ssh_socket` | 是 | `true`、`false` | `false` | auth mount 解析时生效；doctor 检查可用性 | 是否挂载 SSH agent socket。只在 agent 需要 SSH 认证时开启。 |
| `auth.mount_docker_socket` | 是 | `true`、`false` | `false` | doctor 作为高权限风险提示/阻断 | 是否挂载 Docker socket。权限很高，`doctor` 会提示风险；普通 benchmark agent 不应开启。 |

#### `[setup]` 参数

| 参数 | 必填 | 取值范围 | 示例 | 生效状态 | 解释 |
| --- | --- | --- | --- | --- | --- |
| `setup.preset` | 否 | `none`、`builtin`、`custom` | `"builtin"` | materializer 在 agent 执行前落实；不支持时阻断 | setup 策略。`none` 不做额外准备；`builtin` 使用 HarnessLab 内置规则；`custom` 才允许执行 `setup.commands`。 |
| `setup.required_commands` | 否 | 裸命令名数组；字符可包含字母、数字、`.`、`_`、`+`、`-` | `["claude", "claude-ds"]` | doctor 检查；builtin/custom 提供状态会进入诊断 | setup 完成后必须能找到的命令。不要写路径、shell 管道或带参数命令。 |
| `setup.run_as` | 否 | `root`、`harnesslab`、`current` | `"harnesslab"` | Docker 可落实；host 路径除 `current` 外提前阻断 | agent 命令使用哪个用户运行。host-only task 使用 `current`。 |
| `setup.commands` | 否 | shell command 数组 | `["npm install -g @anthropic-ai/claude-code"]` | 仅 `setup.preset = "custom"` 执行；公开 artifacts 会 redact | 高级自定义 setup 命令。只有 `setup.preset = "custom"` 时有效。不要在这里写 secret。 |

#### `[skills]` 参数

| 参数 | 必填 | 取值范围 | 示例 | 生效状态 | 解释 |
| --- | --- | --- | --- | --- | --- |
| `skills.inherit` | 否 | `true`、`false` | `true` | 解析为 `candidate_effective`；非默认策略需 verified materializer | 是否继承该 agent kind 的默认 skills。首次注册建议保持 `true`。 |
| `skills.allow` | 否 | 当前 catalog 已知 skill 名称数组 | `["code-review"]` | 非空时作为显式目标集合；未知项是字段错误 | skills 白名单。非空时集合来自 `allow`，即使 `inherit = false` 也有效。不能和 `skills.deny` 重复。 |
| `skills.deny` | 否 | 当前 catalog 已知 skill 名称数组 | `["test-runner"]` | 从默认集合或显式 `allow` 中扣除；未知项是字段错误 | skills 黑名单。不能和 `skills.allow` 重复。 |
| `skills.include_paths` | 否 | 路径数组 | `["~/.claude/skills"]` | 只对 skills 合法；tools/hooks 没有 include_paths | 额外 skills 目录。路径类配置放这里，不要放进 `allow` 或 `deny`。 |

#### `[tools]` 参数

| 参数 | 必填 | 取值范围 | 示例 | 生效状态 | 解释 |
| --- | --- | --- | --- | --- | --- |
| `tools.inherit` | 否 | `true`、`false` | `true` | 解析为 `candidate_effective`；非默认策略需 verified materializer | 是否继承默认 tools。首次注册建议保持 `true`。 |
| `tools.allow` | 否 | 当前 catalog 已知 tool 名称数组 | `["bash"]` | 非空时作为显式目标集合；未知项是字段错误 | tools 白名单。非空时集合来自 `allow`，即使 `inherit = false` 也有效。不能和 `tools.deny` 重复。 |
| `tools.deny` | 否 | 当前 catalog 已知 tool 名称数组 | `["web_search"]` | 从默认集合或显式 `allow` 中扣除；未知项是字段错误 | tools 黑名单。不能和 `tools.allow` 重复。 |

#### `[hooks]` 参数

| 参数 | 必填 | 取值范围 | 示例 | 生效状态 | 解释 |
| --- | --- | --- | --- | --- | --- |
| `hooks.inherit` | 否 | `true`、`false` | `true` | 解析为 `candidate_effective`；非默认策略需 verified materializer | 是否继承默认 hooks。首次注册建议保持 `true`。 |
| `hooks.allow` | 否 | 当前 catalog 已知 hook 名称数组 | `["pre_tool_use"]` | 非空时作为显式目标集合；未知项是字段错误 | hooks 白名单。非空时集合来自 `allow`，即使 `inherit = false` 也有效。不能和 `hooks.deny` 重复。 |
| `hooks.deny` | 否 | 当前 catalog 已知 hook 名称数组 | `["post_tool_use"]` | 从默认集合或显式 `allow` 中扣除；未知项是字段错误 | hooks 黑名单。不能和 `hooks.allow` 重复。 |

#### `[usage]` 参数

| 参数 | 必填 | 取值范围 | 示例 | 生效状态 | 解释 |
| --- | --- | --- | --- | --- | --- |
| `usage.parser` | 是 | `none`、`regex`、`json_path` | `"none"` | usage collector 执行；`none` 记录未知用量 | token/cost 采集方式。`none` 表示不解析用量。 |
| `usage.source` | 否 | `agent_stdout`、`agent_stderr`、`agent_logs`、`file:<safe-relative-path>` | `"agent_logs"` | safe path 校验，禁止逃逸 run 目录 | usage parser 的输入来源。`file:` 路径不能逃逸 run 目录。 |
| `usage.input_tokens_key` | 否 | 字符串 | `"input_tokens"` | 结构化/JSON usage 解析时使用 | `json_path` 或结构化解析时表示输入 token 字段名。 |
| `usage.output_tokens_key` | 否 | 字符串 | `"output_tokens"` | 结构化/JSON usage 解析时使用 | `json_path` 或结构化解析时表示输出 token 字段名。 |
| `usage.total_tokens_key` | 否 | 字符串 | `"total_tokens"` | 结构化/JSON usage 解析时使用 | `json_path` 或结构化解析时表示总 token 字段名。 |
| `usage.cost_usd_key` | 否 | 字符串 | `"cost_usd"` | 结构化/JSON usage 解析时使用 | `json_path` 或结构化解析时表示美元成本字段名。 |

#### `[labels]` 参数

`labels` 是开放 key/value map，值都是字符串。它既用于报告展示，也用于部分 benchmark adapter 的提示信息。

| 参数 | 必填 | 取值范围 | 示例 | 生效状态 | 解释 |
| --- | --- | --- | --- | --- | --- |
| `labels.<key>` | 否 | 字符串 key/value | `model = "deepseek"` | 写入 snapshot/report；已知 key 可能被 adapter 消费 | 通用报告标签。适合记录模型、provider、wrapper、策略名等可比较条件。 |
| `labels.model` | 否 | 字符串 | `"deepseek"` | 常用 report/model 标签 | 常用模型标签。报告中用于展示这个 profile 的模型/配置。 |
| `labels.terminal_bench_model` | 否 | 字符串 | `"deepseek"` | Terminal-Bench adapter 消费 | Terminal-Bench adapter 使用的模型标签；未设置时通常可用 `labels.model` 作为 fallback。 |
| `labels.terminal_bench_agent` | 否 | Terminal-Bench 内置 agent 名称 | `"codex"` | Terminal-Bench adapter 消费 | 让 Terminal-Bench 使用官方内置 agent 名称。和 `terminal_bench_agent_import_path` 二选一。 |
| `labels.terminal_bench_agent_import_path` | 否 | Python import path | `"harnesslab_tb_agent:HarnessLabCommandAgent"` | Terminal-Bench adapter 消费；host-agent run_as precheck 适用 | 让 Terminal-Bench 通过 import path 加载 HarnessLab bridge agent。 |
| `labels.terminal_bench_agent_pythonpath` | 否 | 绝对路径或 Python path 字符串 | `"/path/to/HarnessLab/integrations/terminal_bench"` | Terminal-Bench adapter 消费 | 传给 Terminal-Bench bridge 的 Python path。使用源码运行时通常需要设置。 |
| `labels.sandbox_setup_command` | 否 | shell command 字符串 | `"npm install -g @anthropic-ai/claude-code"` | 旧快照/旧 profile 兼容；新注册优先用 `[setup]` | 旧 profile 兼容字段。新注册不要优先使用；应改用 `[setup]`。 |

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
- `setup.required_commands` 是否是裸命令名、是否能由 host 或 builtin setup 提供。
- `skills`、`tools`、`hooks` 是否存在冲突、拼写错误或无法执行的非默认策略。
- 当前 `kind` 是否能 materialize 你声明的能力策略。
- `version_command` 是否能在 bounded probe 中运行，输出会被 redact。
- host auth 继承和 `setup.run_as` 是否会在目标执行路径上失真。
- Docker、benchmark 数据和 usage 配置是否存在明显问题。

看到 blocking error 时不要直接开始 run。先根据 `field_path` 和 `suggested_fix` 修改 profile。

## 6. 处理能力策略

HarnessLab 不会静默忽略无法执行的能力策略。当前 MVP 中，非默认 `skills`、`tools`、`hooks` 策略只有在对应 `kind` 能证明可 materialize 时才允许进入 run。

能力策略的最终集合按下面规则计算：

```text
if allow is non-empty:
  target = allow
else if inherit = true:
  target = default_enabled(kind, domain)
else:
  target = {}

candidate_effective = target - deny
```

这意味着：

- 默认继承通常可用：`inherit = true`，`allow = []`，`deny = []`。
- `allow` 是显式目标集合，不是对默认集合的过滤器。
- `inherit = false` 时，非空 `allow` 仍然有效；例如 `inherit = false` 且 `allow = ["bash"]` 的目标集合就是 `["bash"]`。
- 如果 `allow` 为空且 `inherit = false`，目标集合为空，然后再应用 `deny`。
- 白名单/黑名单是强声明，不是备注。拼错的 `allow` 或 `deny` 项会被 `doctor` 报成精确字段错误，例如 `tools.allow[0]`。
- `deny` 从默认集合或显式 `allow` 集合中扣除。
- 只有 materializer 已验证且策略无错误时，`effective` 才会真正生效；否则 `run` 会提前阻断。
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

## 7. 认证继承和运行用户限制

`auth.inherit` 同时影响 Docker 和 host 执行：

- `auth.inherit = true`：HarnessLab 只传入 `auth.inherit_env` 声明的变量值、task env，以及启动命令需要的最小进程 baseline。
- `auth.inherit = false`：不传入 profile 声明的 env/path 继承。host 进程也不会偷偷继承整个父进程环境。
- `auth.inherit_env` 只写变量名，不写变量值。公开 artifacts 会 redact 已知 secret 和常见敏感 token。

`setup.run_as` 的生效范围也有限制：

- Docker sandbox 中可以 materialize `root`、`harnesslab`、`current`。
- host task、Terminal-Bench import-agent host path、SWE-bench Pro `gold` host path 只支持 `current`。
- 如果 host 路径遇到 `run_as = "root"` 或 `"harnesslab"`，`run` 会在 task 执行前阻断，不会静默降级。

首次注册真实 CLI 时，如果你先跑的是 host/fake smoke 路径，建议用：

```toml
[setup]
run_as = "current"
```

如果后续转向 Docker sandbox，再按隔离需求改成 `harnesslab` 或 `root`。

## 8. 跑一次最小实验

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

## 9. 检查注册是否真的生效

一次成功 run 之后，检查 run 目录中的快照。它证明 HarnessLab 使用的是 materialized runtime profile，而不是临时解释你的原始 TOML：

```bash
find "$HARNESSLAB_HOME/runs" -name agent-runtime.materialized.json | tail -1
```

重点查看：

- `setup_script`：实际执行的 setup。
- `setup_summary`：报告中展示的 setup 摘要。
- `capabilities.skills`、`capabilities.tools`、`capabilities.hooks`：结构化 resolved policy，包含 `available`、`default_enabled`、`candidate_effective`、`effective`、`enforcement` 和错误。
- `skills_summary`、`tools_summary`、`hooks_summary`：报告兼容用的人类可读摘要。
- `agent-version.snapshot.json`：如果配置了 `version_command`，这里记录 redacted probe 状态。
- `report.html`：确认页面展示了 materialized runtime、effective capabilities 和 version probe 状态。

公共 artifact 会 redact 已知 secret 和常见敏感 token；如果你发现密钥值出现在报告、`command.txt` 或公开日志中，应停止使用该 profile 并修正认证写法。`agent-profile.runtime.json` 是 private runtime snapshot，不应当作为可分享 artifact。

## 10. 常见问题

### `doctor` 说 profile load failed

通常是 TOML 语法错误、字段拼写错误或文件名/profile name 不一致。先用 `doctor --json` 看具体 `field_path`。

### `doctor` 说 policy cannot be materialized

你声明了非默认 `skills`、`tools` 或 `hooks` 策略，但当前 agent kind 还不能可靠执行它。首次注册时先恢复默认继承；如果要比较能力策略，创建单独 profile 并等待对应 adapter 支持。

### `doctor` 说 unknown allow 或 unknown deny

你写了当前 catalog 不认识的能力名。`allow` 和 `deny` 都不是备注字段，拼错会阻断。用 `doctor --json` 查看该 domain 的 `available` 列表，按 `field_path` 修改，例如 `tools.allow[0]` 或 `hooks.deny[0]`。

### `inherit = false` 时 `allow` 还有效吗

有效。`allow` 非空时，目标集合直接来自 `allow`，不再看 `inherit`。例如：

```toml
[tools]
inherit = false
allow = ["bash"]
deny = []
```

这个策略的候选集合是 `["bash"]`。是否能进入 `effective`，取决于当前 `kind` 是否有 verified materializer。

### `version_command` 失败会阻断 run 吗

通常不会。`doctor` 会把探测失败报告为 warning，new run 会保存 redacted snapshot，replay 会和 source snapshot 比较并写 warning/event。除非命令本身 malformed，否则它不是 benchmark verdict。

### `run_as = "harnesslab"` 在 host task 上为什么被阻断

因为 host 执行目前没有可靠的用户切换 materializer。HarnessLab 不会假装已经切换用户，也不会静默 fallback 到当前用户。host 路径请使用 `run_as = "current"`。

### run 前就失败，没有创建 benchmark 结果

这是预期行为。HarnessLab 会在 run 前阻断无效 profile、缺失 benchmark 数据、无法 materialize 的策略和明显环境问题，避免留下误导性 benchmark verdict。

### Terminal-Bench 里 setup 失败

看 run artifacts 中的 setup 日志，例如 `agent_setup_stdout.log`、`agent_setup_stderr.log` 和 `agent_setup_command.sha256`。setup failure 是 HarnessLab 执行失败，不应当被当成 agent 解题失败。

### 报告里看不到我想比较的差异

把差异写进 profile：不同模型、不同 wrapper、不同 tools/skills/hooks 策略都应成为不同 profile 名称或 labels。不要只靠外部环境变量隐式改变实验条件。

## 11. 注册完成清单

注册完成前，逐项确认：

- `$HARNESSLAB_HOME/agents/<name>.toml` 存在。
- `name` 与你运行时的 `--agent <name>` 一致。
- `command` 不包含 secret。
- `auth.inherit_env` 只包含变量名。
- `setup.required_commands` 是裸命令名，不是 shell 管道或路径。
- `skills/tools/hooks` 没有无法 materialize 的非默认策略。
- `run_as` 与执行路径匹配：host 路径用 `current`，Docker sandbox 可用 `harnesslab` 或 `root`。
- 如果配置了 `version_command`，`doctor` warning 已被理解，run 目录里有 `agent-version.snapshot.json`。
- `harnesslab doctor` 通过，或 warning 已被你明确接受。
- 最小 smoke run 通过，或失败分类能清楚区分 agent verdict 与执行环境问题。
- run 目录里存在 `agent-runtime.materialized.json`，且 `report.html` 展示的 effective capabilities 与 snapshot 一致。

完成这些检查后，这个 profile 才算可以用于正式 benchmark 对比。
