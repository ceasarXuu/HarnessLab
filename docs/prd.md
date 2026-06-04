# HarnessLab 产品设计文档

> HarnessLab 是一个 harness 评测实验室：用尽量简单的一条命令，在本地隔离环境中运行主流 benchmark，评估一个完整 CLI Agent / harness 的真实任务表现，并生成可复现的实验记录报告。

## 1. 文档信息

| 项目 | 内容 |
|---|---|
| 产品名 | HarnessLab |
| 文档类型 | 产品设计文档 / Product Design Document |
| 阶段 | MVP 定义 |
| 更新日期 | 2026-05-26 |
| 第一用户 | HarnessLab 作者本人，以及需要评测 CLI coding agent 的个人开发者 |
| 主要形态 | 开源框架 + CLI-first 本地工具 |

## 2. 一句话定位

HarnessLab 是一个 **harness 评测实验室**，用于评估完整 CLI Agent 在同一 benchmark 下的综合效果，帮助用户回答：

1. 我应该在 Codex CLI、Claude Code、opencode、Pi Coding Agent 等 harness 中选择哪个？
2. 我自己开发的 harness 相比竞品强在哪里、弱在哪里？
3. 我的 harness 在版本演进或配置调整后，真实 benchmark 表现是否变好？

## 3. 产品目标

### 3.1 MVP 目标

MVP 只解决一个核心闭环：

```text
检测本机 CLI Agent -> 确认/编辑配置 -> 选择 benchmark -> 一条命令运行单个 run -> 生成单文件 HTML 实验报告 -> 支持复现和 resume
```

用户应能在 5 分钟内完成本机 agent 检测与配置确认，并启动一次 Terminal-Bench smoke 或其他已准备好数据的 benchmark run。

### 3.2 长期目标

HarnessLab 长期要成为 coding agent / harness 的本地评测基础设施，让 harness 的优劣从体感判断变成可复现实验记录。

长期方向包括多 run 对比、版本趋势、私有任务集、企业内部评测、失败归因增强和更多 benchmark 生态接入，但这些都不进入 MVP 的产品边界。

## 4. 核心用户

### 4.1 Agent 消费型用户

这类用户已经在使用或准备选择 CLI coding agent，例如：

- Codex CLI
- Claude Code
- opencode
- Pi Coding Agent

他们的问题是：不同 agent 或同一 agent 的不同配置在真实任务上到底谁更好，当前缺乏简单、公平、可复现的比较方式。

### 4.2 Harness 开发者

这类用户正在开发自己的 harness，需要验证：

- 自研 harness 相比主流竞品表现如何。
- 同一个 harness 在不同配置、skills、模型、权限模式下表现如何。
- 版本迭代后是否真的提升，而不是只在主观体验上感觉更好。

MVP 不要求系统理解 harness 内部结构。HarnessLab 评估的是完整 CLI Agent 配置的最终效果。

## 5. 产品原则

| 原则 | 产品要求 |
|---|---|
| 简单优先 | CLI 交互保持短路径，避免复杂 wizard。 |
| 一键运行 | 初始化后，用户应主要通过一条 `run` 命令启动实验。 |
| Benchmark-first | 不自创 benchmark，优先接入市场主流 benchmark。 |
| Harness-level | 评测对象是完整 CLI Agent，不拆解模型、prompt、tools。 |
| 配置可表达差异 | 同一 CLI Agent 的不同模型、skills、权限、模式都通过不同 agent profile 表达。 |
| 本地隔离 | 默认在本地 Docker 隔离环境运行，用户不需要理解 Docker 细节。 |
| 实验记录 | 报告优先服务复盘、排障和复现，而不是营销展示。 |
| 原始评分优先 | benchmark 原始分数是主评分，耗时和 token/cost 是辅助指标。 |

## 6. 明确非目标

MVP 不做：

- 自研 benchmark 数据集。
- 多 run 总排行榜。
- 版本趋势图。
- Web SaaS。
- 多用户、权限、团队协作。
- 企业云端执行。
- 人工评分系统。
- LLM 自动总结报告。
- 用户私有任务集运行。

用户私有任务集是重要未来能力，MVP 需要保留扩展性，但不交付正式产品体验。

## 7. 核心产品对象

### 7.1 Agent Profile

