# HarnessLab Harbor 集成工程计划

> Superseded on 2026-06-15 by
> `docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`.
> This CLI-first Rust + Python Bridge plan is retained as historical context
> only. Do not use it as the implementation source of truth for the WebUI
> redesign.

- Created: 2026-06-15
- Updated: 2026-06-15
- Version: 1.1
- Status: Superseded
- Owner: HarnessLab team
- Related Systems: HarnessLab (Rust), Harbor (Python), Docker
- Risk Level: High
- Plan Type: Full

## 1. Background

HarnessLab 当前是一个 Rust 实现的 CLI 工具，用于评估 CLI coding agent 在 benchmark 上的表现。当前架构存在核心瓶颈：**Runtime 层通过调用外部 runner（如 `tb run`）执行任务，导致 AgentProfile 配置无法在容器内生效**。

Phase 0 评估已确认 Harbor 框架具备以下能力：
- 75+ benchmark 注册，覆盖代码、数学、工具调用、安全审计等领域
- 完善的容器生命周期管理（`BaseEnvironment`）
- 成熟的 agent 注册机制（`BaseInstalledAgent` + `--agent-import-path`）
- Python API 可编程（`Job.create(config)` + `job.run()`）
- Skills/MCP/环境变量配置继承
- `harbor-agents` 包提供 `ClaudeCodeWithSkills` 等高级 agent

## 2. Problem Definition

### 当前行为
- HarnessLab 的 `TerminalBenchAdapter` 调用 `tb run` 作为外部 runner
- `SWEBenchProAdapter` 调用 SWE-bench 官方 evaluator
- AgentProfile 中的 `command`、`env`、`skills`、`tools` 配置被外部 runner 忽略
- 自定义 agent（如 `claude-ds`）无法在容器内生效
- 每接入一个新 benchmark 需要编写大量 Rust runtime 代码

### 期望行为
- HarnessLab 拥有统一的 runtime 层，直接管理容器、agent 安装、agent 执行和 verifier
- AgentProfile 配置完全在容器内生效
- 声明式 agent 注册：用户通过 TOML 配置注册 agent，无需编写代码
- 75+ Harbor benchmark 可直接使用
- 保留 HarnessLab 的 CLI 体验和产品语义

### Gap
- 缺少 Python 执行层来调用 Harbor API
- 缺少 TOML AgentProfile → Harbor AgentConfig 的翻译层
- 缺少 HarnessLab CLI → Harbor Job 的桥接层
- 缺少 Harbor 结果 → HarnessLab Report 的映射层

## 3. Goals

1. **Runtime 统一**：所有 benchmark 执行走 Harbor runtime，不再调用外部 runner
2. **声明式 Agent 注册**：TOML AgentProfile 自动翻译为 Harbor agent，零代码注册
3. **Benchmark 即时可用**：75+ Harbor benchmark 通过 `harnesslab run --benchmark <name>` 直接运行
4. **产品体验保留**：`harnesslab init/doctor/run/report` 命令语义不变
5. **WebUI 可扩展**：Python 层为后续 WebUI 提供后端基础

## 4. Non-goals

- 不重写 Harbor 核心：容器管理、agent 安装、verifier 执行完全复用 Harbor
- 不在 MVP 阶段实现 WebUI：本计划只建立 Python 后端基础
- 不替换 Rust CLI：CLI 层保持 Rust 实现，Python 作为执行引擎
- 不实现 Harbor 的全部 CLI 命令：只桥接 `harbor run` 和 `harbor resume`
- 不实现 Harbor Hub 上传/分享功能
- 不修改 Harbor 源码：纯依赖模式，通过 `pip install harbor` 使用

## 5. Constraints And Assumptions

| Assumption | Verification Method | If Assumption Fails |
|---|---|---|
| Harbor Python API 稳定到足以作为依赖 | Phase 0 已验证 `Job.create()` + `job.run()` | 锁版本 `harbor>=0.13,<0.14` |
| `--agent-import-path` 机制可满足自定义 agent 需求 | Phase 0 已验证 CustomOracle | 改为 fork Harbor |
| Harbor 的 Docker 环境管理与 HarnessLab 需求兼容 | Phase 0 已验证 terminal-bench | 自定义 Environment 子类 |
| Rust CLI 可通过子进程调用 Python 执行层 | 技术上无障碍 | 改为 gRPC/HTTP 桥接 |
| Harbor 结果格式可映射到 HarnessLab Report | 结果 JSON 结构已分析 | 编写适配器转换 |
| 用户机器有 Python 3.12+ 环境 | Harbor 依赖要求 | 需提供 Python 环境引导 |

### Constraints

- **语言边界**：Rust CLI ↔ Python 执行层通过子进程 + JSON 通信
- **版本锁定**：Harbor 版本必须锁定，避免 API breaking change
- **单用户本地**：MVP 不支持多用户、远程执行
- **Docker 必需**：Harbor 的 Docker 环境是唯一 MVP 后端
- **Python 环境管理**：使用 `uv` 自动管理 Python 虚拟环境（见 7.4 Python 环境管理策略）

### 7.4 Python 环境管理策略 (已确定)

**决策：使用 `uv` 自动管理 Python 虚拟环境，同时支持用户自管 Python 环境。**

HarnessLab 需要在用户机器上运行 Python Bridge，但不应要求用户手动配置 Python 环境。采用以下分层策略：

**Layer 1（自动，默认）**：`harnesslab setup bridge` 使用 `uv` 自动完成：
1. 检测系统是否已安装 Python 3.12+
2. 如未安装，使用 `uv python install 3.12` 安装
3. 在 `~/.harnesslab/venvs/` 下创建虚拟环境
4. 在虚拟环境中安装 `harnesslab-bridge` 和 `harbor>=0.13,<0.14`
5. Rust CLI 调用 Python Bridge 时使用该虚拟环境中的 Python

**Layer 2（自管，高级用户）**：用户可自行管理 Python 环境：
- 设置 `HARNESSLAB_PYTHON_PATH` 环境变量指向目标 Python 解释器
- `harnesslab doctor` 检测该 Python 是否满足依赖要求
- 用户需自行安装 `harnesslab-bridge` 和 `harbor`

**Layer 3（fallback）**：如果 uv 不可用且无自管 Python：
- `harnesslab doctor` 提示用户安装 Python 3.12+ 和 pip
- 提供一键安装脚本（macOS: brew, Linux: apt, Windows: winget）

