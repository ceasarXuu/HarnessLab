# BUG-WEB-03: 前端 UI 模型与后端 schema 契约错位

- Created: 2026-06-23
- Updated: 2026-06-24
- Version: 1.2
- Status: Implemented
- Owner / Responsible: project maintainer
- Related Systems: frontend types & api client, ornnlab API routers
- Related Links: [README](README.md), [BUG-WEB-02](02-views-not-consuming-api.md), [bugfix/04-sse-stream-not-realtime.md](../bugfix/04-sse-stream-not-realtime.md)
- Risk Level: Medium
- Plan Type: Standard
- Phase: 2（数据接入，前置）
- Revision Notes: v1.1 31 端点表（F2）+ viewmodel 字段决策（F3）+ query-param（F4）+ mapper 判据（R3）+ SSE Deferred（R4）+ job_dir gap（R7）+ openapi-typescript defer（R9）。v1.2 新增删除字段实施约定（Round 3 N3，mapper 输出 `""` 保持类型不变）+ SSE 行补 `after` query param（Round 3 N1）。来源：vs_review/2026-06-23-web-connectivity-plan-review.md + vs_review/2026-06-24-closure-review-round3.md

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

前端 UI 模型 [types/console.ts](../../../../frontend/src/types/console.ts) 与后端 schema [api/client.ts](../../../../frontend/src/api/client.ts) 的 `Experiment / LeaderboardEntryResponse / ExperimentRun` 字段并不一致；同时，后端已有的 `agents` 与 `system` 端点在前端 `ornnLabApi` 中缺失访问器，且 `ornnLabApi` 现有覆盖的端点也不完整（遗漏 create / delete / events 等）。

典型偏差：

| UI 模型字段（`types/console.ts`） | 后端实际字段 | 状态 |
|---|---|---|
| `ExperimentRecord.state: 'complete' \| 'queued' \| 'running'` | `Experiment.status: string`（可能取 `cancelled / completed / draft / failed / interrupted / queued / running`） | 枚举范围不一致，UI 需扩展或映射 |
| `ExperimentRecord.successRate: string` | 后端无 `successRate` 字段；需从 `ExperimentStateResponse.runs` 派生 | UI 需 mapper 派生 |
| `ExperimentRecord.target` / `owner` / `updatedAt` | 后端 `Experiment.kind / mode / updated_at`（无 `owner`） | 字段名/语义需对齐；`owner` 无后端源 |
| `AgentRecord.*` | `ornnLabApi` 没有 `agents()` 访问器，但 [ornnlab/api/agents.py](../../../../ornnlab/api/agents.py) 已提供 `GET /api/agents` | API client 缺失 |
| `KpiMetric.*` | 后端无对应聚合端点 | 需在前端从 `experiments + leaderboard` 派生，或后续在后端增设 `system/metrics` |

## 证据

- [frontend/src/types/console.ts](../../../../frontend/src/types/console.ts) 第 1–58 行：UI 模型自定义。
- [frontend/src/api/client.ts](../../../../frontend/src/api/client.ts) `ornnLabApi` 只暴露 `experiments / templates / runs / leaderboard`；无 `agents()` `systemStatus()`。
- [ornnlab/api/agents.py](../../../../ornnlab/api/agents.py)、[ornnlab/api/system.py](../../../../ornnlab/api/system.py)、[ornnlab/api/benchmarks.py](../../../../ornnlab/api/benchmarks.py) 均已挂载在 [ornnlab/app.py](../../../../ornnlab/app.py)。

## 后端端点完整清单（F2 修正）

