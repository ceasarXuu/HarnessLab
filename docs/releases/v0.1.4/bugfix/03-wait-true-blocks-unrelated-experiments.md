# BUG-03: wait=true 等待无关 experiment 的 run

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.2
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: experiments API, worker_service
- Related Links: [README](README.md), [BUG-09](09-worker-recreates-experiment-service-per-run.md), [BUG-10](10-worker-serial-run-execution.md)
- Risk Level: Medium
- Plan Type: Lightweight
- Phase: 4（依赖 worker 架构定型）

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

API `POST /api/experiments/{experiment_id}/run?wait=true` 的语义应是等待当前 experiment 的所有 run 完成。但当前实现调用 `QueueWorkerService.wait_until_idle()`，该方法等待全局 worker drain 完成，因此会被其他 experiment 的 queued/running run 阻塞。

如果 experiment A 已经排入大量 run，此时对 experiment B 发起 `wait=true`，B 的请求可能直到 A 的 run 全部执行结束后才返回。

## 证据

文件: `ornnlab/api/experiments.py`

```python
@router.post("/{experiment_id}/run")
async def run_experiment(experiment_id, request, wait: bool = False):
    service = ExperimentService(request.app.state.settings)
    service.enqueue(experiment_id)
    worker = request.app.state.worker
    if wait:
        await worker.wait_until_idle()  # 等待全局队列空闲
    else:
        worker.start()
    return service.get(experiment_id)
```

文件: `ornnlab/services/worker_service.py`

```python
async def _run_until_no_queued_runs(self) -> int:
    while True:
        service = ExperimentService(self.settings)
        run = service.dequeue_next_run()  # 不带 experiment_id 过滤
        if run is None:
            return processed
```

## 修复决策

不要把 `wait=true` 实现为“等待 worker 全局 idle”。即使给 `wait_until_idle(experiment_id)` 增加参数，只要方法内部仍然执行：

```python
if task is not None and not task.done():
    await task
    return
```

就会继续等待已经运行的全局 worker，仍然无法保证只等待当前 experiment。

因此，`wait=true` 应改为：

1. enqueue 当前 experiment。
2. 确保后台 worker 已启动。
3. 等待当前 experiment 的所有 run 进入 terminal 状态。
4. 不抢占全局 drain，不依赖 `_loop_lock`。

## 修复方案

### 1. 新增 experiment-scoped terminal wait

```python
TERMINAL_RUN_STATUSES = {"completed", "failed", "cancelled", "interrupted"}

async def wait_experiment_terminal(
    self,
    experiment_id: str,
    poll_interval_sec: float = 0.2,
) -> None:
    while True:
        state = ExperimentService(self.settings).get(experiment_id)
        statuses = {run["status"] for run in state["runs"]}
        if statuses and statuses.issubset(TERMINAL_RUN_STATUSES):
            return
        await asyncio.sleep(poll_interval_sec)
```

该方法只观察当前 experiment 的 run 状态，不等待全局 worker task 完成。

### 2. API 调整

```python
service.enqueue(experiment_id)
worker = request.app.state.worker
worker.start()
if wait:
    await worker.wait_experiment_terminal(experiment_id)
return service.get(experiment_id)
```

即使后台 worker 正在处理其他 experiment，`wait=true` 也只等待当前 experiment 的状态变化。

### 3. 与 BUG-10 的关系

BUG-10 如果引入 `dequeue_next_run(experiment_id)`，可以用于同步测试或局部 drain，但不应作为 HTTP `wait=true` 的主要语义。HTTP request 不应持有 worker drain lock，也不应阻止后台 worker 继续处理全局队列。

### 4. 超时策略

当前 API 可先不暴露超时参数，但内部建议保留可测试参数。未来如需避免 HTTP 长时间挂起，可新增 query 参数：

```text
POST /api/experiments/{experiment_id}/run?wait=true&timeout_sec=300
```

v0.1.4 非目标：不新增公开 API 参数，只修正当前 wait 语义。

## 非目标

- 不改变 `worker.start()` 的全局后台处理行为。
- 不改变 `dequeue_next` 默认处理全局队列的行为。
- 不在本 bug 中引入新的公开 timeout 参数。

## 竞争分析

- 若当前 experiment 的 run 已被后台 worker 执行，状态轮询会在其 terminal 后返回。
- 若后台 worker 尚未启动，API 会先调用 `worker.start()`。
- 若当前 experiment 的某些 run 被 cancel，terminal wait 会在所有 run 成为 terminal 后返回。
- 若 worker 崩溃且 run 卡在 queued/running，BUG-02 recovery 和 worker error handling 应负责恢复；本 bug 不掩盖 worker 层异常。

## Acceptance Criteria（目标，未完成）

- [x] `run?wait=true` 不再等待无关 experiment 的 run 完成。
- [x] `wait=true` 路径会启动后台 worker，并等待当前 experiment 的 run 全部 terminal。
- [x] `wait=false` 行为保持不变：enqueue 后立即返回并启动后台 worker。
- [x] 如果后台 worker 已在处理全局队列，当前 experiment terminal 后 `wait=true` 可返回，不必等 worker 全局 idle。
- [x] 现有 experiment API 和 worker 测试无回归。

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | wait 只等当前 experiment | 创建 A（慢 run）+ B（快 run），B run?wait=true | B terminal 后返回，A 可仍在 running/queued |
| 回归测试 | 单 experiment wait | 创建 1 experiment，run?wait=true | 返回时该 experiment terminal |
| 回归测试 | 后台 worker | run（无 wait）后轮询状态 | 最终 terminal |
| 竞态测试 | worker 已运行 | 启动全局 worker 后再对 B wait=true | 不等待无关 A 全部完成 |

## 回滚策略

单次代码 commit 可直接 `git revert`。API 接口不变，仅修正 `wait=true` 行为。
