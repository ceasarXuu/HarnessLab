# OrnnLab v1.0.5 发布笔记

- 发布版本：v1.0.5（Build Set）
- 发布状态：Released
- 发布日期：2026-08-17
- 源码提交：`main` @ `8deec52`（release/v1.0.5 分支合入后打 tag `v1.0.5`）

## Build Set 组成表

| 组件 | 版本 | 说明 |
|---|---|---|
| `ornnlab` npm launcher（`package.json`） | 1.0.5 | 用户安装载体 |
| Python 应用包（`pyproject.toml` + `ornnlab/__init__.py` + `app.py` FastAPI） | 1.0.5 | 后端统一对齐（此前 pyproject 0.2.0 与 app.py 0.3.0 存在漂移） |
| 私有前端包（`frontend/package.json`） | 1.0.5 | 三端一致可追溯 |
| transition 兼容包（`npm/harnesslab-transition`） | 0.1.2（不变） | 仅在 transition/deprecation 发布时变更 |
| Harbor | `>=0.13,<0.14`（不变） | 外部运行时依赖范围 |

## 版本能力摘要

v1.0.5 将 OrnnLab 建设为基于 Harbor 的本地实验控制台（WebUI），核心能力：

- 唯一产品 API `/api/webui/v1`：Jobs、Datasets、Agents、Environments、Leaderboard、System、Hub、Operation 全资源契约。
- 本地 Job 全生命周期：创建（Agent 配置 + 模型单选 + 价格快照）、实时进度（任务分类 + 秒表）、终态完整删除、cancel/resume/rerun-failed。
- Dataset 管理：Harbor registry 下载/取消/迁移/重定位/删除边界、本地导入、任务目录浏览；Job 创建自动登记数据集并指向管理下载目录。
- Leaderboard：只展示跑过 Jobs 的 Dataset，得分统一回退到比例型指标（pass@1 缺失时用 mean）。
- 应用级守护进程（`ornnlab dev start/stop/restart/status/logs`）：崩溃重启、日志脱敏、System 健康看板接入。
- Docker 所有权与回收：实例/run 标签注入、终态/启动幂等回收。

## 发布门禁证据

- 本地全量门禁 `scripts/test-after-change-web.sh`：Ruff、Pyright（0 error / 0 warning）、pytest 246 passed / 4 skipped、前端 135 tests、lint、typecheck、生产构建（JS 371KB / CSS 41KB，400KiB 预算内）、bundle、Storybook smoke/static、launcher 27/27、`test-run-dev-api.sh`、`git diff --check` 全绿。
- 云端 CI（`workflow_dispatch`，run 32074693021）：Python/Frontend ×（ubuntu / macOS / Windows）+ Npm Package Gates **全部通过**。
- 文件行数门禁：所有生产文件 ≤ 500 行。
- 走查台账：6 项问题，5 closed / 1 triaged（环境代理观察项，非阻断），无 open P0/P1。

## 已知边界

- `run_dev.sh`（Bash 启动器）的完整回归在 Windows 上跳过：MSYS 进程树与 Win32 PPID 链不一致，无法整树回收；产品级 `ornnlab dev` daemon 路径（Node 包装器）跨平台正常，由 launcher 测试覆盖。
- 未做系统级开机自启动、登录会话、Windows Service（v1.0.5 明确不做，见 dev-daemon 工程设计）。
- `pass_at_k` 在 Harbor 侧从 k=2 起算，单次尝试 run 的排行榜得分通过 mean 回退展示。

## 回滚

见 `docs/releases/v0.1.3/checklist.md` 回滚章节：停止后端 → `ornnlab backup export` 导出备份 → 记录归档路径 → `cleanup plan/archive` → 通过版本管理回退 → `doctor --logs` 验证 → 必要时 `backup import` 恢复。
