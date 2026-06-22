# BUG-08: 每次 DB 连接重复 ensure_dirs + PRAGMA

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.1
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: sqlite, settings
- Related Links: [README](README.md)
- Risk Level: Low
- Plan Type: Lightweight
- Phase: 5（性能优化，独立）

## 问题描述

`sqlite.connect` 每次被调用时都执行 `settings.ensure_dirs()`（7 次 `mkdir` 系统调用）
和两条 PRAGMA 语句。每个 service 方法都创建新连接，高频操作下产生大量不必要的
系统调用和 SQLite 元数据操作。

## 证据

文件: `ornnlab/storage/sqlite.py` 第 11-16 行

```python
def connect(settings: Settings) -> sqlite3.Connection:
    settings.ensure_dirs()                        # 7 次 mkdir 系统调用
    conn = sqlite3.connect(settings.db_path)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA foreign_keys = ON")
    conn.execute("PRAGMA journal_mode = WAL")     # WAL 是持久设置，只需设一次
    return conn
```

文件: `ornnlab/settings.py` 第 63-74 行

```python
def ensure_dirs(self) -> None:
    for path in [
        self.home,           # mkdir
        self.logs_dir,       # mkdir
        self.agents_dir,     # mkdir
        self.generated_agents_dir,  # mkdir
        self.experiments_dir,       # mkdir
        self.exports_dir,           # mkdir
        self.archive_dir,           # mkdir
    ]:
        path.mkdir(parents=True, exist_ok=True)   # 7 次 stat + mkdir
```

单次 experiment run 流程中 `ensure_dirs` 至少被调用 8-10 次，每次 7 个 `mkdir` 系统调用。

WAL 模式的额外问题：`PRAGMA journal_mode = WAL` 是数据库级别的持久设置，
设置一次后后续连接无需重复设置。重复执行会产生不必要的 SQLite 内部锁和 I/O。

## 修复方案

在 `app.py` 的 `create_app` 中调用一次 `settings.ensure_dirs()` 和
`sqlite.initialize()`，`connect` 只负责创建连接和设置 `foreign_keys`：

```python
# sqlite.py
_ensured_dirs: set[str] = set()

def connect(settings: Settings) -> sqlite3.Connection:
    key = str(settings.db_path)
    if key not in _ensured_dirs:
        settings.ensure_dirs()
        _ensured_dirs.add(key)
    conn = sqlite3.connect(settings.db_path)
    conn.row_factory = sqlite3.Row
    conn.execute("PRAGMA foreign_keys = ON")
    conn.execute("PRAGMA busy_timeout = 5000")  # 配合 BUG-10 并行执行
    return conn
```

注意：`PRAGMA journal_mode = WAL` 已在 `sqlite.initialize` 中通过 migration
脚本设置，`connect` 中无需重复。`PRAGMA foreign_keys = ON` 是连接级设置，
每次连接必须设置。`PRAGMA busy_timeout` 也是连接级设置。

## 线程安全评估

模块级 `_ensured_dirs` set 在 asyncio 单线程环境下安全。uvicorn 默认单进程
单线程运行 asyncio 事件循环，不会多线程访问此 set。如果未来切换到多进程模式
（如 gunicorn workers），每个进程有独立的 `_ensured_dirs`，也安全。

## 验收标准

- [x] `ensure_dirs` 每个 db_path 只调用一次（通过 `_ensured_dirs` 去重）
- [x] `PRAGMA journal_mode = WAL` 不在 `connect` 中重复设置
- [x] `PRAGMA foreign_keys = ON` 每次连接仍正确设置
- [x] `PRAGMA busy_timeout = 5000` 新增设置（配合 BUG-10）
- [x] 现有测试全部通过，无回归

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | 连接功能 | 多次 connect + 查询 | 每次连接都能正常读写 |
| 正确性验证 | ensure_dirs 去重 | mock ensure_dirs，多次 connect | ensure_dirs 只被调用 1 次 |
| 回归测试 | 现有测试 | 运行全套 python 测试 | 全部通过 |

## 回滚策略

单次 commit，`git revert` 即可。`_ensured_dirs` 是纯优化缓存，回滚后恢复
每次 ensure_dirs 的行为，功能不变。
