# v1.0.5 工程计划与进度

- Status: Active
- Created: 2026-07-09
- Updated: 2026-07-10
- Scope: Harbor WebUI 产品化实施进度、阶段计划、联调门禁

## 1. 文档定位

本文只维护实施计划和进度，不重复定义产品范围和技术架构。

- 产品范围见 [PRD](prd.md)。
- 技术边界见 [技术设计](technical-design.md)。
- API 契约见 [前后端接口规范](../../architecture/frontend-api-contract.md)。
- 前端治理规则见 [前端治理说明](../../architecture/frontend-webui-governance.md)。

## 2. 当前进度

截至 2026-07-10，v1.0.5 已完成 Stage 1“前端 mock 产品化”，Stage 2“前端契约层建设”正在推进。当前默认仍为 mock 模式；Jobs/Datasets 的列表、详情和附属读资源已迁移到 resource hooks，Agents、Environments、Leaderboard、System 仍待迁移。`api` 模式不展示未迁移资源的 fixture，也不执行模拟写操作。

| 工作项 | 状态 | 当前证据 | 下一步 |
|---|---|---|---|
| 产品范围收敛 | Done | [PRD](prd.md) 已明确六个一级页面、New Job 二级页面、Job/Dataset/Agent/Environment/Leaderboard/System 职责 | 后续需求变化继续更新 PRD |
| 版本文档治理 | Done | [README](README.md) 已收敛为版本文档入口；新增 [技术设计](technical-design.md) 与本文 | 后续只在对应权威文档更新主题 |
| 前端重建 | Done | `frontend/` 已采用 React/Vite/Storybook；旧 Vue demo 不作为开发基础 | 继续按正式前端治理推进 |
| 主页面 mock | Done | Jobs、Datasets、Agents、Environment、Leaderboard、System、New Job、New Agent 已具备 mock 页面与主要交互 | 继续修 UI 细节和状态覆盖 |
| 样式拆分 | Done | `frontend/src/styles/` 已按 token/base/layout/controls/tables/surfaces/screens/run-builder 拆分 | 后续禁止回到巨型样式文件 |
| Storybook 基线 | Done | App、Screens、Controls、RunBuilder 等 story 已覆盖 theme/locale、路由、empty、downloading、operation-running、confirm、task bulk、verifier skip 等 Stage 1 状态 | Stage 2 接入 API hook 后继续补真实 loading/error/permission 状态 |
| i18n 基线 | Done | `i18n.zh.ts`、`i18n.en.ts`、`i18n.ts` 已覆盖新增通用组件文案；生产 UI 硬编码扫描未命中中英文残留 | 新增文案继续先入 locale 文件 |
| 领域类型治理 | Partial | 已有 `frontend/src/domain/harbor.ts` | 继续把生产 UI 类型从 `frontend/src/mocks/` 剥离 |
| API 契约规范 | In progress | [前后端接口规范](../../architecture/frontend-api-contract.md) 已定义 `/api/webui/v1`、`ApiResponse<T>`、`Operation`；Jobs/Datasets 的列表、详情、events、trials、tasks 已由 `webUiClient.ts`、hooks 和 DTO/ViewModel 接入 | 迁移 Agents、Environments、Leaderboard、System；为写操作建立 Operation client |
| 后端 API 破坏性升级 | Not started | 现有后端仍是 `/api/experiments`、`/api/runs`、`/api/benchmarks` 等旧语义路由 | 直接升级旧 API 到 `/api/webui/v1` 产品契约，不维护新旧两套 |
| 联调门禁 | Stage 1 passed | `typecheck`、unit、lint、build、Storybook smoke、e2e 已通过；但 contract client 和真实 API smoke 未完成 | Stage 2/3 补齐前端 client 和后端破坏性升级后再跑联调门禁 |

## 3. 阶段计划

### Stage 0: 文档与产品范围收敛

状态：Done

完成内容：

- 明确 v1.0.5 是 Harbor WebUI 产品化，不是脱离 Harbor 的独立产品。
- 明确 OrnnLab 仍是基于 Harbor 的实验控制台。
- 明确一级导航：Jobs、Datasets、Agents、Environment、Leaderboard、System。
- 明确 Task 是 Dataset 子概念，Trial 是 Job 子概念。
- 明确 Environment 是 OrnnLab-local 模板管理层，底层映射 Harbor 真实环境字段。
- 明确页面可见功能必须能落到 Harbor、OrnnLab 或本机系统真实能力。

### Stage 1: 前端 mock 产品化

状态：Done

已完成：

- React/Vite 前端已建立。
- 主要页面与抽屉交互已覆盖。
- New Job 参数已多轮收敛：基础、Tasks、验证器、运行策略；输出 tab 已移除。
- Environment、Agent 详情已切分子 tab。
- Jobs、Datasets、Agents 详情已使用右侧抽屉。
- Storybook 已作为组件注册入口。
- 样式文件已拆分。
- Storybook 已补充 App/Screens/RunBuilder/Controls 的 Stage 1 状态矩阵。
- `EditableStringList`、`KeyValueControl`、`NetworkAccessControl` 等相似 add/delete、key-value、allowlist 交互已收敛到通用组件。
- 组件内新增硬编码文案已收敛到 locale 文件，生产 UI 硬编码扫描未发现新增残留。
- Stage 1 测试门禁已通过：
  - `npm run typecheck`
  - `npm test -- --reporter=dot`
  - `npm run lint`
  - `npm run build`
  - `npm run storybook:test`
  - `npm run e2e`

