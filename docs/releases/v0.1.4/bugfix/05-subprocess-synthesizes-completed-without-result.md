# BUG-05: subprocess 模式合成 completed 掩盖异常

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.2
- Status: Draft / Needs upstream verification
- Owner / Responsible: project maintainer
- Related Systems: harbor_subprocess, experiment_service
- Related Links: [README](README.md)
- Risk Level: Low
- Plan Type: Lightweight
- Phase: 1（正确性基础，但必须先验证 Harbor 行为）

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

在 subprocess 模式下，当 Harbor 进程退出码为 0 但未写 `result.json` 时，`ManagedSubprocessHarborRunner` 会合成一个 `status: completed` 的结果。这可能掩盖 Harbor 内部异常：exit 0 但无输出文件通常意味着执行结果不可验证。

不过，该判断依赖 Harbor 的上游行为契约。本 bug 不能在未验证 Harbor 成功路径的前提下直接落地为确定性修复。

## 前置验证

必须先确认当前支持的 Harbor 版本在正常完成路径下是否总是写 `result.json`。

验证对象：

- Harbor CLI：`harbor run --config <config>` 成功路径
- OrnnLab 支持的 Harbor 版本范围，例如 0.13.x
- 特殊路径：dry-run、`n_tasks=0`、全部 trial 被跳过、全部 trial cancelled、全部 trial errored

验证方法：

1. 阅读 Harbor 源码中 CLI `run` 命令和 `Job.run()` 成功路径。
2. 增加 real Harbor smoke test：执行最小 job，断言 exit code 与 result file 行为。
3. 明确文档化契约：
   - 正常成功是否必定写 `result.json`
   - 哪些合法场景允许 exit 0 且无 result file
   - 发生缺失 result 时应被 OrnnLab 视为 failed、interrupted 还是 upstream_protocol_error

## 当前证据

文件: `ornnlab/services/harbor_subprocess.py`

```python
def _read_or_write_result(path: Path, return_code: int) -> dict[str, Any]:
    if path.exists():
        return json.loads(path.read_text(encoding="utf-8"))
    result = {"status": "completed", "score": None, "subprocess_returncode": return_code}
    atomic_write_text(path, json.dumps(result, indent=2, sort_keys=True))
    return result
```

问题分析：

- `return_code=0` 表示进程层面成功，但不必然等价于 benchmark 结果完整。
- 合成 `completed` 会让 run 和 experiment 进入成功终态。
- `score=None` 虽然通常不会进入 leaderboard，但用户会看到 completed，容易误判。

## 修复决策

只有当前置验证证明“Harbor 正常成功必定写 `result.json`”时，才能把缺失 result 的 exit 0 视为异常。

验证通过后，建议不要简单标记为普通 `interrupted`，而是显式携带 failure class/code：

```python
result = {
    "status": "interrupted",
    "score": None,
    "subprocess_returncode": return_code,
    "failure_class": "harbor_protocol",
    "failure_code": "missing_result_json_after_success_exit",
    "warning": "harbor exited 0 but did not produce result.json",
}
```

上层 `ExperimentService` 需要把该 failure metadata 写入 report，避免只看到 interrupted 而缺少根因。

## 修复方案（验证通过后）

```python
def _read_or_write_result(path: Path, return_code: int) -> dict[str, Any]:
    if path.exists():
        return json.loads(path.read_text(encoding="utf-8"))

    result = {
        "status": "interrupted",
        "score": None,
        "subprocess_returncode": return_code,
        "failure_class": "harbor_protocol",
        "failure_code": "missing_result_json_after_success_exit",
        "warning": "harbor exited 0 but did not produce result.json",
    }
    atomic_write_text(path, json.dumps(result, indent=2, sort_keys=True))
    return result
```

如果验证发现 Harbor 存在合法的 exit 0 + missing result 场景，则必须改为条件分支，不能一律标记为 interrupted。

## 非目标

- 不在本 bug 中修改 Python API runner 的 result 写入语义。
- 不在未验证 Harbor 行为前改变生产路径状态。
- 不引入 schema 变更。

## Acceptance Criteria（目标，未完成）

- [ ] 完成 Harbor 成功路径契约验证，并记录结果。
- [ ] 若验证通过，exit 0 且缺失 `result.json` 时不再合成 `completed`。
- [ ] 合成异常结果包含 `failure_class`、`failure_code` 和 warning。
- [ ] 有 `result.json` 的 exit 0 路径行为不变，读取真实 result。
- [ ] 覆盖 subprocess smoke/mocked 测试，现有 harbor_subprocess 测试无回归。

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 前置验证 | Harbor 正常成功 | real Harbor 最小 job | exit 0 且生成 result.json，或记录例外契约 |
| 正确性验证 | exit 0 + 无 result.json | mock subprocess exit 0 且不写 result | status != `completed`，包含 failure metadata |
| 回归测试 | exit 0 + 有 result.json | mock subprocess 写真实 result | 读取并返回真实 result |
| 回归测试 | 非零退出 | mock subprocess 非零返回 | 仍抛 RuntimeError 或既有失败路径 |

## 回滚策略

若代码变更后发现 Harbor 合法场景依赖 exit 0 + missing result，可 `git revert`。无 schema 变更，无数据迁移。
