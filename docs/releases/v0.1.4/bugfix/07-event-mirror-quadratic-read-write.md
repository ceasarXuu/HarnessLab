# BUG-07: 事件镜像 O(n²) 读写

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.2
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: event_service
- Related Links: [README](README.md), [BUG-10](10-worker-serial-run-execution.md)
- Risk Level: Medium
- Plan Type: Standard
- Phase: 5（性能优化，依赖 BUG-10 并行模型）

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

`EventService._mirror` 每次追加一条事件时，先读取整个 JSONL mirror 文件，再拼接新行后重写整个文件。一个有 N 条事件的 experiment，mirror 写入总 I/O 为 O(N²)。长跑实验或高频事件会放大本地文件 I/O。

## 证据

文件: `ornnlab/services/event_service.py`

```python
previous = path.read_text(encoding="utf-8") if path.exists() else ""
atomic_write_text(Path(path), f"{previous}{line}\n")
```

问题点：

- 每次 append 都读全量历史内容。
- 每次 append 都写全量历史内容。
- JSONL 本身适合按行追加。
- DB 表 `experiment_events` 才是 source of truth，mirror 是诊断副本。

## 修复方案

改为按行追加：

```python
with path.open("a", encoding="utf-8") as handle:
    handle.write(f"{line}\n")
    handle.flush()
```

这样每条事件只写一行，整体复杂度从 O(N²) 降为 O(N)。

## 并发安全说明

原文中用 `PIPE_BUF` 证明普通文件 append 并发安全，这个表述需要删除。`PIPE_BUF` 是 pipe/FIFO 相关语义，不应直接作为普通文件并发 append 的保证。

v0.1.4 可接受的约束是：

- 默认运行形态是本地单用户、单进程 asyncio WebUI。
- `_mirror` 中没有 `await`，单次同步写入在当前进程内不会被协程切分。
- DB 是事件 source of truth；API 和 SSE 不应依赖 JSONL mirror。
- 如果未来支持多进程或多线程并发写同一 mirror 文件，需要显式引入 per-path lock 或文件锁。

## 性能影响估算

| 事件数 | 当前重写总量约 | 追加写总量约 |
|--------|----------------|--------------|
| 100    | ~1 MB          | ~20 KB       |
| 500    | ~25 MB         | ~100 KB      |
| 1000   | ~100 MB        | ~200 KB      |

## Acceptance Criteria（目标，未完成）

- [x] `_mirror` 使用 JSONL 追加写，不再读取并重写全量文件。
- [x] 文档和代码注释不再用 `PIPE_BUF` 作为普通文件 append 的并发安全依据。
- [x] DB 仍是 event source of truth；SSE 和 API 查询不依赖 mirror 文件。
- [x] 单进程并发追加测试下，JSONL 行完整且可解析。
- [x] 1000 条事件 mirror 写入表现为线性增长。
- [x] 现有 event service 测试无回归。

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | 事件追加 | append 10 条事件，读取 JSONL | 10 行完整 JSONL，每行可解析 |
| 回归测试 | 事件查询 | append 后 list_after | DB 顺序和内容正确 |
| 性能验证 | 1000 事件写入 | append 1000 条事件并统计文件大小 | 近似 O(N)，非 O(N²) |
| 并发验证 | 单进程并发追加 | 多协程 append 同一 aggregate | 无损坏行；DB 记录完整 |

## 回滚策略

单次代码 commit 可直接 `git revert`。mirror 文件只是诊断副本，DB 不受影响。