后续延伸：

- API-specific loading、error、permission、operation polling 状态需要在 Stage 2 建立 `frontend/src/api/` 和 MSW/mock client 后继续补齐。
- 生产 UI 类型继续从 mock fixture 中剥离，作为 Stage 2 的核心任务。

### Stage 2: 前端契约层建设

状态：In progress（以下全部验收项通过前不得标记完成）

目标：

- 新增 `frontend/src/api/`。
- 定义 typed WebUI client。
- 定义 `ApiResponse<T>`、`Operation`、DTO。
- 建立 mock client；不建立 legacy adapter。
- 页面通过 data hook 消费 domain model，不直接读取 mock seed data。

#### 完成验收矩阵

Stage 2 仅覆盖前端契约层和离线 mock 行为；后端真实 `/api/webui/v1` 实现、Operation 轮询/SSE 和真实联调属于 Stage 3 及后续阶段。以下每项均需要代码、自动化验证和 Storybook 证据，缺任一项即为未完成。

| ID | 验收项 | 完成定义 |
|---|---|---|
| S2-0 | Jobs / Datasets 基线 | 列表、详情及 events、trials、tasks 全部经 DTO、client、hook、ViewModel 读取；API 模式请求失败不回退 fixture。 |
| S2-1 | 类型与 fixture 隔离 | `app`、`screens`、`ui/components` 不导入 `mocks`；生产类型来自 `domain` 或 `api`；mock 只作为离线 client、MSW、Storybook 与测试夹具。 |
| S2-2 | 六类资源读取 | Jobs、Datasets、Agents、Environments、Leaderboard、System 的列表、详情和页面附属只读资源都有 DTO、client、hook、ViewModel、mock client、MSW handler 和 loading/empty/error 页面状态。 |
| S2-3 | 契约边界 | `frontend/src/api/` 完整承载 `ApiResponse`、`ApiError`、`ApiMeta`、`Operation`、DTO、HTTP/mock client、hook；不含 React 组件、旧路由兼容或旧字段泄漏。 |
| S2-4 | 写操作 Operation 边界 | 所有可见写操作经 client 返回 `Operation`；页面不直接伪造完成。mock 模式可显式模拟 Operation，API 模式不得伪造成功。 |
| S2-5 | MSW / Storybook 状态矩阵 | 每个已接入资源都有 contract-accurate MSW；所有 Screen 覆盖 loaded、loading、empty、error、operation-running、destructive-confirm、dark/light、zh/en，并具备技术设计中规定的资源专项状态。 |
| S2-6 | 质量门禁 | `typecheck`、unit、lint、build、Storybook smoke、e2e 均通过；e2e 运行前确认 `4174` 无旧监听；同一交互的 Storybook、Vitest、e2e 断言一致。 |
| S2-7 | 文档与治理 | PRD、技术设计、契约、工程计划和功能清单的资源名称、可见操作与阶段状态一致；新增文案、组件和样式分别按 i18n、Storybook 和样式分层规则登记。 |

已完成：

- 新增 `frontend/src/api/contract.ts`，提供通用 `ApiResponse<T>`、分页模型与 Jobs/Datasets 首批结构化 DTO。
- 新增 `frontend/src/api/webUiClient.ts`，提供 `/api/webui/v1` HTTP client 读取接口。
- 新增 `frontend/src/api/mockClient.ts`，以现有 fixture 适配首批 Jobs/Datasets 契约，供离线开发、测试和后续 Storybook 使用。
- 新增 mock client 契约测试，覆盖 Job 搜索、Task split 筛选和 not-found 错误包络。
- 新增 runtime client 模式：`VITE_ORNNLAB_DATA_MODE=api` 时只请求 `/api/webui/v1`，请求失败不会回退到 mock；默认 mock 模式仅用于离线开发。
- 新增通用资源 hook，并将 App 的 Jobs、Datasets 列表迁移为通过 client + DTO/ViewModel 读取；页面具备 loading 与 API unavailable 状态。
- Job 详情、Job events、Job trials、Dataset 详情、Dataset tasks 与 New Job Task 白名单已迁移到 client + hook；App 不再直接导入这些 detail fixture。
- MSW 只保留已有 DTO、client 和 hook 消费者的 Jobs/Datasets HTTP 路由；未迁移资源不再伪装为 `/api/webui/v1` 接口。
- `api` 模式不回退未迁移资源的 fixture，且禁用全部 mock 写操作，避免出现本地伪造成功。
- 补充 HTTP response envelope 校验、网络失败测试、MSW Jobs/Datasets 路由与包络测试，以及 App 的 API unavailable Storybook 状态。
- 统一 Leaderboard 正式路由为 `GET /leaderboard`；`/leaderboards` 不作为 v1.0.5 契约路径。

