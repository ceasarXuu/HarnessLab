# BUG-08: 每次 DB 连接重复 ensure_dirs + PRAGMA

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.2
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: sqlite, settings
- Related Links: [README](README.md), [BUG-10](10-worker-serial-run-execution.md)
- Risk Level: Low
- Plan Type: Lightweight
- Phase: 5（性能优化，配合 BUG-10 并发执行）

## 状态说明

本文档是修复计划，不表示实现已经完成。验收项均为目标状态，只有对应代码和测试落地后才可改为 `[x]`。

## 问题描述

`sqlite.connect` 每次调用都会执行 `settings.ensure_dirs()`，并重复设置 `foreign_keys` 与 `journal_mode=WAL`。高频 service 调用会产生不必要的目录检查和 SQLite 元数据操作。

## 证据

当前连接逻辑：

```python
def connect(settings: Settings) -> sqlite3.Connection:
    settings.ensure_dirs()
    conn = sqlite3.connect(settings.db_path)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA foreign_keys = ON")
    conn.execute("PRAGMA journal_mode = WAL")
    return conn
```

`ensure_dirs()` 会检查并创建多个目录。该操作幂等，但不需要在每个 DB 连接上重复执行。

## 修正后的判断

原方案中“WAL 已由 migration 设置”的说法不准确。当前 migration 文件只建表，没有设置 WAL。整改后应明确：

- `journal_mode=WAL` 是 DB 级设置，应在 `sqlite.initialize(settings)` 阶段设置一次。
- `foreign_keys=ON` 是连接级设置，仍需每次连接设置。
- `busy_timeout=5000` 也是连接级设置，应每次连接设置，以配合 BUG-10 的并行 worker。

## 修复方案

1. 为 `settings.ensure_dirs()` 增加按 `Settings.home` 去重的轻量缓存。
2. `connect()` 保留连接级设置：`foreign_keys=ON` 与 `busy_timeout=5000`。
3. `connect()` 不再重复设置 `journal_mode=WAL`。
4. `initialize()` 在 schema migration 前后显式设置 `journal_mode=WAL`。
5. 测试中使用完整 home path 作为缓存 key，避免多个临时目录互相污染。

示意：

```python
_ensured_dirs: set[str] = set()

# connect(): ensure dirs once per home, then set connection-level pragmas
# initialize(): set WAL once for the database
```

## 与 BUG-10 的关系

BUG-10 引入并行 run 后，SQLite 写事务会串行化。`busy_timeout=5000` 用于降低短时写竞争直接报 `database is locked` 的概率。超时后仍应暴露异常，由 worker 错误处理记录。

## 风险评估

- 单进程 asyncio 场景下，模块级 `_ensured_dirs` 缓存足够。
- 多进程场景下，每个进程各自维护缓存；重复目录检查仍是幂等的。
- 必须确认所有入口都会先完成 `sqlite.initialize(settings)`，否则 WAL 可能未设置。

## Acceptance Criteria（目标，未完成）

- [x] `ensure_dirs` 对同一个 `Settings.home` 不再每次连接都重复执行。
- [x] `PRAGMA journal_mode = WAL` 从 `connect` 移到 DB 初始化阶段。
- [x] `PRAGMA foreign_keys = ON` 每次连接仍正确设置。
- [x] `PRAGMA busy_timeout = 5000` 每次连接设置。
- [x] 测试覆盖不同临时 home，避免缓存污染。
- [x] 现有 storage/experiment/worker 测试无回归。

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | 连接功能 | 多次 connect + 查询 | 每次连接都能正常读写 |
| 正确性验证 | ensure_dirs 去重 | mock ensure_dirs，多次 connect 同一 home | ensure_dirs 只调用一次 |
| 隔离测试 | 多 Settings.home | 两个临时 home 分别 connect | 两者都完成目录初始化 |
| PRAGMA 验证 | foreign_keys/busy_timeout | 新连接查询 PRAGMA | 连接级设置存在 |
| 回归测试 | 相关 python 测试 | 运行 storage/experiment/worker 测试 | 全部通过 |

## 回滚策略

单次代码 commit 可直接 `git revert`。无 schema 变更。
