# v1.0.5 工程计划与进度

- 状态：Active
- 更新：2026-07-13
- 判定规则：验收项未逐项满足、质量门未通过或独立审计仍有阻断项时，Stage 不得标记完成。

## 1. 阶段总览

| Stage | 名称 | 状态 | 说明 |
|---|---|---|---|
| 0 | 产品与文档范围收敛 | Done | 六个一级页面、资源层级与双向一致性原则已确定 |
| 1 | mock 前端产品化 | Done | React/Vite/Storybook、页面、抽屉、主题、语言与主要交互完成 |
| 2 | 前端契约层 | Done | DTO、HTTP/mock client、MSW、ViewModel、Operation 轮询与旧接口隔离完成 |
| 3 | 后端 API 破坏性升级 | Done | `/api/webui/v1` 已成为唯一产品 API；全量质量门、Codex Web Preview 和两轮 OpenCode 审计均已完成 |
| 4 | API 模式联调 | Done | 已以真实 `/api/webui/v1`、Docker、Harbor 与 Hub 可观察状态完成全栈验证；直接启动前端仍默认 mock |
| 5 | 发布前硬化 | Done | 严格 API 构建配置、跨平台启动与 CI、真实 Harbor 条件回归、两轮独立 Codex 审查均已闭环 |
| 6 | Dataset 存储位置管理 | In progress | S6-01 至 S6-05 已完成并通过前后端门禁；S6-06 等待对抗性审查结论 |
| 7 | 应用级守护进程 | Done | 应用级 daemon、CLI 生命周期、崩溃重启、System 状态接入与 hardening 测试已落地；全量门禁通过，最终 subagent 复审 APPROVED |

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
| S4-08 | 质量与独立审计 | `scripts/test-after-change-web.sh` 全绿；两轮 OpenCode 默认模型只读审计均为 `NO BLOCKERS` | Done |

## 4. Stage 5 验收矩阵

Stage 5 的唯一目标是证明 v1.0.5 可作为本地 WebUI 产品进入发布准备状态。以下任一项未完成、未取得当前提交的证据，或独立审计仍有阻断项时，Stage 5 保持 `In progress`。

| ID | 验收项 | 完成证据 | 状态 |
|---|---|---|---|
| S5-01 | API 模式配置 | 直接前端开发未设置模式时仍为 mock；`run_dev.sh`、`ornnlab dev` 和生产 build 默认为 API；显式非法值或 mock 生产 build 在启动/构建前失败 | Done |
| S5-02 | 跨平台产品启动器 | `ornnlab dev` 先等待后端健康，再启动带 API proxy 的前端；proxy 健康通过后才输出 URL，任一进程退出或收到终止信号会收敛子进程 | Done |
| S5-03 | 本机全栈启动回归 | `scripts/test-run-dev-api.sh` 使用独立 `ORNNLAB_HOME` 和随机端口，验证 `run_dev.sh` 的直接 API、Vite proxy、非法模式拒绝和退出清理 | Done |
| S5-04 | 跨平台 CI | CI `#29149465969` 在 Ubuntu、macOS、Windows 的 Python 与前端门禁均通过；Windows 路径、命令解析、file:// URI、日志换行、npm spawn 与文档清单路径均有回归覆盖 | Done |
| S5-05 | 真实 Harbor 条件回归 | `ORNNLAB_REAL_HARBOR=1` 下 Python API smoke、subprocess smoke 和 cancel recovery 全部通过（3 passed, 414s）；未显式启用或 Docker 不可用时明确 skip。公共基准回归不要求 Hub 凭证 | Done |
| S5-06 | 发布包与性能检查 | npm pack 内容和启动器依赖由 `verify-npm-reservation-package.sh` 验证；生产 build 后最大 JS 不超过 400 KiB、CSS 不超过 50 KiB，Storybook 静态构建仍在全量门禁中 | Done |
| S5-07 | 最终质量与独立审计 | 全量本地门禁通过（84 passed / 3 skipped）；两轮独立 Codex 审查完成：Round 1 发现启动器树清理与 S5-05 证据阻断，均已修复；Round 2 对 `e3baa83` 复审为 PASS，无阻断。当前提交跨平台 CI `#29149465969` 为绿色 | Done |

## 5. Stage 6 验收矩阵

Stage 6 的目标是让用户选择任意本地父目录下载 Harbor registry Dataset，并由 OrnnLab 持久化管理唯一副本及其可用性。以下任一项未完成时，Stage 6 保持 `In progress`。

