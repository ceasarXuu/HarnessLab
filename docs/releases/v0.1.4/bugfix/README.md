# v0.1.4 Bugfix 修复计划

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.1
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: ornnlab backend, worker, queue, event, storage
- Risk Level: High
- Plan Type: Standard

## 状态说明

本目录是 v0.1.4 的修复计划和设计审查清单，不表示实现已经落地。各文档中的验收项均使用未勾选的 `[ ]`，表示目标验收标准；只有对应代码和测试提交后才应改为 `[x]`。

本次整改仅调整 bugfix 文档和执行计划，不修改运行时代码。

## 问题总览

| # | 文档 | 级别 | 类型 | 涉及文件 | 当前处理建议 |
|---|------|------|------|----------|--------------|
| 01 | [TOCTOU 竞态](01-toctou-cancel-overwrites-completed-run.md) | 高 | 正确性 | experiment_service.py | 修正 SQL 方案，保留执行事实但不覆盖 cancelled 终态 |
| 02 | [崩溃恢复盲区](02-crash-recovery-blind-spot-dequeue-to-running.md) | 高 | 正确性 | queue_service.py, experiment_service.py, recovery_service.py | 修复 dequeue 原子性，并修正 orphan recovery 计数 |
| 03 | [wait=true 等待无关 experiment](03-wait-true-blocks-unrelated-experiments.md) | 中 | 正确性 | experiments.py, worker_service.py | 改为等待指定 experiment terminal，不依赖全局 worker idle |
| 04 | [SSE 不工作](04-sse-stream-not-realtime.md) | 中 | 正确性 | experiments.py, event_service.py | 增加轮询流、run 级事件合并和终止竞态保护 |
| 05 | [subprocess 合成 completed](05-subprocess-synthesizes-completed-without-result.md) | 低 | 正确性 | harbor_subprocess.py | 先验证 Harbor 行为，再决定状态策略 |
| 06 | [状态派生重复](06-duplicated-inconsistent-status-derivation.md) | 低 | 正确性 | experiment_utils.py, recovery_service.py | 低风险，优先统一到 experiment_utils |
| 07 | [事件镜像 O(n²)](07-event-mirror-quadratic-read-write.md) | 中 | 性能 | event_service.py | 改追加写，但并发安全论证需从 PIPE_BUF 改为锁/单进程约束 |
| 08 | [DB 连接重复开销](08-db-connect-repeated-ensure-dirs-pragma.md) | 中 | 性能 | sqlite.py, settings.py | WAL 一次性设置应落在 initialize，不声称已有 migration |
| 09 | [Worker 重建 Service](09-worker-recreates-experiment-service-per-run.md) | 中 | 性能/架构 | worker_service.py | 重新表述为执行上下文隔离，不夸大实例创建收益 |
| 10 | [Worker 串行执行](10-worker-serial-run-execution.md) | 高 | 性能/架构 | worker_service.py, settings.py, sqlite.py | 补 task 异常消费、并发配置校验和 scoped dequeue 签名 |
| 11 | [Profile 重复哈希](11-profile-compiler-redundant-hashing.md) | 低 | 性能 | profile_compiler.py, agent_service.py | 影响极低，v0.1.4 建议推迟；若修应做 compiled artifact cache |

## 跨文档冲突