Agent Profile 是 HarnessLab 的评测对象。它代表一个完整 CLI Agent 配置，而不是单纯模型。

例子：

- `codex-default`
- `codex-gpt5-high`
- `claude-code-sonnet-default`
- `claude-code-sonnet-skill-a`
- `opencode-default`
- `pi-coding-agent-default`

同一个 CLI Agent 只要配置不同，就应注册成不同 profile。HarnessLab 不负责解释这些差异，只负责把它们作为可复现实验条件保存下来。

### 7.2 Benchmark

Benchmark 是任务来源与评分来源。MVP 必须支持：

| Benchmark | MVP 要求 |
|---|---|
| Terminal-Bench | 必须完整跑通。这里指第三方 Terminal-Bench benchmark，而不是 HarnessLab 自建任务集。 |
| SWE-bench Pro | 必须完整跑通。这里指 Scale AI 发布的 SWE-bench Pro benchmark，而不是 HarnessLab 自建 SWE 子集。 |

Harbor 是重要参考对象，但 MVP 不直接依赖 Harbor，也不把 Harbor 作为独立 P0 benchmark。本文中的 Harbor-style 只表示：HarnessLab 的 terminal task 体验应兼容 Harbor 常见的任务组织思路，便于未来导入或适配 Harbor 生态任务。

SWE-bench Pro 是 MVP 的明确验收目标。普通 SWE-bench 属于同一产品方向；如果数据和 evaluator 兼容，可以复用同一产品体验，但不作为独立 P0 验收项。

外部 benchmark 身份：

