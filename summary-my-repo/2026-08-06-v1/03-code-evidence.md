# 代码证据

本文件为 `00-overview.md` 与 `02-core-logic.md` 中的核心论断提供代码证明。所有行号基于当前工作区。

### S01

- File: `ornnlab/app.py:29-70`
- Claim: 后端启动时统一完成迁移、历史脱敏、运行恢复、孤儿容器清理、Operation 对账与 worker 生命周期管理。

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
    container_proxy = ContainerProxyRuntime()
    worker = QueueWorkerService(active_settings, container_proxy)

    @asynccontextmanager
    async def lifespan(app: FastAPI):
        await app.state.container_proxy.start()
        if QueueService(active_settings).queued_count() > 0:
            app.state.worker.start()
        yield
        tasks = list(app.state.operation_tasks.values())
        for task in tasks:
            task.cancel()
        if tasks:
            await asyncio.gather(*tasks, return_exceptions=True)
        await app.state.worker.close()
        await app.state.container_proxy.close()
```

- Interpretation: 启动顺序保证恢复先于 worker 执行，避免崩溃盲区被新执行覆盖；容器代理与 worker 随应用生命周期启停。

### S02

- File: `ornnlab/api/webui.py:25-34`
- Claim: `/api/webui/v1` 是唯一产品 API 前缀，且业务路由经统一 router 聚合。

```python
router = APIRouter(prefix="/api/webui/v1", tags=["webui"])
router.include_router(resources_router)
```

- Interpretation: `app.py` 只 include 这一个 router，旧 experiments/runs/agents 等路由已删除，符合「唯一契约」原则。

### S03

- File: `ornnlab/api/webui.py:466-500`
- Claim: 所有响应使用统一包络 `{data, error, meta}`，且查询参数有白名单校验。

```python
def _data(request: Request, data: object) -> dict:
    return {"data": data, "error": None, "meta": {"requestId": _request_id(request)}}

def _page(request: Request, items: list[dict], cursor: str | None, limit: int) -> dict:
    offset = int(cursor or "0")
    page = items[offset : offset + limit]
    next_cursor = str(offset + limit) if offset + limit < len(items) else None
    ...

def _require_query(request: Request, allowed: set[str]) -> None:
    unsupported = sorted(set(request.query_params) - allowed)
    if unsupported:
        raise ValueError(f"unsupported query parameters: {', '.join(unsupported)}")
```

- Interpretation: 包络与分页在路由层统一，前端契约层（`ApiResponse<T>`）与之一一对应。

### S04

- File: `ornnlab/services/experiment_service.py:276-335`
- Claim: Job 执行主路径依次完成代理策略准备、Harbor 配置构建、产物写入与运行标记。

```python
    async def _run_one(self, run: dict) -> None:
        now = now_iso()
        webui_config = self._webui_run_config(run["id"])
        job_dir = _resolve_job_dir(
            webui_config.get("jobs_dir"), self.settings.experiments_dir / run["id"] / "harbor-job"
        )
        proxy_policy = RuntimeProxyPolicy({}, {}, 0)
        try:
            overrides = webui_config.get("harbor_overrides") or {}
            agent_config = self.agent_configs.config(run["agent_id"], webui_config.get("model"))
            if _uses_docker_environment(overrides):
                proxy_policy = await self.container_proxy.prepare_policy(
                    _explicit_proxy_names(agent_config, overrides),
                    automatic_proxy_allowed=_automatic_proxy_allowed(
                        agent_config, overrides
                    ),
                )
            ...
            config = self.builder.build(
                agent_config,
                run["benchmark_name"],
                run["benchmark_version"],
                run["n_tasks"],
                run["n_attempts"],
                run["n_concurrent"],
                job_dir,
                job_name=webui_config.get("job_name", run["id"]),
                overrides=overrides,
                runtime_container_env_defaults=proxy_policy.container_env_defaults,
                owner_run_id=run["id"],
            )
            snapshot = self.engine.capability_snapshot()
            artifact_paths = self.builder.write_run_artifacts(config, snapshot)
        except Exception as exc:
            await proxy_policy.close()
            await self.failures.mark_failed(run, job_dir, exc)
            return
        if not self._mark_run_running(run, job_dir, config.job_name, now):
            ...
            return
