# BUG-09: Worker 每轮重建 ExperimentService

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.2
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: worker_service, experiment_service
- Related Links: [README](README.md), [BUG-10](10-worker-serial-run-execution.md), [BUG-03](03-wait-true-blocks-unrelated-experiments.md)
- Risk Level: Medium
- Plan Type: Standard
- Phase: 4（Worker 架构，与 BUG-10 联合修复）

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

`QueueWorkerService._run_until_no_queued_runs` 在每次循环中创建新的 `ExperimentService`，连带创建多个子服务对象。这个现象存在，但性能收益不应被夸大：这些对象本身较轻量，真正影响吞吐的主因是 BUG-10 中 worker 串行执行 run。

## 与 BUG-10 的关系

本项必须作为 BUG-10 的架构配套，而不是单独性能主线。

原方案中“50 个 run 从 50 组 service 对象降为 1 + max_concurrent 组”的说法不准确。如果并行执行时每个 run task 内部仍创建一个独立 `ExperimentService`，那么总创建次数仍接近每 run 一次；减少的是调度层重复创建和同时存活对象数量，而不是总实例数。

## 证据

当前 worker 循环：

```python
while True:
    service = ExperimentService(self.settings)
    run = service.dequeue_next_run()
    if run is None:
        return processed
    task = asyncio.create_task(service.execute_dequeued_run(run))
    await task
```

`ExperimentService.__init__` 会创建 EventService、HarborConfigBuilder、HarborEngine、ProfileCompiler、QueueService、ReportService、TemplateService 等子对象。

## 修复决策

以 BUG-10 的有界并行为主线，BUG-09 只负责明确 service 生命周期：

1. worker 调度层使用轻量 dequeue context，不在循环中为每次 dequeue 重建完整执行上下文。
2. 每个 run 的执行 task 拥有独立执行上下文，避免并行 run 之间共享可变 service 状态。
3. 不把该修复描述为大幅减少总对象创建次数；它主要是架构清晰度和并行隔离收益。

## 修复方案

```python
async def _execute_run(self, run: dict) -> None:
    service = ExperimentService(self.settings)
    await service.execute_dequeued_run(run)

async def _run_until_no_queued_runs(self, experiment_id: str | None = None) -> int:
    dequeue_service = ExperimentService(self.settings)
    pending: set[asyncio.Task[None]] = set()
    # BUG-10 在这里实现 max_concurrent 调度
```

关键点：

- `dequeue_service` 只负责 dequeue，不执行 run。
- `_execute_run` 内部为每个 run 创建一次独立 `ExperimentService`。
- 如果未来要进一步优化，应拆出更轻量的 `RunExecutor`，而不是在并行 run 之间共享同一个完整 `ExperimentService`。

## 非目标

- 不在本 bug 中实现跨 run 的全局 service 单例。
- 不把 `ExperimentService` 作为并行 run 的共享 mutable context。
- 不单独承诺显著性能提升；主要吞吐收益来自 BUG-10。

## Acceptance Criteria（目标，未完成）

- [ ] worker 调度循环不再每轮都创建完整执行 service 只为 dequeue。
- [ ] 每个 run task 拥有独立执行上下文，避免并行 run 状态串扰。
- [ ] 文档不再声称总 service 实例数从 N 降为 `1 + max_concurrent`。
- [ ] 与 BUG-10 的并行执行模型兼容。
- [ ] 现有 worker/experiment 测试无回归。

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 回归测试 | 单 run 执行 | 创建 + run?wait=true | status == `completed` |
| 回归测试 | batch 执行 | 2 benchmark × 1 agent | 全部 terminal |
| 架构验证 | dequeue context | 检查 worker 调度层只创建一个 dequeue service | 不随 run 数重复创建 dequeue service |
| 并发验证 | 独立执行上下文 | 2 个 run 并行执行 | 状态、事件、report 不串扰 |

## 回滚策略

与 BUG-10 联合回滚。单次代码 commit 可直接 `git revert`。无 schema 变更。
