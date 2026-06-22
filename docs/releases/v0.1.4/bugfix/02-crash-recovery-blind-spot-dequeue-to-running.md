# BUG-02: dequeue 与 mark_running 间的崩溃恢复盲区

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.1
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: queue_service, experiment_service, recovery_service
- Related Links: [README](README.md), [BUG-01](01-toctou-cancel-overwrites-completed-run.md), [BUG-06](06-duplicated-inconsistent-status-derivation.md)
- Risk Level: High
- Plan Type: Standard
- Phase: 2（崩溃恢复，依赖 BUG-06 状态定义）

## 问题描述

`QueueService.dequeue_next` 将 `queue_items.state` 改为 `running`，但此时 `runs.status`
仍为 `queued`。`_mark_run_running` 在稍后才将 `runs.status` 改为 `running`。

如果进程在 `dequeue_next` 之后、`_mark_run_running` 之前崩溃：
- `runs.status` 停在 `queued`
- `queue_items.state` 停在 `running`
- `RunRecoveryService.reconcile_startup` 只恢复 `status='running'` 的 run
- **该 run 永久卡死，无法恢复，也无法被重新 dequeue**

## 证据

文件: `ornnlab/services/queue_service.py` 第 37-43 行

```python
def dequeue_next(self, experiment_id: str | None = None) -> dict | None:
    # ...
    conn.execute(
        "UPDATE queue_items SET state = ?, dequeued_at = ? WHERE run_id = ?",
        ("running", now, run["id"]),          # queue_items → running
    )
    return run                                # 但 runs.status 仍为 "queued"
```

文件: `ornnlab/services/experiment_service.py` 第 286-296 行

```python
def _mark_run_running(self, run, job_dir, harbor_job_name, now):
    with sqlite.connect(self.settings) as conn:
        conn.execute(
            "UPDATE runs SET status = ?, ..."  # runs → running
        )
```

文件: `ornnlab/services/recovery_service.py` 第 28-31 行

```python
def _running_runs(self) -> list[dict]:
    with sqlite.connect(self.settings) as conn:
        return sqlite.rows(
            conn,
            "SELECT * FROM runs WHERE status = 'running' ...",  # 只查 running
        )
```

崩溃窗口:

```
dequeue_next()           ← queue_items.state="running", runs.status="queued"
  ↓ (崩溃发生在这里)
_mark_run_running()      ← runs.status="running" (永远执行不到)
```

重启后：
- `reconcile_startup` 查 `runs.status='running'` → 查不到该 run
- `dequeue_next` 查 `queue_items.state='queued'` → 查不到该 run
- **run 永久卡在中间状态**

## 修复方案

方案 A（推荐）：在 `dequeue_next` 中同时更新 `runs.status` 为 `running`，消除窗口：

```python
def dequeue_next(self, experiment_id: str | None = None) -> dict | None:
    # ...
    conn.execute(
        "UPDATE queue_items SET state = ?, dequeued_at = ? WHERE run_id = ?",
        ("running", now, run["id"]),
    )
    conn.execute(
        "UPDATE runs SET status = ?, updated_at = ? WHERE id = ?",
        ("running", now, run["id"]),  # 同步更新 runs.status
    )
    return run
```

方案 A 的已知窗口：`dequeue_next` 把 `runs.status` 改为 `running` 后，如果后续
`builder.build()` 抛异常，`_mark_run_failed` 会兜底标记为 failed。但此时 run
处于 `running` 状态且无 `job_dir`——recovery 服务重启时会发现此 run，检查
`result.json` 不存在，标记为 `interrupted`。这个行为是正确的。

方案 B：扩展 `reconcile_startup` 同时处理 `queue_items.state='running'` 但
`runs.status` 非 terminal 的孤儿记录。

方案 A 更简洁，从根源消除不一致窗口。

## 存量数据修复

生产环境中已存在的卡死 run 需要一次性修复。在 `reconcile_startup` 中添加
孤儿 queue_items 扫描：

```python
def reconcile_startup(self) -> dict[str, int]:
    # 现有逻辑：恢复 status='running' 的 run
    running = self._running_runs()
    # ... 现有恢复逻辑 ...

    # 新增：修复孤儿 queue_items
    orphaned = self._orphaned_queue_items()
    for run in orphaned:
        self._reconcile_run(run)
        counts["recovered"] += 1

    return counts

def _orphaned_queue_items(self) -> list[dict]:
    """查找 queue_items.state='running' 但 runs.status 非 terminal 的记录"""
    with sqlite.connect(self.settings) as conn:
        return sqlite.rows(
            conn,
            "SELECT r.* FROM queue_items q JOIN runs r ON r.id = q.run_id "
            "WHERE q.state = 'running' "
            "AND r.status NOT IN ('completed', 'failed', 'cancelled', 'interrupted')",
        )
```

## 验收标准

- [x] `dequeue_next` 在同一事务中同时更新 `queue_items.state` 和 `runs.status` 为 `running`
- [x] `reconcile_startup` 能发现并修复 `queue_items.state='running'` 但 `runs.status` 非 terminal 的孤儿记录
- [x] 模拟 dequeue 后崩溃→重启，run 被正确恢复为 `interrupted` 或 `recovered`
- [x] 现有测试全部通过，无回归

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | dequeue 原子性 | 调用 dequeue_next 后检查 runs.status | runs.status == "running" |
| 正确性验证 | 崩溃恢复 | 手动制造 queue_items.state='running' + runs.status='queued' 的孤儿记录，调用 reconcile_startup | 孤儿记录被恢复为 interrupted 或 recovered |
| 回归测试 | 正常 run 流程 | 创建 + run?wait=true | status == "completed"，无回归 |
| 回归测试 | 现有 recovery 测试 | 运行 test_recovery_service.py | 全部通过 |

## 回滚策略

`dequeue_next` 的修改是单次 commit。如果引入问题：
1. `git revert` 恢复 dequeue_next 原始实现
2. 存量数据修复逻辑（`_orphaned_queue_items`）可保留，它是只读扫描 + 安全恢复，无副作用
3. 无 schema 变更，无需数据回滚
