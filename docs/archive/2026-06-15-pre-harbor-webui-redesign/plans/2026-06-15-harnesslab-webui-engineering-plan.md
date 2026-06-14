# HarnessLab WebUI 工程计划

> Superseded on 2026-06-15 by
> `docs/plans/2026-06-15-harbor-webui-redesign-engineering-plan.md`.
> This v2 draft captured the first WebUI pivot, but it is no longer canonical:
> the v3 plan adds project-state evaluation, SQLite metadata, SSE streams,
> explicit Harbor API boundaries, failure classification, and phased gates.

- Created: 2026-06-15
- Updated: 2026-06-15
- Version: 2.0
- Status: Superseded
- Owner: HarnessLab team
- Related Systems: Harbor (Python), Docker, Vue 3, FastAPI
- Risk Level: Medium
- Plan Type: Full
- Replaces: v1.x (CLI-first plan, deprecated)

## 1. Background

### 1.1 产品方向重大调整 (v2.0)

v1.x 计划以 Rust CLI 为核心，通过 Python Bridge 调用 Harbor。v2.0 调整方向：

- **WebUI 优先**：CLI 不再是一等公民，改为 `python -m harnesslab web` 启动本地 Web 服务
- **Vue 3 + FastAPI + Harbor**：纯 Web 前后端分离架构，简单优先
- **大胆重写**：不保留现有 Rust 代码资产，以 WebUI 体验为中心重新设计
- **三大核心模块**：Agent 管理 / 实验管理 / 榜单

### 1.2 复用资产

| 资产 | 来源 | 复用方式 |
|---|---|---|
| Harbor 框架 | Phase 0 已验证 | 作为 Python 依赖，执行引擎 |
| 75+ benchmark 注册 | Harbor | 动态发现，无需适配 |
| 7 种 AgentKind 定义 | v1.x 计划 | Agent 模板库预设 |
| Agent Materializer 方案 | v1.x 计划 | `~/.harnesslab/agents/` + PYTHONPATH |
| 报告格式 | Harbor | 直接复用 result.json 渲染 |
| 生命周期管理协议 | v1.x 计划 | heartbeat / 健康检查 / orphan 清理 |

### 1.3 PRD 引用

详细产品设计参见 [PRD: HarnessLab WebUI](../../prd/2026-06-15-harnesslab-webui-prd.md)。

核心产品决策：
- 纯本地单用户，无认证，文件系统持久化
- Agent 创建：模板 + 向导模式（7 个预设模板）
- 实验流程：配置 → 保存 → 运行 → 报告，支持模板复用和克隆
- 实时反馈：进度概览 + 可展开 WebSocket 日志流
- Docker 状态可见 + 问题诊断

## 2. Architecture Overview

### 2.1 System Architecture

```
┌────────────────────────────────────────────────────┐
│                   Browser (Vue 3)                   │
│  ┌──────────┐ ┌──────────┐ ┌───────────────────┐  │
│  │  Agent   │ │  实验    │ │      榜单          │  │
│  │  Manager │ │  Manager │ │   Leaderboard      │  │
│  └────┬─────┘ └────┬─────┘ └─────────┬─────────┘  │
│       │            │                │              │
│       └────────────┼────────────────┘              │
│                    │ HTTP REST + WebSocket          │
└────────────────────┼───────────────────────────────┘
                     │
┌────────────────────┼───────────────────────────────┐
│              FastAPI Backend (Python)               │
│  ┌─────────────────┼─────────────────────────────┐ │
│  │            API Layer                           │ │
│  │  /api/agents  /api/experiments  /api/leaderboard │
│  └────────┬───────┼────────┬─────────────────────┘ │
│           │       │        │                        │
│  ┌────────┴───┐ ┌─┴──────┐ ┌─┴───────────────────┐ │
│  │ Agent      │ │ Experi-│ │ Leaderboard          │ │
│  │ Service    │ │ ment   │ │ Service              │ │
│  │            │ │ Service│ │                      │ │
│  └─────┬──────┘ └───┬────┘ └──────────────────────┘ │
│        │             │                               │
│  ┌─────┴─────────────┴─────┐ ┌────────────────────┐ │
│  │     Harbor Engine        │ │  File Storage      │ │
│  │  (Job.create/run/cancel) │ │  /agents /exps ... │ │
│  └──────────────────────────┘ └────────────────────┘ │
└──────────────────────────────────────────────────────┘
```

