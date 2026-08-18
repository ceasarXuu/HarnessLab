# Harbor 升级兼容（v1.1.0）

本专题负责 v1.1.0 将 Harbor 运行时依赖从 `0.13.x`（0.13.2）升级到最新稳定版 `0.21.0` 的兼容性分析与升级执行。

- [兼容性分析与升级方案](compatibility-analysis.md)：版本态势、实测兼容矩阵、硬破坏点、升级步骤、验证门禁与回滚。
- 升级门禁与失败策略以既有 [Harbor 升级流程](../../../playbooks/harbor-upgrade-procedure.md) 为准。
- 历史 Harbor 固定边界（`>=0.13,<0.14`）依据见 [v1.0.5 发布笔记](../../v1.0.5/ornnlab-1.0.5.md)。

## 主题状态

| 项目 | 值 |
|---|---|
| 状态 | Implemented（已完成兼容改造与门禁验证） |
| 更新 | 2026-08-19 |
| 源版本 | Harbor `0.13.2`（2026-06-11） |
| 目标版本 | Harbor `0.21.0`（2026-08-10，最新稳定版） |
| 目标依赖约束 | `harbor>=0.21,<0.22`（已更新 `pyproject.toml` 并 `uv lock`） |

## 关键结论

- Harbor 上游已从 0.13.2 推进 8 个 minor 版本至 0.21.0；CLI 与 Python API 绝大部分向后兼容（已在隔离环境实测 0.21.0）。
- 唯一硬破坏点：`harbor.auth.handler` 在 0.18.0 认证重构中被移除（GoTrue 会话 → 个人 API Key），`webui_system_service.hub_connection` 必须显式改写，禁止依赖现有 try/except 静默降级为 disconnected。
- 0.21.0 新增 `harbor job/trial regrade`、leaderboard dataset 版本过滤、`harbor version` 等能力，与 ornnlab 的 rerun / leaderboard 场景直接相关，纳入 v1.1.0 评估。

## 文档规则

- 升级范围（是否改 pin、是否改造 hub_connection、是否接入 regrade）先在工程计划中确认，再执行实现。
- 兼容性结论必须可追溯：本文档中的结论均来自隔离环境（`harbor==0.21.0`）实测，不写猜测性断言。
- 不添加静默兼容 fallback；任何版本相关行为都必须显式实现并配测试与 doctor 诊断。
