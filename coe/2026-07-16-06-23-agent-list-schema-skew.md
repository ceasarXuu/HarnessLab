# Problem P-001: Agent 列表因进程与数据库版本不一致而加载失败
- Status: fixed
- Created: 2026-07-16 06:23
- Updated: 2026-07-16 06:26
- Objective: 恢复 Agent 列表，并确认运行中后端与当前数据库 schema 一致。
- Symptoms:
  - Agent 目录为空，页面显示“无法加载 Agent”。
- Expected behavior:
  - Agent API 返回 Harbor 内置 Agent 和已保存模板。
- Actual behavior:
  - `GET /api/webui/v1/agents` 返回 500。
- Impact:
  - 本地开发环境无法查看和编辑 Agent。
- Reproduction:
  - 打开 `http://127.0.0.1:5173/#agents`。
- Environment:
  - macOS，本地 `main`，提交 `e89c4df`，OrnnLab dev daemon。
- Known facts:
  - 数据库已执行 migration 006，旧表已删除；后端进程早于 migration 006 启动。
- Ruled out:
  - 前端静态资源异常；请求经前端代理和直连后端均稳定返回同一个 500。
- Fix criteria:
  - 重启后 API 返回 200，Agent 列表非空，后端日志不再出现旧表查询。
- Current conclusion: 重启 daemon 后进程与 schema 已重新对齐，Agent API 恢复。
- Related hypotheses:
  - H-001
- Resolution basis:
  - E-003
- Close reason:
  - 原症状验证通过。

## Hypothesis H-001: 后端仍在运行迁移前加载的旧代码
- Status: confirmed
- Parent: P-001
- Claim: 后端进程在代码升级前启动，仍查询已被 migration 006 删除的 `webui_agent_configs`。
- Layer: root-cause
- Factor relation: single
- Depends on:
  - none
- Rationale:
  - Python 服务不会自动重新加载已导入模块，daemon 仅在子进程退出时重启。
- Falsifiable predictions:
  - If true: 日志包含旧表查询错误，后端启动时间早于 migration 006，重启后恢复。
  - If false: 当前源码仍查询旧表，或重启后仍返回同一错误。
- Diagnostic evidence plan:
  - Prediction or clause under test: 日志和运行时启动时间应同时证明版本偏斜。
  - Signal: API 响应、后端 traceback、daemon state、SQLite schema。
  - Capture method: curl API、读取日志和 state、查询 schema_migrations。
  - Event name or marker:
    - `sqlite3.OperationalError: no such table: webui_agent_configs`
  - Correlation keys:
    - requestId `7945392116064e6d9f9fd843375a6120`
  - Differentiates from:
    - 前端渲染错误或 Agent 数据为空。
  - Supports if:
    - 旧进程查询已删除表且其启动时间早于迁移时间。
  - Refutes if:
    - 运行中源码不查询旧表且错误来自其他路径。
  - Instrumentation status: none
  - Instrumentation lifecycle:
    - none
- Evidence gate: satisfied
- Related evidence:
  - E-001
  - E-002
- Conclusion: 日志、schema 与进程时间共同确认版本偏斜。
- Repair design readiness: ready
- Next step: 已完成。
- Blocker:
  - none
- Close reason:
  - not closed

## Evidence E-001: Agent API 查询已删除旧表
- Related hypotheses:
  - H-001
- Direction: supports
- Type: diagnostic-log
- Source: `~/.ornnlab/dev-service/logs/backend.log`
- Prediction or plan link:
  - H-001 日志预测。
- Matched signal:
  - `sqlite3.OperationalError: no such table: webui_agent_configs`
- Correlation keys:
  - requestId `7945392116064e6d9f9fd843375a6120`
- Raw content:
  ```text
  GET /api/webui/v1/agents -> 500
  sqlite3.OperationalError: no such table: webui_agent_configs
  ```
- Interpretation: 运行中的代码仍执行已从当前源码删除的旧查询。
- Time: 2026-07-16 06:21

## Evidence E-002: 后端启动时间早于 migration 006
- Related hypotheses:
  - H-001
- Direction: supports
- Type: environment
- Source: daemon state 与 SQLite schema_migrations
- Prediction or plan link:
  - H-001 进程时间预测。
- Matched signal:
  - backend 启动于 02:20，migration 006 应用于 05:57。
- Correlation keys:
  - backendPid `28332`
- Raw content:
  ```text
  backendStartTime: Thu Jul 16 02:20:53 2026
  006_unify_agent_configs: 2026-07-15 21:57:23 UTC
  ```
- Interpretation: 迁移发生时旧后端没有重新加载新代码。
- Time: 2026-07-16 06:22

## Evidence E-003: 重启后 Agent API 恢复
- Related hypotheses:
  - H-001
- Direction: supports
- Type: fix-validation
- Source: `ornnlab dev restart` 与 Agent API 探针
- Prediction or plan link:
  - H-001 重启恢复预测及 P-001 fix criteria。
- Matched signal:
  - 新后端启动后返回 HTTP 200 和非空 Agent 列表。
- Correlation keys:
  - requestId `fb1297c566b94090acb0e55d15e0a4a7`
- Raw content:
  ```text
  HTTP/1.1 200 OK
  Content-Length: 18318
  GET /api/webui/v1/agents HTTP/1.1 200 OK
  ```
- Interpretation: 当前代码、数据库 schema 和前端代理已一致，原故障消失。
- Time: 2026-07-16 06:26
