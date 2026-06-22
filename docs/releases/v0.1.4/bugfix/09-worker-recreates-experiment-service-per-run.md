# BUG-09: Worker 每轮重建 ExperimentService

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.1
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: worker_service, experiment_service
- Related Links: [README](README.md), [BUG-10](10-worker-serial-run-execution.md), [BUG-03](03-wait-true-blocks-unrelated-experiments.md)
- Risk Level: Medium
- Plan Type: Standard
- Phase: 4（Worker 架构，与 BUG-10 联合修复）

## 问题描述

`QueueWorkerService._run_until_no_queued_runs` 在每次循环中创建全新的
`ExperimentService` 实例，连带创建 8 个子服务对象。虽然这些对象本身是轻量的，
但在批量执行场景下属于不必要的重复初始化开销。

## 与 BUG-10 的关系

本修复与 [BUG-10](10-worker-serial-run-execution.md) 存在架构冲突，必须联合设计。

- BUG-09 倾向：复用单个 ExperimentService 实例
- BUG-10 倾向：并行执行需要每个 run 独立实例以避免状态竞争

**联合决策**：并行执行时每个 run 创建独立 ExperimentService 实例（满足 BUG-10
并行约束），但在同一 run 内部不重复创建（消除当前 BUG-09 的浪费）。worker 层
维护一个轻量的调度器，不为每个 run 预创建 service，而是在 task 内部按需创建
一次。

## 证据

文件: `ornnlab/services/worker_service.py` 第 53-62 行

```python
async def _run_until_no_queued_runs(self) -> int:
    processed = 0
    while True:
        service = ExperimentService(self.settings)  # 每轮新建
        run = service.dequeue_next_run()
        if run is None:
            return processed
        processed += 1
        task = asyncio.create_task(service.execute_dequeued_run(run))
        self._active_runs[run["id"]] = task
        try:
            await task
        finally:
            self._active_runs.pop(run["id"], None)
```

文件: `ornnlab/services/experiment_service.py` 第 20-30 行

```python
class ExperimentService:
    def __init__(self, settings: Settings):
        self.settings = settings
        self.events = EventService(settings)
        self.builder = HarborConfigBuilder(settings)
        self.engine = HarborEngine()
        self.compiler = ProfileCompiler(settings)
        self.queue = QueueService(settings)
        self.reports = ReportService(settings)
        self.templates = TemplateService(settings)
        # 每次创建 8 个对象
```

一个有 50 个 run 的 batch experiment 会创建 50 组共 400 个服务对象。

## 修复方案

在并行执行模型（BUG-10 联合修复）中，每个 run 的 task 内部创建一次
ExperimentService，不再在 worker 调度循环中预创建：

```python
async def _execute_run(self, run: dict) -> None:
    """每个 run 独立的执行上下文，内部创建一次 service"""
    service = ExperimentService(self.settings)  # 只创建一次
    await service.execute_dequeued_run(run)

async def _run_until_no_queued_runs(self, max_concurrent: int = 2) -> int:
    processed = 0
    pending: set[asyncio.Task] = set()
    # 复用同一个 dequeue service（只用于 dequeue，无状态）
    dequeue_service = ExperimentService(self.settings)

    while True:
        done = {t for t in pending if t.done()}
        for t in done:
            pending.discard(t)
            self._active_runs.pop(t.get_name(), None)

        if len(pending) >= max_concurrent:
            await asyncio.wait(pending, return_when=asyncio.FIRST_COMPLETED)
            continue

        run = dequeue_service.dequeue_next_run()
        if run is None:
            if pending:
                await asyncio.wait(pending, return_when=asyncio.ALL_COMPLETED)
            return processed

        processed += 1
        task = asyncio.create_task(self._execute_run(run), name=run["id"])
        self._active_runs[run["id"]] = task
        pending.add(task)
```

收益：
- dequeue 用单个 service 实例（BUG-09 目标）
- 每个 run 的执行在独立 service 中（BUG-10 并行约束）
- 50 个 run 从 50 组 400 对象降为 1 + max_concurrent 组对象

## 验收标准

- [x] worker 调度循环中不再每轮创建 ExperimentService（仅用 1 个 dequeue_service）
- [x] 每个 run 的 task 内部创建一次 ExperimentService（不重复）
- [x] 与 BUG-10 的并行执行模型兼容
- [x] 现有测试全部通过，无回归

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 回归测试 | 单 run 执行 | 创建 + run?wait=true | status == "completed" |
| 回归测试 | batch 执行 | 2 benchmark × 1 agent | 全部 completed |
| 正确性验证 | service 复用 | 检查 dequeue_service 不为每个 run 重建 | 1 个 service 实例用于 dequeue |

## 回滚策略

与 BUG-10 联合回滚。单次 commit，`git revert` 即可。无 schema 变更。