| ID | 验收项 | 当前证据 | 状态 |
|---|---|---|---|
| S6-01 | 路径契约与持久化 | `005_dataset_storage_locations.sql` 记录存储类型、唯一 Dataset 子目录、父目录偏好与路径可用状态；既有 local 记录迁移为 external | Done |
| S6-02 | Registry 下载与取消 | 下载写入用户选择父目录下的标记子目录；同名冲突拒绝；取消/失败只清理有 OrnnLab 标记的临时目录 | Done |
| S6-03 | 迁移、重定位与删除边界 | managed Dataset 支持异步移动；路径丢失可重新定位或移除登记；存在的 managed 目录只能删除，不能直接遗留未登记目录 | Done |
| S6-04 | 本地导入边界 | external 导入仅登记与加载；重新定位或移除登记不会删除用户原始目录；删除请求被 API 拒绝 | Done |
| S6-05 | 前端与 mock 对等 | API/mock/MSW 使用同一 DTO、Operation 与错误语义；Dataset 和 New Job 路径控件通过本机原生目录选择器回填只读路径；Storybook 覆盖 managed、external、path-unavailable 状态 | Done |
| S6-06 | 回归与审查 | 已通过后端 API/文件边界、前端交互、Storybook 与全量门禁；待独立对抗性审查确认无阻断项 | In progress |

## 6. Stage 7 应用级守护进程

Stage 7 的目标是把本地 WebUI 前后端服务从“依赖终端前台进程”升级为“OrnnLab 应用级守护进程管理”。该阶段不安装 launchd、systemd、Windows Service，不做登录或开机自启动。主题索引见 [应用级守护进程](dev-daemon/README.md)，具体执行见 [工程设计](dev-daemon/engineering-design.md)。

| ID | 验收项 | 当前证据 | 状态 |
|---|---|---|---|
| S7-01 | 范围收敛 | 专题目录、工程计划和实现设计已明确只做应用级守护，不做系统级开机自启动 | Done |
| S7-02 | CLI 生命周期 | `ornnlab dev start/stop/restart/status/logs` 已接入应用级 daemon，状态写入 `~/.ornnlab/dev-service` | Done |
| S7-03 | 崩溃重启 | 已实现 backend/frontend 退出检测、退避重启、失败阈值、用户 stop 不复活和未知 PID fail closed | Done |
| S7-04 | System 页接入 | `/api/webui/v1/system/health` 已读取真实 dev-service state，并在 daemon/child 不可信时返回 unavailable/degraded | Done |
| S7-05 | 日志与诊断 | daemon/backend/frontend 日志、稳定事件名、私有权限和密钥脱敏已实现并有回归测试 | Done |
| S7-06 | 质量门 | `scripts/test-after-change-web.sh` 已通过：Python 97 passed / 3 skipped，前端 16 files / 65 tests，Storybook smoke/static build，launcher 22/22；最终 subagent 复审 `APPROVED` | Done |

## 7. 已实施内容

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

## 8. 已完成验证

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

### Stage 4 质量门与独立审计

- `bash scripts/test-after-change-web.sh` 已通过：Ruff、Pyright（0 error / 0 warning）、Python 测试（75 passed / 3 skipped）、前端测试（16 files / 58 tests）、lint、typecheck、build、Storybook smoke/static build 和 `git diff --check` 均为绿色。
- OpenCode 默认模型（`deepseek-v4-pro`）已完成两轮独立只读审计，结论均为 `NO BLOCKERS`。审计确认 API 失败不会回退 mock、旧产品路由和非 Harbor retry 语义不存在、Job/Trial 只读取 Harbor 原生结果、Operation 使用真实轮询、`run_dev.sh` 以 API 模式启动。
- 审计记录的非阻断项已转入 Stage 5：部署时校验 `VITE_ORNNLAB_DATA_MODE` 的严格取值、为 API 模式增加自动化启动/健康检查、补充真实 Harbor 条件测试覆盖。它们不改变 Stage 4 已验收的真实联调结论。
- 联调收尾日志发现删除 Dataset 后可能触发禁用详情资源的手动刷新；`useWebUiResource` 现统一拦截禁用资源，避免向 API 发送空资源标识。该回归已有前端测试覆盖。
- 上述修复提交后已补充最终 OpenCode 默认模型只读复核，结论仍为 `NO BLOCKERS`；复核确认 API 模式的启用资源加载不受影响，且当前工作区无未提交文件。

### Stage 5 跨平台与发布硬化验证

- `bash scripts/test-after-change-web.sh` 全量门禁通过：Ruff、Pyright（0 error / 0 warning）、Python 测试（78 passed / 3 skipped）、前端测试（16 files / 59 tests）、lint、typecheck、build（JS 343 KiB / CSS 31.52 KiB）、Storybook smoke/static build、launcher 测试（3 passed）、`test-run-dev-api.sh` 和 `git diff --check` 均为绿色。
- 新增 `ornnlab/services/command_line.py` 跨平台命令解析器：在 Windows 上使用非 posix shlex，避免反斜杠路径被破坏；替换了 `docker_orphan_service`、`harbor_subprocess` 和 `webui_system_service` 中的 `shlex.split`。
- 修复 Windows `file:///C:/...` URI 路径解析：`harbor_results._file_uri_path` 剥离驱动器前的多余斜杠。
- 修复 Harbor 子进程日志写入的换行转换：`log_path.open("a", newline="")` 防止 Windows CRLF 自动转换。
- 修复 Windows 下 Node `spawnAttached` 使用 `shell: true` 以便通过 PATH 找到 npm。
- 修复 `run_dev.sh` 中 Vite ANSI 颜色码导致 URL 解析失败：Vite 输出包含 `\x1b[36m` 等转义码，grep 捕获了带转义码的 URL，使 `${FRONTEND_URL%/}` 无法去除尾部斜杠，健康检查 URL 出现双斜杠。修复方式为 sed 剥离 ANSI 码后再 grep。
- `ORNNLAB_REAL_HARBOR=1` 真实 Harbor 回归全部通过：Python API smoke（约 3min 33s）、subprocess smoke 和 cancel recovery（共 3 passed, 414s），验证了真实 Job 运行、结果文件布局和取消清理证据。

