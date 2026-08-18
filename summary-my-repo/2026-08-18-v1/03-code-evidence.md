# 03 代码证据

本文件证明 `00-overview.md` 与 `02-core-logic.md` 中的核心主张。

## S01

- File: `ornnlab/app.py:29-70`
- Claim: 后端启动先做目录/迁移/恢复/孤儿清理/Operation 对账，然后只挂 WebUI 路由，并在 lifespan 里启动容器代理与队列 worker。

```python
def create_app(settings: Settings | None = None) -> FastAPI:
    active_settings = settings or Settings.from_env()
    active_settings.ensure_dirs()
    sqlite.initialize(active_settings)
    redact_historical_event_payloads(active_settings)
    startup_recovery = RunRecoveryService(active_settings).reconcile_startup()
    startup_docker_cleanup = DockerOrphanService(
        instance_id=active_settings.instance_id
    ).cleanup_orphans(_active_run_ids(active_settings))
    operation_tasks: dict[str, asyncio.Task[None]] = {}
    interrupted_operations = WebUiOperationService(
        active_settings, operation_tasks
    ).reconcile_interrupted()
    dataset_service = WebUiDatasetService(active_settings)
    interrupted_downloads = dataset_service.reconcile_interrupted_downloads()
    container_proxy = ContainerProxyRuntime()
    worker = QueueWorkerService(active_settings, container_proxy)

    @asynccontextmanager
    async def lifespan(app: FastAPI):
        await app.state.container_proxy.start()
        if QueueService(active_settings).queued_count() > 0:
            app.state.worker.start()
        yield
        # cancel leftover operations, close worker and proxy
```

- Interpretation: 恢复发生在 worker 启动前；应用只装配 WebUI 所需状态。同文件第 119 行 `app.include_router(webui.router)` 证明没有第二套产品路由。

## S02

- File: `ornnlab/api/webui.py:30-31,179-201`
- Claim: 唯一产品前缀是 `/api/webui/v1`；创建、取消、恢复、重跑失败任务都走同一包络。

```python
router = APIRouter(prefix="/api/webui/v1", tags=["webui"])
router.include_router(resources_router)

@router.post("/jobs")
async def create_job(payload: CreateJobInput, request: Request) -> dict:
    _require_query(request, set())
    job, operation = await _jobs(request).create_job(payload)
    return _data(request, {"job": job, "operation": operation})

@router.post("/jobs/{job_id}/cancel")
async def cancel_job(job_id: str, request: Request) -> dict:
    return _data(request, {"operation": _jobs(request).cancel_job(job_id)})

@router.post("/jobs/{job_id}/resume")
async def resume_job(job_id: str, request: Request) -> dict:
    return _data(request, {"operation": _jobs(request).resume_job(job_id)})

@router.post("/jobs/{job_id}/rerun-failed")
async def rerun_failed_job(job_id: str, request: Request) -> dict:
    return _data(request, {"operation": _jobs(request).rerun_failed_job(job_id)})
```

- Interpretation: 前端不应再调用 `/api/experiments` 或 `/api/runs`。恢复和重跑是一等路由，不是隐藏 CLI。

## S03

- File: `ornnlab/services/webui_job_service.py:98-184`
- Claim: 创建 Job 必须使用已保存 Agent 和该 Agent 模型集合中的模型，并把 Harbor overrides 与价格快照写入 `webui_job_configs`。

```python
    async def create_job(self, request: CreateJobInput) -> tuple[dict, dict]:
        config = request.config
        agent = self.profiles.resolve_agent(config.agent_name)
        if config.model_name not in agent["models"]:
            raise ValueError("selected model is not configured for this Agent")
        pricing = pricing_snapshot(agent, config.model_name)
        environment = self.profiles.get_environment(config.environment_preset_id)
        # ... ExperimentService.create(..., mode="webui")
        stored = {
            "agent_harness": agent["harness"],
            "harbor_overrides": overrides,
            "model": config.model_name,
            "pricing": pricing,
        }
        if request.run_immediately:
            QueueService(self.settings).enqueue_experiment(created["experiment"]["id"])
            self.worker.start()
            operation = self.operations.complete("run-job", "job", run["id"], "Job queued")
        else:
            operation = self.operations.complete("create-job", "job", run["id"], "Job created")
        return self.get_job(run["id"]), operation
```

- Interpretation: Harness 模板不能直接跑。价格在创建时固化。立即运行只是入队，不是在 HTTP 请求里同步执行 Harbor。

## S04

