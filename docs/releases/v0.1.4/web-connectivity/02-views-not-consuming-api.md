# BUG-WEB-02: Views 未消费 `ornnLabApi`，仅显示静态快照

- Created: 2026-06-23
- Updated: 2026-06-24
- Version: 1.1
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: frontend views, frontend api client
- Related Links: [README](README.md), [BUG-WEB-03](03-contract-gap-vs-backend.md), [BUG-WEB-04](04-loading-error-empty-states.md), [bugfix/04-sse-stream-not-realtime.md](../bugfix/04-sse-stream-not-realtime.md)
- Risk Level: High
- Plan Type: Standard
- Phase: 2（数据接入，主线）
- Revision Notes: v1.1 拆 PR 切片（R1）：04 基础设施独立 PR 先合，View 切换可在同 PR 或后续 PR；引用 BUG-WEB-03 viewmodel 字段决策；显式声明 SSE 不在本立项范围（R4）。来源：vs_review/2026-06-23-web-connectivity-plan-review.md

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

[frontend/src/api/client.ts](../../../../frontend/src/api/client.ts) 已经导出 `apiClient` 和 `ornnLabApi`，但**全仓库范围内没有任何文件 import 它**：

```
$ rg "from '@/api" frontend/src   # 0 matches
```

四个 View 全部从 [App.vue](../../../../frontend/src/App.vue) 通过 prop 接收同一份 [consoleSnapshot.ts](../../../../frontend/src/data/consoleSnapshot.ts) 静态对象。这意味着：

- 任何后端的实际状态变化都不会反映到界面；
- 演示截图与运行时观感一致，掩盖了后端联调缺口；
- 类型契约 [types/console.ts](../../../../frontend/src/types/console.ts) 与后端 schema 实际上**从未在运行时被对齐**。

## 证据

- [frontend/src/App.vue](../../../../frontend/src/App.vue) 中：`const snapshot = buildConsoleSnapshot()` 后直接传给 `AppShell`。
- [frontend/src/views/DashboardView.vue](../../../../frontend/src/views/DashboardView.vue) 等四个 View 仅 `defineProps<{ snapshot: ConsoleSnapshot }>()`，无 `fetch` / `onMounted` / API import。
- [frontend/src/api/client.ts](../../../../frontend/src/api/client.ts) 中 `ornnLabApi` 提供 `experiments / leaderboard / templates / run / runReport ...` 全套方法但无消费者。

## 修复方案

1. 引入 view-model 与 mapper 层（具体字段映射与数据源决策在 [BUG-WEB-03 § UI viewmodel 字段数据源决策](03-contract-gap-vs-backend.md#ui-viewmodel-字段数据源决策f3-新增)）。**注意**：按 BUG-WEB-03 R3 判据，仅枚举映射 / 重命名 / 聚合场景引入 mapper；1:1 直传字段不引入 mapper。
2. 改造每个 View，使用 `onMounted` + `ref` + `try/catch` 的最小数据接入：
   - `DashboardView` ← `ornnLabApi.experiments()` + `ornnLabApi.leaderboard()`（用于 Top N 与 KPI 派生）
   - `AgentsView` ← 新增（在 BUG-WEB-03 中补齐）`ornnLabApi.agents.list()`
   - `ExperimentsView` ← `ornnLabApi.experiments()`
   - `LeaderboardView` ← `ornnLabApi.leaderboard()`
3. 把 [consoleSnapshot.ts](../../../../frontend/src/data/consoleSnapshot.ts) 移到 `frontend/src/__fixtures__/` 或 `tests/fixtures/`，作为 Storybook / 单测的 fixture，不再作为运行时数据源。
4. **PR 切片策略（R1 修正）**：[BUG-WEB-04](04-loading-error-empty-states.md) 的基础设施（`StatePanel` + `asyncState`）应**独立于 View 改动先行合并**；View 切换可在同一 PR（含 04 基础设施）或后续 PR 中进行。避免一个 PR 同时改 4 个 View + AsyncState 原语 + StatePanel + fixture 迁移导致 diff 过大不可 review。
5. **SSE 实时事件流不在本立项范围**（R4）：View 切换时若触手可及看到 `/events/stream` 端点，不要临时接入 EventSource；SSE 接入等 [bugfix/04](../bugfix/04-sse-stream-not-realtime.md) 落地后再追加。

## Acceptance Criteria

- [ ] 四个 View 在挂载后均会调用 `ornnLabApi` 并把响应映射到 UI；后端响应变更时刷新页面可见到变化。
- [ ] `frontend/src/data/consoleSnapshot.ts` 在生产路径上无引用（grep 验证）；如保留则迁移至 fixtures 目录。
- [ ] `npm --prefix frontend run typecheck` 通过。
- [ ] 至少一个 View 拥有"挂载即调用 API"的单元/集成测试（细节在 [BUG-WEB-05](05-integration-test-gap.md)）。
- [ ] View 切换 PR 中 loading / error / empty 三态均可在浏览器手动复现（依赖 [BUG-WEB-04](04-loading-error-empty-states.md) 基础设施已先行合并）。

## 风险与回滚

- 切换为真实数据后，若后端无数据，UI 可能空白——由 BUG-WEB-04 兜底。
- 回滚方式：恢复 `App.vue` 中的 `snapshot` prop 传递与 `consoleSnapshot` 引用。
