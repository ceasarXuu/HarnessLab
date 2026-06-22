# v0.1.4 Bugfix 修复计划

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.0
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: ornnlab backend, worker, queue, event, storage
- Risk Level: High
- Plan Type: Standard

## 问题总览

| # | 文档 | 级别 | 类型 | 涉及文件 |
|---|------|------|------|----------|
| 01 | [TOCTOU 竞态](01-toctou-cancel-overwrites-completed-run.md) | 高 | 正确性 | experiment_service.py |
| 02 | [崩溃恢复盲区](02-crash-recovery-blind-spot-dequeue-to-running.md) | 高 | 正确性 | queue_service.py, experiment_service.py, recovery_service.py |
| 03 | [wait=true 等待无关 experiment](03-wait-true-blocks-unrelated-experiments.md) | 中 | 正确性 | experiments.py, worker_service.py |
| 04 | [SSE 不工作](04-sse-stream-not-realtime.md) | 中 | 正确性 | experiments.py |
| 05 | [subprocess 合成 completed](05-subprocess-synthesizes-completed-without-result.md) | 低 | 正确性 | harbor_subprocess.py |
| 06 | [状态派生重复](06-duplicated-inconsistent-status-derivation.md) | 低 | 正确性 | experiment_utils.py, recovery_service.py |
| 07 | [事件镜像 O(n²)](07-event-mirror-quadratic-read-write.md) | 高 | 性能 | event_service.py |
| 08 | [DB 连接重复开销](08-db-connect-repeated-ensure-dirs-pragma.md) | 中 | 性能 | sqlite.py, settings.py |
| 09 | [Worker 重建 Service](09-worker-recreates-experiment-service-per-run.md) | 中 | 性能 | worker_service.py |
| 10 | [Worker 串行执行](10-worker-serial-run-execution.md) | 中 | 性能 | worker_service.py |
| 11 | [Profile 重复哈希](11-profile-compiler-redundant-hashing.md) | 低 | 性能 | profile_compiler.py |

## 跨文档冲突

| 文档对 | 冲突类型 | 说明 |
|--------|----------|------|
| BUG-09 ↔ BUG-10 | 架构冲突 | BUG-09 要复用 ExperimentService，BUG-10 要并行执行需独立实例。合并为 Phase 4 联合修复 |
| BUG-07 ↔ BUG-10 | 并发安全 | BUG-10 引入并行后，BUG-07 的 append 修复需考虑并发写入 |
| BUG-01 ↔ BUG-02 | 同文件修改 | 都修改 experiment_service.py 的 _run_one / queue 流程，需协调 |
| BUG-03 ↔ BUG-10 | worker 语义 | BUG-03 要按 experiment 过滤，BUG-10 要全局并行，worker 调度模型需统一设计 |

## 执行顺序

```
Phase 1: 正确性基础（无冲突）
  BUG-06 (状态派生统一) → 无依赖
  BUG-05 (subprocess 状态) → 无依赖

Phase 2: 崩溃恢复（依赖 BUG-06 的状态定义）
  BUG-02 (dequeue 原子性) → 依赖 BUG-06

Phase 3: 竞态修复（依赖 BUG-02 的 queue 一致性）
  BUG-01 (TOCTOU) → 依赖 BUG-02

Phase 4: Worker 架构（BUG-09 和 BUG-10 联合设计）
  BUG-09 + BUG-10 (联合修复) → 依赖 BUG-01, BUG-02
  BUG-03 (wait 过滤) → 依赖 worker 架构定型

Phase 5: 性能优化（依赖 worker 架构）
  BUG-07 (事件追加) → 依赖 BUG-10 的并行模型
  BUG-08 (DB 连接) → 独立
  BUG-11 (哈希缓存) → 独立

Phase 6: SSE（依赖事件系统稳定）
  BUG-04 (SSE 实时流) → 依赖 BUG-07
```

## Phase 依赖关系图

```
Phase 1                Phase 2           Phase 3
┌──────────┐      ┌──────────┐     ┌──────────┐
│ BUG-06   │─────▶│ BUG-02   │────▶│ BUG-01   │
│ BUG-05   │      └──────────┘     └────┬─────┘
└──────────┘                             │
                                         ▼
Phase 4           ┌──────────────────────────────┐
┌──────────┐      │  BUG-09 + BUG-10 (联合修复)  │
│ BUG-03   │◀─────┤                              │
└──────────┘      └──────────────┬───────────────┘
                                 │
                    ┌────────────┼────────────┐
                    ▼            ▼            ▼
Phase 5        ┌──────────┐ ┌──────────┐ ┌──────────┐
               │ BUG-07   │ │ BUG-08   │ │ BUG-11   │
               └────┬─────┘ └──────────┘ └──────────┘
                    │
                    ▼
Phase 6        ┌──────────┐
               │ BUG-04   │
               └──────────┘
```