仍未完成：

- Agents、Environments、Leaderboard、System 仍使用离线 fixture，尚未迁移到统一 data hook。
- 写操作、Operation 状态流、权限状态和所有资源的 loading/empty/error Storybook 矩阵尚未建立；现阶段仅允许 mock mode 的显式 UI 模拟。
- 后端尚未提供 `/api/webui/v1`；此项属于 Stage 3，不能以 mock mode 代替真实联调。

运行记录：执行 `npm run e2e` 前先确认 `4174` 未被旧的 preview/server 占用；否则 `start-server-and-test` 可能复用现有监听器，使测试不能证明它正在验证最新构建。可先执行 `lsof -nP -iTCP:4174 -sTCP:LISTEN` 检查。

验收：

- screen/component 不再从 `frontend/src/mocks/` 导入生产类型。
- mock 只用于 fixture、MSW、Storybook 和测试。
- Job、Dataset、Agent、Environment、Leaderboard、System 资源都能通过统一 data hook 读取。
- 所有写操作返回 Operation 状态，不在页面内伪造完成。

### Stage 3: 后端 API 破坏性升级

状态：Not started

目标：

- 直接升级现有后端 API 对外契约到 `/api/webui/v1`。
- 不保留 `/api/experiments`、`/api/runs`、`/api/benchmarks` 等旧产品契约作为正式入口。
- 现有 route 文件、service 和 worker 可以复用或重组，但对外资源语义必须改为 Job、Dataset、Agent、Environment、Leaderboard、System、Operation。
- 给耗时动作建立 Operation 模型。
- 明确 SSE 或轮询策略。

首批资源：

- `GET /jobs`
- `GET /jobs/{jobId}`
- `POST /jobs`
- `POST /jobs/{jobId}/cancel`
- `GET /datasets`
- `GET /datasets/{datasetId}`
- `POST /datasets/{datasetId}/download`
- `POST /operations/{operationId}/cancel`
- `GET /agents`
- `GET /environments`
- `GET /leaderboard`
- `GET /system/health`

### Stage 4: 只读联调

状态：Not started

目标：

- 先接只读列表和详情，不接写操作。
- Jobs、Datasets、Agents、Environment、Leaderboard、System 从真实 API 读取。
- 保持 mock client 可切换，用于 Storybook 和离线开发。

验收：

- Mock mode 和 API mode 使用同一套 domain model。
- 列表、详情抽屉、空态、错误态都可通过 Storybook 和 e2e 验证。
- API unavailable 时有全局错误态，不导致页面崩溃。

### Stage 5: 写操作与 Operation 联调

状态：Not started

目标：

- 创建并运行 Job。
- 取消 Job。
- 下载/取消下载 Dataset。
- 删除本地 Dataset。
- 删除 custom Agent。
- 检查更新、重启服务、清理缓存。

验收：

- 写操作都有二次确认或明确状态反馈。
- Operation 进度可观察。
- 失败状态可恢复，不静默吞错。
- 破坏性操作不直接不可恢复删除。

### Stage 6: 发布前硬化

状态：Not started

目标：

- 完整测试门禁。
- Storybook 状态矩阵补齐。
- e2e 覆盖主链路。
- 文档与实现一致性复核。
- Harbor 能力双向一致性复核。

最低门禁：

```bash
cd frontend
npm run typecheck
npm test
npm run lint
npm run build
npm run storybook:test
npm run e2e
```

## 4. 当前阻塞与风险

| 风险 | 影响 | 处理方式 |
|---|---|---|
| 前端资源迁移未完成 | Agents、Environments、Leaderboard、System 仍绑定 fixture，会放大联调返工 | Stage 2 继续迁移剩余读资源并补状态矩阵 |
| 后端旧 API 尚未破坏性升级 | React 页面无法稳定消费 v1.0.5 产品契约 | Stage 3 直接升级旧 API，不做新旧并行和前端 legacy adapter |
| API 状态矩阵未收口 | 仅 Jobs/Datasets 已有 API loading/error 样板，其余页面仍以 fixture story 为主 | Stage 2 为剩余资源补 client-backed story 与 MSW handler 测试 |
| Harbor 能力边界仍需核验 | UI 可能重新出现 fake-only action | 功能清单只做证据，PRD 和技术设计只收敛确认后的能力 |
| Environment / Agent / MCP 语义复杂 | 容易把 Harbor、OrnnLab-local 和 harness 责任混在一起 | PRD 保持产品语义，技术设计保持契约边界 |

## 5. 下一轮建议

Stage 2 下一轮：

1. 将 Agents、Environments、Leaderboard、System 和详情附属资源迁移到统一 client/hook。
2. 为每个资源补齐 contract-accurate MSW handler、loading/error/empty/permission Storybook 状态。
3. 完成读资源迁移后进行一次新的独立对抗性审查；Stage 3 再直接升级后端旧 API。

完成 Stage 2 后，进入 Stage 3：把现有后端 API 直接破坏性升级到 `/api/webui/v1`，再做只读联调。
