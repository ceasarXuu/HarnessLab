# BUG-WEB-03: 前端 UI 模型与后端 schema 契约错位

- Created: 2026-06-23
- Updated: 2026-06-23
- Version: 1.0
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: frontend types & api client, ornnlab API routers
- Related Links: [README](README.md), [BUG-WEB-02](02-views-not-consuming-api.md)
- Risk Level: Medium
- Plan Type: Standard
- Phase: 2（数据接入，前置）

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

前端 UI 模型 [types/console.ts](../../../../frontend/src/types/console.ts) 与后端 schema [api/client.ts](../../../../frontend/src/api/client.ts) 的 `Experiment / LeaderboardEntryResponse / ExperimentRun` 字段并不一致；同时，后端已有的 `agents` 与 `system` 端点在前端 `ornnLabApi` 中缺失访问器。

典型偏差：

| UI 模型字段（`types/console.ts`） | 后端实际字段 | 状态 |
|---|---|---|
| `ExperimentRecord.state: 'complete' \| 'queued' \| 'running'` | `Experiment.status: string`（可能取 `cancelled / completed / draft / failed / interrupted / queued / running`） | 枚举范围不一致，UI 需扩展或映射 |
| `ExperimentRecord.successRate: string` | 后端无 `successRate` 字段；需从 `ExperimentStateResponse.runs` 派生 | UI 需 mapper 派生 |
| `ExperimentRecord.target` / `owner` / `updatedAt` | 后端 `Experiment.kind / mode / updated_at` | 字段名/语义需对齐 |
| `AgentRecord.*` | `ornnLabApi` 没有 `agents()` 访问器，但 [ornnlab/api/agents.py](../../../../ornnlab/api/agents.py) 已提供 `GET /api/agents` | API client 缺失 |
| `KpiMetric.*` | 后端无对应聚合端点 | 需在前端从 `experiments + leaderboard` 派生，或后续在后端增设 `system/metrics` |

## 证据

- [frontend/src/types/console.ts](../../../../frontend/src/types/console.ts) 第 1–58 行：UI 模型自定义。
- [frontend/src/api/client.ts](../../../../frontend/src/api/client.ts) `ornnLabApi` 只暴露 `experiments / templates / runs / leaderboard`；无 `agents()` `systemStatus()`。
- [ornnlab/api/agents.py](../../../../ornnlab/api/agents.py)、[ornnlab/api/system.py](../../../../ornnlab/api/system.py)、[ornnlab/api/benchmarks.py](../../../../ornnlab/api/benchmarks.py) 均已挂载在 [ornnlab/app.py](../../../../ornnlab/app.py)。

## 修复方案

1. 在 [frontend/src/api/client.ts](../../../../frontend/src/api/client.ts) 补齐缺失访问器：
   - `agents.list() / get(id) / compile(id)`
   - `system.status() / doctor()`
   - `benchmarks.list()`（按 [ornnlab/api/benchmarks.py](../../../../ornnlab/api/benchmarks.py) 实际签名）
   - 同步增加对应 Response 类型（保持以后端 schema 为契约源）。
2. 新建 `frontend/src/api/mappers/`（或就近放在 `views/<page>/mappers.ts`），把后端 schema 映射为 UI view-model：
   - `toExperimentRecord(experiment, runs?) -> ExperimentRecord`
   - `toAgentRecord(...)`
   - `toKpiMetrics(experiments, leaderboard)`
   - `toLeaderboardSeed(leaderboardEntries)`
3. UI 枚举（如 `ExperimentState`）按后端实际状态集扩展，或在 mapper 中聚合为 UI 友好状态（例如 `failed/interrupted/cancelled → 'stopped'`）。映射规则在文档中显式列出。
4. 不在本 PR 修改 [types/console.ts](../../../../frontend/src/types/console.ts) 的语义边界，避免与 [BUG-WEB-02](02-views-not-consuming-api.md) 的改动互相耦合；本 PR 仅做"客户端 + 映射器 + 类型扩展"。

## Acceptance Criteria

- [ ] `ornnLabApi` 暴露 `agents / system / benchmarks` 命名空间，与后端实际端点一一对应。
- [ ] 所有 mapper 函数有对应单测，覆盖典型 + 边界（空数组、缺失字段、未知 status）。
- [ ] 文档（本文件或同目录补充说明）列出最终的字段映射表，便于 review。
- [ ] `npm --prefix frontend run typecheck` 通过；类型上 UI view-model 不直接 import 后端 Response 类型（依赖在 mapper 层）。

## 风险与回滚

- 增加 mapper 层会使前端结构略复杂；属于必要分层成本，不回滚。
- 类型扩展不破坏现有 View 静态使用方式（[BUG-WEB-02](02-views-not-consuming-api.md) 切换前后均兼容）。