| 文档对 | 冲突类型 | 说明 | 整改决策 |
|--------|----------|------|----------|
| BUG-09 ↔ BUG-10 | 架构冲突 | BUG-09 原先强调复用 ExperimentService，BUG-10 并行执行需要隔离上下文 | 以 BUG-10 并行为主；BUG-09 改为减少调度层重复创建和明确执行上下文边界 |
| BUG-07 ↔ BUG-10 | 并发安全 | BUG-10 引入并行后，BUG-07 的 JSONL mirror append 需要并发约束 | 文档不再使用 PIPE_BUF 作为普通文件保证；未来多进程需加 per-path lock 或文件锁 |
| BUG-01 ↔ BUG-02 | 同文件修改 | 都修改 experiment_service.py 的 `_run_one` / queue 流程 | 先修 dequeue/recovery，再修 cancel/result 竞态 |
| BUG-03 ↔ BUG-10 | worker 语义 | BUG-03 不能通过等待全局 worker idle 实现 scoped wait | BUG-03 改为等待指定 experiment 的 run 状态达到 terminal |
| BUG-05 ↔ 外部 Harbor | 外部依赖 | 是否缺失 result.json 属异常，取决于 Harbor 成功路径契约 | 标记为需要前置验证，不直接作为确定性修复落地 |
| BUG-08 ↔ BUG-10 | SQLite 并发 | 并行 worker 需要 busy_timeout；WAL 只需一次性配置 | `foreign_keys` 和 `busy_timeout` 保持连接级；`journal_mode=WAL` 移到 initialize |
| BUG-11 ↔ Agent compile 模型 | 优化粒度 | 只缓存 hash 无法避免 compile 主逻辑，且会引入陈旧 hash 风险 | 推迟；若做，应缓存编译产物而非单独 schema 化 profile_hash |

## 执行顺序

```text
Phase 0: 文档状态修正
  全部 Acceptance Criteria 统一为未完成目标项，避免 Draft 文档被误读为已修复

Phase 1: 低风险正确性
  BUG-06 (状态派生统一) → 无依赖，优先落地
  BUG-05 (subprocess 状态) → 先完成 Harbor 行为验证，再决定代码变更

Phase 2: 崩溃恢复
  BUG-02 (dequeue 原子性 + orphan recovery) → 依赖 BUG-06 的统一状态定义

Phase 3: 竞态修复
  BUG-01 (TOCTOU) → 依赖 BUG-02 的 queue 一致性；SQL 方案必须保留执行事实

Phase 4: Worker 架构
  BUG-10 (有界并行) → 主线修复
  BUG-03 (wait scoped terminal) → 与 BUG-10 的 scoped dequeue/status polling 协调
  BUG-09 (执行上下文边界) → 作为 BUG-10 的架构说明，不单独夸大性能收益

Phase 5: 事件与 DB 性能
  BUG-07 (JSONL append) → 依赖 BUG-10 的并发模型说明
  BUG-08 (DB connect/WAL/busy_timeout) → 配合 BUG-10 并发写竞争

Phase 6: SSE
  BUG-04 (实时事件流) → 依赖 BUG-07 的事件系统稳定；需合并 experiment + run events

Deferred:
  BUG-11 (Profile 重复哈希) → v0.1.4 不建议单独 schema 化 profile_hash
```

## Phase 依赖关系图

```text
Phase 0
┌──────────────────────┐
│ 文档状态与验收项修正 │
└──────────┬───────────┘
           ▼
Phase 1                Phase 2           Phase 3
┌──────────┐      ┌──────────┐     ┌──────────┐
│ BUG-06   │─────▶│ BUG-02   │────▶│ BUG-01   │
│ BUG-05*  │      └──────────┘     └────┬─────┘
└──────────┘                             │
                                         ▼
Phase 4           ┌──────────────────────────────┐
┌──────────┐      │  BUG-10 主线 + BUG-09 说明   │
│ BUG-03   │◀─────┤  有界并行 / scoped wait      │
└──────────┘      └──────────────┬───────────────┘
                                 │
                    ┌────────────┴────────────┐
                    ▼                         ▼
Phase 5        ┌──────────┐              ┌──────────┐
               │ BUG-07   │              │ BUG-08   │
               └────┬─────┘              └──────────┘
                    │
                    ▼
Phase 6        ┌──────────┐
               │ BUG-04   │
               └──────────┘

Deferred       ┌──────────┐
               │ BUG-11   │
               └──────────┘

* BUG-05 先做 Harbor 行为验证，验证不通过则重新设计。
```
