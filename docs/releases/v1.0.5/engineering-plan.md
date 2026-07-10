# v1.0.5 工程计划与进度

- 状态：Active
- 更新：2026-07-11
- 判定规则：验收项未逐项满足、质量门未通过或独立审计仍有阻断项时，Stage 不得标记完成。

## 1. 阶段总览

| Stage | 名称 | 状态 | 说明 |
|---|---|---|---|
| 0 | 产品与文档范围收敛 | Done | 六个一级页面、资源层级与双向一致性原则已确定 |
| 1 | mock 前端产品化 | Done | React/Vite/Storybook、页面、抽屉、主题、语言与主要交互完成 |
| 2 | 前端契约层 | Done | DTO、HTTP/mock client、MSW、ViewModel、Operation 轮询与旧接口隔离完成 |
| 3 | 后端 API 破坏性升级 | Done | `/api/webui/v1` 已成为唯一产品 API；全量质量门、Codex Web Preview 和两轮 OpenCode 审计均已完成 |
| 4 | API 模式联调 | In progress | 以真实 `/api/webui/v1`、Docker、Harbor 与 Hub 可观察状态验证全栈行为；不改变产品默认 mock |
| 5 | 发布前硬化 | Not started | 真实环境回归、跨平台、性能与发布检查 |

## 2. Stage 3 验收矩阵

Stage 3 的唯一目标是把现有后端直接升级为 WebUI 产品契约，不维护新旧路由、DTO 或 adapter。以下任一项未完成，Stage 3 都保持 `In progress`。

| ID | 验收项 | 当前证据 | 状态 |
|---|---|---|---|
| S3-01 | 唯一入口 | `ornnlab/app.py` 只注册 `ornnlab.api.webui`，根路径为 `/api/webui/v1` | Done |
| S3-02 | 旧产品路由移除 | `ornnlab/api/{agents,benchmarks,experiments,leaderboard,runs,system,templates}.py` 已删除；API 测试断言旧路由 404 | Done |
| S3-03 | 统一包络与错误 | `ApiResponse<T>`、request id、统一异常映射和超参拒绝已实现 | Done |
| S3-04 | 全量 client 路由 | `WebUiClient` 的 Jobs、Datasets、Agents、Environment、Leaderboard、System、Hub、Operation 全部有后端实现与 mock/MSW 对照 | Done |
| S3-05 | Job 真实映射 | New Job payload 映射 Harbor `JobConfig` 真实字段；custom Agent profile 与 Environment 模板在后端展开 | Done |
| S3-06 | Harbor 字段校验 | Agent/Environment 通过 Harbor Pydantic 模型校验；MCP transport、TPU、无效/移除字段均有拒绝测试 | Done |
| S3-07 | Operation | `webui_operations` 持久化、后台执行、进度、失败、取消、前端轮询与 mock 同语义 | Done |
| S3-08 | 真实系统与恢复语义 | Job cancel/resume、Dataset import/download/sync/delete、缓存清理、服务更新/重启返回真实可观察结果 | Done |
| S3-09 | 前端约束一致 | New Job 只选 custom Agent；built-in Agent/Environment 不出现可编辑未保存交互；mock 也拒绝 built-in Job | Done |
| S3-10 | 文档收敛 | 当前 PRD、技术设计、工程计划、API 契约与活跃代码一致；历史专题与 Playwright 历史资料均已归档 | Done |
| S3-11 | 质量门 | 前后端全量测试、lint、build、Storybook smoke/static build 与 Codex Web Preview 验收均已通过 | Done |
| S3-12 | 独立审计 | OpenCode 首轮审计的阻断项已修复；第二轮使用 `deepseek-v4-pro` 只读复审，结论为 `NO BLOCKERS` | Done |

## 3. Stage 4 验收矩阵

Stage 4 的唯一目标是在不提供 API-to-mock 回退的前提下，证明现有 WebUI 可通过真实 `/api/webui/v1` 驱动。以下任一项未完成，Stage 4 保持 `In progress`。

