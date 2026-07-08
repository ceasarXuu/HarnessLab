# v1.0.5 前端重建架构决策

- Status: Demo implemented; integration boundary needs implementation
- Created: 2026-06-28
- Updated: 2026-07-09
- Decision owner: project maintainer
- Official reference:
  - Harbor Viewer: `https://github.com/harbor-framework/harbor/tree/main/apps/viewer`
  - Harbor Hub: `https://hub.harborframework.com/`

> 文档定位：本文记录前端重建背景和架构决策支撑信息。v1.0.5 当前技术权威入口见 [技术设计](technical-design.md)，实施进度见 [工程计划与进度](engineering-plan.md)。

## 1. 决策

v1.0.5 前端不再延续旧 Vue demo。新的 `frontend/` 以 React/Vite demo 重新建立，
作为后续 Harbor 官方 Viewer 对齐开发的入口。

v1.0.5 WebUI 需要重建，并与 Harbor 官方 Viewer 保持架构方向一致。当前已经落地的前端基线是：

- React 19
- Vite
- Tailwind CSS v4
- lucide-react
- Storybook
- SPA 模式，不启用 SSR

以下能力是正式化和联调阶段的目标基线，尚不能在文档中表述为已落地依赖：

- React Router 或等价路由层
- TanStack Query 或等价 server-state 层
- TanStack Table 或等价数据表格层
- shadcn/Radix 风格的可访问基础组件层

当前 `frontend/package.json` 只代表已经落地并可验证的依赖。任何新增路由、server-state、
table 或基础组件库，都需要单独完成选型、安装、Storybook 示例和回归验证后，才能从
“目标基线”移动到“已落地基线”。

官方 Harbor Hub 的公开站点用于视觉参考；真正的本地 WebUI 产品架构以 Harbor
仓库中的 `apps/viewer` 为准。

## 2. 背景

OrnnLab 当前仍是基于 Harbor 的实验控制台。v1.0.5 的阶段目标是先补齐 Harbor
WebUI 能力，用 UI 交互替代日常 Harbor CLI 操作。

前一个 `frontend/` 是 Vue 3 + Vite demo，用于早期 operations console 验证。它已经不能作为
Harbor WebUI 的正式基础：

- 与 Harbor 官方 Viewer 的 React/shadcn/TanStack 技术栈不一致；
- 不能直接复用官方 Viewer 的组件、表格、路由和交互模型；
- 已有页面仍围绕 OrnnLab demo dashboard，而不是 Harbor Dataset/Job 嵌套模型；
- 继续保留会让后续开发误以为要在 Vue demo 上做增量演进。

## 3. 官方架构观察

Harbor 官方前端分两类：

| Surface | 官方位置 | 架构 | 对 OrnnLab v1.0.5 的意义 |
|---|---|---|---|
| Harbor Viewer | `apps/viewer` | React Router + Vite SPA + Tailwind + shadcn/ui + TanStack Query/Table | 本地 WebUI 主架构基线 |
| Harbor Hub / Docs | `hub.harborframework.com` 和 `docs/` | Next + React + Tailwind + fumadocs | 视觉、导航、表格密度和文案风格参考 |

Harbor Viewer 的生产模式是把前端 build 输出复制到 Python package 的
`src/harbor/viewer/static/`，再由 FastAPI server 服务 SPA。这与 OrnnLab 的本地
FastAPI 产品形态兼容。

## 4. 非目标

- 不在 Vue demo 上继续加 Harbor 功能。
- 不做纯命令包装 UI。
- 不用 Next 重建 v1.0.5 本地应用，除非后续明确需要 SSR、文档站能力或云端 Hub 页面。
- 不 fork Harbor core。
- 不做误导性假入口；未接入真实 Harbor 能力的入口必须隐藏、禁用或明确标注 pending。

## 5. 重建边界

重建后的前端应优先对齐 Harbor 官方 Viewer 的信息架构：

- Datasets catalog
- Dataset detail with task definitions
- Jobs
- New Job
- Job detail with trial list/detail
- Agents
- Leaderboard
- Compare
- Auth status

OrnnLab 仍可保留自己的产品组织层：

- Templates
- System doctor
- Reports

但这些页面必须服务 Harbor WebUI 主链路，不应重新变成独立 demo dashboard。

## 6. API 与后端约束

前端重建不改变 Harbor 执行权威：

- Harbor subprocess / API 仍是 job 执行边界；
- FastAPI 后端仍负责本地数据、system doctor、job lifecycle、artifact path 和 event stream；
- UI 发起的是结构化 API 调用，不允许让用户复制自由文本命令作为主路径；
- 每个核心操作必须能映射到被替代的 Harbor CLI 操作，并保留 JobConfig 或等价 CLI 作为审计证据。

联调前需要把后端暴露给前端的资源面统一到 WebUI 契约。推荐公开根路径为：

- `/api/webui/v1`

现有 `/api/experiments`、`/api/runs`、`/api/benchmarks`、`/api/leaderboard`、`/api/system`、`/api/agents` 可以继续作为后端内部或兼容路由，但 React 页面不得直接适配这些路由字段。若后端 facade 暂时未实现，前端必须在 `frontend/src/api/` 中建立 contract adapter。

优先兼容或借鉴 Harbor Viewer API 形态：

- `/api/config`
- `/api/jobs`
- `/api/jobs/{jobName}`
- `/api/jobs/{jobName}/tasks`
- `/api/jobs/{jobName}/trials`
- `/api/task-definitions`
- `/api/run-options`
- `/api/run-status`
- `/api/auth/status`

OrnnLab 自有 API 可以保留，但需要逐步收敛到 Harbor Dataset/Task 与 Job/Trial 的嵌套用户模型。

联调期统一采用 `ApiResponse<T>`、`Operation`、SSE/轮询状态观察和 `Idempotency-Key`。下载、清理缓存、重启服务、创建 Job、取消 Job 等耗时或破坏性动作不得在页面内部伪造完成状态。

## 7. 迁移策略

1. 删除旧 Vue demo，清理启动脚本、CI 和验证脚本中的 Vue/Vite 假设。
2. 以 Harbor `apps/viewer` 为参考重建 `frontend/`。
3. 先恢复最小 P0 闭环：Jobs 默认页、Datasets catalog/detail、Agents、Leaderboard、New Job、Job detail、System doctor。
4. 新增 `frontend/src/domain/` 和 `frontend/src/api/`，把 mock fixture、DTO、ViewModel 和页面状态拆开。
5. 再补 Dataset 内 task detail、Job 内 trial detail、artifact viewer、auth/upload/share。
6. 每个页面从 Storybook、unit test、Playwright smoke 和 FastAPI contract 开始建立测试。

## 8. 验收原则

- 仓库中不得残留可启动的旧 Vue demo。
- 新前端 package 不得引入 Vue 依赖。
- 新前端页面结构、表格、按钮、输入、tabs、toast、空状态和 loading 状态应优先复用当前项目已有组件；确需引入 shadcn/Radix/TanStack 时，应先补选型说明、Storybook 示例和测试门禁。
- 新前端必须支持 Storybook 工作流。
- Storybook 必须覆盖 loaded、loading、empty、error、operation-running、dark/light、zh/en 等关键状态，而不只是展示 happy path。
- e2e、unit test 和 Storybook play 的断言口径必须一致；任一测试仍断言已移除 UI 时，不能进入联调。
- 新前端必须有一条真实 Harbor job 的端到端 UI smoke，至少覆盖配置、启动、状态读取和详情页查看。
- 默认开发命令在新前端落地前不得假装存在完整 WebUI。
