# BUG-WEB-05: 前端缺少 API 层与 View 层的集成测试

- Created: 2026-06-23
- Updated: 2026-06-24
- Version: 1.1
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: frontend tests, scripts/test-after-change-web.sh
- Related Links: [README](README.md), [BUG-WEB-02](02-views-not-consuming-api.md), [BUG-WEB-04](04-loading-error-empty-states.md), [bugfix/04-sse-stream-not-realtime.md](../bugfix/04-sse-stream-not-realtime.md)
- Risk Level: Medium
- Plan Type: Standard
- Phase: 3（测试基建）
- Revision Notes: v1.1 增加测试断言深度要求（R10）：≥1 个 View 测试需做特定输入→特定 DOM 文本断言，而非仅断言"调用了 API"或"未抛错"。来源：vs_review/2026-06-23-web-connectivity-plan-review.md

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

当前前端只有：

- 工具函数单测：[frontend/src/utils/leaderboard.test.ts](../../../../frontend/src/utils/leaderboard.test.ts)
- 一条 e2e 导航 smoke：[frontend/tests/e2e/navigation.spec.ts](../../../../frontend/tests/e2e/navigation.spec.ts)

完全没有对 `ornnLabApi` / View 数据流的覆盖。一旦 [BUG-WEB-02](02-views-not-consuming-api.md) 接入真实 API，回归保护几乎为零。

## 证据

- [frontend/vitest.config.ts](../../../../frontend/vitest.config.ts)、[frontend/playwright.config.ts](../../../../frontend/playwright.config.ts) 已就绪。
- [scripts/test-after-change-web.sh](../../../../scripts/test-after-change-web.sh) 已存在但未覆盖联调路径。

## 修复方案

1. **API client 单测**（Vitest + `fetch` mock）：
   - 覆盖 `createApiClient` 拼路径、`Content-Type` 注入、非 2xx → `ApiError`、JSON 解析失败保护。
   - 覆盖 `ornnLabApi` 每个 method 的 URL/HTTP method/payload 正确性。
   - 覆盖 `query` 参数传递（呼应 [BUG-WEB-03](03-contract-gap-vs-backend.md) F4 apiClient 能力扩展）。
2. **View 集成测试**（Vitest + `@vue/test-utils` + 局部 `fetch` mock 或 MSW，按依赖最小化原则优先 mock fetch）：
   - 每个 View 至少 1 个 happy path、1 个 error path、1 个 empty path（呼应 [BUG-WEB-04](04-loading-error-empty-states.md)）。
   - **断言深度要求（R10 新增）**：≥1 个 View 的 happy path 测试必须做**特定输入 → 特定 DOM 文本**断言，而非仅断言"调用了 API"或"未抛错"。例如：mock `ornnLabApi.experiments()` 返回含 `{ name: "exp-001", status: "completed" }` 的数组，断言页面渲染出 `exp-001` 文本与 `complete` 状态标签。这确保 mapper 层与 View 模板的数据流真正贯通，而非仅验证函数被调用。
3. **e2e smoke 扩展**（Playwright）：
   - 在现有 `navigation.spec.ts` 基础上增加 "首屏拉到数据" 断言；后端不可用时跳过或 xfail。
4. **脚本编排**：
   - 更新 [scripts/test-after-change-web.sh](../../../../scripts/test-after-change-web.sh) 串起 `typecheck / lint / vitest / e2e`；CI 中接入。
5. SSE 实时事件测试不在本立项范围，等 [bugfix/04](../bugfix/04-sse-stream-not-realtime.md) 落地后追加。

## Acceptance Criteria

- [ ] `npm --prefix frontend run test` 通过，新增 API client 与 View 集成测试覆盖率纳入报告。
- [ ] ≥1 个 View 集成测试包含"特定输入 → 特定 DOM 文本"断言（R10），而非仅断言 API 被调用。
- [ ] `npm --prefix frontend run e2e` smoke 含 "首屏数据" 断言；可在 CI 上稳定运行。
- [ ] `scripts/test-after-change-web.sh` 一键串起 typecheck/lint/单测/e2e，CI 接入。
- [ ] 文档记录测试夹具（fixture）来源：仅复用 [consoleSnapshot.ts](../../../../frontend/src/data/consoleSnapshot.ts) 迁移后的 fixtures，不引入网络依赖。

## 风险与回滚

- 增加 CI 时长；通过 vitest `--pool vmThreads --maxWorkers=1` 控制资源（与现有脚本一致）。
- 测试本身不影响运行时；回滚为删除新增测试文件与脚本变更。
