# 核心逻辑走查

## 1. 核心文件表

| 文件 | 角色 | 关键输出/影响 | 证据 |
|---|---|---|---|
| `ornnlab/app.py` | 应用装配与生命周期 | 启动恢复、孤儿清理、Operation 对账、worker 生命周期 | S01 |
| `ornnlab/api/webui.py` | 唯一 API 入口与统一包络 | `/api/webui/v1/*` 全部路由、错误映射、参数白名单 | S02, S03 |
| `ornnlab/services/webui_job_service.py` | Job 领域服务 | 创建/查询/取消/resume/排行榜 | S04（上游） |
| `ornnlab/services/experiment_service.py` | 实验与运行状态机 + 执行主路径 | `_run_one` 驱动 Harbor 执行全过程 | S04 |
| `ornnlab/services/harbor_engine.py` | Harbor 配置构建与引擎分发 | `harbor.config.json` / `harbor.capability.json` | S05 |
| `ornnlab/services/harbor_subprocess.py` | 托管 Harbor CLI 子进程 | `job.log`、`result.json`、取消清理 | S06 |
| `ornnlab/services/worker_service.py` | 后台队列调度 | 并发执行、取消、闲置退出 | S07 |
| `ornnlab/services/queue_service.py` | SQLite 队列状态机 | `queue_items` + `runs` 状态转换 | S08 |
| `ornnlab/services/webui_operation_service.py` | 异步 Operation 生命周期 | 进度持久化、取消、重启对账 | S09 |
| `ornnlab/services/recovery_service.py` | 崩溃/重启恢复 | 终态收敛或 `interrupted` | S10 |
| `ornnlab/services/webui_job_deletion.py` | Job 完整删除 | 事务清理 + 产物安全删除 | S11 |
| `ornnlab/services/model_pricing.py` | 模型价格快照与成本 | Job 成本展示 | S12 |
| `frontend/src/api/runtimeClient.ts` | 前端模式选择 | mock/api 双模式 | S13 |
| `ornnlab/storage/migrations/*` | schema 演进 | 唯一 Schema 事实 | S14 |
| `.github/workflows/ci.yml` | 质量门 | 三平台门禁 + 可选真实 Harbor 冒烟 | S15 |

## 2. 端到端工作流：创建并执行一个 Job

1. **创建**：前端 `NewRunPage` → `runDraftToCreateJobRequest` → `POST /jobs`。后端校验 Agent 存在、`model_name` 属于该 Agent 的模型集合（`webui_job_service.py:66-67`）。
2. **落库**：`ExperimentService.create` 写入 `experiments` + `runs`（draft）；`webui_job_configs` 保存 `harbor_overrides`、环境预设、模型与价格快照；`leaderboard_eligible` 依 verifier 开关计算。
3. **入队**：`run_immediately` 时 `QueueService.enqueue_experiment`（draft → queued）并 `worker.start()`。
4. **出队执行**：`QueueWorkerService._run_until_no_queued_runs` 按 `queue_position` 顺序出队（S07/S08），受 `worker_max_concurrent`（默认 2）限制；每个 run 一个 `asyncio.Task`。
5. **配置构建**：`_run_one` 读取 WebUI config → 解析 Agent 配置 → 若用 Docker 环境则准备代理策略（`ContainerProxyRuntime`）→ `HarborConfigBuilder.build` 产出 `HarborJobConfigView`（S05）→ 原子写 `harbor.config.json` 与能力快照（S01 中 `write_run_artifacts`）。
6. **标记运行**：`_mark_run_running` 用条件 UPDATE 防竞态（仅非终态可置 running），随后发 `harbor.job.running` 事件（payload 已脱敏）。
7. **执行**：`HarborEngine.run`（默认 subprocess）→ `ManagedSubprocessHarborRunner.run`：`harbor run --config`，stdout 实时镜像到 `job.log`，取消时对进程组 SIGTERM → SIGKILL 并写 `harbor.cleanup.json`（S06）。
8. **收尾**：读取 `result.json` 映射终态（S06 中 `_status_from_result_payload`）→ 写报告摘要 → 条件 UPDATE `runs`（防止覆盖取消/失败态）→ `queue.finish` → 发 `harbor.job.completed` → 清理 Docker 资源 → 若实验所有 run 终态则收敛 `experiments.status`。
9. **展示**：`_job_dto` 聚合 trial 进度、成本、token 用量、运行时长、可恢复性等（`webui_job_service.py:334-376`）。

