# 目录职责图

## 1. 顶层结构

```text
HarnessLab/
├── ornnlab/                  # Python 后端包（事实源）
├── frontend/                 # React 19 + Vite 前端（事实源）
├── lib/ bin/ npm/            # npm launcher 源码（事实源）
├── tests/                    # Python + Node 测试与注册表（事实源）
├── scripts/                  # 门禁与验证脚本（事实源）
├── docs/                     # 文档体系（事实源，中文主语言）
├── coe/                      # 持续观察/事故反思记录
├── vs_review/                # 对抗性审查记录
├── integrations/terminal_bench/  # 遗留 terminal-bench 适配器（参考）
├── jobs/                     # 示例 Harbor Job 产物（gitignore）
├── artifacts/                # 验证产物（gitignore）
├── .omx/                     # 外部审计记录
├── .github/workflows/ci.yml  # 手动触发 CI
├── pyproject.toml / uv.lock  # Python 依赖与打包
├── package.json / npm_publish.sh # npm launcher 打包
└── run_dev.sh                # 全栈联调启动器
```

## 2. 各顶层目录职责

| 目录 | 职责 | 维护要点 |
|---|---|---|
| `ornnlab/` | FastAPI 后端：`api/` 路由、`services/` 业务、`models/` DTO、`storage/` SQLite 与迁移、`settings.py` | 只新增 `/api/webui/v1` 下的路由；业务逻辑放 `services/`，路由层保持薄 |
| `frontend/` | React 19 + Vite 前端：`src/app` 路由装配、`src/api` 契约层、`src/domain` 领域模型、`src/mocks` 离线夹具、`src/screens` 页面、`src/ui/components` 共享组件、`src/styles` | 遵循 `technical-design.md` 分层：app 不读 mock fixture，domain 不导入 API，api 不放页面 |
| `lib/ bin/ npm/` | npm launcher：bootstrap、开发服务、app 级 daemon、更新/卸载、源码托管 | 改动需跑 `npm run test:launcher` 与跨平台脚本 |
| `tests/` | `tests/python/`（pytest）、`tests/node/`（launcher）、注册表 `REQUIREMENTS.toml` / `WEB_REQUIREMENTS.toml` / `WEB_TEST_REGISTRY.toml` | 新增测试需同步注册表；docker 标记测试需要真实 Docker/Harbor |
| `scripts/` | 门禁：`test-after-change-web.sh` 全量门禁、行数检查、包验证、rebrand 守卫 | 门禁脚本本身由 CI 与本地共用 |
| `docs/` | 版本化文档：`releases/`（版本权威）、`architecture/`（架构与契约）、`playbooks/`（操作）、`spikes/`、`archive/`（归档） | 版本文档按 `docs/releases/v<version>/` 维护，不建单一总 PRD |
| `coe/` | 事故/根因反思记录（如 Docker 代理、Job 计数、容器所有权回收） | 每次疑难问题解决后按惯例新增 |
| `vs_review/` | 对抗性审查结论存档（2026-05 至 07） | 代码变更后按 AGENTS.md 流程执行审查并归档 |
| `integrations/terminal_bench/` | 遗留 terminal-bench Python 适配器与测试 | 当前产品方向不再依赖，仅参考 |
| `.github/workflows/ci.yml` | 手动触发的三组 CI：python-web、frontend-web、real-harbor-docker-smoke | 自动触发已被用户禁用，勿改回 |

## 3. 重要子目录

### `ornnlab/`

- `api/`：`webui.py`（唯一路由）、`webui_resources.py`（聚合 Job 删除与模型价格路由）、`webui_job_deletion.py`、`webui_model_pricing.py`。
- `services/`：约 40 个服务模块，核心为 `experiment_service.py`（执行主路径）、`harbor_engine.py`（配置构建）、`harbor_subprocess.py`（托管子进程）、`worker_service.py`/`queue_service.py`（队列）、`webui_*` 系列（WebUI 领域服务）、`recovery_service.py`（恢复）、`docker_orphan_service.py`/`container_proxy_runtime.py`（Docker 生命周期与代理）。
- `storage/`：`sqlite.py`（连接/迁移）、`paths.py`（原子写）、`migrations/001-009`（schema 演进）。
- `models/`：`webui.py`（DTO/输入模型）、`harbor.py`（Harbor 视图与能力快照）。

### `frontend/src/`

- `api/`：`contract.ts`（DTO 契约）、`webUiClient.ts`（HTTP client 接口）、`runtimeClient.ts`（模式选择）、`mockClient.ts`/`mockOperations.ts`/`mockQueries.ts`（mock 实现）、`hooks.ts`（资源 hooks 与 Operation 轮询）、`viewModels.ts`（唯一展示格式化层）、`requestMappers.ts`（RunDraft → CreateJobRequest）。
- `mocks/`：`demo*.ts`（离线数据）、`mswHandlers.ts`（MSW）。
- `screens/`：六个一级页面（Jobs/Agents/Environments/Datasets/Leaderboard/System）+ NewJob/NewAgent。
- `ui/components/`：共享控件（RunBuilder、JobsTable、DetailDrawer、AppShell 等），每个可复用组件有 Storybook。

## 4. 事实源 vs 生成/派生物

| 类别 | 路径 |
|---|---|
| 事实源 | `ornnlab/`、`frontend/src/`、`frontend/package.json`、`tests/`、`scripts/`、`docs/`、`lib/ bin/ npm/`、`pyproject.toml`、`uv.lock`、`.github/workflows/ci.yml` |
| 生成/忽略 | `.venv/`、`frontend/node_modules/`、`frontend/dist/`、`.pytest_cache/`、`.ruff_cache/`、`*.pyc`、`jobs/`、`artifacts/` |
| 参考/归档 | `docs/archive/`、`integrations/terminal_bench/`、`vs_review/`、`coe/`、`.omx/` |

## 5. 新工作应放在哪里

- 新 API：在 `ornnlab/api/webui.py` 或新 router（经 `webui_resources.py` 聚合）注册，业务逻辑放 `ornnlab/services/webui_*`，并先更新 API 契约文档。
- 新前端页面：`frontend/src/screens/` + `src/ui/components/`，数据访问只经 `src/api/hooks.ts`。
- 新 schema 变更：`ornnlab/storage/migrations/00N_*.sql`，由 `sqlite.initialize` 顺序执行。
- 新门禁：`scripts/` 或 `tests/WEB_TEST_REGISTRY.toml` 注册。
- 新版本文档：新建 `docs/releases/v<version>/`。
