# v0.1.4 Web 服务调通修复计划

- Created: 2026-06-23
- Updated: 2026-06-24
- Version: 1.1
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: frontend (Vue 3 + Vite), ornnlab FastAPI backend, dev server proxy
- Related Links: [bugfix/README](../bugfix/README.md), [frontend/src/api/client.ts](../../../../frontend/src/api/client.ts), [ornnlab/app.py](../../../../ornnlab/app.py)
- Risk Level: Medium
- Plan Type: Standard
- Revision Notes: v1.1 拆 PR 切片（R1）：04 基础设施独立 PR 先行；新增量化验收指标（R5）。来源：vs_review/2026-06-23-web-connectivity-plan-review.md

## 状态说明

本目录是 v0.1.4 的「Web 服务调通」修复立项，不表示实现已经落地。所有 Acceptance Criteria 使用未勾选的 `[ ]`，仅在对应代码、配置与测试合入后才可改为 `[x]`。

## 背景

v0.1.4 阶段已经存在：

- **后端**：FastAPI 应用 [ornnlab/app.py](../../../../ornnlab/app.py) 注册了 `system / agents / benchmarks / experiments / runs / templates / leaderboard` 七组 `/api/*` 路由，并挂接 SQLite、Queue、Worker、Recovery 等 service 层；
- **前端**：Vue 3 + Vite 控制台脚手架（[frontend/](../../../../frontend)）已实现 `Dashboard / Agents / Experiments / Leaderboard` 四个路由页面，并在 [frontend/src/api/client.ts](../../../../frontend/src/api/client.ts) 中完成了 `apiClient` + `ornnLabApi` 的封装，类型与后端契约对齐。

但当前前端**只显示静态快照**，与后端零联动。具体证据：

1. 全仓库范围内 `@/api/client` 没有任何 import；所有 View 都消费 [frontend/src/data/consoleSnapshot.ts](../../../../frontend/src/data/consoleSnapshot.ts) 的硬编码数据。
2. [frontend/vite.config.ts](../../../../frontend/vite.config.ts) 未配置 `server.proxy`，dev 模式下 `/api` 请求会落到 Vite 自身，无法到达 FastAPI。
3. 前端没有加载态 / 错误态 / 空态的统一处理，也没有针对真实 API 的契约测试或回归测试。

这属于「前端 ↔ 后端尚未调通」的集成缺口，需要单独立项推进，避免被混入 `bugfix/` 的运行时正确性修复。

## 问题总览