**决策理由**：
- `uv` 是 Python 生态事实标准工具，安装简单，启动快
- 虚拟环境隔离避免污染用户系统 Python
- 支持用户自管满足企业环境约束
- Harbor 升级时，只需重建虚拟环境，不影响 agent 文件

## 6. Current State

### HarnessLab 代码库现状
- Rust workspace：5 个 crate（cli, core, adapters, infra, report）
- 已实现：CLI skeleton, core models, config, agent registry, artifact store, events, fake-terminal, Docker sandbox, run orchestrator, report, resume, replay
- 已实现 adapter：terminal-bench（通过 `tb run`）, swe-bench-pro（通过外部 runner）
- 已有 40+ contract tests, 30+ integration tests
- 已有 `benchmark-compatibility-strategy.md` 定义了统一任务目录结构

### Harbor 代码库现状
- Python 包：`harbor` v0.13.2, `harbor-agents` v0.2.0
- 核心类：`Job`, `Trial`, `BaseEnvironment`, `BaseAgent`, `BaseInstalledAgent`, `AgentFactory`, `EnvironmentFactory`, `VerifierFactory`
- 75+ benchmark adapter（通过 registry）
- 30+ 内置 agent（claude-code, codex, aider, openhands 等）

## 7. Overall Technical Design

### 7.1 架构总览

```
┌─────────────────────────────────────────────────────────────┐
│                    HarnessLab Rust CLI                       │
│  harnesslab init / doctor / run / resume / replay / report  │
└──────────────────────────┬──────────────────────────────────┘
                           │ subprocess + JSON
                           ▼
┌─────────────────────────────────────────────────────────────┐
│              HarnessLab Python Bridge (新增)                  │
│                                                             │
│  ┌─────────────┐  ┌──────────────┐  ┌───────────────────┐  │
│  │ Profile     │  │ Bridge       │  │ Result            │  │
│  │ Translator  │  │ Orchestrator │  │ Mapper            │  │
│  │             │  │              │  │                   │  │
│  │ TOML →      │  │ JobConfig →  │  │ Harbor result →   │  │
│  │ AgentConfig │  │ Job.create() │  │ HarnessLab report │  │
│  └─────────────┘  └──────────────┘  └───────────────────┘  │
│                                                             │
│  ┌─────────────────────────────────────────────────────┐    │
│  │           Agent Materializer (新增)                   │    │
│  │                                                     │    │
│  │  TOML AgentProfile → Python .py file                │    │
│  │  → write to ~/.harnesslab/agents/                   │    │
│  │  → PYTHONPATH injection for Harbor import           │    │
│  └─────────────────────────────────────────────────────┘    │
└──────────────────────────┬──────────────────────────────────┘
                           │ Python API
                           ▼
┌─────────────────────────────────────────────────────────────┐
│                    Harbor Framework                          │
│                                                             │
│  Job.create(config) → Trial → Environment → Agent → Verify  │
│  75+ benchmark adapters  |  30+ built-in agents             │
└─────────────────────────────────────────────────────────────┘
```

### 7.2 关键设计决策

**决策 1：Rust CLI + Python Bridge 的双进程架构**

Rust CLI 保持用户交互和配置管理，Python Bridge 负责 Harbor API 调用。

- 优点：保留 Rust CLI 性能和单二进制体验；复用 Harbor Python 生态
- 缺点：跨进程通信开销；需要管理 Python 环境
- 替代方案：纯 Rust 重写 Harbor runtime（工作量巨大，不可行）；纯 Python 重写 CLI（放弃 Rust 投资）

**决策 2：Agent Materializer 模式**

将 TOML AgentProfile 动态翻译为 Python 类文件，写入 `~/.harnesslab/agents/` 目录，通过 `PYTHONPATH` 注入使 Harbor 可发现。

```python
# 生成的 agent 文件示例：~/.harnesslab/agents/codex_default/codex_default.py
from harbor.agents.installed.base import BaseInstalledAgent, with_prompt_template

class CodexDefaultAgent(BaseInstalledAgent):
    @staticmethod
    def name() -> str:
        return "codex-default"

    def version(self) -> str | None:
        return "1.0.0"

    async def install(self, environment):
        await self.exec_as_root(environment, "npm install -g @openai/codex")

    @with_prompt_template
    async def run(self, instruction, environment, context):
        await self.exec_as_agent(
            environment,
            f"codex exec --full-auto {shlex.quote(instruction)}"
        )
```

- 优点：零代码注册；TOML 配置完全驱动 agent 行为；不侵入 Harbor 安装目录，Harbor 升级不影响 agent 文件
- 缺点：动态代码生成需要严格的模板和安全审查；需要管理 PYTHONPATH
- 替代方案：预编译 agent 包（灵活性不足）；直接复制到 Harbor site-packages（升级时文件丢失）

**决策 3：Benchmark 名称映射**

Harbor 的 benchmark 名称（如 `terminal-bench@2.0`）映射到 HarnessLab 的 benchmark 名称（如 `terminal-bench`）。

- 映射表维护在 Python Bridge 中
- 用户使用 HarnessLab 名称，Bridge 自动转换为 Harbor 格式
- 支持版本选择：`harnesslab run --benchmark terminal-bench@2.0`

**决策 4：结果格式映射**

Harbor 的 `result.json` 映射到 HarnessLab 的 `result.json` + `report.html`。

| Harbor 字段 | HarnessLab 字段 | 映射逻辑 |
|---|---|---|
| `trial.result.reward` | `task.score` | 直接映射 |
| `trial.result.exception_info` | `task.failure_class` | 分类映射 |
| `trial.timing_info` | `task.duration` | 直接映射 |
| `job.stats.n_input_tokens` | `run.usage.tokens_in` | 聚合 |
| `trial.paths.artifacts_dir` | `task.artifacts/` | 路径映射 |

### 7.3 依赖关系

| Dependency | Type | Current Status | Blocking Risk | Handling Plan |
|---|---|---|---|---|
| Harbor Python 包 | third-party | v0.13.2, PyPI | API breaking change | 锁版本 `>=0.13,<0.14` |
| harbor-agents 包 | third-party | v0.2.0, PyPI | 低 | 可选依赖 |
| Python 3.12+ | environment | 需用户安装 | 用户无 Python | 提供安装引导 |
| Docker | system | 已验证可用 | 低 | doctor 检查 |
| Rust CLI ↔ Python Bridge 通信协议 | internal | 待设计 | 协议不稳定 | JSON schema 版本化 |