### 2.2 Data Directory Structure

```
~/.harnesslab/
├── agents/                  # Agent 配置文件 (TOML)
│   ├── my-claude.toml
│   └── my-codex.toml
├── agents-py/               # Materialized agent Python files
│   └── my_claude.py
├── experiments/             # 实验记录 (JSON)
│   ├── exp-20260615-001/
│   │   ├── config.json      # 实验配置快照
│   │   ├── result.json      # Harbor 原始结果
│   │   └── log.txt          # 运行日志
│   └── exp-20260615-002/
├── templates/               # 实验模板 (JSON)
│   └── smoke-test.json
├── reports/                 # 生成的 HTML 报告
├── leaderboard/             # 榜单缓存 (JSON)
└── db/                      # 元数据索引 (JSON)
    ├── agents-index.json    # agent 索引
    └── experiments-index.json
```

### 2.3 Technology Stack

| 层 | 技术 | 理由 |
|---|---|---|
| 前端 | Vue 3 + Vite + Vue Router | 简单优先，生态成熟 |
| UI 组件 | 原生 CSS / Tailwind（按需） | 不过度工程化 |
| 后端框架 | FastAPI | Python 原生，与 Harbor 同语言 |
| 实时通信 | WebSocket (FastAPI built-in) | 实验日志推送 |
| 执行引擎 | Harbor Python API | Phase 0 已验证 |
| 数据存储 | 本地文件系统 (TOML/JSON) | 无数据库依赖 |
| Python 环境 | uv 虚拟环境管理 | 隔离依赖，自动安装 |
| 包管理 | pip + pyproject.toml | 标准 Python 打包 |

### 2.4 API Design (RESTful)

```
# Agent
GET    /api/agents              # 列出所有 agent
POST   /api/agents              # 创建 agent
GET    /api/agents/:id          # 获取 agent 详情
PUT    /api/agents/:id          # 更新 agent
DELETE /api/agents/:id          # 删除 agent
GET    /api/agent-templates     # 获取预设模板列表

# Experiment
GET    /api/experiments         # 列出所有实验
POST   /api/experiments         # 创建实验
GET    /api/experiments/:id     # 获取实验详情
DELETE /api/experiments/:id     # 删除实验
POST   /api/experiments/:id/run      # 启动实验
POST   /api/experiments/:id/cancel   # 取消实验
GET    /api/experiments/:id/report   # 获取报告 (HTML)
WS     /api/experiments/:id/logs     # WebSocket 实时日志

# Templates
GET    /api/templates           # 列出模板
POST   /api/templates           # 保存模板（从实验）
DELETE /api/templates/:id       # 删除模板

# Leaderboard
GET    /api/leaderboard?benchmark=X   # 按 benchmark 排名

# System
GET    /api/system/status       # Docker/Harbor 状态
GET    /api/benchmarks          # 可用的 benchmark 列表
```

## 3. Phase Plan

### Phase Gate Overview

```
Phase 0 ──► Phase 1 ──► Phase 2 ──► Phase 3 ──► Phase 4 ──► Phase 5
 基础        Agent      实验引擎    实验管理    榜单+      打磨发布
 脚手架      管理                   +实时日志   Dashboard
│          │          │          │          │          │
▼ Gate     ▼ Gate     ▼ Gate     ▼ Gate     ▼ Gate     ▼
后端启动   Agent CRUD  Harbor     完整实验   榜单可用   全链路
Vue 页面   向导可用    run 成功   流程跑通    Dashboard  稳定
```

### Phase 0: Project Scaffold & Infrastructure

**目标**：搭建 Vue + FastAPI 项目骨架，确保前后端通信正常，Harbor 可用。

**工作量**：2-3 天

**Tasks**：

