# Harbor 升级兼容性分析与升级方案（v0.13.x → v0.21.0）

## Document Control

| 文档版本 | 工程版本 | 更新 | 变更 |
|---|---|---|---|
| 1.0 | ornnlab v1.1.0 | 2026-08-19 | 新建：Harbor 0.13.2 → 0.21.0 升级兼容分析与方案 |
| 1.1 | ornnlab v1.1.0 | 2026-08-19 | 实施完成：依赖升级、兼容改造、门禁与真实 Docker 冒烟证据（见第 8 节） |

## 1. 现状盘点

- 依赖声明：`pyproject.toml` 中 `harbor>=0.13,<0.14`，当前安装 **0.13.2**（2026-06-11）。
- 运行时依赖面（按代码引用盘点）：
  - **CLI 子进程**：`harbor run --config`（`harbor_subprocess.py`）、`harbor job resume --job-path`（`webui_job_service.py:346`）、`harbor cache clean --force --no-cache-dir`（`webui_system_service.py:88`）。
  - **Python API**：`DockerEnvironment` 子类化并覆写私有方法（`owned_docker_environment.py`）、`TaskConfig/TaskPaths/Task`、`AgentName/AgentConfig/EnvironmentConfig/ResourceMode`、`EnvironmentType`、`AgentFactory`、`RegistryClientFactory`、`sync_dataset`、`auth.handler`。
  - **数据面**：`harbor.config.json`、`result.json`（legacy 与 native 两种布局）、job 目录、`trial/result.json`、`harbor.cleanup.json`、`harbor.capability.json`。

## 2. 上游版本态势

| 版本 | 发布日期 | 备注 |
|---|---|---|
| 0.13.2 | 2026-06-11 | 当前固定版本 |
| 0.14.0 → 0.15.0 | 2026-06-17 → 06-19 | 功能与修复 |
| 0.16.0 / 0.16.1 | 2026-06-27 / 06-28 | 并发限制、agent-end hooks 等 |
| 0.17.0 / 0.17.1 | 2026-07-03 | trial lock schema、exec 参数改名 |
| 0.18.0 | 2026-07-07 | **认证重构**（GoTrue → 个人 API Key） |
| 0.19.0 | 2026-07-17 | 功能与修复 |
| 0.20.0 | 2026-07-18 | `--json` 输出改为纯 JSON 等 |
| **0.21.0** | 2026-08-10 | **升级目标（最新稳定版）** |

另存在 `0.21.1.dev*` 每日构建（不稳定），不纳入升级目标。

## 3. 兼容性实测（隔离环境安装 `harbor==0.21.0` 验证）

| ornnlab 依赖面 | 0.21.0 实测 | 说明 |
|---|---|---|
| CLI `harbor run --config` | ✅ | 仍为 `harbor job start` 别名，`--config` 参数保留 |
| CLI `harbor job resume --job-path` | ✅ | 参数与语义保留 |
| CLI `harbor cache clean --force --no-cache-dir` | ✅ | 参数保留 |
| `harbor.environments.docker.docker`：`DockerEnvironment`、`_sanitize_docker_compose_project_name`、`_docker_compose_paths`、`_run_docker_compose_command` | ✅ | 私有 API 仍在 |
| `harbor.models.task.task.Task` / `task.config.TaskConfig` / `task.paths.TaskPaths` | ✅ | 模块与属性可用 |
| `harbor.models.agent.name.AgentName`（含 `.values()`） | ✅ | pydantic Enum 保留 |
| `harbor.models.trial.config.AgentConfig / EnvironmentConfig / ResourceMode` | ✅ | pydantic 模型保留 |
| `harbor.models.environment_type.EnvironmentType` | ✅ | 保留 |
| `harbor.agents.factory.AgentFactory` | ✅ | 保留 |
| `harbor.registry.client.factory.RegistryClientFactory` | ✅ | 保留 |
| `harbor.cli.sync.sync_dataset` | ✅ | 保留 |
| **`harbor.auth.handler`（`get_auth_handler` / `AuthHandler`）** | ❌ | **0.18.0 移除，模块不存在** |

## 4. 硬破坏点与改造方案

### 4.1 `harbor.auth.handler` 移除（0.18.0 认证重构）

- **现状代码**：`ornnlab/services/webui_system_service.py:41`：
  ```python
  from harbor.auth.handler import get_auth_handler
  authenticated = await (await get_auth_handler()).is_authenticated()
  ```
- **影响**：升级后触发 `ImportError`，被现有 `try/except Exception` 吞掉，`hub_connection` 静默返回 `{"status": "disconnected"}`。按文档规则禁止静默 fallback，必须显式改造。
- **改造方案**：Harbor 0.21.0 认证模型为个人 API Key，新增探测接口：
  ```python
  from harbor.auth.credentials import read_stored_credentials
  authenticated = read_stored_credentials() is not None
  ```
  保持返回契约不变（`{"status": "connected" | "disconnected"}`），并补充未认证/已认证两种状态的单元测试。
- **待确认**：是否同时在 doctor / system health 中展示 Harbor 认证状态（纳入 v1.1.0 工程计划确认）。

### 4.2 其余变更（当前 ornnlab 未受影响，仅记录）

