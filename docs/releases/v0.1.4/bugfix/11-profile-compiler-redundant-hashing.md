# BUG-11: ProfileCompiler 重复哈希

- Created: 2026-06-22
- Updated: 2026-06-22
- Version: 1.1
- Status: Draft
- Owner / Responsible: project maintainer
- Related Systems: profile_compiler, agent_service
- Related Links: [README](README.md)
- Risk Level: Low
- Plan Type: Lightweight
- Phase: 5（性能优化，独立）

## 问题描述

`ProfileCompiler.compile` 每次调用都对 profile JSON 做 SHA256 哈希，对生成的
agent.py 做文件 SHA256 哈希。同一 profile 被多次编译时（例如每次 experiment run
都会调用 `_agent_config` → `compiler.compile`），重复计算相同的哈希值。

## 实际影响评估

SHA256 计算一次约 1-10 微秒（取决于 JSON 大小），50 个 run 重复计算的总开销
约 0.05-0.5 毫秒。性能影响极小，但修复成本低，且方案 B 能提供额外的
完整性校验价值。

## 证据

文件: `ornnlab/services/profile_compiler.py` 第 56-78 行

```python
def compile(self, profile: AgentProfile) -> dict:
    # ...
    return {
        "mode": "built-in",
        "agent_config": {...},
        "manifest": self._manifest(profile, generated_file=None),
    }

def _manifest(self, profile: AgentProfile, generated_file: Path | None) -> dict:
    profile_json = profile.model_dump_json(exclude_none=True)
    generated_hash = None
    if generated_file is not None and generated_file.exists():
        generated_hash = hashlib.sha256(generated_file.read_bytes()).hexdigest()
    return {
        "profile_id": profile.id,
        "profile_hash": hashlib.sha256(profile_json.encode("utf-8")).hexdigest(),
        # ...
    }
```

调用链：`experiment_service._run_one` → `_agent_config(run["agent_id"])` →
`compiler.compile(profile)` → `_manifest()` → SHA256 计算。

一个 experiment 有 N 个 run 使用同一 agent 时，profile JSON 被哈希 N 次，
结果完全相同。

## 修复方案

方案 B（预计算，推荐）：在 agent compile 阶段（`AgentService.compile`）一次性
计算并持久化 profile_hash，run 时直接读取：

```python
# AgentService.compile 时写入 hash
def compile(self, agent_id: str) -> dict:
    # ...
    result = self.compiler.compile(profile)
    profile_hash = result["manifest"]["profile_hash"]
    with sqlite.connect(self.settings) as conn:
        conn.execute(
            "UPDATE agents SET status = ?, harbor_import_path = ?, "
            "profile_hash = ?, updated_at = ? WHERE id = ?",
            ("compiled", result["agent_config"].get("import_path"),
             profile_hash, now_iso(), agent_id),
        )
    # ...
```

需要 schema migration 添加 `profile_hash` 列：

```sql
-- migrations/004_agent_profile_hash.sql
ALTER TABLE agents ADD COLUMN profile_hash TEXT;
```

run 时从 DB 读取已存储的 hash，跳过重复计算：

```python
# experiment_service._agent_config
def _agent_config(self, agent_id: str) -> dict:
    with sqlite.connect(self.settings) as conn:
        rows = sqlite.rows(conn, "SELECT profile_path, profile_hash FROM agents WHERE id = ?", (agent_id,))
    # ...
    if rows[0]["profile_hash"]:
        # 使用已存储的 hash，跳过重复计算
        result = self.compiler.compile(profile, precomputed_hash=rows[0]["profile_hash"])
    else:
        result = self.compiler.compile(profile)
    return result["agent_config"]
```

方案 B 更符合"编译一次、多次执行"的模型，且能检测 profile 被篡改的情况
（run 时 profile 文件的 hash 与 DB 中存储的不一致则报警）。

## 验收标准

- [x] `agents` 表新增 `profile_hash` 列（schema migration 004）
- [x] `AgentService.compile` 计算 hash 并持久化到 DB
- [x] `_agent_config` 从 DB 读取 hash，跳过重复计算
- [x] 现有测试全部通过，无回归

## 测试计划

| 测试类型 | 测试项 | 方法 | 通过标准 |
|----------|--------|------|----------|
| 正确性验证 | hash 持久化 | compile agent 后检查 DB | profile_hash 列有值 |
| 正确性验证 | run 时复用 | 同一 agent 跑 2 个 run，检查 compile 调用次数 | compile 只调用 1 次（第 2 次 run 复用 hash） |
| 回归测试 | 现有 agent 测试 | 运行 test_agent_api.py | 全部通过 |
| 迁移测试 | schema migration | 从旧 DB 启动，检查 migration 004 | profile_hash 列存在，值为 NULL |

## 回滚策略

schema migration 004 添加的列不影响现有功能。如需回滚：
1. `git revert` 代码变更
2. `ALTER TABLE agents DROP COLUMN profile_hash`（SQLite 3.35+ 支持）
3. 或者保留该列不删除（空列不影响功能）
