# PRD: HarnessLab WebUI

- Status: Ready for implementation
- Created: 2026-06-15
- Updated: 2026-06-15
- Owner / requester: HarnessLab team
- Source request: 以 WebUI 为第一优先级，重构 HarnessLab 为基于 Web 的 agent 评估平台，核心提供 agent 管理和实验管理两大模块

## Requester Review Summary

- Key decisions:
  - WebUI 优先，CLI 降级为辅助工具（`harnesslab web` 启动本地服务）
  - 纯本地单用户项目，无认证机制
  - 大胆重写架构，以 WebUI 体验为中心，不保守保留现有 Rust 代码
  - 底层复用 Harbor 框架作为执行引擎
- Important exceptions:
  - 报告格式直接复用 Harbor 现有格式，不做额外包装
  - 数据持久化使用本地文件系统（TOML/JSON/HTML），不引入数据库
- Must-confirm before implementation:
  - 前端框架选择（React vs Vue）
  - 后端框架是否使用 FastAPI

## 1. Background And Product Intent

HarnessLab 当前是一个 Rust CLI 工具，用于评估 CLI coding agent 在 benchmark 上的表现。产品方向调整为：**以 WebUI 为第一优先级**，让用户通过简单易用的界面快速管理 agent 配置、启动实验、查看结果和榜单排名。

底层执行引擎复用 Harbor 框架（已通过 Phase 0 评估验证），HarnessLab 专注于提供优秀的交互体验和产品语义。

## 2. Goals And Success Criteria

### Goals
1. **Agent 管理**：用户可通过 WebUI 完成 agent 的增删改查、安装/执行命令配置、环境变量、skills 等外围配置
2. **实验管理**：用户可通过 WebUI 创建实验、配置参数、一键运行、复用配置、查看实验报告
3. **榜单排名**：按 benchmark 维度展示各 agent 的得分排名
4. **零门槛上手**：新用户 5 分钟内完成第一个 agent 注册 + 实验运行

### Success Criteria
| Metric | Target |
|---|---|
| 首次使用到完成第一个实验 | < 5 分钟 |
| Agent 创建（模板+向导） | < 2 分钟完成配置 |
| 实验启动（从已配置 agent） | < 3 次点击 |
| 实验模板复用率 | > 50% 的实验从模板/克隆创建 |
| 页面加载时间 | < 2 秒 |

## 3. Users And Usage Context

### 主要用户
- **AI 应用开发者**：需要评估自己开发的 agent 在各种 benchmark 上的表现，注册新 agent、配置环境、运行实验、对比结果
- **非技术决策者**：查看实验报告和榜单排名，了解不同 agent 的能力对比，不需要操作 agent 配置

### 使用场景
- 开发者本地运行，单机使用
- 通过 `harnesslab web` 命令启动本地 HTTP 服务，浏览器访问 `localhost:xxxx`
- 所有数据存储在本地文件系统，无需网络

## 4. Scope

### In Scope

**Agent 管理模块**：
- Agent 列表展示（名称、类型、状态、最近实验得分）
- Agent 创建（模板 + 向导模式）
- Agent 编辑（分步结构化表单：安装命令、执行命令、环境变量、skills、超时等）
- Agent 删除
- Agent 详情页（配置信息、关联实验、得分趋势）

**实验管理模块**：
- 实验列表展示（名称、状态、agent、benchmark、进度、耗时）
- 实验创建（选择 agent → 选择 benchmark → 配置参数）
- 实验配置保存为模板（可复用）
- 从模板/历史实验克隆创建新实验
- 一键运行实验
- 实验运行中实时反馈（进度概览 + 可展开日志）
- 实验报告查看（复用 Harbor 格式）
- 实验删除

**榜单模块**：
- 按 benchmark 筛选
- 按得分排名展示各 agent
- 支持查看历史排名变化

**运行环境**：
- Docker 状态可见（可用/不可用、容器状态）
- 问题诊断（实验失败时展示容器日志）

### Out Of Scope
- 用户认证/登录系统
- 多用户/团队协作
- 远程部署（MVP 仅本地 localhost）
- 数据库持久化（仅文件系统）
- Harbor Hub 上传/分享功能
- 移动端适配
- 国际化（仅中文）
- 自定义 benchmark 注册（使用 Harbor 现有 75+ benchmark）

## 5. Core User Journey

