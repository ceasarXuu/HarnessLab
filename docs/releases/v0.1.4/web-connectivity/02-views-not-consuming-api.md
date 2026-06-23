# BUG-WEB-02: Views 未消费 `ornnLabApi`，仅显示静态快照

- Created: 2026-06-23
- Updated: 2026-06-23
- Version: 1.0
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: frontend views, frontend api client
- Related Links: [README](README.md), [BUG-WEB-03](03-contract-gap-vs-backend.md), [BUG-WEB-04](04-loading-error-empty-states.md)
- Risk Level: High
- Plan Type: Standard
- Phase: 2（数据接入，主线）

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

1. 引入 view-model 与 mapper 层（具体字段映射在 [BUG-WEB-03](03-contract-gap-vs-backend.md)），让 View 不直接耦合后端 schema。
2. 改造每个 View，使用 `onMounted` + `ref` + `try/catch` 的最小数据接入：
   - `DashboardView` ← `ornnLabApi.experiments()` + `ornnLabApi.leaderboard()`（用于 Top N 与 KPI 派生）
   - `AgentsView` ← 新增（在 BUG-WEB-03 中补齐）`ornnLabApi.agents()`
   - `ExperimentsView` ← `ornnLabApi.experiments()`
   - `LeaderboardView` ← `ornnLabApi.leaderboard()`
3. 把 [consoleSnapshot.ts](../../../../frontend/src/data/consoleSnapshot.ts) 移到 `frontend/src/__fixtures__/` 或 `tests/fixtures/`，作为 Storybook / 单测的 fixture，不再作为运行时数据源。
4. 与 [BUG-WEB-04](04-loading-error-empty-states.md) 一并提交，确保每个 View 在 loading / error / empty 状态下有明确呈现。

## Acceptance Criteria

- [ ] 四个 View 在挂载后均会调用 `ornnLabApi` 并把响应映射到 UI；后端响应变更时刷新页面可见到变化。
- [ ] `frontend/src/data/consoleSnapshot.ts` 在生产路径上无引用（grep 验证）；如保留则迁移至 fixtures 目录。
- [ ] `npm --prefix frontend run typecheck` 通过。
- [ ] 至少一个 View 拥有"挂载即调用 API"的单元/集成测试（细节在 [BUG-WEB-05](05-integration-test-gap.md)）。
- [ ] 与 [BUG-WEB-04](04-loading-error-empty-states.md) 同 PR 提交，loading / error / empty 三态均可在浏览器手动复现。

## 风险与回滚

- 切换为真实数据后，若后端无数据，UI 可能空白——由 BUG-WEB-04 兜底。
- 回滚方式：恢复 `App.vue` 中的 `snapshot` prop 传递与 `consoleSnapshot` 引用。