| ID | 验收项 | 完成证据 | 状态 |
|---|---|---|---|
| S4-01 | 联调启动器 | `run_dev.sh` 同时启动后端与 5173 前端，默认 `VITE_ORNNLAB_DATA_MODE=api`，`ORNNLAB_API_TARGET` 可覆盖代理目标；直接 `npm run dev` 仍默认 mock | Done |
| S4-02 | API 读取全覆盖 | Jobs、Datasets、Agents、Environments、Leaderboard、System 与 Hub 均通过 5173 proxy 返回真实后端；Jobs/Datasets API 失败测试断言显示错误而不回退 mock | Done |
| S4-03 | 资源写操作 | custom Agent、custom Environment 与 Job leaderboard 开关已通过真实 API 创建、更新、删除或恢复，并读取最终资源状态 | Done |
| S4-04 | Dataset 异步操作 | Harbor registry 的 `hello-world@1.0` 已完成下载、Operation 轮询至 100%、读取本地路径和大小，随后删除本地数据并确认回到未下载状态 | Done |
| S4-05 | 真实 Harbor Job | `run-c8ca0d433ab1` 使用 Docker、`hello-world@1.0` 和 custom `nop` Agent 完成；原生 Job、Trial、事件日志与产物路径均已读回 | Done |
| S4-06 | Docker 与 Hub 可观察性 | System 读取实际 Colima Docker 状态；本机无 Hub 凭证时 Header 显示 `Hub disconnected`，未伪造连接成功 | Done |
| S4-07 | 前端 API 可视验收 | Codex Web Preview 已在 API 模式打开六个一级页面、New Job 和真实 Job 抽屉；`hello-world@1.0` 在 UI 中完成下载、Operation 刷新与确认删除。页面无 mock 回退，控制台无 error | Done |
| S4-08 | 质量与独立审计 | 全量前后端门禁、真实联调日志、首轮 OpenCode 审计、Block 修复与第二轮无 Block 审计 | Pending |

## 4. 已实施内容

### 后端

- 新增 `ornnlab/api/webui.py`、`ornnlab/models/webui.py` 与 `webui_*_service.py`，按产品资源而不是旧实验术语暴露接口。
- 新增 migration `004_webui_resources.sql`，持久化 Operation、custom Agent、Environment、Dataset 元数据和 Job 运行配置。
- Job 创建只接受 custom Agent profile；该 profile 以 Harbor `AgentConfig` 校验，Environment 以 `EnvironmentConfig` 校验。
- Job 输出只读取 Harbor 实际 `TrialResult` 字段；不存在的进度、日志或验证器字段返回空值，不编造。
- 通过事件服务返回 Job 事件；通过 Operation 表返回异步状态与错误。
- 添加 `tests/python/test_webui_api.py`，验证 API 包络、旧路由移除、资源写操作、字段拒绝、Operation、系统失败语义与真实 Trial DTO 边界。

### 前端

- `frontend/src/api/` 是唯一接口边界；HTTP client 不访问旧 API，API 失败不回退 mock。
- mock client、MSW、Storybook 使用同一 DTO 和写操作约束。
- New Job 已去除 `split`、custom verifier、`env_file`、输出 tab 与虚构环境字段；仅提交 Harbor 支持的 Job 级字段。
- built-in Agent 仅展示 Harbor Harness 身份；模型、凭证、Skills、MCP 和 kwargs 编辑只对 custom Agent 提供。
- Environment UI 只展示当前 Harbor `EnvironmentConfig` 映射字段，移除 Docker image、network mode、healthcheck、workdir 和无效 warning suppression。

## 5. 已完成验证

本轮已取得以下明确结果：

```text
frontend npm test                         15 files, 52 tests passed
frontend npm run lint                     passed
frontend npm run typecheck                passed
frontend npm run build                    passed
frontend npm run storybook:test           passed
frontend npm run storybook:build          passed
.venv/bin/python -m pytest tests/python -q 64 passed, 3 skipped
.venv/bin/python -m ruff check ornnlab tests/python passed
bash -n run_dev.sh && bash -n scripts/test-after-change-web.sh passed
git diff --check                          passed
Codex Web Preview                         #agents、#environments、#jobs/new 已验收
```

