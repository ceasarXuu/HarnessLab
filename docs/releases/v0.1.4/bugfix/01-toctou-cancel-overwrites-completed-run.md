# BUG-01: 完成的 run 被误标 cancelled（TOCTOU 竞态）

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.2
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: experiment_service, worker_service
- Related Links: [README](README.md), [BUG-02](02-crash-recovery-blind-spot-dequeue-to-running.md)
- Risk Level: High
- Plan Type: Standard
- Phase: 3（竞态修复，依赖 BUG-02 queue 一致性）

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

在 `ExperimentService._run_one` 中，`engine.run(config)` 返回成功结果后，代码会检查 DB 中 run 的状态是否已被外部 `cancel_run` 改为 `cancelled`。如果取消操作恰好发生在 `engine.run` 返回与 `_is_run_cancelled` 检查之间，成功执行结果会被丢弃，run 保持 cancelled，score、result_path 和 report 均不写入。

这是一个 TOCTOU（Time-of-Check to Time-of-Use）竞态条件。

## 证据

文件: `ornnlab/services/experiment_service.py`

```python
try:
    result = await self.engine.run(config)
except asyncio.CancelledError:
    # ...
except Exception as exc:
    # ...

if self._is_run_cancelled(run["id"]):
    self.events.append(
        "run",
        run["id"],
        "harbor.job.cancelled",
        {"source": "cancelled_during_engine_run"},
    )
    return  # 成功 result 被丢弃

# 后续才写 score、report、result_path，并更新 DB 为 completed/failed/etc.
```

竞态时间线:

```text
T1: engine.run(config) 开始执行
T2: cancel_run() 被调用，DB 中 run.status 改为 "cancelled"
T3: engine.run(config) 返回成功 result
T4: _is_run_cancelled() 返回 True
T5: return，成功 result 被丢弃
```

## 修复决策

需要同时满足两个原则：

1. **保留执行事实**：一旦 `engine.run` 已返回 result，`result_path`、`score`、`harbor_job_id` 等事实信息不应被丢弃。
2. **尊重外部 cancel 终态**：如果 DB 已被用户请求标记为 `cancelled`，不要把最终状态强行覆盖成 `completed`。

因此不能只用一个条件 UPDATE：

```sql
UPDATE runs SET ... WHERE id = ? AND status NOT IN ('cancelled', 'failed', 'interrupted')
```

因为当 `rowcount == 0` 时，执行事实也不会写入 DB，仍然无法满足“result 已获取则保留结果”的目标。

## 修复方案

删除 `engine.run` 成功返回后的 `_is_run_cancelled()` 早退逻辑，改成两阶段更新。

### 1. 生成 summary/report

即使 run 已被 cancel，也生成一份基于真实 result 的 summary，供诊断和 artifact 查看使用：

```python
report_path = self.reports.write_summary(
    run["id"],
    result["status"],
    job_dir,
    result.get("score"),
)
finished = now_iso()
```

### 2. 始终保留执行事实

```python
with sqlite.connect(self.settings) as conn:
    conn.execute(
        "UPDATE runs SET "
        "result_path = COALESCE(result_path, ?), "
        "score = COALESCE(score, ?), "
        "harbor_job_id = COALESCE(harbor_job_id, ?), "
        "updated_at = ? "
        "WHERE id = ?",
        (
            result["result_path"],
            result.get("score"),
            result.get("harbor_job_id"),
            finished,
            run["id"],
        ),
    )
```

### 3. 仅在非 terminal 状态下更新最终状态

```python
cursor = conn.execute(
    "UPDATE runs SET status = ?, "
    "finished_at = COALESCE(finished_at, ?), "
    "report_path = COALESCE(report_path, ?), "
    "updated_at = ? "
    "WHERE id = ? AND status NOT IN ('cancelled', 'failed', 'interrupted')",
    (result["status"], finished, report_path, finished, run["id"]),
)
```

### 4. 事件与 queue 状态

```python
if cursor.rowcount == 0:
    self.events.append(
        "run",
        run["id"],
        "harbor.job.completed_but_cancelled",
        {
            "score": result.get("score"),
            "status": result["status"],
            "result_path": result["result_path"],
            "report_path": report_path,
        },
        severity="warning",
    )
else:
    self.queue.finish(run["id"], result["status"])
    self.events.append("run", run["id"], "harbor.job.completed", result)
```

若 `cancel_run` 已经把 `queue_items.state` 写为 `cancelled`，`completed_but_cancelled` 分支不再覆盖 queue 终态。

## CancelledError 语义

`asyncio.CancelledError` 仍然表示 worker task 本身被取消，继续走取消/中断路径：

- 如果 DB 已是 `cancelled`：追加 `harbor.job.cancelled` 事件并返回。
- 如果 DB 不是 `cancelled`：标记为 `interrupted`，避免 task 生命周期异常被误认为用户取消。

本修复只改变 `engine.run` 已经成功返回后的处理，不改变 task cancellation 的语义。

## 风险评估

- 对用户取消语义的影响：DB 终态仍保持 `cancelled`，但会补充真实执行结果作为诊断事实。
- 对 leaderboard 的影响：如果 leaderboard 查询只看 `status='completed'`，已取消 run 不会进入榜单；如果未来需要展示 cancelled-but-completed 的事实，应通过事件或 report 展示。
- 对 report 的影响：如果 `cancel_run` 已写 cancelled report，DB `report_path` 默认不覆盖；真实 result report 可通过 `completed_but_cancelled` 事件 payload 找到。

## Acceptance Criteria（目标，未完成）

- [x] `engine.run` 成功返回后，即使 DB 已被标记 `cancelled`，`result_path`、`score`、`harbor_job_id` 仍被保留。
- [x] 如果 DB 已是 `cancelled`，run 终态不被覆盖为 `completed`。
- [x] cancelled-but-completed 场景追加 `harbor.job.completed_but_cancelled` warning 事件，并包含 result/report 路径。
- [x] `CancelledError` 仍然正确触发取消或 interrupted 路径。
- [x] 覆盖竞态单测，现有 experiment/worker 测试无回归。

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | 正常完成路径 | 创建 + run?wait=true | status == `completed`，score 写入 |
| 正确性验证 | 运行中 cancel | 创建 + run + cancel_run | status == `cancelled` |
| 竞态验证 | 完成结果与 cancel 交错 | mock engine.run 返回成功，在结果处理前调用 cancel_run | status 保持 `cancelled`，result metadata 写入，事件包含 `completed_but_cancelled` |
| 回归测试 | experiment service | 运行 test_experiment_service.py | 全部通过 |

## 回滚策略

单次代码 commit 可直接 `git revert`。无 schema 变更，无数据迁移。回滚后恢复原有行为，但成功结果可能再次被取消竞态丢弃。
