# BUG-03: wait=true 等待无关 experiment 的 run

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.1
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: experiments API, worker_service
- Related Links: [README](README.md), [BUG-09](09-worker-recreates-experiment-service-per-run.md), [BUG-10](10-worker-serial-run-execution.md)
- Risk Level: Medium
- Plan Type: Lightweight
- Phase: 4（依赖 worker 架构定型）

## 问题描述

API `POST /api/experiments/{experiment_id}/run?wait=true` 的语义是等待**当前
experiment** 的所有 run 完成。但 `QueueWorkerService.wait_until_idle` 会等待
**全局队列**中所有 run 完成，包括其他 experiment 的 run。

如果 experiment A 有大量 queued run，此时对 experiment B 发起 `wait=true`，
B 的请求会阻塞直到 A 的所有 run 执行完毕才返回，造成不必要的延迟和混淆。

## 证据

文件: `ornnlab/api/experiments.py` 第 33-44 行

```python
@router.post("/{experiment_id}/run")
async def run_experiment(experiment_id, request, wait: bool = False):
    service = ExperimentService(request.app.state.settings)
    service.enqueue(experiment_id)
    worker = request.app.state.worker
    if wait:
        await worker.wait_until_idle()   # ← 等待全局队列空闲
    else:
        worker.start()
    return service.get(experiment_id)
```

文件: `ornnlab/services/worker_service.py` 第 52-62 行

```python
async def _run_until_no_queued_runs(self) -> int:
    while True:
        service = ExperimentService(self.settings)
        run = service.dequeue_next_run()      # ← 不带 experiment_id 过滤
        if run is None:
            return processed
        # ...
```

`dequeue_next_run()` 调用 `QueueService.dequeue_next()` 时 `experiment_id=None`，
会 dequeue 全局队列中任意 experiment 的 run。

## 修复方案

为 `wait_until_idle` 增加 experiment 作用域过滤。依赖 BUG-10 的并行 worker
架构（`_run_until_no_queued_runs` 已支持 `experiment_id` 参数）：

```python
# worker_service.py
async def wait_until_idle(self, experiment_id: str | None = None) -> None:
    task = self._task
    if task is not None and not task.done() and task is not asyncio.current_task():
        await task
        return
    await self._drain(experiment_id)

async def _drain(self, experiment_id: str | None = None) -> None:
    async with self._loop_lock():
        event_service = EventService(self.settings)
        event_service.append("queue", "queue", "queue.worker_started", {})
        processed = await self._run_until_no_queued_runs(experiment_id=experiment_id)
        event_service.append("queue", "queue", "queue.worker_idle", {"processed": processed})
```

```python
# experiments.py
if wait:
    await worker.wait_until_idle(experiment_id)  # 只等当前 experiment
```

## 非目标

- 不改变后台 worker（`worker.start()`）的行为：后台 worker 仍然处理全局队列
- 不改变 `dequeue_next` 的默认行为：不传 experiment_id 时仍处理全局队列

## 竞争分析

`wait_until_idle(experiment_id)` 过滤后只处理该 experiment 的 run。但后台
worker 可能同时在处理全局队列。两个 drain 循环通过 `_loop_lock` 互斥，不会
同时 dequeue。`wait_until_idle` 获取 lock 后会处理当前 experiment 的 queued
run，后台 worker 如果已经持有 lock，则 `wait_until_idle` 会等待其完成后
再开始。

## 验收标准

- [x] `wait_until_idle(experiment_id)` 只处理指定 experiment 的 run
- [x] `wait_until_idle()` 无参数时仍处理全局队列（向后兼容）
- [x] 后台 worker 行为不变
- [x] 现有测试全部通过，无回归

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | wait 只等当前 experiment | 创建 A（2 run）+ B（1 run），B run?wait=true | B 返回时 A 的 run 可能仍在运行 |
| 回归测试 | wait 全局行为 | 1 experiment，run?wait=true | status == "completed" |
| 回归测试 | 后台 worker | 1 experiment，run（无 wait）+ 随后 check | 最终全部 completed |

## 回滚策略

单次 commit，`git revert` 即可。API 接口不变，仅行为优化。