## 8. Phased Execution Plan

### Phase 1: Python Bridge Foundation

#### Objective
建立 Python Bridge 包的基础结构，实现 Rust CLI → Python Bridge → Harbor 的最小可运行路径。

#### Entry Criteria
- Phase 0 评估完成（GO 决策已确认）
- Harbor v0.13.2 已验证可用
- HarnessLab Rust CLI 已有 `run` 命令框架

#### Entry Criteria Checks
| Entry Criterion | Check Method | Evidence / Output | Owner |
|---|---|---|---|
| Phase 0 GO | 评估报告已输出 | 本仓库 docs/ | team |
| Harbor 可用 | `harbor --version` | 终端输出 | team |
| Rust CLI 编译 | `cargo build` | 编译成功 | team |

#### Design Approach

创建 `harnesslab-bridge` Python 包，作为 Rust CLI 和 Harbor 之间的桥接层。

包结构：
```
harnesslab-bridge/
├── pyproject.toml
├── src/
│   └── harnesslab_bridge/
│       ├── __init__.py
│       ├── cli.py              # Python 侧入口，被 Rust CLI 调用
│       ├── profile_translator.py  # TOML → AgentConfig 翻译
│       ├── agent_materializer.py  # TOML → Python agent 类文件生成
│       ├── bridge_orchestrator.py # JobConfig 构建 + Job.create() + job.run()
│       ├── result_mapper.py    # Harbor result → HarnessLab result 映射
│       ├── benchmark_resolver.py # HarnessLab benchmark name → Harbor dataset
│       └── models.py           # 通信协议模型（JSON schema）
├── tests/
│   ├── test_profile_translator.py
│   ├── test_agent_materializer.py
│   ├── test_bridge_orchestrator.py
│   └── test_result_mapper.py
└── templates/
    └── agents/                 # Agent 类模板（Jinja2）
        ├── installed_agent.py.j2
        └── external_agent.py.j2
```

#### Implementation Tasks

1.1 创建 `harnesslab-bridge` Python 包骨架
- `pyproject.toml`：依赖 `harbor>=0.13,<0.14`, `pydantic`, `jinja2`, `tomli`
- 包结构和 `__init__.py`

1.2 实现通信协议模型 (`models.py`)
- `BridgeRunRequest`：Rust CLI → Python Bridge 的请求模型
  - `agent_profile_path: str`
  - `benchmark_name: str`
  - `benchmark_version: str | None`
  - `split: str | None`
  - `n_concurrent: int`
  - `n_attempts: int`
  - `n_tasks: int | None`
  - `extra_env: dict[str, str]`
  - `jobs_dir: str`
- `BridgeRunResult`：Python Bridge → Rust CLI 的响应模型
  - `status: "success" | "error" | "interrupted"`
  - `job_dir: str`
  - `result_path: str`
  - `report_path: str | None`
  - `stats: RunStats`
  - `error: str | None`

1.3 实现 `cli.py` 入口
- 解析 JSON stdin 或命令行参数
- 调用 `BridgeOrchestrator.run()`
- 输出 JSON 结果到 stdout

1.4 实现 `benchmark_resolver.py`
- 以 `harbor datasets list` 动态发现为主路径，硬编码映射仅用于名称格式转换（如 `class-name` → `class_name`）
- 支持版本选择（`terminal-bench@2.0`）
- 支持列出可用 benchmark

1.5 实现最小 `bridge_orchestrator.py`
- 构建 `JobConfig`
- 调用 `Job.create(config)`
- 调用 `job.run()`
- 收集结果

1.6 实现结构化日志基础设施
- 定义 correlation ID（`run_id`），跨 Rust CLI → Python Bridge → Harbor → Docker Container 四层传递
- Python Bridge 使用 `structlog` 输出 JSON-lines 日志到 stderr
- 所有日志行包含 `run_id`、`phase`、`timestamp`
- `harnesslab doctor --logs` 收集最近运行失败的相关日志

#### Deliverables
- `harnesslab-bridge` Python 包，可 `pip install -e .`
- 最小可运行路径：`python -m harnesslab_bridge --agent oracle --benchmark terminal-bench@2.0 --n-tasks 1`

#### Testing And Validation
| Validation Item | Method | Passing Standard |
|---|---|---|
| 包安装 | `pip install -e .` | 无错误 |
| Harbor API 调用 | 运行 oracle agent on terminal-bench | 1/1 Mean: 1.000 |
| 通信协议 | JSON 序列化/反序列化 | round-trip 一致 |
| Benchmark 解析 | `terminal-bench` → `terminal-bench@2.0` | 正确映射 |

#### Exit Criteria
- `harnesslab-bridge` 可通过 Python 直接调用 Harbor 运行 terminal-bench
- 通信协议模型定义完成并通过测试

#### Review Plan
- 代码审查：包结构、依赖选择、通信协议设计
- 架构审查：Rust ↔ Python 通信方式是否合理

#### Risks And Fallback
| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| Harbor API breaking change | 高 | import 错误 | 锁版本 | fork Harbor |
| Python 环境缺失 | 中 | 用户报错 | doctor 检查 | 内嵌 Python（PyInstaller） |
| 通信协议设计不当 | 中 | 反复修改 | 早期验证 | 改用 gRPC |

#### Gate To Next Phase
- Python Bridge 可成功运行 `harbor run` 等价操作
- 通信协议模型稳定

---

### Phase 2: Agent Materializer

#### Objective
实现 TOML AgentProfile → Harbor Agent 的自动翻译，让用户通过 TOML 配置即可注册自定义 agent。

#### Entry Criteria
- Phase 1 完成
- `harnesslab-bridge` 可运行内置 agent

#### Design Approach

**Agent Materializer 工作流**：

```
TOML AgentProfile
    │
    ▼
ProfileTranslator.parse(toml_path)
    │
    ├─ kind = "claude-code" → 复用 Harbor 内置 ClaudeCode agent
    ├─ kind = "codex"       → 复用 Harbor 内置 Codex agent
    ├─ kind = "custom"      → 生成 Python agent 类文件
    └─ kind = 其他内置      → 映射到 Harbor 对应 agent
    │
    ▼
AgentMaterializer.materialize(translated_profile)
    │
    ├─ 内置 agent → 直接使用 agent name
    └─ 自定义 agent → 生成 .py 文件 → 写入 ~/.harnesslab/agents/
    │
    ▼
AgentConfig(name=..., import_path=..., kwargs=...)
(harbor 通过 PYTHONPATH 或 --agent-import-path 发现 agent)
```