### Stage 5 独立审计

- 首轮 OpenCode 只读审计（`deepseek/deepseek-v4-pro`）结论为 `APPROVED`，无阻断项。发现 7 项非阻断警告：W1 驱动字母校验、W2 嵌套引号理论风险、W3 split_command 测试覆盖、W4 _file_uri_path 测试覆盖、W5 shell injection 未来风险、W6 taskkill 错误处理、W7 ANSI 剥离仅覆盖 SGR。
- W1（`isalpha()` 驱动字母校验）、W3（POSIX 模式和空白拒绝测试）、W4（POSIX 路径、远程 host、非字母前缀测试）已修复并提交（`55d6241`）。
- 第二轮 OpenCode 只读复审确认 W1/W3/W4 修复正确且无回归；W2/W5/W6/W7 均为非阻断级别，不阻碍 Stage 5 完成。结论为 `APPROVED`。

## 9. 后续执行顺序

1. 完成 Stage 6 的独立对抗性审查，确认 Dataset 任意父目录下载、导入、移动和删除边界无阻断项。
2. 进入 Stage 7，先实现应用级守护进程核心生命周期和状态文件，再接入 System 页。
3. 持续保持唯一 `/api/webui/v1` 契约；发现缺口时直接升级该契约，不引入旧 API adapter、API-to-mock 回退或第二套 DTO。

## 10. 运行经验

- 前端默认是 mock；要验证真实后端必须显式使用 `VITE_ORNNLAB_DATA_MODE=api`，不能因为页面仍可显示而假设 API 已被调用。
- Operation 完成需要至少经历提交、轮询和资源刷新；测试必须等待最终列表状态，不可仅断言按钮已点击。
- mock 写操作必须区分同步完成与真实异步 Operation；只有异步资源操作完成后才变更可见资源，防止 mock 掩盖 API 模式的时序问题。
- 资源刷新期间保留上一份成功数据，避免写操作完成后短暂清空下拉列表或列表项。
- Codex Web Preview 验收前确认运行的是当前 `5173` 开发服务，并在页面加载后检查 mock/API 模式是否符合目标。
- `run_dev.sh` 的健康探针只能请求当前 `/api/webui/v1/system/health`，不得重新引入旧 `/api/system/status`。
- 当前视觉验收使用 Codex Web Preview；Playwright 仅作为历史资料归档，不是活跃测试门禁。
- 后端全量测试使用 `.venv/bin/python`，不依赖系统 Python；真实 Dataset 导入测试会触发第三方 Supabase 客户端初始化 warning。
- `scripts/test-after-change-web.sh` 是 Stage 4 的最终质量门；它同时覆盖类型边界、版本文档清单与前后端构建，避免只运行局部测试后误判联调完成。
- 文档目录清单由 `tests/python/test_rebrand_verification.py` 约束；新增或收敛 v1.0.5 活跃文档时必须同步更新验证脚本，防止治理文档与仓库实际文件漂移。
- Vite dev server 输出包含 ANSI 颜色码（如 `\x1b[36m`），即使输出重定向到文件也不自动剥离；从日志解析 URL 时必须先 `sed` 剥离 ANSI 码再 grep，否则 URL 中嵌入的转义码会破坏后续的字符串处理。
- Windows 下 `shlex.split` 使用 posix 模式会把反斜杠路径中的 `\` 当作转义符，导致路径损坏；跨平台命令解析必须根据 `os.name` 切换 posix 模式。
- Windows 下 Node `child_process.spawn` 不支持直接执行 `npm`（它不是 `.exe`），必须设置 `shell: true` 让系统通过 PATH 解析；进程终止使用 `taskkill /t /f` 而非进程组信号。
- 跨平台文档清单不能直接比较 `str(Path)`；仓库相对路径和归档筛选一律使用 `Path.as_posix()`，避免 Windows 反斜杠造成误报。
- Windows 的 `asyncio.Process.terminate()` 是强制终止，不会执行 Unix `SIGTERM` handler；取消回归应验证进程已退出和清理记录已写入，信号 handler 断言只适用于 POSIX。
- 高密度 JSDOM 页面断言在 Windows runner 上可明显慢于 macOS；为该测试显式声明合理时限，避免默认 5 秒把平台性能差异误判为功能失败。
