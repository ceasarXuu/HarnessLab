# 核心逻辑走查

## 1. 核心文件表

| 文件 | 角色 | 输入 | 输出 | 证据 |
|---|---|---|---|---|
| `ornnlab/app.py` | 应用装配与生命周期 | `Settings` | FastAPI app、启动恢复计数 | S01 |
| `ornnlab/api/webui.py` | 唯一 API 入口 | HTTP + Pydantic | `{data,error,meta}` 包络 | S02 |
| `ornnlab/services/webui_job_service.py` | Job 门面 | `CreateJobInput`、job id | Job DTO + Operation | S03, S10 |
| `ornnlab/services/experiment_service.py` | 执行主路径 | 出队 run、WebUI config | Harbor 产物 + `runs` 状态 | S04 |
| `ornnlab/services/harbor_engine.py` | 配置构建与引擎分发 | Agent/overrides/jobs_dir | `harbor.config.json` + runner 结果 | S05 |
| `ornnlab/services/harbor_subprocess.py` | 托管 Harbor CLI | config + runtime env | `job.log`、`result.json`、清理证据 | S06 |
| `ornnlab/services/worker_service.py` | 后台队列调度 | 队列 + 并发上限 | 并发 `asyncio.Task` | S07 |
| `ornnlab/services/queue_service.py` | SQLite 队列状态机 | experiment/run id | `draft→queued→running` | S08 |
| `ornnlab/storage/sqlite.py` | 连接与迁移 | `Settings` | WAL SQLite + schema 010 | S08 |
| `ornnlab/services/webui_operation_service.py` | 异步 Operation | type/resource/work | 可轮询 Operation DTO | S09 |
| `ornnlab/services/recovery_service.py` | 崩溃/重启恢复 | running/orphan rows | recovered / interrupted | S01 |
| `ornnlab/services/webui_job_deletion.py` | 终态 Job 删除 | job id | 记录 + 独占产物消失 | S11 |
| `ornnlab/services/webui_job_dto.py` | Job 展示映射 | run 行 + Harbor result | 两轴 Job DTO | S12 |
| `ornnlab/services/model_pricing.py` | 价格快照与成本 | Agent + Harbor tokens | 不可变 snapshot / `costUsd` | S14 |
| `frontend/src/api/runtimeClient.ts` | 前端模式选择 | `VITE_ORNNLAB_DATA_MODE` | HTTP 或 mock client | S13 |
| `frontend/src/app/App.tsx` | hash 路由与资源装配 | WebUiClient | 六个一级页 | S16 |
| `frontend/vite.config.ts` | dev proxy 与生产守卫 | env | `/api` → 8765；禁 mock build | S13 |
| `bin/ornnlab.js` | npm 安装与 daemon 入口 | argv | setup / dev / web | S17 |
| `scripts/test-after-change-web.sh` | 本地全量门禁 | 仓库 | ruff/pyright/pytest/前端/launcher | S15 |
| `.github/workflows/ci.yml` | 云端三平台门 | `workflow_dispatch` | Python / Frontend / npm / 可选 Harbor | S15 |

改错这些文件的后果：

- 动 `app.py` 路由注册或启动顺序，会破坏唯一契约或让 worker 覆盖恢复结果。
- 动 `_run_one` 的条件 UPDATE / Docker cleanup，会丢取消语义或泄漏容器。
- 动 `runtimeClient` / Vite 守卫，会让生产页面静默回到 mock。
- 动删除归属校验，会误删共享 `jobsDir` 或其他 Job。

## 2. 端到端工作流

### 2.1 安装、开发启动与质量门

1. 用户 `npm install -g ornnlab` 后执行 `ornnlab`：无参数时先 bootstrap 再 `runDev`（S17）。
2. launcher 检查 git/uv/node/npm，可选 Docker，把源码放到 `~/.ornnlab/launcher/source`（仓库内开发则用 CWD）。
3. 后端 `uv run ornnlab web` → `create_app`（S01）→ `127.0.0.1:8765`。
4. 前端 Vite 默认 mock；`run_dev.sh` / daemon 设 `VITE_ORNNLAB_DATA_MODE=api` 并 proxy `/api`（S13）。
5. 健康探针是 `GET /api/webui/v1/system/live` 或 `/system/health`，不是旧 `/api/system/status`。
6. 变更后本地门禁是 `scripts/test-after-change-web.sh`（S15）。

### 2.2 创建并执行一个 Job

1. 前端 `NewRunPage` → `runDraftToCreateJobRequest` → `POST /api/webui/v1/jobs`（S02）。
2. `WebUiJobService.create_job`：解析已保存 Agent；`model_name` 必须属于该 Agent 的模型集合；打价格快照；解析 Environment；必要时登记 Dataset 下载目录（S03, S14）。
3. `ExperimentService.create(..., mode="webui")` 写一个 experiment + 一个 draft run。
4. `webui_job_configs` 保存 `harbor_overrides`、jobs_dir、模型、pricing；`leaderboard_eligible` 在 skip verifier 时强制 false。
5. `run_immediately` 时 `enqueue_experiment`（draft → queued）并 `worker.start()`（S08, S07）。
6. worker 按 `queue_position` 出队，默认最多 2 个并发（`ORNNLAB_WORKER_MAX_CONCURRENT`）。
7. `_run_one`（S04）：
   - 编译 Agent → Harbor `AgentConfig`，所选模型覆盖 `model_name`。
   - Docker 环境：准备 `ContainerProxyRuntime` policy，并把 environment 改写成 `OwnedDockerEnvironment`（S05）。
   - 原子写 `harbor.config.json` / `harbor.capability.json`。
   - `_mark_run_running` 条件更新；已取消则不再启动 Harbor。
   - `harbor.job.running` 事件只带脱敏摘要，不含完整 config。
   - `HarborEngine.run` 默认走 subprocess（S05, S06）。