**TOML → Python 映射规则**：

| TOML 字段 | Python agent 方法 | 映射逻辑 |
|---|---|---|
| `kind` + `command` | `install()` + `run()` | 按 kind 选择安装模板 |
| `setup.preset = "builtin"` | `install()` 中使用预设安装命令 | 按 kind 查表 |
| `setup.commands` | `install()` 中追加自定义命令 | 直接映射 |
| `command` + `input_mode` | `run()` 中的 `exec_as_agent()` 调用 | 模板渲染 |
| `env` | `AgentConfig.env` | 直接传递 |
| `auth.inherit_env` | `AgentConfig.env` | 从宿主环境读取并传递 |
| `skills.*` | `AgentConfig.skills` + `AgentConfig.kwargs` | 映射到 Harbor skills 机制 |
| `timeout_sec` | `AgentConfig.override_timeout_sec` | 直接映射 |
| `setup.run_as` | `run()` 中的 `user` 参数 | 映射 |

**内置 Kind 映射表**：

| HarnessLab kind | Harbor agent name | 说明 |
|---|---|---|
| `claude-code` | `claude-code` | 直接使用内置 |
| `codex` | `codex` | 直接使用内置 |
| `opencode` | `opencode` | 直接使用内置 |
| `pi-coding-agent` | `pi` | 直接使用内置 |
| `aider` | `aider` | 直接使用内置 |
| `openhands` | `openhands` | 直接使用内置 |
| `custom` | 动态生成 | Materializer 生成 |

#### Implementation Tasks

2.1 实现 `profile_translator.py`
- 解析 TOML AgentProfile
- 按 kind 分类处理
- 输出 `TranslatedProfile` 数据结构

2.2 实现 Agent 模板系统 (`templates/agents/`)
- `installed_agent.py.j2`：BaseInstalledAgent 子类模板
  - 变量：`class_name`, `agent_name`, `install_commands`, `run_command`, `run_user`
- `external_agent.py.j2`：BaseAgent 子类模板（用于 host execution 场景）

2.3 实现 `agent_materializer.py`
- 内置 kind → 直接返回 Harbor agent name
- 自定义 kind → 渲染模板 → 写入 `~/.harnesslab/agents/` 目录
- 管理 agent 模块的生命周期（生成、注册、清理）
- 通过 PYTHONPATH 或 `--agent-import-path` 绝对路径使 Harbor 可发现 agent

2.4 实现 Agent 目录管理
- 默认目录：`~/.harnesslab/agents/`
- 目录结构：`~/.harnesslab/agents/<agent_name>/<agent_name>.py` + `__init__.py`
- 支持 `harnesslab agent cleanup` 清理不再需要的生成文件
- `doctor` 命令检查 agents 目录中文件的有效性

2.5 集成到 `bridge_orchestrator.py`
- 在 `run()` 前调用 Materializer
- 将翻译结果注入 `AgentConfig`

2.6 实现 Agent Materializer 审计日志
- 记录每次 Materialize 操作：agent name, kind, 生成文件路径, 时间戳
- 记录 TOML 解析错误时的上下文（文件名、行号、错误详情）
- `harnesslab doctor` 可读取审计日志诊断 agent 注册问题

#### Deliverables
- `profile_translator.py` + 测试
- `agent_materializer.py` + 测试
- Agent 模板文件
- 端到端验证：TOML → Python agent → Harbor 运行

#### Testing And Validation
| Validation Item | Method | Passing Standard |
|---|---|---|
| 内置 kind 映射 | `claude-code` → Harbor `claude-code` | agent name 正确 |
| 自定义 agent 生成 | TOML custom → .py 文件 | 文件语法正确，可 import |
| 自定义 agent 运行 | harbor run with generated agent | agent 被加载执行 |
| Skills 传递 | TOML skills → Harbor skills | skills 在容器内生效 |
| 环境变量传递 | TOML env → Harbor agent env | 环境变量在容器内可见 |

#### Exit Criteria
- 所有内置 kind 可正确映射到 Harbor agent
- 自定义 agent 可通过 TOML 配置生成并运行
- Skills 和环境变量正确传递

#### Risks And Fallback
| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| 模板渲染安全 | 高 | 注入攻击 | 沙箱化模板渲染 + 输入转义 | 限制模板能力 |
| Agent 文件在 Harbor 升级后丢失 | ~~中~~ 已消除 | ~~文件复制失败~~ | ~~动态检测~~ 改用 `~/.harnesslab/agents/` 独立目录 | n/a |
| `~/.harnesslab/agents/` 权限或磁盘问题 | 低 | 写入失败 | doctor 检查目录可写 | 使用临时目录 |
| 内置 kind 映射不完整 | 低 | 新 agent 不在表中 | 维护映射表 | 降级为 custom |

#### Gate To Next Phase
- 至少 3 个内置 kind 验证通过（claude-code, codex, custom）
- 自定义 agent 端到端验证通过

---

### Phase 3: Rust CLI Integration

#### Objective
将 Python Bridge 集成到 Rust CLI 中，用户通过 `harnesslab run` 命令即可调用 Harbor runtime。

#### Entry Criteria
- Phase 2 完成
- Agent Materializer 可工作

#### Design Approach

**Rust CLI 调用 Python Bridge 的方式**：

```
harnesslab run --agent codex-default --benchmark terminal-bench
    │
    ▼
Rust CLI 解析参数
    │
    ├─ 读取 AgentProfile TOML
    ├─ 构建 BridgeRunRequest JSON
    │
    ▼
调用 Python Bridge 子进程
    python -m harnesslab_bridge run --request-json '<json>'
    │
    ▼
Python Bridge 执行
    ├─ 翻译 AgentProfile
    ├─ Materialize agent
    ├─ 构建 JobConfig
    ├─ Job.create() + job.run()
    ├─ 映射结果
    │
    ▼
输出 BridgeRunResult JSON 到 stdout
    │
    ▼
Rust CLI 解析结果
    ├─ 生成 report.html
    ├─ 输出摘要
    └─ 返回 exit code
```

