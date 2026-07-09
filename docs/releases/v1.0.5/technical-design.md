# v1.0.5 技术设计

- Status: Active
- Created: 2026-07-09
- Updated: 2026-07-10
- Scope: Harbor WebUI 前端架构、契约边界、治理规则和联调设计

## 1. 文档定位

本文是 v1.0.5 的技术权威入口。产品范围以 [PRD](prd.md) 为准，实施状态以 [工程计划与进度](engineering-plan.md) 为准。

支撑资料：

- [v1.0.5 前端重建架构决策](frontend-rebuild-architecture.md)
- [前后端联调准备设计基线](frontend-backend-integration-readiness.md)
- [Harbor CLI-to-UI 替代架构](harbor-cli-to-ui-architecture.md)
- [Harbor WebUI 功能覆盖清单](harbor-webui-feature-coverage-checklist.md)
- [前后端接口规范](../../architecture/frontend-api-contract.md)
- [前端治理说明](../../architecture/frontend-webui-governance.md)

## 2. 技术目标

v1.0.5 的技术目标不是继续扩展 mock demo，而是把当前前端收敛成可进入前后端联调的正式 WebUI 基础。

必须满足：

- 页面和组件不直接依赖旧后端路由字段。
- 生产 UI 类型不再从 `frontend/src/mocks/` 导出。
- mock 数据只作为 Storybook、测试和离线预览 fixture。
- 所有后端接入走 WebUI contract client；mock client 只用于 Storybook、测试和离线开发，不做 legacy adapter。
- 所有耗时或破坏性动作通过统一 `Operation` 状态表达。
- Storybook 覆盖主要页面状态，而不是只展示 happy path。
- i18n、样式、基础控件、抽屉、弹窗、toast、表格都在统一组件和样式层治理。

## 3. 当前前端基线

已落地并可在 `frontend/package.json` 验证的基线：

- React 19
- Vite
- Tailwind CSS v4
- lucide-react
- Storybook
- Vitest
- Playwright
- MSW / msw-storybook-addon
- TypeScript / ESLint

目标基线但尚未落地为依赖的能力：

- React Router 或等价路由层
- TanStack Query 或等价 server-state 层
- TanStack Table 或等价表格层
- shadcn/Radix 风格的可访问基础组件库

新增这些依赖前，必须先补选型说明、Storybook 示例和回归验证。文档不得把目标基线写成已落地事实。

## 4. 前端分层

| 层 | 责任 | 禁止事项 |
|---|---|---|
| `frontend/src/app/` | 应用装配、全局偏好、页面切换、资源级状态装配 | 不承载资源业务模型和后端字段转换 |
| `frontend/src/domain/` | WebUI 领域模型、状态枚举、未格式化业务字段、ViewModel 边界 | 不导入 mock seed data |
| `frontend/src/api/` | WebUI contract client、DTO、`ApiResponse`、`Operation`、mock client、data hook | 不导入 React 组件，不实现旧路由兼容层，不把旧后端字段泄漏到页面 |
| `frontend/src/mocks/` | Storybook、测试、离线 demo fixture 和 MSW handlers | 不作为生产 UI 类型来源 |
| `frontend/src/screens/` | 页面级编排、资源状态组合、路由页面 | 不直接 `fetch`，不直接适配旧路由 |
| `frontend/src/ui/components/` | 可复用组件、pattern 组件、受 Storybook 约束的交互单元 | 不知道后端路由 |
| `frontend/src/styles/` | token、base、layout、controls、tables、surfaces、screen 专属样式 | 不恢复单个巨型样式文件 |

`frontend/src/domain/harbor.ts` 与 `frontend/src/api/` 已完成 Stage 2 所需分层：所有资源均经统一 client/data hook 获取，所有页面写操作均经过 `Operation` 边界。当前等待最终独立审查；通过后可进入 Stage 3 的后端破坏性升级，不能跳过该阶段直接联调。

## 5. API 契约

v1.0.5 对前端暴露统一 WebUI API 语义，根路径为：

```text
/api/webui/v1
```

权威接口规范见 [前后端接口规范](../../architecture/frontend-api-contract.md)。

现有后端旧路由：

- `/api/experiments`
- `/api/runs`
- `/api/benchmarks`
- `/api/leaderboard`
- `/api/system`
- `/api/agents`

v1.0.5 不维护新旧两套接口，也不建设前端 legacy adapter。上述旧路由需要被破坏性升级为 `/api/webui/v1` 下的 WebUI 产品契约；实现层可以复用现有 service、worker 和事件流，但对外资源语义、响应包络、错误模型和异步 Operation 必须按新契约重塑。

