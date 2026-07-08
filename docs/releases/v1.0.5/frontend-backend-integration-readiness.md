# v1.0.5 前后端联调准备设计基线

- Status: Ready for implementation planning
- Created: 2026-07-09
- Owner / requester: OrnnLab
- Scope: Harbor WebUI 当前 mock 前端进入真实后端联调前的架构与治理修复

## 1. 设计结论

当前前端已经具备 Harbor WebUI 的主要页面和交互形态，但还不能直接进入联调。联调前必须先修复四个设计边界：

1. 前端页面不能继续直接依赖 `frontend/src/mocks/` 的类型和 seed data。
2. 前端看到的 API 必须是稳定 WebUI 契约，而不是散落的旧 `/api/experiments`、`/api/runs`、`/api/benchmarks` 路由。
3. Storybook 必须从页面展示工具升级为接口状态评审面，覆盖 loading、empty、error、permission、operation-running 等状态。
4. e2e、unit、Storybook play 的断言口径必须一致，不能一个测试期待旧 UI，另一个测试期待新 UI。

## 2. API 边界

联调期对前端暴露统一 WebUI 资源语义：

- `Job`
- `Trial`
- `Dataset`
- `Task`
- `Agent`
- `EnvironmentProfile`
- `LeaderboardEntry`
- `SystemHealth`
- `Operation`

推荐后端新增 `/api/webui/v1` facade。旧路由可以继续保留，但只作为后端内部或兼容层：

- `/api/experiments`
- `/api/runs`
- `/api/benchmarks`
- `/api/leaderboard`
- `/api/system`
- `/api/agents`

如果后端暂时不能新增 facade，前端也必须在 `frontend/src/api/` 里建立 contract adapter，由 adapter 调旧路由并转换成 WebUI 契约。页面和组件不得直接适配旧路由字段。

## 3. 前端分层

联调前新增或调整为以下边界：

| 层 | 责任 | 禁止事项 |
|---|---|---|
| `src/domain/` | WebUI 领域模型、状态枚举、格式化前的业务字段 | 不能导入 mock 数据 |
| `src/api/` | typed client、DTO、ApiResponse、Operation、legacy adapter | 不能导入 React 组件 |
| `src/mocks/` | Storybook、离线 demo、测试夹具 | 不能作为生产 UI 类型来源 |
| `src/screens/` | 页面编排、空态/错误态/加载态组合 | 不能直接 `fetch` |
| `src/ui/components/` | 可复用组件和 pattern 组件 | 不能知道后端路由 |
| `src/app/` | shell、路由、全局偏好和资源级 data hook 装配 | 不能承载所有资源业务逻辑 |

迁移时允许短期存在 mock adapter，但类型必须从 `src/domain/` 或 `src/api/contract` 导出。

## 4. DTO 与 ViewModel

后端 DTO 不应直接成为 UI 展示模型。

示例原则：

- 后端返回 token 数字和单位，UI 再格式化为 `18.4M`。
- 后端返回 cost 数值和 currency，UI 再格式化为 `$3.42`。
- 后端返回 started/finished/duration 秒数，UI 再格式化为 `01:02:03`。
- 后端返回 score 结构，UI 再按百分比或 `87/100` 展示。
- 后端返回 artifact path 数组和类型，UI 再分组成路径卡片。

这能避免 `HarborJob.score`、`cost`、`trials` 这类 UI 字符串继续扩散成后端契约。

## 5. Operation 统一模型

以下动作不得以同步按钮状态硬编码完成：

- 创建并运行 Job
- 取消 Job / Trial
- 下载 Dataset
- 取消 Dataset 下载
- 删除本地 Dataset
- 上传 Job 到 Hub
- 移除 Leaderboard
- 检查更新
- 重启 OrnnLab 服务
- 清理 Docker / Harbor 本地缓存

它们统一返回 `Operation`，前端通过轮询或 SSE 观察：

- queued
- running
- completed
- failed
- cancelled

按钮只负责发起动作和展示 operation 状态，不在页面内部伪造完成结果。

## 6. Storybook 状态矩阵

Storybook 进入联调前至少覆盖以下状态。

### App / Shell

- 默认深色与浅色
- 中文与英文
- Hub 已连接 / 未连接 / 过期
- API unavailable 全局错误

### Jobs

- loaded
- empty
- loading
- failed
- Job drawer loaded
- Job drawer event stream disconnected
- operation running: cancel / resume / upload

### New Job

- default valid draft
- dataset 无 tasks
- verifier skip 强制不进 leaderboard
- task 搜索后批量选择只影响搜索结果
- run operation queued / failed validation

### Datasets

- registry dataset 未下载
- downloading with progress
- downloaded
- local import
- delete confirm
- split filter empty
- task row expanded

### Agents

- built-in 不可删除
- custom 可删除
- missing secret
- skills 空 / 多路径
- MCP compose sidecar / external service / stdio

### Environment

- built-in profile
- custom profile
- network off / allowlist
- healthcheck enabled / disabled
- GPU only / TPU only
- advanced collapsed / expanded

### System

- healthy
- degraded
- service update available
- cache clean operation running
- destructive confirm
- toast countdown

## 7. 测试门禁

进入联调前的最低门禁：

1. `npm run typecheck`
2. `npm test`
3. `npm run lint`
4. `npm run build`
5. `npm run storybook:test`
6. `npm run e2e`

当前已知 e2e 红灯必须先修复：Agent drawer 场景仍在寻找已移除的 `Capability config` 文案。该问题是测试口径漂移，不是功能设计问题。

## 8. 联调进入条件

满足以下条件后再开始逐接口接入：

- `src/api/` 存在 WebUI contract client。
- `src/domain/` 存在前端领域模型。
- `src/mocks/` 只作为 fixture，不再导出生产 UI 类型。
- App 不再直接从 mock seed 初始化资源状态。
- 每个 screen 至少有 loaded、loading、empty、error story。
- e2e 全绿。
- PRD、接口契约和功能清单对同一能力使用同一命名。

## 9. 首批落地顺序

1. 修复 e2e 断言漂移，恢复全绿基线。
2. 新增 `src/domain/` 与 `src/api/contract`，先不接真实服务。
3. 将 mock seed 改为 contract fixture。
4. 建立 jobs、datasets、agents、environments 的 data hook。
5. 后端新增 `/api/webui/v1` facade 或前端 adapter 兼容旧路由。
6. 先接只读列表，再接详情，再接 Operation 类写操作。

## 10. 不进入首批联调的内容

- Harbor Hub upload/share 的完整权限模型。
- Plugin 管理。
- Dataset authoring / manifest editor。
- Task debug/check/start-env 高级作者工具。
- 多用户和团队权限。

这些能力保留在功能覆盖清单中，不作为联调前阻塞项。