**Python 环境管理**：

Rust CLI 需要确保 Python Bridge 可用。策略：

1. **检测**：`doctor` 命令检查 `harnesslab-bridge` 是否已安装
2. **安装引导**：如果未安装，提示用户运行 `pip install harnesslab-bridge`
3. **路径发现**：自动检测 `harnesslab-bridge` 的安装位置
4. **版本检查**：确保 `harnesslab-bridge` 和 `harbor` 版本兼容

#### Implementation Tasks

3.1 修改 Rust CLI `run` 命令
- 新增 `--runtime harbor` 参数（默认值）
- 构建BridgeRunRequest
- 调用 Python Bridge 子进程
- 解析 BridgeRunResult
- 生成 report.html

3.2 实现 Python Bridge 调用器（Rust）
- `BridgeExecutor` struct：管理 Python 子进程
- 超时控制
- 进度流式输出（Python Bridge 通过 stderr 输出进度）
- 信号转发（Ctrl+C）

3.2.1 实现 Bridge 生命周期管理协议
- **Heartbeat 机制**：Python Bridge 每 5 秒向 stderr 输出 `{"type": "heartbeat", "ts": <iso8601>}` 心跳行
- **健康检查**：Rust `BridgeExecutor` 检测心跳超时（默认 30 秒无心跳视为 stale）
- **Graceful Degradation**：stale 检测后发送 SIGTERM，等待 10 秒后 SIGKILL
- **Orphan 清理**：进程退出时（正常/异常）确保子进程树完全终止；使用进程组（`setpgid`）管理
- **Docker 容器泄漏防护**：Bridge 进程退出时，Python 侧通过 `atexit` 注册清理回调，调用 Harbor 的 `job.cancel()` 清理容器
- **Bridge 重启策略**：stale 后自动重试（最多 1 次），失败后返回明确错误给用户
- **结构化 stderr 协议**：所有进度/心跳/错误信息使用 JSON-lines 格式，Rust 侧按行解析，过滤非 JSON 行

3.3 修改 `doctor` 命令
- 检查 Python 3.12+ 可用性
- 检查 `harnesslab-bridge` 安装状态
- 检查 `harbor` 安装状态和版本
- 检查 Docker 可用性

3.4 修改 `benchmark list` 命令
- 调用 Python Bridge 获取 Harbor benchmark 列表
- 合并 HarnessLab 本地 benchmark 和 Harbor benchmark
- 展示 benchmark 名称、版本、任务数

3.5 修改 `agent list` 命令
- 展示 Harbor 内置 agent
- 展示自定义 agent（TOML profile）

3.6 实现 `harnesslab-bridge` 安装命令
- `harnesslab setup bridge`：安装 Python Bridge 及其依赖
- 自动创建虚拟环境或使用 uv

3.7 实现 Bridge 运行日志收集
- Rust CLI 侧：收集子进程 stderr 中的 JSON-lines 日志
- 将运行日志写入 `~/.harnesslab/logs/<run_id>/bridge.log`
- `harnesslab doctor --logs` 展示最近失败的运行日志摘要

#### Deliverables
- Rust CLI 可通过 `harnesslab run` 调用 Harbor runtime
- `doctor` 检查 Python Bridge 状态
- `benchmark list` 展示 Harbor benchmark

#### Testing And Validation
| Validation Item | Method | Passing Standard |
|---|---|---|
| CLI → Bridge 调用 | `harnesslab run --agent oracle --benchmark terminal-bench --split smoke` | 运行成功 |
| 信号转发 | Ctrl+C 中断运行 | 优雅停止 |
| Doctor 检查 | `harnesslab doctor` | 检测 Python Bridge 状态 |
| Benchmark 列表 | `harnesslab benchmark list` | 展示 Harbor benchmark |

#### Exit Criteria
- `harnesslab run` 可成功运行 Harbor benchmark
- `harnesslab doctor` 可检测所有依赖
- 信号处理正确

#### Risks And Fallback
| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| Python 子进程启动慢 | 低 | 用户感知延迟 | 预热 Python 进程 | 接受延迟 |
| 跨进程通信失败 | 高 | JSON 解析错误 | 严格 schema 验证 + 协议版本协商 | 重试机制 |
| 信号丢失 | 中 | Ctrl+C 无响应 | 信号处理器 + 进程组管理 | kill 子进程 |
| Bridge 挂死 | 高 | 心跳超时 | 3.2.1 生命周期管理协议 | 自动重启 + 用户提示 |

#### Gate To Next Phase
- `harnesslab run` 端到端验证通过
- `harnesslab doctor` 完整检查通过

---

### Phase 4: Result Mapping & Report

#### Objective
将 Harbor 的执行结果映射到 HarnessLab 的报告格式，生成单文件 HTML 报告。

#### Entry Criteria
- Phase 3 完成
- `harnesslab run` 可执行并产出 Harbor 结果

#### Design Approach

**结果映射流程**：

```
Harbor Job Result (result.json)
    │
    ▼
ResultMapper.map()
    │
    ├─ Job stats → Run summary
    ├─ Trial results → Task results
    │   ├─ reward → score
    │   ├─ exception_info → failure classification
    │   ├─ timing_info → duration
    │   └─ artifact paths → artifact references
    ├─ Token usage → Usage data
    │
    ▼
HarnessLab Result (result.json)
    │
    ▼
Report Generator (Rust, askama)
    │
    ▼
report.html
```

**Harbor Trial Result → HarnessLab Task Result 映射**：

| Harbor 字段 | HarnessLab 字段 | 转换逻辑 |
|---|---|---|
| `trial_name` | `task_id` | 解析 task name |
| `reward` | `score` | 直接映射 |
| `exception_info.exception_type` | `failure_class` | 分类映射表 |
| `exception_info.message` | `failure_message` | 直接映射 |
| `timing_info.started_at` | `started_at` | 直接映射 |
| `timing_info.finished_at` | `finished_at` | 直接映射 |
| `agent_context.n_input_tokens` | `usage.tokens_in` | 聚合 |
| `agent_context.n_output_tokens` | `usage.tokens_out` | 聚合 |
| `agent_context.cost_usd` | `usage.cost_usd` | 聚合 |
| `verifier_result.reward` | `verifier_score` | 直接映射 |

**Failure Classification 映射**：