- File: `ornnlab/services/experiment_service.py:276-423`
- Claim: `_run_one` 是执行内核：构建 Harbor 配置、条件进入 running、托管引擎、取消优先于晚到成功、终态后回收 Docker。

```python
        try:
            result = await self.engine.run(config, runtime_env=proxy_policy.subprocess_env)
        except asyncio.CancelledError:
            if self._is_run_cancelled(run["id"]):
                self.events.append(..., "harbor.job.cancelled", ...)
                return
            await self.failures.mark_interrupted(...)
            return
        finally:
            await proxy_policy.close()
            await cleanup_run_docker_resources(...)
        cursor = conn.execute(
            "UPDATE runs SET status = ?, ..."
            "WHERE id = ? AND status NOT IN ('cancelled', 'failed', 'interrupted')",
            (result["status"], finished, report_path, finished, run["id"]),
        )
        if updated == 0:
            self.events.append(..., "harbor.job.completed_but_cancelled", ...)
        else:
            self.queue.finish(run["id"], result["status"])
```

- Interpretation: 取消是 CAS。Harbor 成功不能覆盖用户取消。Docker 回收在 `finally`，与任务成败无关。

## S05

- File: `ornnlab/services/harbor_engine.py:141-211`
- Claim: 默认引擎是托管 CLI 子进程；`python-api` 是可选路径，且拒绝自动 Docker 代理。

```python
    async def run(self, config: HarborJobConfigView, runtime_env: dict[str, str] | None = None) -> dict:
        if self.mode == "subprocess":
            return await ManagedSubprocessHarborRunner().run(config, extra_env=runtime_env)
        if self.mode == "python-api":
            if runtime_env:
                raise RuntimeError(
                    "automatic Docker proxy inheritance requires the subprocess Harbor engine"
                )
            return await PythonApiHarborRunner().run(config)
```

- Interpretation: 硬取消和宿主代理注入都绑定在 subprocess 上。`capability_snapshot.supports_cancel` 仅在该模式为真。同文件 `HarborConfigBuilder.build` 会把 Docker 环境改写成带 `instance_id`/`run_id` 的 `OwnedDockerEnvironment`。

## S06

- File: `ornnlab/services/harbor_subprocess.py:59-119`
- Claim: Harbor 以独立进程组运行，日志实时镜像；取消时杀进程组并留下清理证据。

```python
                process = await asyncio.create_subprocess_exec(
                    *self.command,
                    "--config",
                    str(runtime_config_path),
                    stdout=asyncio.subprocess.PIPE,
                    stderr=asyncio.subprocess.STDOUT,
                    start_new_session=True,
                    env=_subprocess_env(extra_env),
                )
            except asyncio.CancelledError:
                cleanup = await _terminate_process_group(process, self.terminate_grace_sec)
                atomic_write_text(job_dir / CLEANUP_FILE_NAME, json.dumps(cleanup, ...))
                raise
```

- Interpretation: 这是 v1.0.5 能“取消正在跑的 Job”的根因。Harbor 0.13 没有 `Job.cancel` API。

## S07

- File: `ornnlab/services/worker_service.py:36-40,86-131`
- Claim: worker 按并发上限出队，每个 run 一个可取消的 asyncio task。

```python
    def cancel_run(self, run_id: str) -> bool:
        task = self._active_runs.get(run_id)
        if task is None or task.done():
            return False
        task.cancel()
        return True

            run = dequeue_service.dequeue_next_run(experiment_id)
            # ...
            task = asyncio.create_task(self._execute_run(run), name=run["id"])
            self._active_runs[run["id"]] = task
            pending.add(task)
```

- Interpretation: 取消 Job 的进程内动作就是 cancel 这个 task，从而触发 S06 的进程组终止。默认并发来自 `Settings.worker_max_concurrent`（环境变量，默认 2）。

## S08

- File: `ornnlab/services/queue_service.py:12-38` 与 `ornnlab/storage/sqlite.py:26-42`
- Claim: draft run 入队时同时改 `queue_items` 和 `runs.status`；schema 由 `migrations/*.sql` 按文件名顺序演进到 010。

```python
                conn.execute(
                    "INSERT OR REPLACE INTO queue_items("
                    "run_id, queue_position, state, enqueued_at"
                    ") VALUES (?, ?, ?, ?)",
                    (run["id"], position, "queued", now),
                )
                conn.execute(
                    "UPDATE runs SET status = ?, updated_at = ? WHERE id = ?",
                    ("queued", now, run["id"]),
                )
```