### Journey 1: 首次使用，完成第一个实验

1. 用户安装 HarnessLab，执行 `harnesslab web`
2. 浏览器自动打开 `localhost:xxxx`，进入 Dashboard
3. Dashboard 显示空状态引导："还没有 Agent，创建一个开始实验"
4. 用户点击"创建 Agent"，进入模板选择页
5. 选择模板（如 "claude-code"），进入向导
6. 向导步骤：
   - Step 1: 基本信息（名称、描述）
   - Step 2: 安装命令（预设模板已填充，可修改）
   - Step 3: 执行命令（预设模板已填充，可修改）
   - Step 4: 环境变量和 skills 配置
   - Step 5: 确认并创建
7. Agent 创建完成，跳转到 Agent 详情页
8. 用户点击"运行实验"
9. 弹窗选择 benchmark（如 terminal-bench），设置参数（split、并发数等）
10. 点击"运行"，实验开始
11. 实时展示进度（已完成 N/总数）和可展开的实时日志
12. 实验完成，自动跳转到报告页

### Journey 2: 对比多个 agent 的表现

1. 用户进入 Dashboard，点击"实验管理"
2. 点击"创建实验"
3. 选择多个 agent（如 agent-A、agent-B），选择同一个 benchmark
4. 可选：保存为实验模板
5. 运行实验
6. 实验完成后，查看对比报告
7. 进入榜单，按该 benchmark 查看排名

### Journey 3: 非技术用户查看报告

1. 用户获得报告链接或文件路径
2. 浏览器打开报告（HTML 格式，复用 Harbor 格式）
3. 查看 agent 在各任务上的得分、耗时、token 消耗
4. 进入榜单查看该 benchmark 下的全局排名

## 6. Interaction And Information Design

### 6.1 全局布局

```
┌──────────────────────────────────────────────────────────┐
│  HarnessLab                              Docker: ● 运行中 │
├──────────┬───────────────────────────────────────────────┤
│          │                                               │
│  Agent   │                                               │
│  管理    │            主内容区                             │
│          │                                               │
│  实验    │                                               │
│  管理    │                                               │
│          │                                               │
│  榜单    │                                               │
│          │                                               │
│          │                                               │
├──────────┴───────────────────────────────────────────────┤
│  v0.1.0  |  harbor v0.13.2  |  localhost:3000            │
└──────────────────────────────────────────────────────────┘
```

- 左侧：固定导航栏（Agent 管理 / 实验管理 / 榜单）
- 顶部：Docker 状态指示器
- 底部：版本信息状态栏

### 6.2 Agent 管理页面

**列表视图**：
- 表格展示：名称、类型、状态（就绪/配置中）、最近实验得分、操作（编辑/删除/运行实验）
- 顶部：搜索框 + "创建 Agent"按钮
- 空状态：引导文案 + 创建按钮

**创建向导**：
- Step 1: 选择模板（展示预设模板卡片：claude-code, codex, aider, openhands, 自定义）
- Step 2: 基本信息（名称输入框 + 描述 textarea）
- Step 3: 安装命令（结构化：包管理器选择 + 包名 + 自定义脚本，支持多行）
- Step 4: 执行命令（命令模板 + 参数占位符 `{instruction}` 提示）
- Step 5: 外围配置（环境变量键值对列表、skills 目录路径、超时时间、并发数）
- Step 6: 确认（展示所有配置的摘要，确认创建）

**编辑页**：
- 与创建向导相同的分步表单，预填充已有数据
- 每步独立保存或最后统一保存

### 6.3 实验管理页面

**列表视图**：
- 表格展示：实验名称、agent、benchmark、状态（等待中/运行中/已完成/失败）、进度、耗时、操作
- 顶部：筛选器（按状态、agent、benchmark）+ "创建实验"按钮
- 空状态：引导文案

**创建实验**：
- 第一步：选择 agent（单选或多选，多选即为对比模式）
- 第二步：选择 benchmark（单选或多选，多选即为批量模式）
- 第三步：配置参数（split、并发数、任务数、超时等）
- 第四步：确认并选择"立即运行"或"保存为模板"

**实验运行中**：
- 进度条：已完成 N / 总数
- 当前任务名
- 实时日志：可展开/折叠的终端风格日志流
- 取消按钮

**实验模板**：
- 从已完成的实验"保存为模板"
- 模板列表：名称、agent、benchmark、创建时间
- 从模板创建实验：预填充配置，可修改后运行