| Harbor Exception | HarnessLab Failure Class |
|---|---|
| `AgentTimeoutError` | `execution/agent_timeout` |
| `EnvironmentStartTimeoutError` | `execution/sandbox_create_failed` |
| `SandboxBuildFailedError` | `execution/sandbox_create_failed` |
| `VerifierTimeoutError` | `benchmark/verifier_timeout` |
| `NonZeroAgentExitCodeError` | `execution/agent_nonzero_exit` |
| 其他 `ValueError` | `benchmark/test_failed` |

#### Implementation Tasks

4.1 实现 `result_mapper.py`
- 解析 Harbor `result.json`
- 映射到 HarnessLab `result.json` 格式
- 处理 token usage 聚合
- 处理 failure classification

4.2 修改 Rust CLI report 生成
- 接收 Python Bridge 返回的结果
- 使用现有 askama 模板生成 HTML 报告
- 确保 Harbor 结果格式与现有报告兼容

4.3 实现 artifact 路径映射
- Harbor 的 artifact 目录结构 → HarnessLab 的 artifact 目录结构
- 日志文件路径映射
- Trajectory 文件路径映射

4.4 实现 resume 支持
- 解析 Harbor 的 resume 机制
- 映射到 HarnessLab 的 `harnesslab run resume` 命令
- 确保 resume 后的结果正确合并

#### Deliverables
- `result_mapper.py` + 测试
- 修改后的 report 生成
- Resume 支持

#### Testing And Validation
| Validation Item | Method | Passing Standard |
|---|---|---|
| 结果映射 | Harbor result → HarnessLab result | 字段完整且正确 |
| 报告生成 | 运行后打开 report.html | 报告可读且完整 |
| Resume | 中断后 resume | 结果正确合并 |
| Failure 分类 | 各类错误场景 | 分类正确 |

#### Exit Criteria
- 完整 run 后可生成 HarnessLab 格式的 HTML 报告
- Resume 功能正常

#### Risks And Fallback
| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| Harbor 结果格式变更 | 中 | 字段缺失 | 版本锁定 + schema 验证 | 降级为原始输出 |
| 报告模板不兼容 | 低 | 渲染错误 | 适配层 | 新模板 |

#### Gate To Next Phase
- 端到端运行 + 报告生成 + resume 全部通过

---

### Phase 5a: Legacy Adapter Deprecation

#### Objective
将现有 Rust adapter 标记为 deprecated，通过 feature flag 保持可回退，同时全面验证 Harbor runtime 覆盖所有关键 benchmark。

#### Entry Criteria
- Phase 4 完成
- Harbor runtime 可完整运行并产出报告

#### Design Approach

**Deprecation 策略（非删除）**：

不物理删除旧 Rust adapter 代码，而是通过以下机制管理过渡：

1. **Feature Flag**：`--runtime legacy` 保留旧路径（默认 `--runtime harbor`）
2. **Deprecation Warning**：使用 `--runtime legacy` 时输出 deprecation 警告和迁移指引
3. **全面验证**：在标记 deprecated 之前，验证 Harbor runtime 覆盖 top-10 benchmark

**Top-10 Benchmark 验证清单**：

| # | Benchmark | Agent | 验证状态 |
|---|---|---|---|
| 1 | terminal-bench@2.0 | oracle |  |
| 2 | swebench-verified@1.0 | oracle |  |
| 3 | aider-polyglot@1.0 | oracle |  |
| 4 | crewai@1.0 | oracle |  |
| 5 | gaia@1.0 | oracle |  |
| 6 | usaco@1.0 | oracle |  |
| 7 | math@1.0 | oracle |  |
| 8 | swebench-multilingual@1.0 | oracle |  |
| 9 | agent-harness@1.0 | oracle |  |
| 10 | swe-rebench@1.0 | oracle |  |

**保留的旧代码**：
- `runner/external/` 目录完整保留（标记为 `#[deprecated]`）
- `ExternalRunnerKind` 保留（标记为 `#[deprecated]`）
- 旧 adapter 保留但通过 feature flag 控制是否编译

#### Implementation Tasks

5a.1 在 `harnesslab run` 中保留 `--runtime` 参数
- `--runtime harbor`：默认，走 Python Bridge
- `--runtime legacy`：走旧路径，输出 deprecation 警告

5a.2 标记旧代码为 deprecated
- 给 `ExternalRunnerKind` 添加 `#[deprecated(since = "0.1.0", note = "use --runtime harbor")]`
- 给旧 adapter 模块添加 deprecation 注释

5a.3 执行 top-10 benchmark 验证
- 逐一运行上述 10 个 benchmark
- 记录通过/失败结果
- 失败的 benchmark 分析根因并修复或记录为 known issue

5a.4 更新测试
- 保留旧路径的 contract tests
- 新增 Harbor 路径的并行测试
- CI 中两个路径都运行

#### Deliverables
- `--runtime legacy` 保留旧路径
- Deprecation 警告
- Top-10 benchmark 验证报告

#### Testing And Validation
| Validation Item | Method | Passing Standard |
|---|---|---|
| Top-10 benchmark 验证 | `harnesslab run --benchmark <name> --split smoke` | >= 8/10 通过 |
| Legacy 路径仍可用 | `harnesslab run --runtime legacy --benchmark terminal-bench` | 输出 deprecation 警告 |
| Deprecation 警告 | 运行 legacy 路径 | 明确指引用户迁移 |

#### Exit Criteria
- Top-10 benchmark >= 80% 通过 Harbor runtime
- `--runtime legacy` 保留且可用
- Deprecation 警告清晰

#### Risks And Fallback
| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| Harbor 不支持某个 benchmark | 中 | 运行失败 | Harbor registry 确认 + 在验证报告中记录 | 该 benchmark 保留 legacy 路径 |
| 验证耗时长 | 低 | 进度延迟 | 并行验证 + 使用 smoke split | 缩小到 top-5 |

#### Gate To Next Phase
- Top-10 benchmark 验证完成
- Deprecation 标记完成

---

### Phase 5b: Legacy Adapter Removal

#### Objective
在 Phase 5a 验证通过且至少一个版本观察期后，物理删除旧 Rust adapter 代码。

#### Entry Criteria
- Phase 5a 完成
- 至少一个发布版本中旧路径保留为 deprecated（观察期 >= 2 周）
- 无用户报告旧路径的不可替代问题
- Top-10 benchmark 全部通过 Harbor runtime