以 [ornnlab/api/*.py](../../../../ornnlab/api/) 的 `@router.*` 装饰器为契约源，共 **31 个端点**。下表逐一标注 `ornnLabApi` 覆盖状态：

| # | Router | Method | Endpoint | Query Params | ornnLabApi 现有方法 | 状态 |
|---|---|---|---|---|---|---|
| 1 | system | GET | `/api/system/status` | — | ❌ | 待补 |
| 2 | system | POST | `/api/system/doctor` | `logs: bool = False` | ❌ | 待补 |
| 3 | system | GET | `/api/system/docker-orphans` | — | ❌ | **遗漏**（self-pass 也未提及） |
| 4 | agents | GET | `/api/agents` | — | ❌ | 待补 `agents.list()` |
| 5 | agents | POST | `/api/agents` | — | ❌ | 待补 `agents.create()` |
| 6 | agents | GET | `/api/agents/{agent_id}` | — | ❌ | 待补 `agents.get(id)` |
| 7 | agents | POST | `/api/agents/{agent_id}/compile` | — | ❌ | 待补 `agents.compile(id)` |
| 8 | agents | POST | `/api/agents/validate` | — | ❌ | 待补 `agents.validate()` |
| 9 | agents | PUT | `/api/agents/{agent_id}` | — | ❌ | 待补 `agents.update(id)` |
| 10 | agents | DELETE | `/api/agents/{agent_id}` | — | ❌ | 待补 `agents.delete(id)` |
| 11 | benchmarks | GET | `/api/benchmarks` | — | ❌ | 待补 `benchmarks.list()` |
| 12 | experiments | GET | `/api/experiments` | — | ✅ `experiments()` | 已覆盖 |
| 13 | experiments | POST | `/api/experiments` | — | ❌ | 待补 `createExperiment()` |
| 14 | experiments | GET | `/api/experiments/{id}` | — | ✅ `experiment(id)` | 已覆盖 |
| 15 | experiments | POST | `/api/experiments/{id}/run` | `wait: bool = False` | ✅ `runExperiment(id)` | **部分**（未传 `wait` query param，见 F4） |
| 16 | experiments | POST | `/api/experiments/{id}/cancel` | — | ✅ `cancelExperiment(id)` | 已覆盖 |
| 17 | experiments | DELETE | `/api/experiments/{id}` | — | ❌ | 待补 `deleteExperiment(id)` |
| 18 | experiments | POST | `/api/experiments/{id}/clone` | — | ✅ `cloneExperiment(id)` | 已覆盖 |
| 19 | experiments | POST | `/api/experiments/{id}/save-template` | — | ✅ `saveExperimentTemplate(id, name)` | 已覆盖 |
| 20 | experiments | GET | `/api/experiments/{id}/report` | — | ✅ `experimentReport(id)` | 已覆盖 |
| 21 | experiments | GET | `/api/experiments/{id}/events` | `after: int = 0` | ❌ | 待补 `experimentEvents(id, after)` |
| 22 | experiments | GET | `/api/experiments/{id}/events/stream` | `after: int = 0` | ❌ | **Deferred**（SSE，见 [bugfix/04](../bugfix/04-sse-stream-not-realtime.md)；实施时勿漏 `after` query param） |
| 23 | experiments | GET | `/api/experiments/{id}/runs` | — | ❌ | 待补 `experimentRuns(id)` |
| 24 | runs | GET | `/api/runs/{run_id}` | — | ✅ `run(id)` | 已覆盖 |
| 25 | runs | POST | `/api/runs/{run_id}/cancel` | — | ✅ `cancelRun(id)` | 已覆盖 |
| 26 | runs | GET | `/api/runs/{run_id}/events` | `after: int = 0` | ❌ | 待补 `runEvents(id, after)` |
| 27 | runs | GET | `/api/runs/{run_id}/report` | — | ✅ `runReport(id)` | 已覆盖 |
| 28 | templates | GET | `/api/templates` | — | ✅ `templates()` | 已覆盖 |
| 29 | templates | POST | `/api/templates` | — | ✅ `createTemplate(name, config)` | 已覆盖 |
| 30 | templates | DELETE | `/api/templates/{template_id}` | — | ❌ | 待补 `deleteTemplate(id)` |
| 31 | leaderboard | GET | `/api/leaderboard` | `benchmark: str \| None = None` | ✅ `leaderboard(benchmark?)` | 已覆盖 |

**统计**：已覆盖 13 / 31；待补 17；Deferred 1（SSE stream）。

### R7：ExperimentRun 缺 `job_dir` 字段

后端 [ornnlab/models/experiment.py#L48](../../../../ornnlab/models/experiment.py) 的 `RunView` 包含 `job_dir: str | None = None`，但前端 [frontend/src/api/client.ts](../../../../frontend/src/api/client.ts) 的 `ExperimentRun` 接口未声明该字段。本次 plan-only 阶段在端点表内标注；实施时在 `ExperimentRun` 接口补 `job_dir: string | null`。

## UI viewmodel 字段数据源决策（F3 新增）

逐字段分析 [types/console.ts](../../../../frontend/src/types/console.ts) 中每个 UI 模型字段的后端数据源，给出 **保留 / 派生 / 删除** 决策。后端 schema 来源：[ornnlab/models/experiment.py](../../../../ornnlab/models/experiment.py) `ExperimentView` / `RunView`、[ornnlab/storage/migrations/001_initial.sql](../../../../ornnlab/storage/migrations/001_initial.sql) `agents` 表、[frontend/src/api/client.ts](../../../../frontend/src/api/client.ts) `LeaderboardEntryResponse`。

### `ExperimentRecord`（UI）← `Experiment` + `ExperimentRun[]`（后端）

| UI 字段 | 后端源 | 决策 | 说明 |
|---|---|---|---|
| `id` | `Experiment.id` | 保留（直传） | — |
| `name` | `Experiment.name` | 保留（直传） | — |
| `owner` | **无** | **删除** | 后端无 owner 概念；本地单机应用无多用户 |
| `state` | `Experiment.status` | 派生（mapper 聚合） | 后端 7 态 → UI 3 态映射：`completed→complete`、`queued/draft→queued`、`running/failed/interrupted/cancelled→running`（或扩展 UI 枚举，见修复方案 3） |
| `target` | `Experiment.kind` | 派生（重命名） | `kind: 'batch'\|'comparison'\|'single'` 即 UI 所需的 target 语义 |
| `updatedAt` | `Experiment.updated_at` | 派生（重命名） | snake_case → camelCase |
| `successRate` | `ExperimentRun[]` | 派生（聚合） | `completed / total`，需 `experiment(id)` 取 runs |

### `AgentRecord`（UI）← Agent（后端 `agents` 表）

后端 `agents` 表字段：`id, name, kind, harbor_agent_name, harbor_import_path, model_name, status, profile_path, created_at, updated_at`。

| UI 字段 | 后端源 | 决策 | 说明 |
|---|---|---|---|
| `name` | `agents.name` | 保留（直传） | — |
| `owner` | **无** | **删除** | 后端无 owner 概念 |
| `queue` | **无** | **删除** | 后端无 queue 概念；本地单机 |
| `health` | `agents.status` | 派生（枚举映射） | `compiled→healthy`、`draft→warming`、`deleted/error→blocked` |
| `activeRuns` | `runs` 表 | 派生（聚合查询） | 需 `GET /api/experiments/{id}/runs` 或后端增设聚合端点；本期可先固定 0 或从 system/doctor 派生 |
| `lastHeartbeat` | **无** | **删除** | 后端无心跳机制 |
| `successRate` | `runs` 表 | 派生（聚合） | 同 ExperimentRecord.successRate；本期可先固定 `'—'` |

### `KpiMetric`（UI）← `experiments + leaderboard` 聚合

| UI 字段 | 后端源 | 决策 | 说明 |
|---|---|---|---|
| `label / value / delta / trend / description` | **无直接端点** | 派生（前端聚合） | 从 `experiments()` + `leaderboard()` 在前端计算；如 "总实验数 = experiments.length"、"最高分 = max(leaderboard.score)" |

### `AlertItem`（UI）

| UI 字段 | 后端源 | 决策 | 说明 |
|---|---|---|---|
| `title / detail / severity` | **无** | **删除**（本期） | 后端无 alerts 端点；可未来从 `system/doctor` 派生，但本期不引入 |

### `LeaderboardSeed`（UI）← `LeaderboardEntryResponse`（后端）

| UI 字段 | 后端源 | 决策 | 说明 |
|---|---|---|---|
| `agent` | `agent_id` | 派生（重命名） | — |
| `score` | `score` | 保留（直传） | — |
| `successRate` | **无直接源** | 派生（聚合）或删除 | 需跨表 join runs；本期可删除或固定 |
| `experiments` | **无直接源** | 派生（聚合）或删除 | 需跨表 join experiments；本期可删除或固定 |

### 决策汇总

- **删除**（无后端源且非核心）：`ExperimentRecord.owner`、`AgentRecord.owner`、`AgentRecord.queue`、`AgentRecord.lastHeartbeat`、`AlertItem.*`
- **派生（重命名）**：`target←kind`、`updatedAt←updated_at`、`agent←agent_id`
- **派生（枚举映射）**：`state←status`、`health←status`
- **派生（聚合）**：`successRate`、`activeRuns`、`KpiMetric.*`
- **保留（直传）**：`id`、`name`、`score`

#### 删除字段的实施约定（N3 澄清）

本 PR（[BUG-WEB-03](03-contract-gap-vs-backend.md)）**不修改** [frontend/src/types/console.ts](../../../../frontend/src/types/console.ts) 的字段定义（见本文件 §修复方案 4 范围边界）。为了兼容"类型不动 + 字段无后端源"的张力，**mapper 输出统一规则**：

- mapper 函数返回的 viewmodel 对象中，**被标记为"删除"的字段填充 `""`（字符串）/`0`（数值）/`[]`（数组）**，保持类型签名不变。
- 这些字段的**模板渲染由 [BUG-WEB-02](02-views-not-consuming-api.md) View 切换 PR 负责**：要么删除对应模板段落（如 `AgentsView` 的 Queue/Heartbeat 列、`DashboardView` 的 Priority alerts 区块），要么改为占位文案（`"—"`）。
- 真正从 `types/console.ts` 移除已删除字段，留到 v0.1.5 PRD（与 OpenAPI 自动类型生成一并评估）。这样本期可保持类型稳定，View 改动局限在模板层，符合"分层成本最小"原则。

## 修复方案

### 1. apiClient 能力扩展（F4 新增）

当前 [frontend/src/api/client.ts](../../../../frontend/src/api/client.ts) 的 `ApiClient.post` 签名为 `post<TRequest, TResponse>(path, payload)`，**无法传递 query param**（如 `POST /api/experiments/{id}/run?wait=true`、`POST /api/system/doctor?logs=true`）。扩展条款：

```ts
export interface ApiClient {
  get<TResponse>(path: string, query?: Record<string, string | number | boolean>): Promise<TResponse>
  post<TRequest, TResponse>(
    path: string,
    payload: TRequest,
    query?: Record<string, string | number | boolean>,
  ): Promise<TResponse>
}
```

实现侧用 `URLSearchParams` 拼接 query string。现有调用点（无 query）保持兼容。

### 2. 补齐缺失访问器

在 [frontend/src/api/client.ts](../../../../frontend/src/api/client.ts) 补齐上表"待补"的 17 个端点访问器（SSE stream 除外），按 router 命名空间分组：

- `system.status() / doctor(logs?) / dockerOrphans()`
- `agents.list() / create(payload) / get(id) / compile(id) / validate(payload) / update(id, payload) / delete(id)`
- `benchmarks.list()`
- `createExperiment(payload) / deleteExperiment(id) / experimentEvents(id, after?) / experimentRuns(id)`
- `runEvents(id, after?)`
- `deleteTemplate(id)`
- `runExperiment(id, wait?)` — 扩展支持 `wait` query param

同步增加对应 Response 类型（保持以后端 schema 为契约源）。

### 3. mapper 层（R3 修正判据）

**mapper 判据**（修正 self-pass R3）：

- **仅 1:1 字段复制**（如 `id`、`name`）→ **不引入 mapper**，View 直接消费 `ornnLabApi` 返回类型。
- **枚举映射 / 重命名 / 聚合 / 多对一派生** → **保留 mapper**（如 `state←status`、`target←kind`、`successRate`、`KpiMetric`）。

按此判据，新建 `frontend/src/api/mappers/`（或就近放在 `views/<page>/mappers.ts`），仅包含：

- `toExperimentRecord(experiment, runs?) -> ExperimentRecord`（枚举映射 + 聚合 successRate）
- `toAgentRecord(agent, activeRuns?) -> AgentRecord`（枚举映射 + 聚合）
- `toKpiMetrics(experiments, leaderboard)`（多对一聚合）
- `toLeaderboardSeed(entry) -> LeaderboardSeed`（重命名 + 可选聚合）

UI 枚举（如 `ExperimentState`）按后端实际状态集扩展，或在 mapper 中聚合为 UI 友好状态（例如 `failed/interrupted/cancelled → 'stopped'`）。映射规则在文档中显式列出。

### 4. 范围边界

- 不在本 PR 修改 [types/console.ts](../../../../frontend/src/types/console.ts) 的语义边界，避免与 [BUG-WEB-02](02-views-not-consuming-api.md) 的改动互相耦合；本 PR 仅做"客户端 + 映射器 + 类型扩展"。
- **SSE 实时事件流（`/events/stream`）不在本立项范围**，等 [bugfix/04](../bugfix/04-sse-stream-not-realtime.md) 落地后再追加客户端访问器与测试。

## Acceptance Criteria

- [x] `ornnLabApi` 暴露 `agents / system / benchmarks` 命名空间，与后端实际端点一一对应（31 端点表全覆盖，SSE stream 显式 Deferred）。
- [x] `ApiClient.get / post` 支持 `query` 参数；`runExperiment(id, wait?)` 可传递 `wait` query param。
- [x] `ExperimentRun` 接口包含 `job_dir: string | null` 字段（R7）。
- [x] 所有 mapper 函数有对应单测，覆盖典型 + 边界（空数组、缺失字段、未知 status）。
- [x] 文档（本文件）列出最终的字段映射表与 viewmodel 数据源决策，便于 review。
- [x] `npm --prefix frontend run typecheck` 通过；类型上 UI view-model 不直接 import 后端 Response 类型（依赖在 mapper 层）。

## Implementation

- **Batch 1** commit `2fd7541`：
  - [frontend/src/api/client.ts](../../../../frontend/src/api/client.ts) `ApiClient` 扩展 get/post/put/delete + query 参数（`URLSearchParams`）；`ornnLabApi` 新增 17 个访问器（`system / agents / benchmarks` 命名空间 + 实验/runs/templates 缺口）；`ExperimentRun` 补 `job_dir: string | null`；空 body DELETE 返回 undefined
  - [frontend/src/api/mappers.ts](../../../../frontend/src/api/mappers.ts) + [frontend/src/api/mappers.test.ts](../../../../frontend/src/api/mappers.test.ts)：4 个 mapper（`toExperimentRecord` / `toAgentRecord` / `toKpiMetrics` / `toLeaderboardSeed`）+ 26 个 vitest 测试，覆盖枚举映射、N3 空值约定、边界场景

## 风险与回滚

- 增加 mapper 层会使前端结构略复杂；按 R3 判据仅对必要场景引入，避免过度工程。
- 类型扩展不破坏现有 View 静态使用方式（[BUG-WEB-02](02-views-not-consuming-api.md) 切换前后均兼容）。

## Maintenance Follow-up（R9 defer 到 v0.1.5）

**OpenAPI 自动类型生成**：当前前端 Response 类型手工维护，与后端 schema 漂移风险高。v0.1.5 PRD 评估引入 [openapi-typescript](https://github.com/drwpow/openapi-typescript) 或同类工具，从 FastAPI 的 `/openapi.json` 自动生成 TypeScript 类型，替代手工 `Experiment / ExperimentRun / LeaderboardEntryResponse` 等接口。本期仅在本文档记录评估意向，不实施。