`pytest` 的 3 个 warning 来自 Starlette TestClient 和 Supabase 客户端的第三方 deprecation warning，不是测试失败。

OpenCode 首轮审计发现的 Job 得分尺度、`jobsDir` 实际使用、mock 异步生命周期、Dataset 取消下载、终态 Job 取消、Agent 超时映射、旧路由和 Playwright 门禁问题均已修复。第二轮只读审计会话 `ses_0b3178c56ffeipe58UCLgtb7bw` 的结论为 `NO BLOCKERS`。

复审记录了三项非阻断债务，均不影响 Stage 3 的接口升级目标：mock 中的历史展示路径、`runs.score` 与 WebUI DTO 的不同读取来源，以及归档审计记录中的历史文件引用。`pass_at_k` 键型优先级已统一为 Harbor 原生整数键优先、JSON 字符串键回退。其余债务不属于活跃 API、质量门或迁移残留；后续如需处理，应单列维护任务，避免在 API 模式联调阶段混入范围变更。

### Stage 4 联调证据

- `run_dev.sh` 已以 API 模式启动后端和前端；通过 `http://127.0.0.1:5173/api/webui/v1/system/health` 可读取真实健康信息。
- 六类资源和 Hub 状态均经 5173 proxy 调用唯一 `/api/webui/v1` 契约。断网错误测试覆盖 Jobs 与 Datasets，明确禁止回退 seed/mock 数据。
- 自定义 Agent、Environment 和 Job 排行榜开关已在真实 API 受控读写；临时 Environment 已删除。
- `hello-world@1.0` 已实际下载、轮询和删除，验证了 Harbor Dataset 的异步状态转移及本地状态刷新。
- `run-c8ca0d433ab1` 已由 Harbor 在 Docker 中完成。原生结果采用 `jobs_dir/<job_name>/result.json` 及每 Trial 的 `<trial>/result.json` 布局，前端只展示这些实际可读字段和绝对日志路径。
- Codex Web Preview 已验证 Jobs、Datasets、Agents、Environments、Leaderboard、System、New Job 和真实 Job 抽屉；`hello-world@1.0` 的 UI 下载、进度、刷新与确认删除均已验证，浏览器控制台没有错误。

## 6. 剩余执行顺序

1. Stage 4 以 `VITE_ORNNLAB_DATA_MODE=api` 按 Jobs、Datasets、Agents、Environments、Leaderboard、System 的顺序切换真实接口。
2. 在本机 Harbor、Docker 与 Hub 连接可用时执行端到端资源操作验证，重点观察 Operation 轮询、错误映射和资源刷新。
3. 不在 Stage 4 引入兼容旧 API 的 adapter、回退 mock 或第二套 DTO；发现契约缺口时直接升级唯一 `/api/webui/v1` 契约。

## 7. 运行经验

- 前端默认是 mock；要验证真实后端必须显式使用 `VITE_ORNNLAB_DATA_MODE=api`，不能因为页面仍可显示而假设 API 已被调用。
- Operation 完成需要至少经历提交、轮询和资源刷新；测试必须等待最终列表状态，不可仅断言按钮已点击。
- mock 写操作必须区分同步完成与真实异步 Operation；只有异步资源操作完成后才变更可见资源，防止 mock 掩盖 API 模式的时序问题。
- 资源刷新期间保留上一份成功数据，避免写操作完成后短暂清空下拉列表或列表项。
- Codex Web Preview 验收前确认运行的是当前 `5173` 开发服务，并在页面加载后检查 mock/API 模式是否符合目标。
- `run_dev.sh` 的健康探针只能请求当前 `/api/webui/v1/system/health`，不得重新引入旧 `/api/system/status`。
- 当前视觉验收使用 Codex Web Preview；Playwright 仅作为历史资料归档，不是活跃测试门禁。
- 后端全量测试使用 `.venv/bin/python`，不依赖系统 Python；真实 Dataset 导入测试会触发第三方 Supabase 客户端初始化 warning。
