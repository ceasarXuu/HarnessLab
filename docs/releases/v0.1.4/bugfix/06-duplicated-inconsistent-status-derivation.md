# BUG-06: experiment 状态派生逻辑重复且不一致

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.1
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: experiment_service, recovery_service
- Related Links: [README](README.md), [BUG-02](02-crash-recovery-blind-spot-dequeue-to-running.md)
- Risk Level: Low
- Plan Type: Lightweight
- Phase: 1（正确性基础，无依赖）

## 问题描述

experiment 最终状态的派生逻辑存在于两处，且优先级顺序不一致。维护时容易遗漏
其中一处，导致同一组 run 状态在不同代码路径下产生不同的 experiment 状态。

## 证据

文件 A: `ornnlab/services/experiment_utils.py` 第 26-43 行

```python
def derive_experiment_status(statuses: Iterable[str]) -> str:
    unique = set(statuses)
    if unique == {"completed"}:
        return "completed"
    if "completed" in unique and "failed" in unique:
        return "partially_failed"
    if "failed" in unique:
        return "failed"
    if "cancelled" in unique:
        return "cancelled"
    if "interrupted" in unique:
        return "interrupted"
    if "running" in unique:
        return "running"
    if "draft" in unique:          # ← 有 draft 分支
        return "draft"
    return "queued"
```

文件 B: `ornnlab/services/recovery_service.py` 第 210-227 行

```python
def _derive_experiment_status(statuses: list[str]) -> str:
    unique = set(statuses)
    if unique == {"completed"}:
        return "completed"
    if "completed" in unique and "failed" in unique:
        return "partially_failed"
    if "failed" in unique:
        return "failed"
    if "cancelled" in unique:
        return "cancelled"
    if "interrupted" in unique:
        return "interrupted"
    if "running" in unique:
        return "running"
    return "queued"                # ← 无 draft 分支，直接返回 queued
```

差异：
1. `experiment_utils` 版本有 `"draft" in unique → "draft"` 分支
2. `recovery_service` 版本没有该分支，当所有 run 都是 `draft` 时返回 `queued`
3. 两个函数签名不同（`Iterable[str]` vs `list[str]`），命名不同

## 决策

保留 `draft` 分支。若所有 run 均为 draft（从未入队），experiment 状态应保持
`draft` 而非 `queued`。`queued` 表示已入队等待执行，`draft` 表示尚未入队，
两者语义不同。

## 修复方案

将状态派生逻辑统一到 `experiment_utils.py` 中的 `derive_experiment_status`，
删除 `recovery_service.py` 中的 `_derive_experiment_status`，改为导入使用：

```python
# recovery_service.py
from ornnlab.services.experiment_utils import derive_experiment_status

class RunRecoveryService:
    def _update_experiment_status(self, experiment_id: str) -> None:
        with sqlite.connect(self.settings) as conn:
            rows = sqlite.rows(
                conn,
                "SELECT status FROM runs WHERE experiment_id = ?",
                (experiment_id,),
            )
            status = derive_experiment_status([row["status"] for row in rows])
            # ...
```

## 验收标准

- [x] `recovery_service.py` 中不再有 `_derive_experiment_status` 函数定义
- [x] `recovery_service.py` 从 `experiment_utils` 导入 `derive_experiment_status`
- [x] 所有 run 均为 `draft` 时，`derive_experiment_status` 返回 `draft`
- [x] 现有测试全部通过，无回归

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 回归测试 | experiment 创建后状态 | 创建 experiment，不 enqueue，检查状态 | status == "draft" |
| 回归测试 | experiment 完成后状态 | 创建 + run?wait=true，检查状态 | status == "completed" |
| 回归测试 | recovery 路径状态派生 | 模拟 startup recovery，检查 experiment 状态 | 与 experiment_utils 一致 |
| 正确性验证 | draft 场景 | 所有 run 为 draft 时调用 derive_experiment_status | 返回 "draft" |

## 回滚策略

单次 commit，如引入问题直接 `git revert`。无 schema 变更，无数据迁移。
