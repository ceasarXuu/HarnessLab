# BUG-05: subprocess 模式合成 completed 掩盖异常

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.1
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: harbor_subprocess, experiment_service
- Related Links: [README](README.md)
- Risk Level: Low
- Plan Type: Lightweight
- Phase: 1（正确性基础，无依赖）

## 问题描述

在 subprocess 模式下，当 harbor 退出码为 0 但未写 `result.json` 时，
`ManagedSubprocessHarborRunner` 会合成一个 `status: completed` 的结果。
这可能掩盖 harbor 内部异常——exit 0 但无输出文件通常意味着 harbor
未正常完成工作。

## 前置验证

需要确认 harbor 0.13.2 在正常路径下是否总是写 result.json。

验证方法：检查 harbor 源码中 `harbor run` 命令的成功路径，确认
`Job.run()` 返回后是否一定写 result.json 到 jobs_dir。

假设：harbor 0.13.2 在正常完成时总是写 result.json。exit 0 但无
result.json 属于异常场景（如 harbor 内部 catch 了异常但未设置非零退出码）。

如果验证发现 harbor 在某些正常路径（如 dry-run、n_tasks=0）下不写
result.json，则需额外判断条件，不能一律标记为 interrupted。

## 证据

文件: `ornnlab/services/harbor_subprocess.py` 第 128-131 行

```python
def _read_or_write_result(path: Path, return_code: int) -> dict[str, Any]:
    if path.exists():
        return json.loads(path.read_text(encoding="utf-8"))
    # harbor exit 0 但未写 result.json
    result = {"status": "completed", "score": None, "subprocess_returncode": return_code}
    atomic_write_text(path, json.dumps(result, indent=2, sort_keys=True))
    return result
```

问题分析：
- return_code=0 通常表示成功，但 harbor 可能在内部出错后仍以 0 退出
- 合成的 `status: completed` 会让 run 被标记为成功完成
- `score: None` 虽然不会进入 leaderboard，但 experiment 状态会变为 completed
- 用户看到 "completed" 但实际没有执行结果

## 兼容性影响

状态值从 `completed` 变为 `interrupted` 会影响下游：
- **leaderboard**: 无影响。`score: None` 的 run 本就不进 leaderboard
- **report**: report 会显示 interrupted，更准确
- **experiment 状态**: 如果 experiment 只有这一个 run，状态从 completed 变为 interrupted
- **前端展示**: 前端需能展示 interrupted 状态（当前已支持）

## 修复方案

区分 "有 result.json 的成功" 和 "无 result.json 的可疑成功"：

```python
def _read_or_write_result(path: Path, return_code: int) -> dict[str, Any]:
    if path.exists():
        return json.loads(path.read_text(encoding="utf-8"))
    # harbor exit 0 但未写 result.json，标记为可疑状态
    result = {
        "status": "interrupted",  # 而非 "completed"
        "score": None,
        "subprocess_returncode": return_code,
        "warning": "harbor exited 0 but did not produce result.json",
    }
    atomic_write_text(path, json.dumps(result, indent=2, sort_keys=True))
    return result
```

## 验收标准

- [x] harbor exit 0 且无 result.json 时，合成的 status 为 `interrupted` 而非 `completed`
- [x] harbor exit 0 且有 result.json 时，行为不变（读取并返回真实结果）
- [x] 合成结果包含 `warning` 字段说明原因
- [x] 现有测试全部通过，无回归

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | exit 0 + 无 result.json | 模拟 harbor subprocess exit 0 且不写 result.json | 合成 status == "interrupted"，包含 warning |
| 回归测试 | exit 0 + 有 result.json | 模拟 harbor subprocess exit 0 且写 result.json | 读取并返回真实 result |
| 回归测试 | 现有 subprocess 测试 | 运行 test_harbor_subprocess.py | 全部通过 |

## 回滚策略

单次 commit，如引入问题直接 `git revert`。无 schema 变更，无数据迁移。
