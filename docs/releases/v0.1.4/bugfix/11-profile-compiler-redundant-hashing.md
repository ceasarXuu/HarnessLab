# BUG-11: ProfileCompiler 重复哈希

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.2
- Status: Draft / Deferred for v0.1.4
- Owner / Responsible: project maintainer
- Related Systems: profile_compiler, agent_service
- Related Links: [README](README.md)
- Risk Level: Low
- Plan Type: Deferred
- Phase: Deferred（v0.1.4 不建议单独修）

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

`ProfileCompiler.compile` 每次调用都会对 profile JSON 做 SHA256 哈希；custom-command profile 还会对生成的 `agent.py` 做文件哈希。同一 profile 被多次 run 使用时，会重复计算相同 hash。

该现象成立，但性能影响极低。原方案提出为 `agents` 表增加 `profile_hash` 列；该方案对 v0.1.4 来说成本偏高，并且不能真正避免 compile 主路径。

## 影响评估

SHA256 计算通常是微秒级。即使 50 个 run 重复计算，总体影响仍远小于 Harbor、Docker 和 benchmark job 的执行成本。优化这个点不会明显改善 v0.1.4 的主要性能问题。

真正的重复工作不只是 hash：

- `_agent_config` 每次仍要读取 profile 文件。
- `compiler.compile(profile)` 仍要构造 agent_config。
- custom-command compile 仍可能重写 generated agent 文件。
- 只缓存 `profile_hash` 不能避免上述主逻辑。

## 原方案的问题

原方案建议添加 `agents.profile_hash` schema migration。该方向存在几个问题：

1. 收益过小：只避免一次 hash，无法避免 compile 主路径。
2. 一致性成本：`AgentService.update` 写入新 profile 后必须清空或重算缓存字段。
3. schema 成本不匹配：为微小性能收益增加 migration，会提高回滚和兼容成本。
4. 优化粒度不完整：如果要优化，应缓存 compiled artifact / agent_config，而不是只缓存 hash。

## 修复决策

v0.1.4 不建议单独修 BUG-11。保留为后续设计项：compiled agent cache。

后续完整方案应考虑：

- `AgentService.compile` 生成并持久化 compiled manifest。
- manifest 记录 `profile_hash`、`agent_config`、generated file path、compiler_version。
- `AgentService.update` 将 agent 状态改回 draft，并使 compiled cache 失效。
- run 时优先读取已编译且 hash 匹配的 artifact；不匹配时提示用户重新 compile。

该方向应作为 agent compile/cache 设计，而不是 v0.1.4 的小型 bugfix。

## Acceptance Criteria（目标，未完成；Deferred）

- [ ] v0.1.4 不新增仅用于 `profile_hash` 的 schema migration。
- [ ] 文档明确单独缓存 hash 收益过低，且不能避免 compile 主路径。
- [ ] 如未来修复，应设计 compiled artifact cache，而不是只缓存 hash。
- [ ] 若引入 cache，`AgentService.update` 必须使 cache 失效。
- [ ] 若引入 cache，必须覆盖 cache 命中、失效和重新 compile 流程。

## 测试计划（未来实现时）

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | compiled cache 命中 | compile agent 后多 run 使用 | 后续 run 复用已编译 artifact |
| 正确性验证 | update 失效 | update agent profile 后运行 | cache 不被误用 |
| 回归测试 | built-in agent | built-in agent compile/run | 行为不变 |
| 回归测试 | custom-command agent | custom-command compile/run | 行为不变 |

## 回滚策略

当前仅是文档降级，无代码变更。若未来实现 compiled cache，应作为独立设计和独立 commit 回滚。
