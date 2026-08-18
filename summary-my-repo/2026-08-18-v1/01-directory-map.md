# 目录职责图

## 1. 顶层结构

```text
HarnessLab/
├── ornnlab/                     # Python 后端包（事实源）
├── frontend/                    # React 19 + Vite 前端（事实源）
├── bin/ lib/                    # npm launcher（事实源）
├── npm/harnesslab-transition/   # 旧 harnesslab 命令过渡包
├── tests/                       # Python + Node 测试与注册表
├── scripts/                     # 门禁与验证脚本
├── docs/                        # 版本化文档（中文主语言）
├── integrations/terminal_bench/ # 遗留 Terminal-Bench 适配器
├── coe/                         # 事故/根因反思
├── vs_review/                   # 对抗性审查归档
├── summary-my-repo/             # 本技能生成的内部工程摘要
├── jobs/ artifacts/             # 示例/验证产物（gitignore）
├── .github/workflows/ci.yml     # 手动触发 CI
├── pyproject.toml / uv.lock     # Python 依赖
├── package.json / npm_publish.sh
└── run_dev.sh                   # 全栈联调启动器
```

## 2. 各顶层目录职责

| 目录 | 职责 | 维护要点 |
|---|---|---|
| `ornnlab/` | FastAPI 后端：路由、领域服务、DTO、SQLite 迁移 | 只新增 `/api/webui/v1`；业务放 `services/`，路由保持薄（S01, S02） |
| `frontend/` | React 19 + Vite：页面、契约层、Storybook、Vitest | 遵守分层：`domain` 不 import API/mock；`app` 不读 fixture；页面不回传格式化字符串（S13, S16） |
| `bin/` `lib/` | npm launcher：bootstrap、前台 dev、应用级 daemon、更新/卸载 | 改动跑 `npm run test:launcher`；daemon 状态在 `~/.ornnlab/dev-service`（S17） |
| `npm/harnesslab-transition/` | 旧 `@ceasarxuu/harnesslab` 命令，提示改用 `ornnlab` | 只在过渡/弃用发布时改版本 |
| `tests/` | `tests/python`（pytest）、`tests/node`（launcher）、`WEB_REQUIREMENTS.toml` / `WEB_TEST_REGISTRY.toml` | 新增测试同步注册表；`docker` 标记需要真实 Harbor |
| `scripts/` | 全量门禁、行数、rebrand、bundle、npm pack 验证 | `test-after-change-web.sh` 是本地最终门（S15） |
| `docs/` | `releases/` 版本权威、`architecture/` 契约、`playbooks/` 操作、`archive/` 归档 | 产品只写当前 `docs/releases/v<version>/`，不建单一总 PRD |
| `integrations/terminal_bench/` | 官方 Terminal-Bench 命令 agent 适配器 | 不是 Harbor WebUI 主路径，不进默认 CI |
| `coe/` | 疑难问题复盘 | 难修问题闭环后按惯例新增 |
| `vs_review/` | 对抗性审查报告 | Stage 6 / resume 等闭环证据在这里 |
| `summary-my-repo/` | 内部工程摘要包 | 生成物，不是产品权威 |

## 3. 重要子目录

### `ornnlab/`

- `cli.py`：`web` / `doctor` / `backup` / `cleanup` / `version`。Python 控制台脚本入口。
- `app.py`：FastAPI 工厂。启动恢复、孤儿清理、Operation 对账、统一错误包络（S01）。
- `settings.py`：`ORNNLAB_HOME`（默认 `~/.ornnlab/data`）、worker 并发、`.ornnlab-home.json` 的稳定 `instance_id`。
- `api/`：
  - `webui.py`：唯一路由前缀 `/api/webui/v1`（S02）。
  - `webui_resources.py`：Job 删除、模型价格、部分资源路由聚合。
  - `webui_deps.py`：从 `request.app.state` 取服务。
