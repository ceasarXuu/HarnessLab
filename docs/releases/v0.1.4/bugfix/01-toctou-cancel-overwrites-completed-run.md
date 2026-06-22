# BUG-01: 完成的 run 被误标 cancelled（TOCTOU 竞态）

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.1
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: experiment_service, worker_service
- Related Links: [README](README.md), [BUG-02](02-crash-recovery-blind-spot-dequeue-to-running.md)
- Risk Level: High
- Plan Type: Standard
- Phase: 3（竞态修复，依赖 BUG-02 queue 一致性）

## 问题描述

在 `ExperimentService._run_one` 中，`engine.run(config)` 返回成功结果后，代码会检查
DB 中 run 的状态是否已被外部 `cancel_run` 改为 `cancelled`。如果取消操作恰好发生在
`engine.run` 返回与 `_is_run_cancelled` 检查之间，**成功的执行结果会被丢弃**，run
被错误标记为 cancelled，score 和 report 均不写入。

这是一个经典的 TOCTOU（Time-of-Check to Time-of-Use）竞态条件。

## 证据

文件: `ornnlab/services/experiment_service.py`

```python
# 第 335-345 行
try:
    result = await self.engine.run(config)       # harbor 执行完成，拿到成功结果
except asyncio.CancelledError:
    # ...
except Exception as exc:
    # ...
# 第 339 行
if self._is_run_cancelled(run["id"]):            # 检查 DB 状态
    self.events.append(
        "run", run["id"], "harbor.job.cancelled",
        {"source": "cancelled_during_engine_run"},
    )
    return                                       # ← 成功的 result 被丢弃!
# 第 347 行后续：写 score、report、更新 DB 为 completed
```

竞态时间线:

```
T1: engine.run(config) 开始执行
T2: cancel_run() 被调用，DB 中 run.status 改为 "cancelled"
T3: engine.run(config) 返回成功 result
T4: _is_run_cancelled() 返回 True
T5: return，成功结果被丢弃
```

## 决策：result 已获取 + DB 已 cancelled 时的最终状态

当 `engine.run` 已成功返回 result 时，**以执行结果为事实来源**，写入 completed
结果。理由：

1. harbor 已完成执行，产生了真实的 score 和 artifacts
2. 丢弃结果等于浪费已消耗的计算资源
3. 用户可以从 report 中看到真实执行情况

但需要处理 `cancel_run` 可能已写了 cancelled report 的情况——使用
`COALESCE` 更新避免覆盖已有 report_path：

```python
# 如果 cancel_run 已写 report_path，不覆盖
conn.execute(
    "UPDATE runs SET status = ?, finished_at = COALESCE(finished_at, ?), "
    "result_path = ?, report_path = COALESCE(report_path, ?), "
    "score = ?, harbor_job_id = ?, updated_at = ? WHERE id = ?",
    (result["status"], finished, result["result_path"], report_path,
     result.get("score"), result.get("harbor_job_id"), finished, run["id"]),
)
```

状态冲突处理：如果 DB 已是 `cancelled`，使用条件更新——仅在 status 不是
terminal 时才更新为 completed：

```python
conn.execute(
    "UPDATE runs SET ... WHERE id = ? AND status NOT IN ('cancelled', 'failed', 'interrupted')",
    (..., run["id"]),
)
```

如果该 UPDATE 影响行数为 0（说明已被外部标记 cancelled），则追加一条事件
记录实际执行结果，但不改变 DB 状态：

```python
if cursor.rowcount == 0:
    self.events.append(
        "run", run["id"], "harbor.job.completed_but_cancelled",
        {"score": result.get("score"), "status": result["status"]},
        severity="warning",
    )
```

## 修复方案

```python
try:
    result = await self.engine.run(config)
except asyncio.CancelledError:
    # 仅在 task 本身被 cancel 时才走取消路径
    if self._is_run_cancelled(run["id"]):
        self.events.append(...)
        return
    await self._mark_run_interrupted(...)
    return
except Exception as exc:
    # ...

# engine.run 已成功返回，写入结果
# 使用条件更新避免覆盖外部 cancel 操作
report_path = self.reports.write_summary(
    run["id"], result["status"], job_dir, result.get("score"),
)
finished = now_iso()
with sqlite.connect(self.settings) as conn:
    cursor = conn.execute(
        "UPDATE runs SET status = ?, finished_at = COALESCE(finished_at, ?), "
        "result_path = ?, report_path = COALESCE(report_path, ?), "
        "score = ?, harbor_job_id = ?, updated_at = ? "
        "WHERE id = ? AND status NOT IN ('cancelled', 'failed', 'interrupted')",
        (result["status"], finished, result["result_path"], report_path,
         result.get("score"), result.get("harbor_job_id"), finished, run["id"]),
    )
    if cursor.rowcount == 0:
        # 已被外部标记 cancelled，记录实际结果但不改变状态
        self.events.append(
            "run", run["id"], "harbor.job.completed_but_cancelled",
            {"score": result.get("score"), "status": result["status"]},
            severity="warning",
        )
    else:
        self.queue.finish(run["id"], result["status"])
        self.events.append("run", run["id"], "harbor.job.completed", result)
```

核心原则：**以执行结果为事实来源**，不因 DB 状态轮询丢弃已获取的成功结果。
同时尊重外部 cancel 操作的最终状态，通过条件 UPDATE 避免冲突。

## 风险评估

修复改变了 CancelledError 的处理语义：
- **之前**：`engine.run` 返回后检查 DB，若 cancelled 则丢弃结果
- **之后**：`engine.run` 返回后写入结果，仅在 DB 未被外部 cancel 时更新状态

对 worker 取消机制的影响：
- worker 的 `task.cancel()` 仍然会触发 `CancelledError`，走取消路径
- `cancel_run` API 仍然会将 DB 状态改为 cancelled
- 唯一变化：如果 `engine.run` 在 cancel 之前已完成，结果会被保留

## 验收标准

- [x] `engine.run` 成功返回后，即使 DB 已被标记 cancelled，执行结果（score、report）仍被写入
- [x] 如果 DB 已是 cancelled，状态不被覆盖为 completed，但追加 `harbor.job.completed_but_cancelled` 事件
- [x] `CancelledError` 仍然正确触发取消路径
- [x] 现有测试全部通过，无回归

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | 正常完成路径 | 创建 + run?wait=true | status == "completed"，score 写入 |
| 正确性验证 | 运行中 cancel | 创建 + run + cancel_run（queued 状态） | status == "cancelled" |
| 正确性验证 | 完成后 cancel 不覆盖 | mock engine.run 返回成功，在返回前调用 cancel_run | report 有内容，事件包含 harbor.job.completed_but_cancelled |
| 回归测试 | 现有 experiment 测试 | 运行 test_experiment_service.py | 全部通过 |

## 回滚策略

单次 commit，如引入问题直接 `git revert`。无 schema 变更，无数据迁移。
竞态条件修复不改变 DB schema 或 API 接口，回滚后恢复原有行为（结果可能被丢弃）。
