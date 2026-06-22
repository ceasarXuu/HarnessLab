# BUG-02: dequeue 与 mark_running 间的崩溃恢复盲区

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.2
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: queue_service, experiment_service, recovery_service
- Related Links: [README](README.md), [BUG-01](01-toctou-cancel-overwrites-completed-run.md), [BUG-06](06-duplicated-inconsistent-status-derivation.md)
- Risk Level: High
- Plan Type: Standard
- Phase: 2（崩溃恢复，依赖 BUG-06 状态定义）

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

`QueueService.dequeue_next` 将 `queue_items.state` 改为 `running`，但此时 `runs.status` 仍为 `queued`。`ExperimentService._mark_run_running` 在稍后才将 `runs.status` 改为 `running`。

如果进程在 `dequeue_next` 之后、`_mark_run_running` 之前崩溃：

- `runs.status` 停在 `queued`
- `queue_items.state` 停在 `running`
- `RunRecoveryService.reconcile_startup` 只恢复 `runs.status='running'` 的 run
- 该 run 无法被 startup recovery 发现，也无法再次被 dequeue

## 证据

文件: `ornnlab/services/queue_service.py`

```python
def dequeue_next(self, experiment_id: str | None = None) -> dict | None:
    # ...
    conn.execute(
        "UPDATE queue_items SET state = ?, dequeued_at = ? WHERE run_id = ?",
        ("running", now, run["id"]),
    )
    return run  # runs.status 仍是 "queued"
```

文件: `ornnlab/services/recovery_service.py`

```python
def _running_runs(self) -> list[dict]:
    return sqlite.rows(
        conn,
        "SELECT * FROM runs WHERE status = 'running' ORDER BY started_at, id",
    )
```

崩溃窗口：

```text
dequeue_next()           → queue_items.state="running", runs.status="queued"
  ↓ crash
_mark_run_running()      → runs.status="running"（没有机会执行）
```

## 修复方案

### 1. dequeue 时同步 claim run 状态

推荐在 `QueueService.dequeue_next` 的同一 DB 事务中同时更新 `queue_items.state` 与 `runs.status`：

```python
def dequeue_next(self, experiment_id: str | None = None) -> dict | None:
    now = now_iso()
    with sqlite.connect(self.settings) as conn:
        # select queued row, preserving existing experiment_id filter
        queued = sqlite.rows(...)
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
        run["status"] = "running"
        return run
```

如果未来支持多进程 worker，需要进一步把 select/update 改成原子 claim：

```sql
UPDATE queue_items
SET state = 'running', dequeued_at = ?
WHERE run_id = ? AND state = 'queued'
```

并检查 `rowcount == 1`。当前本地单进程 worker 下，上述同步更新已经能消除主要恢复盲区。

### 2. `_mark_run_running` 保持幂等

`_mark_run_running` 仍负责写入 `started_at`、`job_dir`、`harbor_job_name` 和 experiment 状态。它可以继续把 run 设为 `running`，但该更新必须是幂等的。

### 3. startup recovery 处理存量孤儿记录

已有 DB 中可能存在 `queue_items.state='running'` 且 `runs.status` 非 terminal 的孤儿记录。startup recovery 应扫描并处理这类记录。

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
```

注意：不能对 orphaned 记录无条件 `counts["recovered"] += 1`，因为 `_reconcile_run` 可能返回 `"interrupted"`。

```python
def _orphaned_queue_items(self) -> list[dict]:
    with sqlite.connect(self.settings) as conn:
        return sqlite.rows(
            conn,
            "SELECT r.* FROM queue_items q JOIN runs r ON r.id = q.run_id "
            "WHERE q.state = 'running' "
            "AND r.status NOT IN ('completed', 'failed', 'cancelled', 'interrupted') "
            "AND r.status != 'running' "
            "ORDER BY q.dequeued_at, r.id",
        )
```

### 4. 与 BUG-01 的顺序关系

本修复应先于 BUG-01 落地。BUG-01 的 cancel/result 竞态修复依赖 queue 与 run 状态的一致性，否则 result 写入和 queue.finish 的条件会更难判断。

## 风险评估

- 如果 `builder.build()` 在 dequeue 后失败，run 已经是 `running`；`_mark_run_failed` 会兜底标记为 `failed`。如果此时进程崩溃，startup recovery 会看到 running run，没有 result 时标记为 `interrupted`，这是可接受的。
- startup orphan recovery 是一次性安全修复：只处理非 terminal run，不触碰已完成、失败、取消或 interrupted 的记录。
- 多进程 worker 场景仍需要更强的 atomic claim；当前计划只覆盖现有单进程本地 WebUI 语义。

## Acceptance Criteria（目标，未完成）

- [ ] `dequeue_next` 在同一事务中同时更新 `queue_items.state` 和 `runs.status` 为 `running`。
- [ ] `_mark_run_running` 对已为 `running` 的 run 保持幂等。
- [ ] `reconcile_startup` 能发现并修复 `queue_items.state='running'` 但 `runs.status` 非 terminal 且非 running 的孤儿记录。
- [ ] orphan recovery 的 `counts` 使用 `_reconcile_run` 返回值，不把 interrupted 误计为 recovered。
- [ ] 模拟 dequeue 后崩溃再重启，run 被正确恢复为 `interrupted` 或 `recovered`。
- [ ] 现有 queue/recovery/experiment 测试全部通过。

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | dequeue 原子性 | 调用 dequeue_next 后检查 runs.status 和 queue_items.state | 二者均为 `running` |
| 正确性验证 | 存量孤儿恢复 | 手动制造 queue_items.running + runs.queued，调用 reconcile_startup | 根据 result.json 是否存在返回 recovered/interrupted，并正确计数 |
| 崩溃恢复 | dequeue 后未 mark_running | 模拟进程重启 | run 不再永久卡死 |
| 回归测试 | 正常 run 流程 | 创建 + run?wait=true | status == `completed` |

## 回滚策略

`dequeue_next` 的修改和 orphan recovery 逻辑应在单次代码 commit 中落地。如引入问题可 `git revert`。无 schema 变更，无需数据回滚。