```

- Interpretation: 配置构建失败会先于运行标记落失败态；`_mark_run_running` 的条件 UPDATE 保证取消竞态不会把已取消 run 置为 running。

### S05

- File: `ornnlab/services/harbor_engine.py:25-90`
- Claim: `HarborConfigBuilder.build` 把 Agent/Environment/Dataset 映射为 Harbor `JobConfig` 的视图模型，并对 Docker 环境注入实例/运行归属。

```python
    def build(
        self,
        agent_config: dict,
        benchmark_name: str,
        benchmark_version: str | None,
        n_tasks: int | None,
        n_attempts: int,
        n_concurrent: int,
        jobs_dir: str,
        job_name: str | None = None,
        overrides: dict[str, Any] | None = None,
        runtime_container_env_defaults: dict[str, str] | None = None,
        owner_run_id: str | None = None,
    ) -> HarborJobConfigView:
        overrides = overrides or {}
        normalized_agent = _normalize_agent_view(agent_config)
        dataset_name = (
            f"{benchmark_name}@{benchmark_version}" if benchmark_version else benchmark_name
        )
        effective_job_name = job_name or f"ornnlab-{_slug(dataset_name)}"
        environment = overrides.get("environment", {"type": "docker", "delete": True})
        if _is_docker_environment(overrides):
            environment = _owned_docker_environment(
                environment,
                self.settings.instance_id,
                owner_run_id or effective_job_name,
            )
            environment = _merge_environment_env_defaults(
                environment,
                runtime_container_env_defaults or {},
                normalized_agent.get("env", {}),
            )
```

- Interpretation: `_owned_docker_environment` 注入实例/运行标签，为孤儿回收与所有权清理提供依据。

### S06

- File: `ornnlab/services/harbor_subprocess.py:40-108`
- Claim: 默认引擎以托管子进程运行 `harbor run --config`，stdout 镜像到 `job.log`，取消时对进程组优雅终止并记录清理证据。

```python
    async def run(
        self,
        config: HarborJobConfigView,
        extra_env: dict[str, str] | None = None,
    ) -> dict:
        job_dir = Path(config.jobs_dir)
        job_dir.mkdir(parents=True, exist_ok=True)
        log_path = job_dir / JOB_LOG_NAME
        config_path = job_dir / CONFIG_FILE_NAME
        executable = self.command[0]
        with _runtime_config(config_path, extra_env) as runtime_config_path:
            try:
                process = await asyncio.create_subprocess_exec(
                    *self.command,
                    "--config",
                    str(runtime_config_path),
                    stdout=asyncio.subprocess.PIPE,
                    stderr=asyncio.subprocess.STDOUT,
                    start_new_session=True,
                    env=_subprocess_env(extra_env),
                )
            ...
            output_task = asyncio.create_task(_mirror_stdout(process, log_path))
            try:
                return_code = await process.wait()
                output = await output_task
            except asyncio.CancelledError:
                cleanup = await _terminate_process_group(process, self.terminate_grace_sec)
                cleanup["reason"] = "task_cancelled"
                cleanup["command"] = self.command
                atomic_write_text(
                    job_dir / CLEANUP_FILE_NAME,
                    json.dumps(cleanup, indent=2, sort_keys=True),
                )
                output_task.cancel()
                await _ignore_cancelled(output_task)
                raise
        if return_code != 0:
            raise RuntimeError(f"harbor subprocess exited with {return_code}: {output[-400:]}")
        result_path = resolve_harbor_result_path(job_dir, config.job_name)
        result = _read_or_write_result(result_path, return_code)
```

- Interpretation: `start_new_session=True` 使进程组终止可覆盖 Harbor 派生的子进程；`harbor.cleanup.json` 为取消审计留痕。

### S07

- File: `ornnlab/services/worker_service.py:86-131`
- Claim: 队列 worker 按并发上限调度 run，并用 task 名称跟踪每个 run。

```python
    async def _run_until_no_queued_runs(
        self,
        experiment_id: str | None = None,
        max_concurrent: int | None = None,
    ) -> int:
        processed = 0
        limit = max_concurrent or self.settings.worker_max_concurrent
        pending: set[asyncio.Task[None]] = set()
        dequeue_service = ExperimentService(self.settings)
        while True:
            done = {task for task in pending if task.done()}
            for task in done:
                pending.discard(task)
                run_id = task.get_name()
                self._active_runs.pop(run_id, None)
                self._consume_task_result(task)
            if len(pending) >= limit:
                done, _ = await asyncio.wait(pending, return_when=asyncio.FIRST_COMPLETED)
                ...
            run = dequeue_service.dequeue_next_run(experiment_id)
            if run is None:
                ...
                return processed
            processed += 1
            task = asyncio.create_task(self._execute_run(run), name=run["id"])
            self._active_runs[run["id"]] = task
            pending.add(task)