| # | Task | Detail |
|---|---|---|
| 0.1 | Python 项目初始化 | `pyproject.toml`，依赖声明（fastapi, uvicorn, harbor, websockets），uv 虚拟环境 |
| 0.2 | FastAPI 最小应用 | `harnesslab/` 包结构，`main.py` 入口，`/api/system/status` health check 端点 |
| 0.3 | Vue 3 项目初始化 | `frontend/` 目录，Vite + Vue Router，基础布局组件（侧边栏 + 主内容区） |
| 0.4 | 开发代理配置 | Vite proxy 到 FastAPI `localhost:8000`，解决 CORS |
| 0.5 | 文件存储层 | `~/.harnesslab/` 目录初始化，`FileStore` 工具类（read/write JSON/TOML） |
| 0.6 | Harbor 验证端点 | `/api/system/status` 返回 Harbor 版本、Docker 状态、可用 benchmark 数量 |
| 0.7 | 启动脚本 | `python -m harnesslab web` 命令：启动 FastAPI + 打开浏览器 |
| 0.8 | 结构化日志基础设施 | correlation ID 注入、请求日志中间件、stderr JSON-lines 格式 |

**Gate Check**：
- [ ] `python -m harnesslab web` 成功启动，浏览器打开 Dashboard 页面
- [ ] `/api/system/status` 返回 Harbor 版本和 Docker 状态
- [ ] 前端侧边栏导航正常渲染
- [ ] 结构化日志正常输出

### Phase 1: Agent Management Module

**目标**：完成 Agent 的 CRUD 全流程，包括模板选择、分步向导、TOML 配置生成。

**工作量**：4-5 天

**Tasks**：

| # | Task | Detail |
|---|---|---|
| 1.1 | Agent 数据模型 | `AgentConfig` Pydantic model，TOML 序列化/反序列化 |
| 1.2 | Agent Service | 增删改查逻辑，索引文件维护，名称唯一性校验 |
| 1.3 | Agent API | `GET/POST/PUT/DELETE /api/agents`，错误处理（进行中实验保护） |
| 1.4 | 预设模板数据 | 7 个 AgentKind 模板定义（claude-code, codex, opencode, pi-coding-agent, aider, openhands, custom），含默认安装/执行命令 |
| 1.5 | 模板 API | `GET /api/agent-templates` |
| 1.6 | Agent 列表页 (Vue) | 表格展示（名称/类型/状态/最近得分/操作），搜索，空状态引导 |
| 1.7 | 创建向导 (Vue) | 6 步向导：选模板 → 基本信息 → 安装命令(分步结构化) → 执行命令(命令模板+占位符) → 外围配置(环境变量/skills/超时) → 确认创建 |
| 1.8 | 编辑页 (Vue) | 复用向导组件，预填充数据，实时校验 |
| 1.9 | 详情页 (Vue) | 配置信息展示，关联实验列表，得分趋势 |
| 1.10 | Agent Materializer | 将 TOML 配置编译为 Python agent 文件，写入 `~/.harnesslab/agents-py/`，注入 PYTHONPATH |
| 1.11 | Agent 管理审计日志 | agent 创建/编辑/删除操作记录，变更 diff |

**Gate Check**：
- [ ] 完整创建 agent 向导流程：选模板 → 填写配置 → 保存 → 列表可见
- [ ] 编辑 agent 后配置正确更新
- [ ] 删除 agent 在无进行中实验时可执行，有则阻止
- [ ] Agent Materializer 生成的 Python 文件语法正确
- [ ] 空状态引导文案正确显示

### Phase 2: Experiment Engine

**目标**：完成 Harbor 集成，实现实验的创建、运行、取消、实时日志推送。

**工作量**：3-4 天

**Tasks**：

| # | Task | Detail |
|---|---|---|
| 2.1 | Experiment 数据模型 | `ExperimentConfig` Pydantic model（agent_id, benchmark, params） |
| 2.2 | Benchmark 发现 | 动态发现 Harbor 已注册的 benchmark 列表，返回名称和元信息 |
| 2.3 | Harbor Job 封装 | `HarborEngine` 类：`create_job()` → `run_job()` → `cancel_job()` |
| 2.4 | 生命周期管理 | heartbeat 检测（5s 间隔），30s 超时 stale 检测 → SIGTERM → SIGKILL |
| 2.5 | WebSocket 日志流 | `WS /api/experiments/:id/logs`，将 Harbor job stdout/stderr 实时推送到前端 |
| 2.6 | 实验状态机 | pending → running → completed/failed/cancelled，状态持久化到索引文件 |
| 2.7 | 结果解析 | Harbor job 完成后解析 `result.json`，存储到 `~/.harnesslab/experiments/` |
| 2.8 | Docker 健康检查 | 启动时 + 每次实验前检查 Docker 可用性，不可用时阻止运行 |
| 2.9 | 错误诊断 | 实验失败时捕获容器日志，附加到实验结果中 |
| 2.10 | 结构化日志 | 实验生命周期事件（创建/启动/完成/失败）JSON-lines 日志 |