所有 JSON 接口统一使用：

- `ApiResponse<T>`
- `ApiError`
- `ApiMeta`
- `Operation`

所有耗时或破坏性动作统一返回 `Operation`，包括：

- 创建并运行 Job
- 取消 Job / Trial
- 下载 Dataset
- 取消 Dataset 下载
- 删除本地 Dataset
- 从 Leaderboard 移除 Job
- 检查更新
- 重启 OrnnLab 服务
- 清理 Docker 缓存
- 清理 `~/.cache/harbor`

## 6. DTO 与 ViewModel

后端 DTO 不直接成为 UI 展示模型。原则如下：

- token、cost、duration、score 在 DTO 中保持结构化值，UI 再格式化。
- artifact path、event、operation 在 DTO 中保持结构化数组或对象，UI 再分组展示。
- 页面不得把已格式化字符串提交回后端。
- mock fixture 可以使用接近真实展示的样例，但生产类型必须来自 `domain` 或 `api`。

示例：

| 数据 | DTO | UI |
|---|---|---|
| tokens | `18400000` + `unit: token` | `18.4M` |
| cost | `3.42` + `currency: USD` | `$3.42` |
| duration | `2520` 秒 | `00:42:00` |
| score | `{ kind: "percent", value: 0.725 }` | `72.5%` |

## 7. Storybook 治理

Storybook 是正式组件注册和评审入口。新增组件、抽屉、表格状态、确认弹窗、toast 或页面状态时，必须同步 story。

当前已具备：

- theme / locale globals
- MSW handlers
- a11y 参数
- App、Controls、JobsTable、RunBuilder、Drawer、MCP 等 story

联调前的状态矩阵基线已落实为共享 `ResourceStatus` / `OperationStatus` 组件与 Screen/App story。Screen story 负责 loaded、empty、抽屉与资源专项交互；App route story 负责 contract client 产生的 loading、API unavailable 与 Hub 状态。不得在纯展示 Screen 内伪造网络状态。

| 页面/组件 | 必须覆盖 |
|---|---|
| App Shell | dark/light、zh/en、Hub connected/disconnected/unavailable、API unavailable |
| Jobs | Screen loaded/empty/drawer/operation；App route loading/failed |
| New Job | default valid、dataset no tasks、verifier skip、task search bulk select、run validation failed |
| Datasets | App route loading/failed；registry not downloaded、downloading、downloaded、local import、delete confirm、split empty、task row expanded |
| Agents | App route loading/failed；built-in immutable、custom deletable、missing secret、skills empty/path configured、MCP variants |
| Environment | App route loading/failed；built-in/custom、network off/allowlist、GPU/TPU、advanced collapsed/expanded、healthcheck enabled/disabled |
| Leaderboard | App route loading/failed；可排名 Dataset 搜索/切换、空排名、Job 抽屉、移除排名 |
| System | App route loading/failed；healthy/degraded、update available、cache operation running、destructive confirm、toast countdown |

## 8. i18n 与样式治理

i18n 规则：

- 组件不得硬编码中文/英文判断。
- 新增文案必须进入 locale 文件。
- 组件状态不能依赖翻译后的字符串比较。
- 中英文都必须覆盖页面标题、表头、按钮、空态、弹窗、toast 和错误文案。

样式规则：

- `frontend/src/styles/index.css` 只做入口。
- token、base、layout、controls、tables、surfaces、run-builder、screens 分层维护。
- 下拉、按钮、输入、switch、tag、key-value、抽屉表格等相似组件必须复用统一组件和样式。
- 抽屉默认宽度是最小可用宽度，并允许用户拖拽调整；拖拽热区应覆盖抽屉左边界全高。

## 9. 测试门禁

进入联调前至少通过：

```bash
cd frontend
npm run typecheck
npm test
npm run lint
npm run build
npm run storybook:test
npm run e2e
```

测试口径必须一致：Storybook play、Vitest、Playwright 不能分别断言不同 UI 版本。

## 10. 联调进入条件

满足以下条件后，才进入真实后端逐接口接入：

- `frontend/src/api/` 已建立 WebUI contract client。
- `frontend/src/domain/` 不再依赖 mock seed data。
- `frontend/src/mocks/` 只提供 fixture、MSW handler 和测试数据。
- 主要页面 Storybook 状态矩阵补齐。
- API 契约和页面可见操作一一对应。
- 进入真实联调前，后端必须将旧 API 破坏性升级为 `/api/webui/v1` WebUI 契约；不保留新旧并行入口，不依赖前端 legacy adapter。
- e2e、unit、storybook smoke 至少达到当前联调最低门禁。