## 3. 重要控制流

### 3.1 启动恢复（防崩溃盲区）

`RunRecoveryService.reconcile_startup` 处理两类残留：状态为 `running` 的 run、队列项 `running` 但 run 状态异常的孤儿。对每个 run：若 `result.json` 存在则按其终态恢复并写报告（`recovered`）；否则标记 `interrupted` + `stale_running_without_result`（S10）。`create_app` 在 worker 启动前执行，避免恢复结果被新 worker 覆盖。

### 3.2 异步 Operation

`submit` 先持久化 `queued` 记录再创建 `asyncio.Task`；`_execute` 内按 `progress` 回调更新 SQLite；取消走 `task.cancel()` + 状态置 `cancelled`；异常写 `OPERATION_FAILED`。重启时 `reconcile_interrupted` 把所有 `queued/running` 且无进程内 task 的项置为 `failed/OPERATION_INTERRUPTED`（S09）。

### 3.3 Job 删除事务

`WebUiJobDeletionService.delete` 在单个事务内：校验终态 → 计算产物根（`experiments/<run_id>`，必要时 `experiments/<experiment_id>`，以及 `jobsDir/<harbor_job_name>` 单层子目录）→ `_validate_stored_artifacts` 校验 `result_path`/`report_path`/事件镜像归属 → 按依赖顺序 DELETE operation/events/configs/queue_items/runs/experiments → 删除文件。文件删除在事务提交前执行，失败会回滚数据库（S11）。

### 3.4 价格与成本

`pricing_snapshot`：`reported` 来源只记录来源（成本以 Harbor 上报 `cost_usd` 为准）；`custom` 使用用户费率；`litellm` 从 Harbor 内置 LiteLLM 目录读取费率。`calculate_cost` 对 cache hit/miss 区分计算（S12）。Job 创建时快照固定，避免后续费率变化改变历史成本。

### 3.5 前端数据模式

`createRuntimeWebUiClient` 按 `VITE_ORNNLAB_DATA_MODE` 选择 HTTP 或 mock client；`readWebUiDataMode` 默认 `PROD ? 'api' : 'mock'`（S13）。`App.tsx` 只经 hooks（`useJobs`/`useDatasets` 等）装配资源，API 模式错误显示错误状态而非回退 mock。

## 4. 关键不变量与假设

- **路由唯一性**：`ornnlab/app.py` 只 `include_router(webui.router)`；新增旧风格路由即破坏契约。
- **状态机**：`runs.status` 合法迁移为 draft → queued → running → {completed, failed, cancelled, interrupted}；`_mark_run_running` 与收尾更新都用条件 UPDATE 防覆盖。
- **删除安全**：删除只接受终态；产物根必须可证明归属；数据库与文件删除原子性由同一事务保证。
- **脱敏**：`harbor.job.running` 事件的 payload 不含 `config`（S14），迁移 009 与运行期构造共同保证。
- **Docker 隔离**：所有托管容器带实例/运行标签，孤儿扫描按 `instance_id` 过滤。
- **价格事实**：历史 Job 成本基于创建时快照；`reported` 不自行计算。
- **前端分层**：domain 不 import API/mock；app 不读 mock fixture；页面不把格式化字符串回传 API。

## 5. 当前锐边与限制

- **真实执行门槛**：Job 必须依赖 Docker + Harbor CLI；CI 冒烟为手动输入（S15）。
- **Stage 6 未闭环**：Dataset 存储位置管理的 S6-06 仍待对抗性审查。
- **运行中进度回退**：`result.json` 未出现前，trial 进度以 `n_tasks × n_attempts` 为计划回退（技术设计 4.2），可能与实际不同。
- **worker 单实例假设**：队列与恢复逻辑面向单进程 WebUI；多实例并发启动未做分布式锁。
- **前端依赖克制**：不引入 React Router / TanStack Query / Radix，新增交互需手写 hooks 并保持分层约束。
- **文件大小门禁**：`webui_dataset_service.py` 498 行、`webui_job_service.py` 491 行接近 500 行上限，后续改动建议先拆模块。