**Gate Check**：
- [ ] `HarborEngine.run_job()` 使用 oracle agent 在 terminal-bench 上成功运行
- [ ] WebSocket 实时日志在前端正确展示
- [ ] 实验完成后 result.json 正确解析并存储
- [ ] Docker 不可用时创建实验被阻止
- [ ] heartbeat 超时后实验被正确终止

### Phase 3: Experiment Management Module

**目标**：完成实验的完整管理流程，包括创建向导、一键运行、模板复用、克隆、报告展示。

**工作量**：4-5 天

**Tasks**：

| # | Task | Detail |
|---|---|---|
| 3.1 | Experiment API | `GET/POST/DELETE /api/experiments`，`POST run/cancel`，`GET report` |
| 3.2 | 实验列表页 (Vue) | 表格（名称/agent/benchmark/状态/进度/耗时），筛选器，空状态 |
| 3.3 | 创建实验弹窗 (Vue) | 选 agent（支持多选=对比）→ 选 benchmark（支持多选=批量）→ 参数配置 → 确认 |
| 3.4 | 一键运行 | 从列表页/agent 详情页直接触发运行按钮 |
| 3.5 | 实时进度展示 | 进度条 + 当前任务名 + 可展开/折叠的日志终端 |
| 3.6 | 取消运行 | 取消按钮，调用 Harbor job.cancel()，状态更新为 cancelled |
| 3.7 | 实验模板 | 保存实验配置为模板 API + 模板列表页 + 从模板克隆创建 |
| 3.8 | 实验克隆 | 从历史实验克隆配置，修改后运行 |
| 3.9 | 报告页 (Vue) | 复用 Harbor result.json 渲染，任务列表/得分/失败分类/token/耗时 |
| 3.10 | 报告一键重新运行 | 从报告页克隆配置并运行 |
| 3.11 | 运行日志持久化 | 实验完成后归档日志文件，历史实验可回看日志 |

**Gate Check**：
- [ ] 完整实验流程：创建 → 运行 → 实时日志 → 完成 → 查看报告
- [ ] 多 agent 对比模式：选择 2 个 agent，运行同一 benchmark，各自生成报告
- [ ] 保存为模板 + 从模板创建实验
- [ ] 克隆历史实验并修改后运行
- [ ] 实验取消：运行中取消，状态变为 cancelled

### Phase 4: Leaderboard & Dashboard

**目标**：完成榜单排名和 Dashboard 首页。

**工作量**：3-4 天

**Tasks**：

| # | Task | Detail |
|---|---|---|
| 4.1 | Leaderboard Service | 按 benchmark 聚合所有 agent 得分，计算排名 |
| 4.2 | Leaderboard API | `GET /api/leaderboard?benchmark=X`，返回排名列表 |
| 4.3 | 榜单页 (Vue) | benchmark 选择器 + 排名表格（排名/agent/得分/通过率/token/耗时） |
| 4.4 | 榜单历史趋势 | agent 多次实验得分变化折线图（简单 SVG/CSS 实现） |
| 4.5 | Dashboard 首页 (Vue) | 三大入口卡片（Agent 管理/实验管理/榜单）+ 最近实验摘要 + 快速操作 |
| 4.6 | Dashboard 统计数据 | agent 数量、实验总数、成功率、最近的 benchmark |
| 4.7 | Docker 状态指示器 | 顶部全局 Docker 状态，绿色/红色/加载中 |

**Gate Check**：
- [ ] 榜单按 benchmark 正确排序
- [ ] Dashboard 展示正确的统计数据
- [ ] Docker 状态实时反映
- [ ] 从榜单点击 agent 名称跳转到详情页

### Phase 5: Hardening & Release

**目标**：稳定性打磨、错误处理完善、日志建设、发布准备。

**工作量**：3-4 天

**Tasks**：

