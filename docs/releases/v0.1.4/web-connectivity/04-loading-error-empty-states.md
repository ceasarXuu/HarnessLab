# BUG-WEB-04: 前端缺少统一的 loading / error / empty 状态

- Created: 2026-06-23
- Updated: 2026-06-23
- Version: 1.0
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: frontend views & components
- Related Links: [README](README.md), [BUG-WEB-02](02-views-not-consuming-api.md)
- Risk Level: Medium
- Plan Type: Standard
- Phase: 2（数据接入，与 02 同 PR）

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

1. 引入轻量 async-state 原语（不引入额外依赖），例如：
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

## Acceptance Criteria

- [ ] 断开后端运行 `npm --prefix frontend run dev` 时，每个 View 显示明确错误态而非空白或异常堆栈。
- [ ] 后端启动但数据为空时，每个 View 显示空态文案而非渲染异常。
- [ ] `StatePanel` 与 `asyncState` 拥有单元测试与 Storybook story。
- [ ] 错误态文案不泄露后端栈/SQL，仅显示用户可理解的摘要 + 可选 "复制详情"。

## 风险与回滚

- 增加约 1 个通用组件 + 1 个工具模块；改动面可控。
- 与 [BUG-WEB-02](02-views-not-consuming-api.md) 同 PR 提交，整体回滚为 `git revert` 该 PR。
