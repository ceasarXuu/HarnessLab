# BUG-06: experiment 状态派生逻辑重复且不一致

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.2
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: experiment_service, recovery_service
- Related Links: [README](README.md), [BUG-02](02-crash-recovery-blind-spot-dequeue-to-running.md)
- Risk Level: Low
- Plan Type: Lightweight
- Phase: 1（正确性基础，无依赖）

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

experiment 状态派生逻辑同时存在于 `experiment_utils.py` 和 `recovery_service.py`。两处实现长期维护会产生漂移风险，并影响 BUG-02 startup recovery、BUG-03 scoped wait、BUG-04 SSE 终止判断等后续修复。

当前 `experiment_utils.derive_experiment_status` 与 `recovery_service._derive_experiment_status` 都在根据 run status 集合推导 experiment status，但函数位置、签名和历史分支顺序不一致。

## 决策

只保留一个 source of truth：

```text
ornnlab/services/experiment_utils.py::derive_experiment_status
```

推荐状态优先级：

```text
all completed      -> completed
completed + failed -> partially_failed
any failed         -> failed
any cancelled      -> cancelled
any interrupted    -> interrupted
any running        -> running
any queued         -> queued
any draft          -> draft
fallback           -> draft
```

`queued` 建议优先于 `draft`。只要一个 experiment 中存在 queued run，就表示它已进入执行队列，不应再被整体视为纯 draft。若产品语义希望 draft 优先，必须在统一函数和测试中显式固定。

## 修复方案

1. 在 `recovery_service.py` 中改用 `derive_experiment_status`。
2. 不再保留 recovery service 内部的重复状态派生函数。
3. 在 `test_experiment_utils.py` 增加状态矩阵测试。
4. recovery service 测试只验证 startup recovery 后 experiment status 与统一函数一致。

示意：

```python
from ornnlab.services.experiment_utils import derive_experiment_status

status = derive_experiment_status(row["status"] for row in rows)
```

## Acceptance Criteria（目标，未完成）

- [ ] recovery service 不再维护第二套 experiment 状态派生逻辑。
- [ ] recovery service 统一调用 `derive_experiment_status`。
- [ ] `derive_experiment_status` 覆盖 draft、queued、running、completed、failed、cancelled、interrupted、partially_failed 的状态矩阵测试。
- [ ] BUG-02、BUG-03、BUG-04 可复用同一状态语义。
- [ ] 现有 recovery/experiment 测试全部通过。

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 单元测试 | 状态矩阵 | 调用 derive_experiment_status | 每种组合返回预期状态 |
| 回归测试 | 创建后状态 | 创建 experiment，不 enqueue | status == `draft` |
| 回归测试 | enqueue 后状态 | 创建后 enqueue | status == `queued` |
| 回归测试 | recovery 路径 | 模拟 startup recovery | experiment 状态与统一函数一致 |

## 回滚策略

单次代码 commit 可直接 `git revert`。无 schema 变更，无数据迁移。