| # | Task | Detail |
|---|---|---|
| 5.1 | 全局错误处理 | 前端统一错误拦截、后端异常处理器、用户友好错误提示 |
| 5.2 | 前端空状态完善 | 所有列表/页面的空状态引导文案和操作按钮 |
| 5.3 | 前端加载状态 | 骨架屏/loading 指示器，避免白屏等待 |
| 5.4 | 后端边缘情况 | 磁盘空间检查、并发实验限制（MVP 单实验）、文件锁 |
| 5.5 | 日志建设 | 全局日志 collector、`doctor --logs` 诊断命令、实验运行日志归档 |
| 5.6 | Docker 问题诊断增强 | 容器状态检查、镜像拉取失败提示、资源不足警告 |
| 5.7 | 性能优化 | 大文件分页加载、WebSocket 重连机制、日志流节流 |
| 5.8 | pyproject.toml 完善 | 入口点配置、依赖版本锁定、`pip install` 可安装 |
| 5.9 | README + 快速开始 | 安装步骤、启动命令、截图 |
| 5.10 | 冒烟测试 | 全链路测试脚本：创建 agent → 运行实验 → 查看报告 → 榜单排序 |

**Gate Check**：
- [ ] 全链路冒烟测试通过
- [ ] `pip install harnesslab && python -m harnesslab web` 一键可用
- [ ] 所有空状态/加载状态/错误状态覆盖率 100%
- [ ] Docker 不可用时所有操作行为正确

## 4. Project Structure

```
HarnessLab/
├── pyproject.toml              # Python 项目配置
├── README.md
├── prd/                        # 产品需求文档
│   └── 2026-06-15-harnesslab-webui-prd.md
├── docs/
│   └── plans/
│       └── 2026-06-15-harnesslab-webui-engineering-plan.md
├── harnesslab/                 # Python 后端包
│   ├── __init__.py
│   ├── main.py                 # FastAPI 应用入口
│   ├── cli.py                  # CLI 命令：`python -m harnesslab web`
│   ├── api/                    # API 路由
│   │   ├── __init__.py
│   │   ├── agents.py           # /api/agents
│   │   ├── experiments.py      # /api/experiments
│   │   ├── templates.py        # /api/templates
│   │   ├── leaderboard.py      # /api/leaderboard
│   │   └── system.py           # /api/system
│   ├── services/               # 业务逻辑
│   │   ├── __init__.py
│   │   ├── agent_service.py    # Agent CRUD + Materializer
│   │   ├── experiment_service.py # Experiment CRUD + state machine
│   │   ├── harbor_engine.py    # Harbor Job 封装
│   │   ├── template_service.py # 模板管理
│   │   ├── leaderboard_service.py # 榜单计算
│   │   └── system_service.py   # Docker/Harbor 状态
│   ├── models/                 # Pydantic 数据模型
│   │   ├── __init__.py
│   │   ├── agent.py            # AgentConfig
│   │   ├── experiment.py       # ExperimentConfig
│   │   └── templates.py        # AgentTemplate
│   ├── storage/                # 文件存储层
│   │   ├── __init__.py
│   │   └── file_store.py       # 通用 JSON/TOML 读写
│   ├── templates/              # 预设 Agent 模板数据
│   │   ├── __init__.py
│   │   └── presets.py          # 7 个 AgentKind 定义
│   └── utils/
│       ├── __init__.py
│       └── logging.py          # 结构化日志工具
├── frontend/                   # Vue 3 前端
│   ├── package.json
│   ├── vite.config.js
│   ├── index.html
│   └── src/
│       ├── main.js
│       ├── App.vue
│       ├── router.js
│       ├── api/                # API 调用封装
│       │   └── index.js
│       ├── components/         # 通用组件
│       │   ├── AppLayout.vue   # 全局布局（侧边栏+内容区）
│       │   ├── DockerStatus.vue # Docker 状态指示器
│       │   └── StatusBar.vue   # 底部状态栏
│       ├── views/              # 页面组件
│       │   ├── Dashboard.vue
│       │   ├── agents/
│       │   │   ├── AgentList.vue
│       │   │   ├── AgentCreate.vue    # 创建向导
│       │   │   ├── AgentEdit.vue
│       │   │   └── AgentDetail.vue
│       │   ├── experiments/
│       │   │   ├── ExperimentList.vue
│       │   │   ├── ExperimentCreate.vue
│       │   │   ├── ExperimentDetail.vue # 运行中 + 报告
│       │   │   └── ExperimentReport.vue
│       │   ├── templates/
│       │   │   └── TemplateList.vue
│       │   └── leaderboard/
│       │       └── Leaderboard.vue
│       └── stores/             # 状态管理 (Pinia 或 reactive)
│           ├── agents.js
│           └── experiments.js
└── tests/                      # 测试
    ├── test_agent_service.py
    ├── test_experiment_service.py
    └── test_integration.py
```