- `services/`：全部产品逻辑。可分成四组：
  - **WebUI 门面**：`webui_job_service.py`、`webui_dataset_service.py`、`webui_profile_service.py`、`webui_system_service.py`、`webui_operation_service.py`。
  - **Job 拆分模块**：`webui_job_{dto,query,progress,runtime,logs,copy,resume,tasks,leaderboard,deletion}.py`。2026-08 为行数门禁把原巨型 Job 服务拆开。
  - **执行内核**：`experiment_service.py`、`harbor_engine.py`、`harbor_subprocess.py`、`queue_service.py`、`worker_service.py`、`recovery_service.py`。
  - **Docker / 代理**：`owned_docker_environment.py`、`docker_orphan_service.py`、`run_docker_cleanup.py`、`container_proxy_runtime.py`、`docker_proxy_target.py`。
- `models/`：`webui.py`（输入/DTO）、`harbor.py`（Harbor 视图）、`experiment.py`、`events.py`、`report.py`。
- `storage/migrations/`：`001`–`010`。最新有效 schema 版本号 `010`；home marker 的 `schema_version` 是产品 home 格式 `2`，不要混用。
- `observability/`：空 stub，不是当前实现。

### `frontend/src/`

- `app/App.tsx`：hash 路由、资源装配、六个一级页（S16）。
- `api/`：`contract.ts`、`webUiClient.ts`、`runtimeClient.ts`、`mockClient.ts`、`hooks.ts`、`viewModels.ts`、`requestMappers.ts`。
- `domain/`：`RunDraft`、`HarborJob`、默认值。不得依赖 API。
- `screens/`：Jobs / Datasets / Agents / Environments / Leaderboard / System + NewJob / NewAgent。
- `ui/components/`：共享控件与抽屉；可复用可见组件需有 Storybook。
- `mocks/`：离线种子与 MSW。只模拟正式 contract。
- `styles/`：token / layout / 页面层，禁止再堆巨型样式文件。

### `lib/` 与 `bin/`

- `bin/ornnlab.js`：npm 公开入口（S17）。
- `bin/harnesslab.js`：仓库内名字占位，**不打进** `ornnlab` pack。
- `lib/bootstrap.js` / `source.js` / `prerequisites.js`：安装工具与源码检出。
- `lib/dev.js`：前台同时起后端和 Vite。
- `lib/dev-daemon.js` + `lib/dev-daemon/`：应用级守护、进程身份、私有日志。

### `docs/`

| 子树 | 权威性 |
|---|---|
| `docs/releases/v1.0.5/` | **当前产品/计划/发布权威** |
| `docs/architecture/frontend-api-contract.md` | **API 字段与包络权威** |
| `docs/architecture/` 其余 | 架构说明；个别治理文档可能滞后 |
| `docs/playbooks/` | 安装、Harbor 升级、开发操作 |
| `docs/archive/` | 历史 PRD/计划/Rust 时代文档，禁止当现行实现读 |

## 4. 事实源 vs 生成/派生物

| 类别 | 路径 |
|---|---|
| 事实源 | `ornnlab/`、`frontend/src/`、`frontend/package.json`、`tests/`、`scripts/`、`docs/`、`bin/` `lib/` `npm/`、`pyproject.toml`、`uv.lock`、`.github/workflows/ci.yml` |
| 运行时数据（不入库） | `~/.ornnlab/data`、`~/.ornnlab/launcher`、`~/.ornnlab/dev-service` |
| 生成/忽略 | `.venv/`、`frontend/node_modules/`、`frontend/dist/`、`frontend/storybook-static/`、`jobs/`、`artifacts/`、`*.pyc` |
| 参考/归档 | `docs/archive/`、`integrations/terminal_bench/`、`vs_review/`、`coe/`、`summary-my-repo/` |

## 5. 新工作应放在哪里

- **新 API**：先改 `docs/architecture/frontend-api-contract.md` 和技术设计，再在 `ornnlab/api/webui.py` 或 `webui_resources.py` 注册，业务进 `ornnlab/services/webui_*`。
- **新前端页面**：`frontend/src/screens/` + `ui/components/`；数据只经 `src/api/hooks.ts`。
- **新 schema**：`ornnlab/storage/migrations/00N_*.sql`，由 `sqlite.initialize` 按文件名排序执行（S08）。
- **新门禁**：`scripts/` 或 `tests/WEB_TEST_REGISTRY.toml`。
- **下一版本文档**：新建 `docs/releases/v<version>/`，不要改写成总 PRD。
- **不要**再往旧 `/api/experiments` 风格路由、ProfileCompiler、generated-agent 或 Vue 目录加代码。