```python
        for migration in sorted(MIGRATIONS_DIR.glob("*.sql")):
            version = migration.stem
            if version in applied:
                continue
            conn.executescript(migration.read_text(encoding="utf-8"))
            conn.execute("INSERT INTO schema_migrations(version) VALUES (?)", (version,))
```

- Interpretation: 队列不是内存结构。新增表必须加迁移文件，不能只改 Python。

## S09

- File: `ornnlab/services/webui_operation_service.py:86-127`
- Claim: 长操作先持久化再丢到 asyncio；同步写也通过 `complete()` 变成已完成 Operation，前端只认这一种写模型。

```python
    def submit(...):
        operation = self.create(...)
        task = asyncio.create_task(self._execute(operation["id"], work), name=operation["id"])
        self.tasks[operation["id"]] = task
        return operation

    def complete(...):
        operation = self.create(...)
        self._set_status(operation["id"], "completed", progress=100, message=message)
        return self.get(operation["id"])

    def cancel(self, operation_id: str) -> dict:
        if operation["status"] in {"completed", "failed", "cancelled"}:
            raise RuntimeError("operation is already terminal")
        task = self.tasks.get(operation_id)
        if task is not None and not task.done():
            task.cancel()
```

- Interpretation: 没有 SSE。重启后进程内 `tasks` 清空，残留 queued/running 会被对账失败，不会自动续跑。

## S10

- File: `ornnlab/services/webui_job_service.py:194-269`
- Claim: Resume / rerun-failed 不走队列 worker，而是 Operation + Harbor 原生 `job resume`。

```python
    def resume_job(self, job_id: str) -> dict:
        if run["status"] not in {"failed", "interrupted"}:
            raise ValueError("only failed or interrupted jobs can be resumed")
        # require job_dir + config.json, clear stale lock
        return self._resume_operation(run, job_path, operation_type="resume-job")

    def rerun_failed_job(self, job_id: str) -> dict:
        if run["status"] not in TERMINAL_STATUSES:
            raise ValueError("only terminal jobs can re-run failed tasks")
        error_types = failed_trial_error_types(job_path)
        return self._resume_operation(..., operation_type="rerun-failed-job",
                                     filter_error_types=error_types)
```

- Interpretation: “恢复”只续未完成 trial；“重跑失败任务”按 error type 过滤。二者都先清理残留、恢复敏感 env、重建 Docker 代理，结束后用 `RunRecoveryService.reconcile_run` 收敛状态。

## S11

- File: `ornnlab/services/webui_job_deletion.py:19-79`
- Claim: 删除只接受终态，先校验产物归属，再在同一事务里删库并删独占文件树。

```python
            if run["status"] not in TERMINAL_STATUSES:
                raise RuntimeError("running or queued jobs must be cancelled before deletion")
            roots = self._artifact_roots(run, delete_experiment)
            self._validate_stored_artifacts(conn, run, roots, experiment_id)
            deleted["runs"] = conn.execute("DELETE FROM runs WHERE id = ?", (job_id,)).rowcount
            if delete_experiment:
                deleted["experiments"] = conn.execute(
                    "DELETE FROM experiments WHERE id = ?", (experiment_id,)
                ).rowcount
            for root in roots:
                _remove_owned_tree(root)
```

- Interpretation: 排行榜没有独立表，删 run 后自然消失。归属不明则整个请求失败，避免半删。

## S12

- File: `ornnlab/services/webui_job_dto.py:22-28` 与 `ornnlab/services/webui_job_progress.py:14-29`
- Claim: 展示层把“全部 trial 已执行”的终态 Job 映射为 `completed`，执行状态与结果质量分离。

```python
    if status in TERMINAL_STATUSES and execution_completed(result, expected_total):
        status = "completed"
```

```python
def execution_completed(result: dict, expected_total: int) -> bool:
    """True when Harbor's native result shows every trial executed.

    Outcome-agnostic: errored/not-passed trials still count as executed...
    """
    return completed >= total
```

- Interpretation: UI 的 Job 状态列语义是执行是否结束，不是得分是否通过。`passed/notPassed/errored` 走 `trial` 结构。

## S13

- File: `frontend/src/api/runtimeClient.ts:7-20` 与 `frontend/vite.config.ts:7-33`
- Claim: 开发默认可 mock，生产必须 API；Vite 把 `/api` 代理到后端，生产 mock 构建直接失败。

```ts
  if (mode === 'api') return createWebUiHttpClient('/api/webui/v1', request)
  if (import.meta.env.PROD) {
    throw new Error('mock data mode is unavailable in production builds')
  }
  return createMockWebUiClient()
```