8. 子进程 `start_new_session=True`，stdout 镜像到 `job.log`，写 PID sidecar；取消时 SIGTERM 进程组再 SIGKILL，并写 `harbor.cleanup.json`（S06）。
9. 读当前 Job 独占的 `result.json`（禁止回退共享 `jobsDir/result.json`）→ 写报告 → 条件 UPDATE `runs`（S04）→ `queue.finish` → 回收带标签 Docker 资源。
10. `job_dto` 聚合 trial 进度、成本、秒表、`canResume`；若 Harbor 显示全部 trial 已执行，终态展示为 `completed`（S12）。

### 2.3 异步 Operation

`submit` 先落 `queued` 再创建 `asyncio.Task`；`complete` 用于同步写操作，保持前端状态模型一致（S09）。

典型异步：Dataset 下载/移动/同步、Job resume/rerun-failed、系统更新/重启/缓存清理。前端 `useOperation` 轮询 `GET /operations/{id}`。服务重启时，无进程内 task 的 queued/running 项对账为 `OPERATION_INTERRUPTED`。

### 2.4 恢复、重跑与取消

- **取消 Job**：终态拒绝；`cancel_run` 持久化 cancelled，再 `worker.cancel_run` 取消 asyncio task，子进程组被杀掉（S03, S06, S07）。
- **Resume**：仅 `failed|interrupted`，且 Harbor job 目录存在可续跑 trial。走 Operation + `harbor job resume --job-path`，会重建 Docker 代理、恢复敏感 env、清陈旧 lock（S10）。
- **Rerun-failed**：终态 Job，按失败 trial 的 error type 加 `--filter-error-type`。
- **启动恢复**：running 且有 `result.json` → 按结果 recovered；否则 `interrupted` / `stale_running_without_result`（S01）。

### 2.5 Job 删除

`WebUiJobDeletionService.delete`（S11）：

1. 拒绝 queued/running。
2. 计算产物根：`experiments/<run_id>`、无其他 run 时的 `experiments/<experiment_id>`、`jobsDir/<harbor_job_name>` 单层子目录。
3. 校验 `result_path` / `report_path` / 事件镜像都在这些根下。
4. 同一事务删除 operations、events、configs、queue_items、runs，必要时 experiments。
5. 文件删除在提交前执行；失败回滚数据库。共享 `jobsDir`、Dataset、Agent、Environment 永不随删。

## 3. 重要控制流

### 3.1 启动顺序（不可颠倒）

`ensure_dirs` → migrate → redact events → recover runs → docker orphan cleanup（跳过仍 running 的 run id）→ interrupt leftover operations/downloads → lifespan 才启动 proxy/worker。先起 worker 会和恢复抢写状态。

### 3.2 两轴状态

- **执行状态**：`draft → queued → running → {completed, failed, cancelled, interrupted}`。
- **结果质量**：`total / passed / notPassed / errored`。全部 trial 有终态结果时，Job 执行态是 completed，即使得分是 0 或有 errored trial（S12）。
- `failed` 应表示环境/setup/引擎级失败，而不是“有失败任务”。DTO remap 用来对齐 Harbor 语义；底层部分 status 函数仍可能把 errored trials 写成 failed，这是已知锐边。

### 3.3 前端数据模式

`readWebUiDataMode`：显式 `api|mock`，非法值直接抛错；生产默认 `api`，开发默认 `mock`（S13）。`App.tsx` 只经 hooks 装配六个一级页（S16）。API 失败展示错误状态，不回退 seed。

### 3.4 Dataset 存储

- `managed`：用户选父目录，OrnnLab 建 `name@version` 子目录并写 `.ornnlab-dataset.json`。只有带标记的目录可移动/删除。
- `external`：只登记用户原目录，`registry_url='local'`（迁移 010 回填）。移除登记不删用户文件。
- 创建 Job 时会自动登记数据集并指向管理下载目录，避免“本地已有副本但下拉仍显示未下载”。

## 4. 关键不变量与假设

- 单进程 WebUI：队列、Operation、daemon 都不是多实例分布式锁。
- Harbor 是执行权威：缺失的 trial 字段保持空，不编造百分比或验证器内部字段。
- 默认引擎是 `subprocess`；`ORNNLAB_HARBOR_ENGINE=python-api` 是不完整备选路径。
- `instance_id` 随机且稳定，不从路径、主机名或网桥推断。
- 事件写入统一脱敏；`harbor.job.running` 不含完整 config。
- 前端不引入 React Router / TanStack Query / Radix；新交互要手写 hooks 并保持分层。

## 5. 当前锐边与限制

- Resume Operation 与普通 worker 执行的进程组/取消语义不完全一致。
- `python-api` 引擎拒绝自动 Docker 代理，也不能硬取消。
- Windows 上 `run_dev.sh` 的完整进程树回归被跳过；产品级路径是 `ornnlab dev` daemon。
- `WEB_TEST_REGISTRY.toml` 有失效文件指针，不能当强制门禁读。
- `observability/`、`LeaderboardService`、`TemplateService` 是遗留内部面，不要误当成产品 API。
- 无 Docker 时不能真实跑 Job；CI 真实 Harbor 冒烟需手动打开。