#### Design Approach

**最终清理**：

1. 移除 `runner/external/` 目录下的所有 runtime adapter
2. 移除 `ExternalRunnerKind` enum
3. 移除 `--runtime` 参数（统一使用 Harbor）
4. 移除旧 adapter 的 contract tests
5. 简化 `harnesslab-infra`（移除不再需要的 Docker 直接调用）

**保留的 Rust 层**（不受删除影响）：
- Agent Registry（TOML 解析、验证、doctor 检查）
- Config Service（全局配置管理）
- Report Service（HTML 报告生成）
- CLI 层（命令解析、用户交互）

#### Implementation Tasks

5b.1 移除 `runner/external/`
- 删除 `terminal_bench_adapter.rs`, `terminal_bench_runtime.rs` 等
- 删除 `swe_bench_pro_adapter.rs` 等
- 清理 `runtime_adapter.rs` 中的 legacy 分发逻辑

5b.2 移除 `ExternalRunnerKind` 和 `--runtime` 参数
- 删除 enum 定义和所有 match 分支
- 移除 CLI 参数定义

5b.3 清理 `harnesslab-infra/`
- 评估并移除不再需要的 Docker 直接调用
- 简化 infra crate

5b.4 清理 `harnesslab-adapters/`
- 移除 runtime 层（保留数据层）

5b.5 清理测试和文档
- 移除 legacy 路径的 contract tests
- 更新 architecture.md, mvp-development-spec.md, technology-decisions.md

#### Deliverables
- 清理后的代码库（无旧 runner 依赖）
- 更新的文档
- 更新的测试套件

#### Testing And Validation
| Validation Item | Method | Passing Standard |
|---|---|---|
| terminal-bench 运行 | `harnesslab run --agent oracle --benchmark terminal-bench` | 成功 |
| swebench-verified 运行 | `harnesslab run --agent oracle --benchmark swebench-verified` | 成功 |
| 无外部 runner 依赖 | `grep -r "tb run" crates/` | 无结果 |
| 编译通过 | `cargo check --workspace` | 0 errors |
| 无 legacy 代码残留 | `grep -r "ExternalRunnerKind" crates/` | 无结果 |

#### Exit Criteria
- 所有 benchmark 运行走 Harbor runtime
- 无外部 runner 依赖
- 测试全部通过
- 观察期无用户回退需求

#### Risks And Fallback
| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| 新发现的不可替代场景 | 高 | 用户报告 | git revert 恢复到 Phase 5a 状态 | 保留该 benchmark 的特殊处理 |
| 大量代码删除导致回归 | 中 | 测试失败 | 逐步删除 + CI 验证 | git revert |

---

### Phase 6: Declarative Agent Registration & WebUI Foundation

#### Objective
完善声明式 agent 注册体验，并为 WebUI 建立后端基础。

#### Entry Criteria
- Phase 5b 完成
- Harbor runtime 完全接管（无 legacy 路径）

#### Design Approach

**声明式 Agent 注册增强**：

1. **Agent Profile Schema v2**：扩展 TOML schema 支持更多 Harbor 能力
   - `mcp_servers`：MCP 服务器配置
   - `skills`：Skills 目录配置
   - `hooks`：生命周期钩子
   - `model`：模型配置

2. **Agent Profile 验证增强**：
   - `doctor` 命令验证 agent profile 与 Harbor 的兼容性
   - 检测 kind 对应的 Harbor agent 是否可用
   - 验证 skills 目录结构

3. **Agent Profile 模板库**：
   - 内置 10+ agent profile 模板
   - 覆盖主流 CLI agent 的常见配置

**WebUI 后端基础**：

1. **FastAPI 后端**：
   - `/api/runs`：运行管理
   - `/api/agents`：Agent 管理
   - `/api/benchmarks`：Benchmark 管理
   - `/api/results`：结果查询
   - WebSocket：实时进度

2. **数据模型**：
   - 复用 `harnesslab-bridge` 的模型
   - 新增 WebUI 特有的模型（用户偏好、运行历史等）

#### Implementation Tasks

6.1 扩展 Agent Profile Schema
- 新增 `mcp_servers` 字段
- 新增 `skills` 配置增强
- 新增 `model` 字段
- 更新 `profile_translator.py`

6.2 增强 `doctor` 命令
- 验证 agent profile 与 Harbor 兼容性
- 验证 skills 目录结构
- 验证 MCP 配置

6.3 创建 Agent Profile 模板库
- 10+ 内置模板
- `harnesslab agent init --template <name>` 命令

6.4 实现 FastAPI 后端骨架
- 基本路由结构
- WebSocket 支持
- 认证（MVP：本地无认证）

6.5 实现 WebUI 前端骨架
- React + Vite
- 基本页面结构：Dashboard, Runs, Agents, Benchmarks

#### Deliverables
- 增强的 Agent Profile schema
- Agent Profile 模板库
- FastAPI 后端骨架
- WebUI 前端骨架

#### Testing And Validation
| Validation Item | Method | Passing Standard |
|---|---|---|
| MCP 配置传递 | TOML → Harbor agent | MCP 服务器在容器内可用 |
| Skills 增强传递 | TOML → Harbor skills | Skills 在容器内生效 |
| WebUI 后端 | API 调用 | 返回正确数据 |
| WebUI 前端 | 浏览器访问 | 页面可渲染 |

#### Exit Criteria
- 声明式 agent 注册覆盖所有 Harbor 能力
- WebUI 后端可提供基本 API
- WebUI 前端可展示运行结果

#### Risks And Fallback
| Risk | Impact | Trigger Signal | Mitigation | Fallback |
|---|---|---|---|---|
| WebUI 范围膨胀 | 中 | 功能蔓延 | 严格 MVP 边界 | 只做 API，不做前端 |
| Agent Profile 复杂度 | 中 | 用户困惑 | 模板引导 | 简化 schema |

#### Gate To Next Phase
- 声明式 agent 注册完整验证
- WebUI 骨架可运行

---

### Phase 7: Hardening & Release

#### Objective
全面加固、性能优化、文档完善，准备发布。

#### Entry Criteria
- Phase 6 完成

#### Implementation Tasks

7.1 性能优化
- Python Bridge 启动速度优化
- 结果映射性能优化
- 大规模 benchmark 运行测试（100+ tasks）