### 6.4 榜单页面

- 顶部：benchmark 选择器（下拉列表，默认 terminal-bench）
- 主区域：排名表格
  - 列：排名、Agent 名称、得分、通过率、token 消耗、运行时间
- 支持点击 agent 名称跳转到详情页
- 支持点击得分查看该次实验报告
- 历史趋势：选择 agent 查看其在多次实验中的得分变化（简单折线图）

### 6.5 实验报告页

- 直接复用 Harbor 的 result.json 渲染为 HTML 报告
- 报告内容：任务列表、每任务得分、失败分类、token 用量、耗时
- 支持从报告页一键克隆实验配置

## 7. Product Rules And State Logic

### Agent 状态

```
[创建] → 就绪 (ready)
就绪 → 编辑中 → 就绪
就绪 → 删除
```

### 实验状态

```
[创建] → 等待中 (pending)
等待中 → 运行中 (running)
运行中 → 已完成 (completed) | 失败 (failed) | 已取消 (cancelled)
已完成 → [查看报告]
已完成 → 保存为模板 (template)
```

### 实验模板状态

```
[从实验保存] → 可用 (active)
可用 → 克隆为实验
可用 → 删除
```

### 规则

- 删除 agent 前检查是否有进行中的实验，如有则阻止并提示
- 同一时间最多运行 1 个实验（MVP 单实验限制，避免资源竞争）
- 实验运行中不可编辑 agent 配置
- Docker 不可用时，创建实验按钮置灰，提示"请先启动 Docker"
- 实验模板不可修改，只能克隆为新实验后修改

## 8. Edge Cases, Errors, And Recovery

### 空状态
- Dashboard 无 agent → 引导创建第一个 agent
- Agent 列表为空 → 引导文案 + 创建按钮
- 实验列表为空 → 引导文案 + 从 agent 页发起实验
- 榜单无数据 → 引导先运行实验

### 错误状态
- Docker 未启动 → 顶部状态栏显示红色指示器 + 操作按钮置灰 + 提示信息
- 实验运行失败 → 列表显示失败状态 + 可展开错误日志 + 重试按钮
- Agent 配置无效 → 向导中实时校验，无效字段红色标记 + 错误提示
- Harbor 不可用 → 全局错误 banner，提示检查 harbor 安装
- 磁盘空间不足 → 实验创建时检查，提示清理空间

### 恢复
- 页面刷新后保持当前导航状态（URL 路由）
- 实验运行中刷新页面：自动恢复进度展示
- 浏览器崩溃后重启实验：需手动重新运行（MVP 不做自动恢复）

## 9. Content And Terminology

| 术语 | 含义 | 用户可见 |
|---|---|---|
| Agent | 被评估的 AI coding agent | 列表、创建、编辑 |
| Benchmark | 评估 agent 的测试集 | 选择、榜单 |
| 实验 (Experiment) | agent + benchmark 的单次运行 | 创建、运行、查看 |
| 实验模板 (Template) | 可复用的实验配置 | 保存、克隆 |
| 榜单 (Leaderboard) | 按 benchmark 的 agent 排名 | 查看 |
| 报告 (Report) | 实验完成后的结果展示 | 查看 |
| Skills | agent 的技能目录配置 | agent 配置 |
| 安装命令 | agent 在容器内的安装步骤 | agent 配置 |
| 执行命令 | agent 在容器内的执行命令 | agent 配置 |

## 10. Acceptance Criteria

### Agent 管理
- [ ] Given 空 agent 列表，when 点击"创建 Agent"，then 进入模板选择页
- [ ] Given 选择模板，when 完成向导所有步骤，then agent 创建成功并在列表中可见
- [ ] Given 已有 agent，when 点击编辑，then 预填充当前配置，可修改
- [ ] Given 已有 agent，when 点击删除，then 确认弹窗后删除
- [ ] Given agent 有进行中的实验，when 点击删除，then 阻止并提示
- [ ] Given agent 配置无效，when 填写表单，then 实时校验并标记错误字段

