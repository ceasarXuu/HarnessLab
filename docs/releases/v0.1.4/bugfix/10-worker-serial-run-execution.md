# BUG-10: Worker 串行执行 run，无法并行

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.2
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: worker_service, experiment_service, sqlite, settings
- Related Links: [README](README.md), [BUG-09](09-worker-recreates-experiment-service-per-run.md), [BUG-03](03-wait-true-blocks-unrelated-experiments.md), [BUG-07](07-event-mirror-quadratic-read-write.md), [BUG-08](08-db-connect-repeated-ensure-dirs-pragma.md)
- Risk Level: High
- Plan Type: Standard
- Phase: 4（Worker 架构，与 BUG-09 联合修复）

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

`QueueWorkerService._run_until_no_queued_runs` 虽然为每个 run 创建了 `asyncio.Task`，但立即 `await` 该 task，导致 run 之间仍是串行执行。一个 experiment 有多个 run 时，无法利用 worker 层并行度缩短总耗时。

## 证据

当前 worker 逻辑：

```python
task = asyncio.create_task(service.execute_dequeued_run(run))
self._active_runs[run["id"]] = task
try:
    await task  # 等当前 run 完成才 dequeue 下一个
finally:
    self._active_runs.pop(run["id"], None)
```

`n_concurrent` 当前只传给 Harbor，用于单个 job 内部 trial 并发；worker 层 run 之间仍完全串行。

## 修复目标

1. 支持有界并行执行多个 run。
2. 默认并发度保守，避免 Docker/SQLite 资源争用。
3. 保留 `cancel_run` 对 active task 的取消能力。
4. 正确消费 task 异常，避免 “Task exception was never retrieved”。
5. 与 BUG-03 的 scoped wait、BUG-09 的执行上下文边界、BUG-08 的 SQLite busy_timeout 协调。

## 配置方案

`Settings` 增加 worker 并发配置：

```python
@dataclass(frozen=True)
class Settings:
    worker_max_concurrent: int = 2
```

从环境变量读取时必须校验：

```python
value = int(os.environ.get("ORNNLAB_WORKER_MAX_CONCURRENT", "2"))
if value < 1:
    raise ValueError("ORNNLAB_WORKER_MAX_CONCURRENT must be >= 1")
```

不要允许 `0` 或负数，否则调度循环可能不执行或进入异常状态。

## Worker 调度方案

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
            pending.remove(task)
            run_id = task.get_name()
            self._active_runs.pop(run_id, None)
            self._consume_task_result(task)

        if len(pending) >= limit:
            done, _ = await asyncio.wait(pending, return_when=asyncio.FIRST_COMPLETED)
            for task in done:
                pending.remove(task)
                run_id = task.get_name()
                self._active_runs.pop(run_id, None)
                self._consume_task_result(task)
            continue

        run = dequeue_service.dequeue_next_run(experiment_id)
        if run is None:
            if pending:
                done, _ = await asyncio.wait(pending, return_when=asyncio.ALL_COMPLETED)
                for task in done:
                    run_id = task.get_name()
                    self._active_runs.pop(run_id, None)
                    self._consume_task_result(task)
            return processed

        processed += 1
        task = asyncio.create_task(self._execute_run(run), name=run["id"])
        self._active_runs[run["id"]] = task
        pending.add(task)
```

## 异常消费

并行 task 完成后必须读取结果：

```python
def _consume_task_result(self, task: asyncio.Task[None]) -> None:
    try:
        task.result()
    except asyncio.CancelledError:
        return
    except Exception as exc:
        self.last_error = f"{type(exc).__name__}: {exc}"
```

原则：

- 单个 run 的业务失败应由 `ExperimentService.execute_dequeued_run` 标记 run failed/interrupted。
- 未被业务层处理的异常要记录到 worker `last_error`，并被测试覆盖。
- 不允许 task 异常无人消费。

## Service 层签名变更

当前 `ExperimentService.dequeue_next_run()` 不接受 experiment scope。若 BUG-10 或 BUG-03 需要 scoped dequeue，必须同步修改签名：

```python
def dequeue_next_run(self, experiment_id: str | None = None) -> dict | None:
    return self.queue.dequeue_next(experiment_id)
```

但 HTTP `wait=true` 不应依赖抢占 scoped drain；BUG-03 应通过等待指定 experiment terminal 实现。

## SQLite 与事件系统要求

- BUG-08 必须为每个 SQLite 连接设置 `busy_timeout=5000`。
- `journal_mode=WAL` 应在初始化阶段设置，而非每次 connect 重复设置。
- BUG-07 的 JSONL mirror append 不应依赖普通文件跨进程原子性假设；DB 是事件 source of truth。

## Docker 资源约束

默认 `worker_max_concurrent=2` 是保守值。并行 run 会并发启动 Harbor/Docker 工作负载。未来可根据 CPU、内存、Docker 资源加入更细的调度策略，但 v0.1.4 只实现简单有界并发。

## Acceptance Criteria（目标，未完成）

- [x] worker 支持有界并行执行 run。
- [x] `worker_max_concurrent` 默认值为 2，可通过 `ORNNLAB_WORKER_MAX_CONCURRENT` 配置。
- [x] `worker_max_concurrent < 1` 会被拒绝并给出清晰错误。
- [x] 并行 task 完成后调用 `task.result()` 或等价逻辑消费异常。
- [x] `cancel_run` 仍能取消 active run task。
- [x] `ExperimentService.dequeue_next_run` 支持可选 `experiment_id` 参数。
- [x] SQLite 连接设置 `busy_timeout=5000`。
- [x] 并行执行时 run 状态、事件和 report 不串扰。
- [x] 现有 worker/experiment/storage 测试无回归。

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | 并行执行 | 2 run，max_concurrent=2 | 两个 run 可同时 running，最终 terminal |
| 正确性验证 | 并发上限 | 4 run，max_concurrent=2 | 同时 running 不超过 2 |
| 配置验证 | 非法并发度 | ORNNLAB_WORKER_MAX_CONCURRENT=0 | 启动时报清晰错误 |
| 异常验证 | task 异常消费 | mock `_execute_run` 抛异常 | `last_error` 记录，无未消费 task 异常 |
| 取消验证 | active run cancel | 运行 fake-slow-cancel 后 cancel | task 被取消，run terminal |
| 回归测试 | 单 run 执行 | 1 run，max_concurrent=2 | status == `completed` |
| 性能验证 | 并行 vs 串行 | 2 个慢 run，max_concurrent=1 vs 2 | 并行耗时明显低于串行 |

## 回滚策略

与 BUG-09 联合回滚。单次代码 commit 可直接 `git revert`。`worker_max_concurrent` 字段有默认值，无 schema 变更。