| Benchmark | 外部来源 | HarnessLab 责任 |
|---|---|---|
| Terminal-Bench | [terminalbench.lol](https://terminalbench.lol/) 及其公开 benchmark 生态 | 提供本地运行、agent 接入、日志采集、报告和复现体验。 |
| SWE-bench Pro | [Scale Labs SWE-bench Pro](https://labs.scale.com/papers/swe_bench_pro) | 提供本地运行、agent 接入、patch/diff 结果记录、报告和复现体验。 |

### 7.3 Split

Split 是 benchmark 对任务集合的逻辑分组，例如：

- `smoke`：极小样本，用于验证配置和运行链路。
- `public`：公开可运行任务集。
- `full`：完整任务集。

每个 benchmark 自己定义可用 split。用户通过 `harnesslab benchmark list` 或 `harnesslab benchmark info <benchmark>` 查看可用 split、任务数量、每个 split 的数据准备状态和预计运行成本。

### 7.4 Run

MVP 的 Run 是最小实验单位：

```text
1 个 agent profile x 1 个 benchmark x 1 个 split
```

MVP 不做多 agent 矩阵。用户要比较多个 agent 时，分别运行多个 run，并查看各自报告。跨 run 总排行后置。

MVP 内允许“人工多 run 对比”，但不做系统聚合能力：

- run id 和目录名应包含 agent、benchmark、split 和时间戳，方便用户人工检索。
- `report open latest` 默认打开全局最新 run。
- 后续可支持按 agent 或 benchmark 筛选 latest，但不作为 MVP 必须项。
- 报告首屏必须展示当前 `report.html` 路径，方便用户并列打开多个报告。
- 不做跨 run 排名、自动 diff 或总榜。

### 7.5 Task Result

Task Result 是报告中的明细单位。它记录某个 benchmark task 的执行结果、评分、耗时、token/cost、日志入口、失败类型和复现信息。

### 7.6 Report

Report 是单个 run 的实验记录。MVP 报告必须是单文件 HTML，便于打开、归档和分享。详细日志、大文件和 workspace artifact 可以保留在 run 目录，通过相对链接访问。

## 8. 用户旅程

### 8.1 首次使用

```bash
harnesslab init
```

初始化行为：

1. 检测本机是否安装 Codex CLI、Claude Code、opencode、Pi Coding Agent。
2. 在全局配置目录生成或更新 agent profile 草稿。
3. 控制台打印检测结果和配置文件路径。
4. 不启动复杂交互，不自动打开文档。

内置 agent 检测标准：

| Agent | 检测方式 |
|---|---|
| Codex CLI | `codex` 命令在 `PATH` 中可用。 |
| Claude Code | `claude` 命令在 `PATH` 中可用；如果存在 `~/.claude`，则同时生成认证继承摘要。 |
| opencode | `opencode` 命令在 `PATH` 中可用。 |
| Pi Coding Agent | `pi` 命令在 `PATH` 中可用，且 `pi coding --version` 或等价版本检查可执行。 |

初始化后的控制台体验应简洁，例如：

```text
Detected agents:
  - codex: found at /usr/local/bin/codex -> ~/.harnesslab/agents/codex-default.toml
  - claude-code: found at /usr/local/bin/claude -> ~/.harnesslab/agents/claude-code-default.toml
  - opencode: not found
  - pi-coding-agent: not found

Next:
  1. Edit agent profiles in ~/.harnesslab/agents/
  2. Run: harnesslab doctor
  3. Run: harnesslab run --agent codex-default --benchmark terminal-bench --split smoke
```

### 8.2 运行前检查

```bash
harnesslab doctor
```

Doctor 是第一次 run 前的强制产品体验。它应检查：

- Docker 是否可用。
- 已注册 agent 命令是否存在。
- agent profile 是否语法合法。
- 必要认证路径或环境变量是否可继承。
- benchmark adapter 和本地缓存状态是否正常。
- smoke task 是否能启动。
- 认证继承路径能否在 Docker dry-run 中被访问。
- benchmark 数据集是否已准备；未准备时给出获取或下载指引。

Doctor 不应泄露密钥值，只展示路径和环境变量名称。

### 8.3 启动实验

推荐首页 demo 命令以 Terminal-Bench 为主，因为它最能体现 CLI Agent 的低成本接入，并且最符合“5 分钟内启动首次 run”的体验承诺：

```bash
harnesslab init
harnesslab doctor
harnesslab run --agent codex-default --benchmark terminal-bench --split smoke
harnesslab report open latest
```

SWE-bench Pro 是同等 P0 能力，但首次运行可能受外部数据集获取、授权、缓存体积和仓库下载影响，不纳入“5 分钟首次启动”的承诺。它应作为第二个示例：

```bash
harnesslab run --agent codex-default --benchmark swe-bench-pro --split public
```

### 8.4 中断与恢复

长 benchmark 运行中断后，用户应能恢复：

```bash
harnesslab run resume <run-dir>
```

Resume 语义：

- 默认跳过状态为 `success` 和 `partial_success` 的 task。
- 默认重跑 Execution Failure 和 Benchmark Failure，避免把运行链路故障或明确失败永久固化。
- 用户可以通过显式参数要求重跑 partial task，但这不是默认行为。
- 报告中必须标记该 run 是否经历过 resume。
- Task 明细中应标记结果来自原始执行还是 resume 后的新执行。

### 8.5 复现

MVP 支持两种复现：

| 类型 | 命令 | 语义 |
|---|---|---|
| 快照复现 | `harnesslab run replay <run-dir>` | 使用 run 目录内保存的完整配置快照复现，追求当时配置 100% 还原。Replay 创建新的 run 目录，不覆盖原 run。 |
| 命令复现 | 报告中复制原始 run 命令 | 使用当前全局配置重新跑，适合用户主动测试新配置。 |

Replay 使用快照中的 agent 配置、benchmark 配置和 task 快照。如果必要数据集或缓存已经被用户清理，replay 应失败并给出明确错误，而不是静默退回到最新 benchmark 数据。

## 9. CLI 产品命令

MVP 命令集：

```bash
harnesslab init
harnesslab doctor
harnesslab agent list
harnesslab benchmark list
harnesslab benchmark info <benchmark>
harnesslab run --agent <agent-profile> --benchmark <benchmark> --split <split>
harnesslab run resume <run-dir>
harnesslab run replay <run-dir>
harnesslab report open latest
harnesslab report open <run-dir>
```

### 9.1 不提供的命令

MVP 不提供 `agent add`。Agent 配置文件由 `init` 生成草稿，用户在全局配置目录中手动编辑。

### 9.2 Run 默认参数

| 参数 | 默认值 | 产品语义 |
|---|---:|---|
| `concurrency` | `4` | 默认并发 4 个 task，可通过命令或配置覆盖。 |
| `attempts` | `1` | 默认每个 task 跑一次。需要稳定性评估时用户显式提高。 |
| `timeout` | benchmark 或 profile 默认 | 超时应归类为执行失败。 |
| `network` | enabled | 默认允许联网，用户可配置限制。 |
| `usage` | optional | 未配置 token/cost parser 时显示 unknown，并明确提示不可比较。 |

### 9.3 Benchmark 发现体验

`harnesslab benchmark list` 应展示：

- benchmark 名称。
- 当前支持状态。
- 可用 split。
- 数据准备状态。
- 是否需要用户额外获取数据集。

`harnesslab benchmark info <benchmark>` 应展示指定 benchmark 的任务数量、split 说明、数据路径、缓存状态、估算运行规模和第一个可复制的 run 命令。

## 10. 全局配置目录

HarnessLab 是与被测项目无关的本地实验工具，配置默认保存在全局目录：

```text
~/.harnesslab/
  config.toml
  agents/
    codex-default.toml
    claude-code-default.toml
    opencode-default.toml
    pi-coding-agent-default.toml
  benchmarks/
  runs/
    <run-id>/
```

Run 目录永久保存，HarnessLab 不自动删除历史记录。清理由用户自己决定。未来可以提供安全清理命令，但不能默认自动清理。

### 10.1 config.toml 最小语义

`config.toml` 用于保存跨 agent 的默认偏好。MVP 至少定义：

| 字段 | 默认值 | 说明 |
|---|---:|---|
| `default_concurrency` | `4` | 未在命令或 run 配置中指定时使用。 |
| `default_attempts` | `1` | 未指定 attempts 时使用。 |
| `runs_dir` | `~/.harnesslab/runs` | run 历史目录。 |
| `benchmarks_dir` | `~/.harnesslab/benchmarks` | benchmark 数据和缓存目录。 |
| `network_default` | `enabled` | 默认是否允许联网。 |
| `usage_default` | `none` | 默认 usage 采集策略。 |

MVP 不要求用户手写 `config.toml`。默认文件由 `init` 生成，用户只在需要覆盖默认行为时编辑。

## 11. Agent Profile 配置体验

Agent 配置必须能用 TOML 表达，并给每个配置项提供详细说明、取值范围和示例。MVP 内置四类模板：

- Codex CLI
- Claude Code
- opencode
- Pi Coding Agent

### 11.0 Agent 注册产品原则

Agent 注册是用户第一次真实接触 HarnessLab 的入口能力。它不能只是“让用户填一个能跑的命令”，而必须是一个可读、可解释、可由其他 agent 自动生成的注册表。

注册体验目标：

- 用户不需要理解 Docker、Terminal-Bench adapter、sandbox setup 或 bridge 细节。
- 用户看到的 profile 字段必须描述“我要注册什么 harness、继承什么配置、启用哪些能力、禁用哪些能力”，而不是一段不可读 shell。
- 其他 agent 可以根据本地 CLI 状态、用户需求和字段说明，生成一个合法 profile，并通过 `harnesslab doctor` 得到可操作的错误信息。
- 同一个 CLI harness 的模型、skills、tools、hooks、权限模式、运行命令差异，都必须能通过多个命名 profile 清晰表达。
- profile 必须同时适合人读、机器生成和 run snapshot 复现；不得依赖隐藏的本机状态来解释关键差异。

注册表必须保留两层表达：

| 层级 | 面向对象 | 作用 |
|---|---|---|
| 语义化字段 | 普通用户和辅助注册的 agent | 描述 agent 类型、启动方式、认证继承、skills/tools/hooks 策略、usage parser。 |
| 高级逃生口 | 少数无法参数化的 harness 维护者 | 暂时表达安装命令、包装命令或特殊权限调整。必须被标记为高级风险，并逐步被参数化 preset 替代。 |

### 11.1 配置字段说明

| 字段 | 必填 | 取值范围 | 说明 |
|---|---|---|---|
| `name` | 是 | 唯一字符串 | Agent profile 名称，也是 `--agent` 参数值。 |
| `kind` | 是 | `codex` / `claude-code` / `opencode` / `pi-coding-agent` / `custom` | 用于选择内置检测和默认配置逻辑。 |
| `display_name` | 否 | 字符串 | 报告中展示的可读名称。 |
| `command` | 是 | 字符串或数组 | 实际启动 agent 的命令模板。 |
| `input_mode` | 是 | `argument` / `stdin` / `file` / `tty` | 任务说明如何传给 agent。`tty` 表示通过伪终端无人值守注入任务文本，不表示人工交互。 |
| `working_dir` | 否 | `workspace` / `repo_root` / 绝对路径模板 | agent 启动目录，默认是任务 workspace。 |
| `timeout_sec` | 否 | 正整数 | 单 task 超时时间。 |
| `env` | 否 | key/value | 额外环境变量，可覆盖继承值。 |
| `auth.inherit` | 否 | `true` / `false` | 是否默认继承本机认证和配置。 |
| `auth.include_paths` | 否 | 路径数组 | 只继承指定配置路径。 |
| `auth.exclude_paths` | 否 | 路径数组 | 从默认继承路径中排除。 |
| `setup.preset` | 否 | `none` / `builtin` / `custom` | sandbox 内 agent 命令准备方式。普通用户优先使用 `builtin`，只有无法参数化时才用 `custom`。 |
| `setup.required_commands` | 否 | 字符串数组 | setup 完成后必须存在的命令，例如 `["claude", "claude-ds"]`。预检查失败则阻断 run。 |
| `setup.run_as` | 否 | `root` / `harnesslab` / `current` | agent 命令使用哪个用户运行。省略时默认 `current`；面向 Docker 的内置模板可以显式写 `harnesslab`。host 路径只支持 `current`。 |
| `setup.commands` | 否 | 字符串数组 | 高级逃生口。仅 `setup.preset = "custom"` 时允许，用于无法参数化的安装或包装命令。 |
| `skills.inherit` | 否 | `true` / `false` | 是否继承该 agent kind 的默认 skills/profile 资源。 |
| `skills.allow` | 否 | 字符串数组 | 白名单。非空时只启用这些 skills。 |
| `skills.deny` | 否 | 字符串数组 | 黑名单。从继承或白名单集合中排除这些 skills。与 `allow` 冲突时配置非法。 |
| `skills.include_paths` | 否 | 路径数组 | 额外挂载或复制的 skills 路径。 |
| `tools.inherit` | 否 | `true` / `false` | 是否继承 agent 默认可用 tools。 |
| `tools.allow` | 否 | 字符串数组 | 工具白名单，名称由对应 agent adapter 解释。 |
| `tools.deny` | 否 | 字符串数组 | 工具黑名单，与 `allow` 冲突时配置非法。 |
| `hooks.inherit` | 否 | `true` / `false` | 是否继承 agent 默认 hooks。 |
| `hooks.allow` | 否 | 字符串数组 | hook 白名单，可按 hook 名称或事件名匹配。 |
| `hooks.deny` | 否 | 字符串数组 | hook 黑名单，与 `allow` 冲突时配置非法。 |
| `usage.parser` | 否 | `none` / `regex` / `json_path` | token/cost 采集方式。命令型 parser 后置到 P1。 |
| `labels` | 否 | key/value | 报告展示用标签，例如模型名、模式、skills 名。 |

### 11.1.1 Agent 能力策略规则

`skills`、`tools`、`hooks` 都使用同一套用户心智模型：

- `inherit = true` 且 `allow = []`：继承该 harness 的默认能力集合，再应用 `deny`。
- `inherit = true` 且 `allow != []`：只启用 `allow` 中列出的能力，再应用 `deny`。
- `inherit = false` 且 `allow = []`：禁用该能力类型。
- `inherit = false` 且 `allow != []`：不继承默认能力，只启用显式 `allow`。
- 同一个条目同时出现在 `allow` 和 `deny` 中时，doctor 必须报错，不允许靠优先级猜测用户意图。
- 如果某个 agent kind 不能可靠执行对应策略，doctor 必须报错并阻断 run；不能只在报告里提示后继续跑出不可比较结果。

每个 profile 的报告和 snapshot 必须展示最终生效的 skills/tools/hooks 策略摘要，方便用户确认本次比较到底比了哪些差异。

### 11.1.2 Setup 参数化原则

`sandbox_setup_command` 这类任意 shell 只能作为旧 profile 的兼容逃生口，不应成为普通用户注册 agent 的主要方式。MVP 的 agent 注册体验优先使用参数化 setup：

```toml
[setup]
preset = "builtin"
required_commands = ["claude", "claude-ds"]
run_as = "current"
commands = []
```

`setup.preset = "custom"` 允许 `setup.commands`，但必须满足：

- 在 doctor 中显示为高风险配置。
- 进入 run snapshot，支持 replay。
- 日志中记录 setup 开始、结束、失败原因和耗时。
- 失败时归类为 `execution/external_runner_setup_failed` 或 profile precheck error，不计入 agent 能力分数。

### 11.2 命令模板变量

| 变量 | 含义 | 典型适用场景 |
|---|---|---|
| `{{instruction}}` | task 说明文本，直接拼入命令行。 | `input_mode: argument` |
| `{{instruction_file}}` | task 说明写入临时文件后的路径。 | `input_mode: file` |
| `{{workspace}}` | 当前 task workspace 路径。 | agent 需要显式指定工作目录时使用。 |
| `{{logs_dir}}` | 当前 task 日志目录。 | agent 支持输出 trajectory 或额外日志时使用。 |
| `{{run_dir}}` | 当前 run 目录。 | 需要引用全局 run 产物时使用。 |

### 11.3 Usage Parser 示例

MVP 允许 usage parser 缺省，但配置了 parser 的 agent 应能把 token/cost 展示到报告中。示例：

```toml
[usage]
parser = "regex"
source = "stderr"

[usage.patterns]
input_tokens = "input_tokens=(\\d+)"
output_tokens = "output_tokens=(\\d+)"
cost_usd = "cost_usd=([0-9.]+)"
```

`regex` 默认从 stdout、stderr 或指定日志文件中提取；`json_path` 用于结构化日志。parser 失败只影响成本展示，不影响 benchmark 主评分，并应以 warning 形式呈现。

### 11.4 Codex 示例

```toml
name = "codex-default"
kind = "codex"
display_name = "Codex CLI Default"
command = "codex exec --full-auto --model gpt-5 {{instruction}}"
input_mode = "argument"
working_dir = "workspace"
timeout_sec = 3600

[auth]
inherit = true
include_paths = ["~/.codex"]

[usage]
parser = "none"

[labels]
model = "gpt-5"
permission_mode = "full-auto"
```

### 11.5 Claude Code 示例

```toml
name = "claude-code-default"
kind = "claude-code"
display_name = "Claude Code Default"
command = "claude -p < {{instruction_file}}"
input_mode = "file"
working_dir = "workspace"
timeout_sec = 3600

[auth]
inherit = true
include_paths = ["~/.claude"]

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
model = "user-configured"
```

### 11.6 opencode 示例

```toml
name = "opencode-default"
kind = "opencode"
display_name = "opencode Default"
command = "opencode run --prompt-file {{instruction_file}}"
input_mode = "file"
working_dir = "workspace"
timeout_sec = 3600

[auth]
inherit = true

[usage]
parser = "none"

[labels]
model = "user-configured"
```

### 11.7 Pi Coding Agent 示例

```toml
name = "pi-coding-agent-default"
kind = "pi-coding-agent"
display_name = "Pi Coding Agent Default"
command = "pi coding run --stdin"
input_mode = "stdin"
working_dir = "workspace"
timeout_sec = 3600

[auth]
inherit = true

[usage]
parser = "none"

[labels]
model = "user-configured"
```

## 12. Benchmark 产品要求

### 12.1 Terminal-Bench

产品目标：

- 验证 CLI Agent 在真实终端任务中的执行能力。
- 兼容市场上已有 terminal benchmark 的任务组织方式，并参考 Harbor 的易用性经验。
- 让用户无需写 adapter 就能运行第三方 Terminal-Bench 任务集。

报告应展示 benchmark 原始得分、每个 task 的 pass/fail、失败分类、耗时和日志入口。

### 12.2 SWE-bench Pro

产品目标：

- 验证 coding agent 在真实仓库修复任务中的能力。
- 报告中应保留最终 diff、benchmark 评分结果、patch 失败原因和 verifier 日志入口。
- 对用户隐藏 benchmark 执行复杂度，保持同一套 `harnesslab run` 体验。

### 12.3 Benchmark 数据获取

产品体验要求：

- Terminal-Bench 数据可以由 HarnessLab 首次运行时自动准备，或在 `doctor` 中提示并自动触发准备步骤。
- SWE-bench Pro 可能需要用户预先获取公开或授权数据集，也可能需要较长时间准备仓库缓存；`doctor` 必须明确提示缺失内容、期望路径和获取指引。
- 所有 benchmark 数据和缓存默认放在 `~/.harnesslab/benchmarks/`。
- 如果数据缺失，`run` 不能模糊失败，必须给出可操作错误。
- 数据准备状态必须细化到 split 级别。例如 `smoke` 可用但 `full` 未准备时，CLI 应展示 split 级状态，而不是只展示 benchmark 整体状态。

### 12.4 未来扩展

MVP 不支持用户私有任务集，但产品结构必须允许未来新增：

- 本地目录任务集。
- 企业内部 repo benchmark。
- 新的 terminal-style benchmark。
- 新的 patch-style benchmark。

## 13. 运行过程产品要求

### 13.1 Docker 隔离

默认在 Docker 隔离环境中运行。用户只需要确保本机安装 Docker，不需要理解镜像、挂载、网络和容器细节。

高级配置可以覆盖网络权限和资源限制，但 MVP 文档应把这些放在高级配置中，不影响默认路径。

### 13.2 认证继承

默认自动检测并继承本机 agent 配置和认证信息。run 前控制台打印摘要：

```text
Auth/config inherited:
  - ~/.codex
  - env: OPENAI_API_KEY
```

不打印密钥值。用户可以在 agent profile 中通过 `include_paths`、`exclude_paths`、`env`、skills、tools 和 hooks 配置覆盖默认继承行为。

### 13.3 运行态输出

控制台运行态应简洁：

- 进度条。
- 已完成 task 数。
- 成功数。
- 失败数。
- 按失败原因分类的实时计数。
- 当前 run 目录。

详细 stdout、stderr、verifier 日志和 diff 只落文件，不默认刷屏。

### 13.4 Token / Cost

Token 和 cost 是可选但一等公民：

- 数据模型、报告和 CLI 都预留字段。
- 没有配置 parser 时显示 `unknown`。
- 报告必须明确提示：当前 agent 未配置 usage parser，因此成本不可比较。
- 耗时和 token/cost 不参与默认排名或 benchmark 主评分。

## 14. 失败分类

失败分类是 MVP 一级能力，必须帮助用户区分“环境/配置坏了”和“agent 真没做对”。

### 14.1 一级分类

| 分类 | 含义 |
|---|---|
| Execution Failure | 运行链路失败，不能直接说明 agent 能力差。 |
| Benchmark Failure | agent 正常执行完成，但 benchmark 判断任务失败。 |
| Partial Success | agent 正常执行完成，benchmark 给出非满分的部分得分。 |

### 14.2 Execution Failure

| 失败码 | 说明 |
|---|---|
| `docker_unavailable` | Docker 不可用或容器启动失败。 |
| `agent_command_missing` | agent 命令不存在。 |
| `auth_failed` | 认证或凭据不可用。 |
| `setup_failed` | benchmark 或任务环境准备失败。 |
| `agent_crashed` | agent 进程异常退出。 |
| `timeout` | 单 task 超时。 |
| `unknown_execution_error` | 未归类执行异常。 |

### 14.3 Warning

Warning 不计入 Execution Failure，也不影响 benchmark 主评分。

| 警告码 | 说明 |
|---|---|
| `usage_parser_failed` | token/cost 解析失败，成本不可比较。 |
| `usage_unknown` | 未配置 usage parser。 |

### 14.4 Benchmark Failure

| 失败码 | 说明 |
|---|---|
| `test_failed` | verifier 或测试未通过。 |
| `patch_apply_failed` | SWE 类任务中 patch 无法应用。 |
| `no_valid_diff` | agent 未产生有效代码变更。 |
| `wrong_answer` | benchmark 判断最终答案错误。 |
| `unknown_benchmark_failure` | 未归类 benchmark 失败。 |

### 14.5 Partial Success

Partial Success 不计入 Execution Failure，也不应被简单等同于失败。报告必须展示原始分数，并在汇总中单独统计 partial task 数量。

## 15. HTML 报告设计

报告定位是实验记录。MVP 报告必须是单文件 HTML。

### 15.1 首屏

首屏展示：

- Run ID。
- Agent profile 名称。
- Benchmark、split、attempts、concurrency。
- Benchmark 原始总分。
- 成功 task 数、失败 task 数。
- Execution Failure 数量。
- Benchmark Failure 数量。
- Partial Success 数量。
- 平均耗时和总耗时。
- token/cost 总量；缺失时明确显示不可比较。
- run 目录和复现命令。
- 报告生成时写入的 `report.html` 绝对路径；CLI 也必须在 run 结束时打印该路径。

### 15.2 Agent 配置快照

报告必须展示运行时使用的 agent 配置摘要，包括：

- `name`
- `kind`
- `display_name`
- 命令模板。
- 输入模式。
- timeout。
- 认证继承摘要。
- labels。
- usage parser 状态。

敏感 env 值必须脱敏。

### 15.3 Task 明细

每个 task 至少展示：

- task id。
- benchmark 原始结果。
- failure class。
- failure code。
- 耗时。
- token/cost。
- 退出码。
- stdout/stderr 链接。
- verifier 日志链接。
- diff 链接；仅 patch-style benchmark 如 SWE-bench Pro 必须产生。
- 复现信息。

### 15.4 报告不做的事

MVP 报告不做：

- 跨 run 排名。
- 自动自然语言结论。
- LLM 失败分析。
- Web dashboard。
- 在线分享服务。

## 16. Run 产物

每个 run 目录至少保存：

```text
~/.harnesslab/runs/<agent>-<benchmark>-<split>-<timestamp>/
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

`diff.patch` 对 Terminal-Bench 等不产生代码 diff 的任务可以缺省。`benchmark.snapshot.json` 和 `task.snapshot.json` 用于 replay；如果 replay 所需的外部数据被用户清理，系统必须明确报错。

这些产物服务于：

- 完整 replay。
- 问题排查。
- 长期实验记录。
- 后续导入多 run 对比系统。

## 17. MVP 验收标准

MVP 必须同时满足：

1. `harnesslab init` 能检测并生成 Codex CLI、Claude Code、opencode、Pi Coding Agent 的配置草稿。
2. `harnesslab doctor` 能检查 Docker、agent 命令、认证继承、配置合法性和 smoke task。
3. 用户能用一条 `harnesslab run` 命令运行 Terminal-Bench benchmark。
4. 用户能用一条 `harnesslab run` 命令运行 SWE-bench Pro。
5. Run 默认 Docker 隔离，默认并发 4，支持 attempts 配置。
6. Run 支持中断后 resume。
7. Run 保存完整配置快照、命令快照、任务快照、日志、diff、评分和失败分类。
8. Run 完成后生成单文件 HTML 报告。
9. 报告明确区分 Execution Failure、Benchmark Failure 和 Partial Success。
10. 报告支持 replay 命令和原始命令复制。
11. token/cost 缺失时明确提示不可比较。
12. 历史 run 永久保存，不自动删除。

一句话验收：

```text
用户在 5 分钟内完成本机 CLI Agent 检测与配置确认，并用一条命令在 Terminal-Bench 或已准备好数据的 SWE-bench Pro 上启动一个隔离、可复现、可 resume 的单 run，结束后得到包含原始评分、耗时、token/cost、配置快照、失败详情和 replay 能力的单文件 HTML 报告。
```

## 18. Roadmap

### P0：单 Run MVP

- CLI init / doctor / run / resume / replay / report open。
- 四个内置 agent profile 模板。
- Terminal-Bench 跑通。
- SWE-bench Pro 跑通。
- 单 run HTML 报告。
- 失败分类。
- token/cost 可选采集。

### P1：多 Run 对比

- 多 run 汇总页。
- agent 横向排行榜。
- 同 agent 不同 profile 对比。
- 多 attempts 稳定性视图。
- 成本/效果散点图。

### P2：回归与私有任务集

- 用户本地私有任务集。
- harness 版本回归视图。
- CI 集成。
- 更细粒度失败归因。
- 可选 LLM 失败分析。

### P3：团队与平台化

- 共享报告。
- 长期趋势。
- 企业内部 benchmark 管理。
- 任务集和运行记录导出。

## 19. 文档边界

本文档只定义产品目标、用户体验、核心对象、MVP 边界和验收标准。

不在本文档中定义：

- 具体技术栈。
- runner 内部实现。
- Docker 镜像构建细节。
- benchmark adapter 内部协议。
- 数据库或文件格式最终实现。
- HTML 报告前端实现方案。

这些内容应在后续技术设计文档中单独展开。