7.2 错误处理增强
- 完善所有错误路径
- 用户友好的错误消息
- 错误恢复机制

7.3 安全审查
- Agent Materializer 模板注入防护
- 环境变量脱敏
- Docker socket 安全

7.4 文档完善
- 用户指南
- Agent 注册指南
- Benchmark 使用指南
- 架构文档更新

7.5 CI/CD
- GitHub Actions 集成
- Python Bridge 测试
- 端到端测试

7.6 发布准备
- 版本号确定
- Changelog
- Release notes

#### Exit Criteria
- 所有测试通过
- 文档完整
- CI/CD 绿灯
- 性能达标

## 9. Phase Gate Overview

```
Phase 1: Python Bridge Foundation
    │ Gate: Python Bridge 可运行 Harbor benchmark
    ▼
Phase 2: Agent Materializer
    │ Gate: TOML → Harbor agent 端到端验证
    ▼
Phase 3: Rust CLI Integration
    │ Gate: harnesslab run 端到端验证
    ▼
Phase 4: Result Mapping & Report
    │ Gate: 报告生成 + resume 验证
    ▼
Phase 5a: Legacy Adapter Deprecation
    │ Gate: Top-10 benchmark 验证 + deprecation 标记
    ▼
Phase 5b: Legacy Adapter Removal
    │ Gate: 无外部 runner 依赖 + 观察期通过
    ▼
Phase 6: Declarative Agent & WebUI Foundation
    │ Gate: 声明式注册 + WebUI 骨架
    ▼
Phase 7: Hardening & Release
    │ Gate: 全部测试通过 + 文档完整
```

## 10. Risks, Dependencies, And Mitigations

| Risk | Probability | Impact | Mitigation | Fallback |
|---|---|---|---|---|
| Harbor API breaking change | Medium | High | 版本锁定 + 兼容层 | Fork Harbor |
| Python 环境管理复杂 | Medium | Medium | uv 虚拟环境自动管理 | 内嵌 Python |
| Rust ↔ Python 通信不稳定 | Low | High | 严格 JSON schema + 重试 | gRPC |
| Agent Materializer 安全风险 | Medium | High | 模板沙箱化 + 输入验证 | 限制自定义 agent |
| Harbor 不支持某个 benchmark | Low | Medium | Harbor registry 确认 | 自定义 adapter |
| 大量 Rust 代码删除导致回归 | Medium | High | 逐步迁移 + 保留旧路径 | 回滚 |
| WebUI 范围膨胀 | Medium | Medium | 严格 MVP 边界 | 只做 API |

## 11. Testing And Validation Strategy

### 11.1 单元测试
- Python Bridge 每个模块独立测试
- Rust CLI 修改部分独立测试

### 11.2 集成测试
- Rust CLI → Python Bridge → Harbor 端到端
- 每个 Phase 的 Gate 测试

### 11.3 冒烟测试
- `harnesslab run --agent oracle --benchmark terminal-bench --split smoke`
- `harnesslab run --agent oracle --benchmark swebench-verified --n-tasks 1`

### 11.4 回归测试
- 保留现有 Rust contract tests
- 新增 Harbor bridge contract tests

## 12. Release, Rollback, And Fallback Strategy

### Release
- Phase 7 完成后发布 v0.1.0
- 同时发布 `harnesslab-bridge` Python 包

### Rollback
- 每个 Phase 完成后 git tag
- 回滚到任意 Phase 的 tag 即可
- Rust CLI 保留 `--runtime legacy` 选项（Phase 5 之前）

### Fallback
- 如果 Harbor 集成失败，可回退到当前 Rust runtime
- 如果 Python Bridge 不稳定，可考虑纯 Rust 重写 Harbor runtime

## 13. Observability And Success Metrics

| Metric | Target | Measurement |
|---|---|---|
| 首次运行时间 | < 10 分钟（从安装到第一次 run 完成） | 用户测试 |
| Benchmark 可用数 | >= 75 | `harnesslab benchmark list` |
| Agent 注册零代码 | 100% TOML 配置 | 验证所有 kind |
| 报告生成成功率 | >= 99% | CI 测试 |
| Resume 成功率 | >= 95% | 集成测试 |

## 14. Open Questions

1. **Harbor 版本策略**：是否跟随 Harbor 最新版本？还是长期锁定 0.13.x？
2. **Agent Materializer 安全边界**：自定义 agent 的模板渲染是否需要限制？如何防止代码注入？
3. **WebUI 优先级**：Phase 6 的 WebUI 是否应该在 MVP 中实现？还是推迟到 v0.2.0？
4. **Harbor fork 时机**：什么条件下应该从依赖模式切换到 fork 模式？
5. **Rust adapter 数据层保留范围**：Phase 5b 迁移后，Rust adapter 的哪些数据层功能需要保留？

### 已决策 (Resolved)

- **Python 环境管理**（原 Q1）：使用 `uv` 自动管理虚拟环境（`~/.harnesslab/venvs/`），支持 `HARNESSLAB_PYTHON_PATH` 自管模式。详见 7.4 Python 环境管理策略。

## 15. Change Log

| Date | Version | Change |
|---|---|---|
| 2026-06-15 | 1.1 | 对抗性审查修复：BF-1 新增 Bridge 生命周期管理协议；BF-2 Phase 5 拆分为 5a(deprecated)+5b(物理删除)并增加 top-10 benchmark 验证；BF-3 Agent Materializer 改为 `~/.harnesslab/agents/` + PYTHONPATH；BF-4 确定 Python 环境管理策略（uv 自动管理）；各 Phase 增加日志/可观测性建设任务 |
| 2026-06-15 | 1.0 | 初始计划 |

## 16. Plan Quality Checklist

- [x] Background and problem definition are clear
- [x] Goals are measurable and non-goals control scope
- [x] Facts, assumptions, constraints, risks, and open questions are separated
- [x] Complexity and plan depth are justified (High risk → Full Plan)
- [x] Work is divided into progressive phases (7 phases)
- [x] Each phase has entry criteria, checks, tasks, deliverables, validation, exit criteria, review plan, risks, fallback, and next gate
- [x] High-risk unknowns are investigated early (Phase 0 completed)
- [x] Risks include trigger signals and mitigations
- [x] Tests and validation have passing standards
- [x] Production impact includes release, rollback, fallback, observability
- [x] The plan does not invent repository or system facts
