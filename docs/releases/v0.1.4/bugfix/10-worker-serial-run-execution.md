# BUG-10: Worker 串行执行 run，无法并行

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.1
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: worker_service, experiment_service, sqlite
- Related Links: [README](README.md), [BUG-09](09-worker-recreates-experiment-service-per-run.md), [BUG-03](03-wait-true-blocks-unrelated-experiments.md), [BUG-07](07-event-mirror-quadratic-read-write.md)
- Risk Level: High
- Plan Type: Standard
- Phase: 4（Worker 架构，与 BUG-09 联合修复）

## 问题描述

`QueueWorkerService._run_until_no_queued_runs` 虽然为每个 run 创建了
`asyncio.Task`，但立即 `await` 该 task，导致 run 之间是串行执行的。一个 experiment
有多个 run 时，只能一个一个跑，无法利用 `n_concurrent` 实现并行。

## 证据

文件: `ornnlab/services/worker_service.py` 第 53-62 行

```python
async def _run_until_no_queued_runs(self) -> int:
    processed = 0
    while True:
        service = ExperimentService(self.settings)
        run = service.dequeue_next_run()
        if run is None:
            return processed
        processed += 1
        task = asyncio.create_task(service.execute_dequeued_run(run))
        self._active_runs[run["id"]] = task
        try:
            await task            # ← 等当前 run 完成才 dequeue 下一个
        finally:
            self._active_runs.pop(run["id"], None)
```

`n_concurrent` 字段只传给 Harbor（单个 job 内的并行 trial），但 worker 层面
run 之间是完全串行的。

影响场景：
- 1 个 experiment，2 个 agent × 3 个 benchmark = 6 个 run
- 当前：6 个 run 串行，假设每个 3 分钟，总计 18 分钟
- 理想：2 个并行（受 max_concurrent 限制），总计 9 分钟

## 修复方案

引入有界并发控制，允许多个 run 同时执行。完整方案见
[BUG-09](09-worker-recreates-experiment-service-per-run.md) 联合修复：

```python
async def _execute_run(self, run: dict) -> None:
    """每个 run 独立的执行上下文"""
    service = ExperimentService(self.settings)
    await service.execute_dequeued_run(run)

async def _run_until_no_queued_runs(
    self,
    max_concurrent: int | None = None,
    experiment_id: str | None = None,
) -> int:
    processed = 0
    if max_concurrent is None:
        max_concurrent = self.settings.worker_max_concurrent  # 从 settings 读取
    pending: set[asyncio.Task] = set()
    dequeue_service = ExperimentService(self.settings)

    while True:
        done = {t for t in pending if t.done()}
        for t in done:
            pending.discard(t)
            self._active_runs.pop(t.get_name(), None)

        if len(pending) >= max_concurrent:
            await asyncio.wait(pending, return_when=asyncio.FIRST_COMPLETED)
            continue

        run = dequeue_service.dequeue_next_run(experiment_id)
        if run is None:
            if pending:
                await asyncio.wait(pending, return_when=asyncio.ALL_COMPLETED)
            return processed

        processed += 1
        task = asyncio.create_task(self._execute_run(run), name=run["id"])
        self._active_runs[run["id"]] = task
        pending.add(task)
```

需要在 `Settings` 中添加 `worker_max_concurrent` 配置：

```python
@dataclass(frozen=True)
class Settings:
    # ... 现有字段 ...
    worker_max_concurrent: int = 2

    @classmethod
    def from_env(cls) -> Settings:
        # ...
        worker_max_concurrent = int(os.environ.get("ORNNLAB_WORKER_MAX_CONCURRENT", "2"))
        return cls(home=home, worker_max_concurrent=worker_max_concurrent)
```

## 并发安全评估

### SQLite 写竞争

WAL 模式下读写不互斥，但多个写事务会串行化。高并行度下可能产生
`database is locked` 错误。

缓解措施：
- SQLite 默认 `busy_timeout` 为 0，需要设置为 5000ms：

```python
def connect(settings: Settings) -> sqlite3.Connection:
    # ...
    conn.execute("PRAGMA busy_timeout = 5000")  # 新增
    return conn
```

- `max_concurrent=2` 时写竞争概率低，每个 run 的写操作集中在状态变更（~5 次），
  5000ms busy_timeout 足以覆盖。

### Docker 资源竞争

并行 run 意味着多个 Docker 容器同时运行。需要评估：
- 内存：每个 harbor job 的容器内存占用，建议 max_concurrent 不超过 CPU 核数
- 磁盘：多个 job_dir 并发写入不同目录，无冲突
- 网络：harbor 拉取镜像可能并发，Docker 自身有镜像层缓存

### 与 BUG-07 的交互

并行执行后，多个 run 可能并发追加同一 experiment 的 JSONL 事件镜像文件。
BUG-07 的 append 修复需要确认并发安全性。详见
[BUG-07](07-event-mirror-quadratic-read-write.md)。

## 验收标准

- [x] worker 支持有界并行执行（max_concurrent 可配置）
- [x] `max_concurrent` 默认值 2，可通过 `ORNNLAB_WORKER_MAX_CONCURRENT` 环境变量配置
- [x] SQLite `busy_timeout` 设置为 5000ms，防止并行写竞争
- [x] 并行执行时 run 状态正确（无串扰、无丢失）
- [x] 现有测试全部通过，无回归

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | 并行执行 | 2 benchmark × 1 agent，max_concurrent=2 | 两个 run 同时 running，全部 completed |
| 正确性验证 | 并发上限 | 4 run，max_concurrent=2 | 同时 running 的 run 不超过 2 |
| 回归测试 | 单 run 执行 | 1 run，max_concurrent=2 | status == "completed" |
| 回归测试 | batch 执行 | 3 benchmark × 1 agent | 全部 completed |
| 性能验证 | 并行 vs 串行 | 2 run（fake-slow-cancel），max_concurrent=1 vs 2 | 并行总时间 < 串行总时间 × 0.75 |

## 回滚策略

与 BUG-09 联合回滚。单次 commit，`git revert` 即可。Settings 中新增的
`worker_max_concurrent` 字段有默认值，回滚后不会影响现有配置。