```

- Interpretation: `_active_runs` 与 task name 绑定使 `cancel_run(run_id)` 能精确取消单个运行。

### S08

- File: `ornnlab/services/queue_service.py:41-73`
- Claim: 出队在一个事务内完成 `queue_items` 与 `runs`/`experiments` 的状态迁移。

```python
    def dequeue_next(self, experiment_id: str | None = None) -> dict | None:
        now = now_iso()
        with sqlite.connect(self.settings) as conn:
            ...
            queued = sqlite.rows(
                conn,
                "SELECT r.* FROM queue_items q JOIN runs r ON r.id = q.run_id "
                f"WHERE q.state = 'queued' {experiment_filter} "
                "ORDER BY q.queue_position LIMIT 1",
                params,
            )
            if not queued:
                return None
            run = queued[0]
            conn.execute(
                "UPDATE queue_items SET state = ?, dequeued_at = ? WHERE run_id = ?",
                ("running", now, run["id"]),
            )
            conn.execute(
                "UPDATE runs SET status = ?, updated_at = ? WHERE id = ?",
                ("running", now, run["id"]),
            )
            ...
            run["status"] = "running"
            return run
```

- Interpretation: SQLite 事务保证「出队即 running」，多 worker 场景也不会重复领取同一 run。

### S09

- File: `ornnlab/services/webui_operation_service.py:85-174`
- Claim: 异步 Operation 先持久化再执行，进度/失败/取消均落库，重启时被对账为中断。

```python
    def submit(
        self, operation_type: str, resource_type: str, resource_id: str | None, work: OperationWork
    ) -> dict:
        operation = self.create(operation_type, resource_type, resource_id)
        ...
        task = asyncio.create_task(self._execute(operation["id"], work), name=operation["id"])
        self.tasks[operation["id"]] = task
        task.add_done_callback(lambda _: self.tasks.pop(operation["id"], None))
        return operation

    async def _execute(self, operation_id: str, work: OperationWork) -> None:
        self._set_status(operation_id, "running", progress=0, message="Running")
        def progress(value: int | None, message: str | None = None) -> None:
            self._set_progress(operation_id, value, message)
        try:
            await work(progress)
        except asyncio.CancelledError:
            self._set_status(operation_id, "cancelled", message="Cancelled")
            raise
        except Exception as exc:
            self._set_status(
                operation_id,
                "failed",
                message=str(exc),
                error=("OPERATION_FAILED", str(exc), {"exception": type(exc).__name__}),
            )
        else:
            self._set_status(operation_id, "completed", progress=100, message="Completed")
```

- Interpretation: 先 `create`（queued）再建 task，保证前端立即可轮询；`reconcile_interrupted` 与 `cancel` 配合形成完整生命周期。

### S10

- File: `ornnlab/services/recovery_service.py:27-71`
- Claim: 启动恢复覆盖 running 残留与队列孤儿，有结果则恢复终态，否则标记 interrupted。

```python
    def reconcile_startup(self) -> dict[str, int]:
        running = self._running_runs()
        orphaned = self._orphaned_queue_items()
        counts = {"recovered": 0, "interrupted": 0}
        experiment_ids: set[str] = set()
        for run in [*running, *orphaned]:
            experiment_ids.add(run["experiment_id"])
            decision = self._reconcile_run(run)
            counts[decision] += 1
        for experiment_id in experiment_ids:
            self._update_experiment_status(experiment_id)
        return counts

    def _orphaned_queue_items(self) -> list[dict]:
        with sqlite.connect(self.settings) as conn:
            return sqlite.rows(
                conn,
                "SELECT r.* FROM queue_items q JOIN runs r ON r.id = q.run_id "
                "WHERE q.state = 'running' "
                "AND r.status NOT IN "
                "('running', 'completed', 'failed', 'cancelled', 'interrupted') "
                "ORDER BY q.dequeued_at, r.id",
            )
```

- Interpretation: 同时处理「run 说 running」与「队列说 running」两类残留，避免崩溃后卡死状态。

### S11

- File: `ornnlab/services/webui_job_deletion.py:19-83`
- Claim: Job 删除只接受终态，并在同一事务内按依赖顺序清理记录与产物。

```python
    def delete(self, job_id: str) -> dict[str, object]:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(conn, "SELECT runs.*, webui_job_configs.config_json FROM runs ...")
            if not rows:
                raise KeyError(job_id)
            run = rows[0]
            if run["status"] not in TERMINAL_STATUSES:
                raise RuntimeError("running or queued jobs must be cancelled before deletion")
            ...
            roots = self._artifact_roots(run, delete_experiment)
            self._validate_stored_artifacts(conn, run, roots, experiment_id)
            deleted = {
                "operations": conn.execute("DELETE FROM webui_operations WHERE resource_type = 'job' AND resource_id = ?", (job_id,)).rowcount,
                "events": conn.execute("DELETE FROM experiment_events WHERE aggregate_id = ?", (job_id,)).rowcount,
                "jobConfigs": conn.execute("DELETE FROM webui_job_configs WHERE run_id = ?", (job_id,)).rowcount,
                "queueItems": conn.execute("DELETE FROM queue_items WHERE run_id = ?", (job_id,)).rowcount,
            }
            deleted["runs"] = conn.execute("DELETE FROM runs WHERE id = ?", (job_id,)).rowcount
            if delete_experiment:
                ...
            for root in roots:
                _remove_owned_tree(root)
        logger.info("Job deletion completed job_id=%s deleted=%s", job_id, deleted)
        return {"deletedJobId": job_id}