| # | 文档 | 级别 | 类型 | 涉及文件 | 当前处理建议 |
|---|------|------|------|----------|--------------|
| 01 | [Vite dev proxy 缺失](01-vite-dev-proxy-missing.md) | 中 | 集成 | frontend/vite.config.ts | 增加 `server.proxy` 把 `/api` 指向 FastAPI；同步 preview 配置 |
| 02 | [Views 未消费 ornnLabApi](02-views-not-consuming-api.md) | 高 | 集成 | frontend/src/views/*.vue, frontend/src/api/client.ts | View 层切换到 `ornnLabApi`，移除/降级 `consoleSnapshot` 为 fallback or fixture |
| 03 | [契约错位与缺失端点](03-contract-gap-vs-backend.md) | 中 | 正确性 | frontend/src/types/console.ts, frontend/src/api/client.ts, ornnlab/api/* | 对齐前端 UI 模型与后端 schema；补齐 agents/system 相关访问器 |
| 04 | [加载/错误/空态缺失](04-loading-error-empty-states.md) | 中 | 体验 | frontend/src/views/*.vue, frontend/src/components/* | 引入统一的 async 状态原语（loading / error / empty / stale） |
| 05 | [集成测试缺口](05-integration-test-gap.md) | 中 | 测试 | frontend/tests/**, scripts/test-after-change-web.sh | 增加 API client 单测、MSW 级 view 集成测试与 e2e smoke |

## 跨文档冲突

| 文档对 | 冲突类型 | 说明 | 整改决策 |
|--------|----------|------|----------|
| BUG-WEB-02 ↔ BUG-WEB-03 | 数据模型 | UI 类型 `ConsoleSnapshot` 与后端 `Experiment / LeaderboardEntryResponse` 字段不一致 | 以后端 schema 为契约源；UI 模型作为派生 view-model，由 mapper 层负责转换（mapper 判据见 [BUG-WEB-03 R3](03-contract-gap-vs-backend.md#3-mapper-层r3-修正判据)） |
| BUG-WEB-02 ↔ BUG-WEB-04 | 改动面 | View 切到真实数据的同时必须有 loading/error UI，否则首版 UX 倒退 | **R1 修正**：04 基础设施（`asyncState` + `StatePanel`）独立 PR 先行合并；View 切换在同 PR 或后续 PR，避免单 PR 过大 |
| BUG-WEB-01 ↔ 运行时部署 | 部署形态 | dev 用 proxy，生产由谁托管前端静态资源尚未决定 | 本计划只交付 dev/preview proxy；生产部署形态在 v0.1.5 PRD 决定 |
| BUG-WEB-05 ↔ bugfix/04 (SSE) | 测试依赖 | 真实 SSE 集成测试需等 BUG-04 SSE 修复 land | 05 仅覆盖 REST endpoint，SSE 测试推迟到 bugfix/04 完成后再追加 |

## 执行顺序

```text
Phase 0: 现状确认 & 契约梳理
  - 列出 UI 当前使用字段 vs 后端响应字段差异（落到 BUG-WEB-03）
  - 不修改运行时代码

Phase 1: 通路打通
  BUG-WEB-01 (Vite dev proxy) → 前置，所有后续手工/自动联调依赖

Phase 2: 数据接入
  BUG-WEB-03 (契约对齐 + mapper) → 为 02 提供干净的数据形态
  BUG-WEB-04 PR-A (asyncState + StatePanel 基础设施) → 独立 PR，不依赖 02/03
  BUG-WEB-02 (Views 切换到 ornnLabApi) → 依赖 03 + 04 PR-A
  BUG-WEB-04 PR-B (View 接入 StatePanel) → 与 02 同 PR 或紧随其后

Phase 3: 测试基建
  BUG-WEB-05 (API client 单测 + view 集成测试 + e2e smoke)

Deferred:
  - SSE 实时事件流接入：依赖 bugfix/04 修复完成
  - 生产部署形态（静态托管 / FastAPI StaticFiles / 反向代理）：v0.1.5 PRD 决定
  - OpenAPI 自动类型生成（openapi-typescript）：v0.1.5 PRD 评估（见 [BUG-WEB-03 Maintenance Follow-up](03-contract-gap-vs-backend.md#maintenance-follow-upr9-defer-到-v0115)）
```

## Phase 依赖关系图

```text
Phase 0 (契约梳理)
        │
        ▼
Phase 1 ── BUG-WEB-01 (proxy)
        │
        ▼
Phase 2 ── BUG-WEB-03 ──► BUG-WEB-02 ◄── BUG-WEB-04 PR-B
        │                  ▲
        │                  │
        └── BUG-WEB-04 PR-A (基础设施，独立先行)
        │
        ▼
Phase 3 ── BUG-WEB-05
```

## 验收

- [ ] `npm --prefix frontend run dev` 启动后，`/api/system/status` 等请求经 Vite proxy 命中本地 FastAPI，浏览器 Network 面板返回 200。
- [ ] Dashboard / Agents / Experiments / Leaderboard 四个页面均显示来自后端的真实数据；无后端可用时显示明确错误态而非空白。
- [ ] `frontend/src/data/consoleSnapshot.ts` 不再作为生产代码路径被消费（保留为测试 fixture 或被删除）。
- [ ] 后端响应与前端类型在 `BUG-WEB-03` 中逐字段对齐，存在偏差时以后端为准并提供 mapper。
- [ ] 新增 API client 单测、至少一个 View 级集成测试与一条 e2e smoke，CI 全绿。
- [ ] 文档与 [bugfix/README](../bugfix/README.md) 交叉链接清晰，便于后续 v0.1.4 收尾汇总。

### 量化验收指标（R5 新增）

"调通"需有可观测目标，避免"再调一调还是收尾"的模糊地带：

- [ ] `scripts/test-after-change-web.sh` 退出码 0（typecheck + lint + vitest + e2e 全绿）。
- [ ] e2e smoke 中至少 1 个真实 API 请求返回 2xx（如 `GET /api/system/status` → 200）。
- [ ] e2e smoke 中至少 1 个 View 首屏渲染出来自后端的真实数据文本（非静态 snapshot）。
- [ ] ≥1 个 View 集成测试包含"特定输入 → 特定 DOM 文本"断言（见 [BUG-WEB-05 R10](05-integration-test-gap.md)）。

## 不在本计划范围内

- 后端 API 的语义性 bug 修复（属于 [bugfix/](../bugfix/)）
- SSE 实时流接入（依赖 bugfix/04）
- 生产部署 / CDN / 静态资源托管策略
- 新增页面或新 UI 能力（仅做"调通"，不扩功能）