## 5. Key Technical Decisions

### Decision 1: 不保留 Rust CLI

**决策**：从零开始，纯 Python 后端 + Vue 前端。

**理由**：
- HarnessLab 处于 demo 阶段，Rust 代码资产少，沉没成本低
- Python 后端与 Harbor 同语言，无需 Bridge 层，消除进程间通信复杂度
- Vue + FastAPI 是成熟的前后端分离方案，团队熟悉度高

### Decision 2: Agent Materializer

**决策**：TOML 配置文件 → Python 文件 → `~/.harnesslab/agents-py/` + PYTHONPATH。

**理由**（同 v1.x 计划）：
- 用户只需编辑 TOML，无需编写 Python 代码
- 独立目录避免 Harbor 升级时文件丢失
- PYTHONPATH 注入是 Python 标准机制，无额外依赖

### Decision 3: WebSocket 实时日志

**决策**：FastAPI 内置 WebSocket 支持，直接将 Harbor job 的 stdout/stderr 转发到前端。

**理由**：
- FastAPI 原生支持 WebSocket，无需额外库
- 简单转发，不做消息队列（MVP 单实验，无并发）
- 日志归档到文件，WebSocket 仅用于运行时展示

### Decision 4: 文件系统持久化

**决策**：所有数据存储为本地文件（TOML/JSON/HTML）。

**理由**：
- 纯本地单用户，无并发读写
- 零部署依赖，无需数据库安装
- 用户可直接查看/编辑配置文件
- 索引文件（JSON）作为元数据缓存

### Decision 5: Python 环境管理

**决策**：使用 `uv` 管理虚拟环境，`pip install harnesslab` 安装。

**理由**：
- uv 速度快，自动管理 Python 版本
- `pyproject.toml` + 入口点，标准 Python 打包
- `~/.harnesslab/` 独立存储用户数据，不污染项目目录

## 6. Risk Assessment

| Risk | Probability | Impact | Mitigation |
|---|---|---|---|
| Harbor API 变更导致不兼容 | Medium | High | 锁定 Harbor 版本范围，CI 中测试兼容性 |
| Docker 环境差异导致运行失败 | Medium | Medium | Docker 健康检查 + 详细诊断信息 |
| 大文件日志导致前端卡顿 | Low | Low | WebSocket 流控 + 虚拟滚动 |
| benchmark 运行时间过长 | Medium | Medium | smoke split 快速验证 + 超时配置 |

## 7. Constraints

### 7.1 技术约束

- Python >= 3.12 + uv
- Docker 必须可用（Colima / Docker Desktop / OrbStack）
- Harbor >= 0.13, < 1.0
- Vue 3 + Vite，Node.js >= 18
- 仅支持 macOS 和 Linux（Windows 不保证）

### 7.2 产品约束

- 纯本地单用户，无认证
- 单实验并发（MVP）
- 文件系统持久化，无数据库
- 不依赖外部 SaaS 服务

### 7.3 启动方式

```bash
# 安装
pip install harnesslab

# 启动 WebUI（自动创建虚拟环境、安装依赖、打开浏览器）
python -m harnesslab web

# 指定端口
python -m harnesslab web --port 3000

# 诊断
python -m harnesslab doctor
```

## 8. Open Questions

| # | Question | Status | Impact |
|---|---|---|---|
| 1 | 榜单历史数据保留策略（全量 vs 最近 N 次） | Open | 存储空间、页面性能 |
| 2 | Harbor 报告是否需要 HarnessLab 包装 UI 还是直接内嵌 | Open | 前端开发量 |
| 3 | 是否需要实验队列（先进先出排队）vs 严格单实验 | Open | 用户体验 |

## 9. Change Log

| Date | Version | Change |
|---|---|---|
| 2026-06-15 | 2.0 | 产品方向重大调整：WebUI 优先，Vue + FastAPI + Harbor 架构，纯 Python 后端，不保留 Rust CLI |
| 2026-06-15 | 1.1 | 对抗性审查修复（CLI-first plan, deprecated） |
| 2026-06-15 | 1.0 | 初始计划（CLI-first plan, deprecated） |