```

- Interpretation: `_remove_owned_tree` 在事务提交前执行，文件删除失败会使整个数据库事务回滚，保证记录与文件原子删除（删除不可恢复，故先校验归属）。

### S12

- File: `ornnlab/services/model_pricing.py:15-70`
- Claim: 价格快照区分 reported/custom/litellm 三种来源，成本计算对 cache hit/miss 分别计费。

```python
def pricing_snapshot(agent: dict[str, Any], model_name: str) -> dict[str, Any]:
    configured = next(
        (
            item
            for item in agent.get("modelPricing", [])
            if isinstance(item, dict) and item.get("modelName") == model_name
        ),
        {"modelName": model_name, "source": "reported"},
    )
    source = configured.get("source", "reported")
    if source == "reported":
        return {"modelName": model_name, "source": source}
    if source == "custom":
        return {
            "modelName": model_name,
            "source": source,
            **{field: float(configured[field]) for field in RATE_FIELDS},
        }
    if source != "litellm":
        raise ValueError(f"unsupported model pricing source: {source}")
    return catalog_pricing(model_name)

def calculate_cost(usage: dict[str, Any], snapshot: dict[str, Any] | None) -> float | None:
    if not snapshot or snapshot.get("source") == "reported":
        return _number_or_none(usage.get("cost_usd"))
    ...
    cached = min(max(cached, 0.0), total_input)
    uncached = total_input - cached
    return (
        uncached * miss_rate + cached * hit_rate + output * output_rate
    ) / 1_000_000
```

- Interpretation: `reported` 信任 Harbor 上报的成本；自定义费率按缓存命中/未命中分开计算，保证历史 Job 成本可复算。

### S13

- File: `frontend/src/api/runtimeClient.ts:1-20`
- Claim: 前端以 `VITE_ORNNLAB_DATA_MODE` 选择 HTTP 或 mock client，生产构建默认 API。

```ts
export function createRuntimeWebUiClient(
  mode: WebUiDataMode = readWebUiDataMode(),
  request: typeof fetch = fetch,
): WebUiClient {
  return mode === 'api' ? createWebUiHttpClient('/api/webui/v1', request) : createMockWebUiClient()
}

export function readWebUiDataMode(): WebUiDataMode {
  return resolveWebUiDataMode(import.meta.env.VITE_ORNNLAB_DATA_MODE, import.meta.env.PROD ? 'api' : 'mock')
}
```

- Interpretation: 开发 `npm run dev` 默认 mock，`run_dev.sh` 与生产构建默认 API；`resolveWebUiDataMode` 拒绝非法值，杜绝静默回退。

### S14

- File: `ornnlab/storage/migrations/009_redact_harbor_running_events.sql:1-6`
- Claim: 迁移层保证历史 `harbor.job.running` 事件不包含敏感 `config`。

```sql
UPDATE experiment_events
SET payload_json = json_remove(payload_json, '$.config')
WHERE event_type = 'harbor.job.running'
  AND json_valid(payload_json)
  AND json_type(payload_json, '$.config') IS NOT NULL;
```

- Interpretation: 新事件在 `harbor_event_payloads.harbor_running_event_payload` 构造时已排除 `config`，此迁移回填历史数据，是事件脱敏的持久保证。

### S15

- File: `.github/workflows/ci.yml:19-48`
- Claim: CI 为手动触发（自动触发被禁用），python-web 门禁覆盖 lint、类型、测试、品牌守卫与行数限制。

```yaml
jobs:
  python-web:
    name: Python Web Gate (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    strategy:
      fail-fast: false
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v4
        with:
          fetch-depth: 0
      - uses: astral-sh/setup-uv@v5
        with:
          enable-cache: true
      - name: Install Python dependencies
        run: uv sync --group dev
      - name: Ruff
        run: uv run ruff check ornnlab tests/python
      - name: Pyright
        run: uv run pyright
      - name: Pytest
        run: uv run pytest tests/python
      - name: OrnnLab rebrand guard
        run: uv run python scripts/verify-ornnlab-rebrand.py
      - name: Python line-count gate
        run: uv run python scripts/check-python-file-length.py ornnlab
```

- Interpretation: 三平台矩阵 + rebrand 守卫 + 行数门禁，与本地 `scripts/test-after-change-web.sh` 保持一致；真实 Harbor 冒烟仅在 `workflow_dispatch` 且输入开启时运行。