### 实验管理
- [ ] Given 已有 agent，when 点击"运行实验"，then 弹出 benchmark 选择 + 参数配置
- [ ] Given 配置完成，when 点击"立即运行"，then 实验状态变为"运行中"，展示进度
- [ ] Given 实验运行中，when 展开日志，then 展示实时日志流
- [ ] Given 实验完成，when 点击查看报告，then 展示 Harbor 格式报告
- [ ] Given 已完成实验，when 点击"保存为模板"，then 模板出现在模板列表中
- [ ] Given 已有模板，when 点击"从模板创建"，then 预填充配置，可修改后运行
- [ ] Given 实验运行中，when 点击取消，then 实验状态变为"已取消"

### 榜单
- [ ] Given 选择 benchmark，when 进入榜单，then 按得分降序展示 agent 排名
- [ ] Given 榜单中有 agent，when 点击 agent 名称，then 跳转到 agent 详情页

### 运行环境
- [ ] Given Docker 不可用，when 进入页面，then 顶部显示红色状态指示器
- [ ] Given Docker 恢复，when 状态变化，then 指示器自动变为绿色
- [ ] Given 实验失败，when 查看详情，then 可展开容器日志帮助诊断

## 11. Review Checklist And Sign-off Questions

- [x] 前端框架选择：Vue（简单优先）
- [x] 是否保留 Rust CLI：不保留，纯 Python 后端启动
- [x] Agent 模板库数量：7 个模板（claude-code, codex, opencode, pi-coding-agent, aider, openhands, custom）
- [ ] Harbor 报告格式是否需要任何 UI 包装，还是直接 iframe/内嵌展示？
- [ ] 榜单需要支持多少历史数据？（全量 vs 最近 N 次）

## 12. Clarification Decision Log

| Topic | Decision | Rationale | Source Round |
|---|---|---|---|
| 目标用户 | 技术用户操作 + 非技术用户查看报告 | 兼顾开发者和决策者 | R1 |
| 实验粒度 | 单agent+单benchmark / 单agent+多benchmark / 多agent对比 | 三种模式覆盖全部使用场景 | R1 |
| 技术栈 | 纯 Web 前后端分离，CLI 降级 | WebUI 优先策略 | R1 |
| 代码策略 | 大胆重写，WebUI 为中心 | Demo 阶段，无历史包袱 | R1 |
| Agent 创建 | 模板 + 向导模式 | 降低上手门槛，预设降低配置错误 | R2 |
| 实验流程 | 配置 → 保存 → 运行 → 报告 | 配置与运行分离，支持复用 | R2 |
| Dashboard | Agent 管理 + 实验管理 + 榜单 | 三大核心模块 | R2 |
| 报告 | 复用 Harbor 格式 | 不做重复工作 | R2 |
| 榜单 | 按 benchmark 得分排名 | 直观对比 agent 能力 | R3 |
| Agent 命令 | 分步结构化输入 | 降低配置错误，引导正确配置 | R3 |
| 实验复用 | 模板 + 克隆 | 覆盖预设复用和临时复用 | R3 |
| 实时反馈 | 进度概览 + 可展开日志 | 既看进度也能 debug | R3 |
| 认证 | 无认证 | 纯本地单用户项目 | R4 |
| 数据持久化 | 本地文件系统 | 无需数据库，简化部署 | R4 |
| Docker 可见性 | 状态可见 + 问题诊断 | 用户需要知道环境状态 | R4 |
| 部署形态 | `harnesslab web` 本地服务 | 单机使用，localhost 访问 | R4 |

## 13. Open Questions And Risks

### Open Questions
1. 榜单历史数据保留策略（全量 vs 最近 N 次）

### 已决策 (Resolved)
- **前端框架**：Vue，简单优先，不做过度工程化
- **启动器**：不保留 Rust CLI，纯 Python 后端启动（FastAPI + uvicorn）
- **Agent 模板库**：预设 7 个模板（claude-code, codex, opencode, pi-coding-agent, aider, openhands, custom）

### Risks
| Risk | Impact | Mitigation |
|---|---|---|
| Harbor 报告格式不够美观 | 中 | 可在报告外层加一层 HarnessLab 包装 UI |
| 实验运行中浏览器关闭 | 中 | 后端独立运行，前端刷新后恢复状态 |
| 大规模 benchmark 运行耗时长 | 低 | 提供 smoke split 快速验证 |

## 14. Implementation Notes

- 技术栈：Vue 3 + FastAPI + Harbor (Python)
- 启动方式：`python -m harnesslab web`（FastAPI + uvicorn, 自动打开浏览器）
- 前端：Vue 3 + Vite，简单组件结构，不过度工程化