```ts
  if (command === 'build' && dataMode !== 'api') {
    throw new Error('Production WebUI builds require VITE_ORNNLAB_DATA_MODE=api.')
  }
      proxy: {
        '/api': {
          target: process.env.ORNNLAB_API_TARGET ?? 'http://127.0.0.1:8765',
          changeOrigin: true,
        },
      },
```

- Interpretation: 页面能显示不等于打到了真实后端。验证真实链路必须显式 `VITE_ORNNLAB_DATA_MODE=api`。

## S14

- File: `ornnlab/services/model_pricing.py:15-69`
- Claim: Job 成本使用创建时快照；`reported` 信任 Harbor `cost_usd`，`litellm`/`custom` 按缓存命中分段计费，缺缓存用量时不把成本当成 0。

```python
    if source == "reported":
        return {"modelName": model_name, "source": source}
    # custom copies three rates; litellm reads catalog_pricing(model_name)

    if not snapshot or snapshot.get("source") == "reported":
        return _number_or_none(usage.get("cost_usd"))
    if cached is None:
        if hit_rate != miss_rate:
            logger.warning("Cannot calculate cache-aware model cost without cache usage", ...)
            return None
```

- Interpretation: 后续改 Agent 价格或 LiteLLM 目录不会改写历史 Job。无法估算时返回空，UI 显示 `-`。

## S15

- File: `scripts/test-after-change-web.sh:1-29` 与 `.github/workflows/ci.yml:6-17`
- Claim: 本地全量门禁覆盖 Python、前端、Storybook、launcher 和联调脚本；云端 CI 只手动触发。

```bash
uv run ruff check ornnlab tests/python
uv run pyright
uv run pytest tests/python
# rebrand + line-count + npm pack + frontend typecheck/lint/test/build
# bundle size + mock-build reject + storybook + launcher + test-run-dev-api.sh
git diff --check
```

```yaml
  # Auto CI disabled per user request; only manual dispatch remains.
  workflow_dispatch:
    inputs:
      real_harbor_smoke:
        description: Run opt-in real Harbor Docker smoke tests
```

- Interpretation: 合并前的默认证据是本地脚本，不是 push 自动 CI。真实 Harbor Docker 冒烟必须显式打开。

## S16

- File: `frontend/src/app/App.tsx:38-80`
- Claim: 前端是手写 hash 路由的单页应用，六个一级页面经 `WebUiClient` 装配，没有 React Router。

```ts
const pageKeys = new Set<PageKey>(['jobs', 'datasets', 'agents', 'environments', 'leaderboard', 'system'])

function readRouteFromHash(): RouteState {
  if (hash === 'jobs/new' || hash === 'new-run') {
    return { page: 'jobs', jobView: 'new', ... }
  }
  return {
    page: pageKeys.has(hash as PageKey) ? (hash as PageKey) : 'jobs',
    ...
  }
}

  const dataMode = injectedDataMode ?? readWebUiDataMode()
  const client = useMemo(() => injectedClient ?? createRuntimeWebUiClient(dataMode), ...)
```

- Interpretation: 新增一级导航必须同时改 `pageKeys`、`AppShell` 和契约，不能私自加 Tasks 页。

## S17

- File: `bin/ornnlab.js:12-31,63-67`
- Claim: 公开安装路径是 npm `ornnlab`；无参数即 bootstrap 后启动本地 WebUI，并提供应用级 daemon 子命令。

```js
Usage:
  ornnlab                    Bootstrap if needed, then start the local WebUI demo
  ornnlab dev start          Start the app-level background dev service
  ornnlab dev stop           Stop the app-level background dev service
  ornnlab web [args...]      Start the FastAPI backend from the managed source checkout

async function main() {
  const [command, ...args] = process.argv.slice(2);
  if (!command) {
    await runDev({ setupIfMissing: true });
    return;
  }
```

- Interpretation: Python `ornnlab web` 只起后端。用户安装后的一键入口是 Node launcher。daemon 明确是应用级，不是 systemd/launchd。

## Traceability Check

| 工作流 / 主张 | Snippet |
|---|---|
| 启动装配与唯一路由 | S01, S02 |
| 创建并执行 Job | S03, S04, S05, S06, S07, S08 |
| Operation 轮询 | S09 |
| Resume / rerun-failed | S02, S10 |
| Job 删除 | S11 |
| 两轴状态 | S12 |
| mock/API 与生产守卫 | S13, S16 |
| 价格快照 | S03, S14 |
| 安装 / daemon | S17 |
| 质量门与 CI | S15 |
