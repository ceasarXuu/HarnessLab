# BUG-07: 事件镜像 O(n²) 读写

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.1
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: event_service
- Related Links: [README](README.md), [BUG-10](10-worker-serial-run-execution.md)
- Risk Level: Medium
- Plan Type: Standard
- Phase: 5（性能优化，依赖 BUG-10 并行模型）

## 问题描述

`EventService._mirror` 每次追加一条事件时，先读取整个 JSONL 镜像文件的全部内容，
再拼接新行后用 `atomic_write_text` 重写整个文件。一个有 N 条事件的 experiment，
镜像写入总数据量为 O(N²)。长跑实验（数百条事件）会显著变慢。

## 证据

文件: `ornnlab/services/event_service.py` 第 67-80 行

```python
def _mirror(self, event_id, aggregate_type, aggregate_id, ...):
    path = self.settings.experiments_dir / aggregate_id / "ornnlab-events.jsonl"
    ensure_parent(path)
    line = json.dumps({...}, sort_keys=True)
    previous = path.read_text(encoding="utf-8") if path.exists() else ""  # 读全量
    atomic_write_text(Path(path), f"{previous}{line}\n")                  # 写全量
```

文件: `ornnlab/storage/paths.py` 第 14-17 行

```python
def atomic_write_text(path: Path, body: str) -> None:
    ensure_parent(path)
    tmp = path.with_name(f".{path.name}.tmp")
    tmp.write_text(body, encoding="utf-8")  # 写临时文件
    tmp.replace(path)                        # 原子替换
```

性能影响估算（每条事件约 200 字节）：

| 事件数 | 每次追加读写量 | 总 I/O 量 |
|--------|---------------|-----------|
| 10     | ~2 KB         | ~11 KB    |
| 100    | ~20 KB        | ~1 MB     |
| 500    | ~100 KB       | ~25 MB    |
| 1000   | ~200 KB       | ~100 MB   |

## 并发安全评估

BUG-10 引入并行执行后，同一 experiment 的多个 run 可能并发调用
`EventService.append` → `_mirror`，并发追加同一 JSONL 文件。

`open(path, "a")` 在 POSIX 下：
- 多个进程/协程对同一文件追加写入，POSIX 保证每次 `write()` 调用是原子的，
  **但仅限于 `PIPE_BUF`（通常 4096 字节）以内的写入**
- 单条 JSONL 事件通常 200-500 字节，远小于 `PIPE_BUF`
- Python `f.write(line)` 在 CPython 中对应一次 `write()` 系统调用（缓冲区已 flush）

结论：单条事件追加在 `PIPE_BUF` 以内是并发安全的。但为保险起见，使用
`with` 语句确保文件描述符正确关闭，并在写入后 flush：

```python
with path.open("a", encoding="utf-8") as f:
    f.write(f"{line}\n")
    f.flush()
```

## 修复方案

JSONL 格式天然支持追加写入，不需要原子写（单行 append 操作在 PIPE_BUF 以内
是原子的）：

```python
def _mirror(self, event_id, ...):
    path = self.settings.experiments_dir / aggregate_id / "ornnlab-events.jsonl"
    ensure_parent(path)
    line = json.dumps({...}, sort_keys=True)
    with path.open("a", encoding="utf-8") as f:
        f.write(f"{line}\n")
        f.flush()
```

将 O(N²) 降为 O(N)。

## 验收标准

- [x] `_mirror` 使用 `open("a")` 追加写入，不再读取全量文件
- [x] 单条事件追加在 PIPE_BUF 以内是并发安全的
- [x] 1000 条事件的镜像写入总 I/O 从 ~100MB 降为 ~200KB
- [x] 现有测试全部通过，无回归

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | 事件追加 | append 10 条事件，读取 JSONL 文件 | 10 行完整 JSONL，每行可解析 |
| 回归测试 | 事件查询 | append 后 list_after | 事件顺序和内容正确 |
| 性能验证 | 1000 事件写入 | append 1000 条事件，计时 | 总 I/O 线性增长（O(N)），非二次 |
| 并发验证 | 并行追加 | 2 个协程同时 append 50 条到同一 experiment | 100 行完整 JSONL，无损坏行 |

## 回滚策略

单次 commit，`git revert` 即可。无 schema 变更。镜像文件仅为诊断便利，
DB 是 source of truth，即使镜像损坏也不影响数据正确性。