| Harbor 版本 | 变更 | 影响判断 |
|---|---|---|
| 0.16.0 | `--agent-import-path` 并入 `--agent` | ornnlab 未使用该 CLI 参数，无影响 |
| 0.17.1 | `harbor exec` 输出参数改名 `--tasks-dir` | ornnlab 未使用 `harbor exec`，无影响 |
| 0.20.0 | `--json` 输出改为纯 JSON（非 Rich 样式） | ornnlab 解析 result.json 文件而非 CLI `--json`，无影响 |
| 0.17.0 | job lock 重构 + trial lock 新增 `schema_version` | ornnlab 结果解析已兼容 legacy/native 两种布局，需在真实 job 冒烟中复核 |

## 5. 值得纳入的增量能力（0.21.0）

- `harbor job regrade` / `harbor trial regrade`：对已记录 trials 重新跑验证，与 ornnlab rerun-failed / 结果修复场景关联。
- leaderboard 创建支持按 dataset 版本过滤（`harbor hub leaderboard create`）。
- `harbor version` 包版本检视命令，可强化 doctor 的 Harbor 版本诊断。
- `harbor auth status` CLI：可作为 hub_connection 的 CLI 级探测替代。

> 是否在 v1.1.0 接入上述能力由工程计划确认；本专题文档不单方面扩大升级范围。

## 6. 升级步骤（遵循既有升级流程）

1. 确认升级范围与目标约束（建议 `harbor>=0.21,<0.22`），记入工程计划。
2. 更新 `pyproject.toml` 依赖约束。
3. 执行 `uv lock` 并核对锁定版本为 0.21.0。
4. 改造 `webui_system_service.hub_connection` 认证探测（见 4.1），补单测。
5. 运行 Harbor API 兼容性测试：`uv run pytest tests/python/test_harbor_engine.py tests/python/test_profile_compiler.py -vv`。
6. 运行全量本地门禁：`scripts/test-after-change-web.sh`。
7. 在 Docker 环境运行真实 Harbor 冒烟：`ORNNLAB_REAL_HARBOR=1 uv run pytest -m docker tests/python/test_real_harbor_cancel_recovery.py`。
8. 检查产出 job 目录：`config.json`、`result.json`、`job.log`、trial 产物与 job lock 布局是否符合解析器预期。
9. 更新 `docs/architecture/technology-decisions.md`、本专题文档与工程计划台账（版本、证据、遗留项）。

## 7. 风险与回滚

- **认证探测回归**：`hub_connection` 若沿用旧 API 会静默降级；改造后需单测覆盖，且不允许吞异常。
- **result / lock 布局漂移**：0.17.0 起的 lock 变化需真实 job 冒烟复核；若取消/结果解析/目录布局发生变化，按既有失败策略停止升级、保留原 pin，等待专项兼容补丁。
- **回滚**：遵循 `docs/releases/v0.1.3/checklist.md` 回滚章节（停止后端 → 备份 → 记录归档 → 版本回退 → `doctor --logs` 验证 → 必要时 `backup import` 恢复），并还原 `pyproject.toml` 与 `uv.lock`。

## 8. 实施记录与验证证据

| 实施项 | 变更/证据 |
|---|---|
| 依赖约束 | `pyproject.toml`：`harbor>=0.13,<0.14` → `harbor>=0.21,<0.22`；`uv lock` 已锁定 `harbor 0.21.0` |
| `hub_connection` 认证探测 | `webui_system_service.py:39`：改用 `harbor.auth.credentials.read_stored_credentials()`；不再吞异常静默降级 |
| `AgentFactory` 兼容 | `agent_capabilities.py:_harbor_agent_class`：`AgentFactory._AGENTS`（0.13 已移除）→ `AgentName(harness)` + `AgentFactory.get_agent_class(name)` |
| `DockerEnvironment` 覆写 | `owned_docker_environment.py:_run_docker_compose_command`：补齐 Harbor 0.21 新增的 `stdin_data` / `on_output` 参数并透传 |
| claude-code 能力变化 | `max_thinking_tokens` 从参数迁移为 `MAX_THINKING_TOKENS` 环境变量（Harbor 0.21 上游变更），`tests/python/test_webui_api.py` 断言已同步 |
| 新增单测 | `tests/python/test_system_api.py`：`hub_connection` 已认证/未认证两种状态 |

### 门禁结果（2026-08-19，本地）

- pytest（非 docker）：**248 passed / 4 deselected**
- 真实 Harbor Docker 冒烟：`test_harbor_real_smoke.py` 1 passed、`test_real_harbor_cancel_recovery.py` 2 passed、`test_real_docker_ownership_cleanup.py` 1 passed
- ruff：All checks passed；pyright：0 errors / 0 warnings
- 前端：Vitest 135 passed、ESLint、`tsc --noEmit` 全绿

### 已知遗留（非阻断）

- Harbor 0.21 对 `memory`/`storage` → `memory_mb`/`storage_mb` 与 `Task.checksum` 发出弃用告警；ornnlab 已使用新字段（`dataset_environment.py:66-67`），仅 Harbor 内部自身告警，无需改动。

## 9. 待办清单

- [x] 更新 `pyproject.toml` + `uv lock`（锁定 0.21.0）
- [x] 改造 `hub_connection` 认证探测 + 单测
- [x] 修复 `AgentFactory` / `DockerEnvironment` 兼容
- [x] 跑 Harbor 兼容性测试与全量门禁
- [x] Docker 真实 Harbor 冒烟与产物复核
- [x] 更新技术决策记录、升级流程、安装快速入门与工程台账
- [ ] v1.1.0 工程计划确认是否接入 0.21.0 增量能力（regrade / leaderboard 版本过滤 / `harbor version`）
