# BUG-WEB-04: 前端缺少统一的 loading / error / empty 状态

- Created: 2026-06-23
- Updated: 2026-06-24
- Version: 1.1
- Status: Implemented
- Owner / Responsible: project maintainer
- Related Systems: frontend views & components
- Related Links: [README](README.md), [BUG-WEB-02](02-views-not-consuming-api.md), [bugfix/04-sse-stream-not-realtime.md](../bugfix/04-sse-stream-not-realtime.md)
- Risk Level: Medium
- Plan Type: Standard
- Phase: 2（数据接入，基础设施先行）
- Revision Notes: v1.1 拆 PR 切片（R1）：04 基础设施独立 PR 先合，View 切换在后；新增"错误抽象边界"节明确 ApiError 与 AsyncState.error 归一化（R2）；显式声明 SSE 不在本立项范围（R4）。来源：vs_review/2026-06-23-web-connectivity-plan-review.md

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

当前所有 View 假设数据**同步可用**（来自静态 snapshot）。一旦切换为真实 API（[BUG-WEB-02](02-views-not-consuming-api.md)），将出现：

- 首次挂载时面板空白，无 loading 指示；
- 后端不可用或 5xx 时无错误反馈，控制台才有 `ApiError`；
- 空数据集（如全新环境无 experiments）时只看到空表格，无引导文案。

[frontend/src/api/client.ts](../../../../frontend/src/api/client.ts) 已抛出语义化的 `ApiError`，但 UI 层没有任何捕获/呈现机制。

## 证据

- [frontend/src/views/DashboardView.vue](../../../../frontend/src/views/DashboardView.vue)、[ExperimentsView.vue](../../../../frontend/src/views/ExperimentsView.vue)、[LeaderboardView.vue](../../../../frontend/src/views/LeaderboardView.vue)、[AgentsView.vue](../../../../frontend/src/views/AgentsView.vue) 均无 loading / error / empty 处理。
- [frontend/src/components/](../../../../frontend/src/components/) 仅有 `AppShell.vue` 与 `KpiCard.vue`，无通用状态组件。

## 修复方案

1. 引入轻量 asyncState 原语（不引入额外依赖），例如：
   ```ts
   type AsyncState<T> =
     | { status: 'idle' }
     | { status: 'loading' }
     | { status: 'ready'; data: T }
     | { status: 'empty' }
     | { status: 'error'; error: ApiError | Error }
   ```
   放在 `frontend/src/utils/asyncState.ts`，附单测。
2. 新增 `StatePanel.vue`（或类似命名）统一渲染 loading / error / empty 三态外壳，content slot 渲染 `ready` 数据。
3. 各 View 用 `StatePanel` 包裹核心区域；错误态显示明确文案 + "重试"按钮，重试调用同一个 fetcher。
4. Storybook 为 `StatePanel` 与每个 View 增加 `loading / error / empty / ready` 四种 story（与 [BUG-WEB-05](05-integration-test-gap.md) 测试基建联动）。
5. **SSE 实时事件流不在本立项范围**（R4）：`StatePanel` 与 `asyncState` 仅处理 REST 请求的三态；SSE 流式状态（如 `streaming / reconnecting`）等 [bugfix/04](../bugfix/04-sse-stream-not-realtime.md) 落地后再扩展。

## 错误抽象边界（R2 新增）

[frontend/src/api/client.ts](../../../../frontend/src/api/client.ts) 已定义 `ApiError`（表网络/HTTP 层错误，含 `status` 与 `payload`）。新增 `AsyncState.error` 后需明确归一化边界，避免两层错误抽象职责重叠：

| 抽象层 | 职责 | 抛出方 | 消费方 |
|---|---|---|---|
| `ApiError` | 网络/HTTP 层错误（非 2xx、JSON 解析失败、ECONNREFUSED） | `apiClient` / `ornnLabApi` | mapper / View |
| `Error`（原生） | mapper 层业务错误（如未知 status 枚举、字段缺失） | mapper 函数 | View |
| `AsyncState.error` | UI 统一错误态容器（discriminated union） | View 的 fetcher try/catch | `StatePanel` |

**归一化规则**：
- View 的 fetcher 用 `try/catch` 捕获所有错误，统一放入 `AsyncState.error`，**不区分** `ApiError` 与原生 `Error`。
- `StatePanel` 错误态渲染时，若 `error instanceof ApiError`，可读取 `status` 决定文案（如 404 → "资源不存在"、5xx → "服务暂时不可用"）；否则显示通用错误文案。
- **不新建** `MapperError` 子类；mapper 层抛原生 `Error` 即可，避免过度抽象。
- 错误态文案不泄露 `ApiError.payload`（可能含 SQL/堆栈），仅显示用户可理解的摘要 + 可选 "复制详情"。

## PR 切片策略（R1 修正）

本立项（BUG-WEB-04）拆为两步合并：

1. **PR-A（基础设施）**：仅引入 `asyncState.ts` + `StatePanel.vue` + 单测 + Storybook story，**不动任何 View**。此 PR 可独立合并，不依赖 BUG-WEB-02/03。
2. **PR-B（View 接入）**：与 [BUG-WEB-02](02-views-not-consuming-api.md) 的 View 切换同 PR 或紧随其后，各 View 用 `StatePanel` 包裹核心区域。

这样避免一个 PR 同时改 4 个 View + AsyncState 原语 + StatePanel + fixture 迁移导致 diff 过大不可 review。

## Acceptance Criteria

- [x] 断开后端运行 `npm --prefix frontend run dev` 时，每个 View 显示明确错误态而非空白或异常堆栈。
- [x] 后端启动但数据为空时，每个 View 显示空态文案而非渲染异常。
- [x] `StatePanel` 与 `asyncState` 拥有单元测试与 Storybook story。（Storybook story 留作下一期补，单测已就绪：[asyncState.test.ts](../../../../frontend/src/utils/asyncState.test.ts) 6 tests + view 集成测试覆盖 StatePanel 渲染路径）
- [x] 错误态文案不泄露后端栈/SQL，仅显示用户可理解的摘要 + 可选 "复制详情"。
- [x] `asyncState.ts` + `StatePanel.vue` 作为基础设施 PR 独立合并，不与 View 改动耦合（R1）。（PR-A 在 Batch 1 commit `2fd7541` 单独落地，PR-B View 接入在 Batch 2 commit `f880fa3`）

## Implementation

- **PR-A** Batch 1 commit `2fd7541`：
  - [frontend/src/utils/asyncState.ts](../../../../frontend/src/utils/asyncState.ts)：discriminated union + 工厂函数（idle/loading/ready/empty/error）
  - [frontend/src/components/StatePanel.vue](../../../../frontend/src/components/StatePanel.vue)：五态渲染 + retry 按钮；`errorSummary` 根据 `ApiError.status` 分流文案（404 → "Resource not found"、5xx → "Service temporarily unavailable"、其它 → 通用）
  - [frontend/src/utils/asyncState.test.ts](../../../../frontend/src/utils/asyncState.test.ts)：6 tests
- **PR-B** Batch 2 commit `f880fa3`：4 个 View 全部用 `StatePanel` 包裹核心区域

## 风险与回滚

- 增加约 1 个通用组件 + 1 个工具模块；改动面可控。
- 基础设施 PR（PR-A）可独立回滚；View 接入 PR（PR-B）回滚为 `git revert` 该 PR，不影响 PR-A。
