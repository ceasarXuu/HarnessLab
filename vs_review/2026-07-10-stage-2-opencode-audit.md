# Stage 2 OpenCode 独立审计

- 日期：2026-07-10
- 审计器：OpenCode `1.15.13`，默认模型 `deepseek/deepseek-v4-pro`
- 会话：`ses_0b456f634ffeyU6Y0hURldHYM9`
- 审计约束：只读；禁止修改、创建、提交或推送文件
- 审计范围：[工程计划](../docs/releases/v1.0.5/engineering-plan.md) 的 S2-0 至 S2-7
- 结论：`CONDITIONAL PASS`

## 结论

Stage 2 的核心契约实现、六类资源读取、Operation 写操作边界、API 不可用处理、生产 UI 与 fixture 隔离，以及所有既有质量门禁均通过。它不满足“100% 完成”的关闭要求：审计发现一个 Medium 和四个 Low 缺口，均应先闭环再把 Stage 2 标记为 Done。

## Findings

| ID | 级别 | 发现 | 证据 |
|---|---|---|---|
| F-M1 | Medium | MSW 与 mock client 忽略 `AgentQuery.status` / `AgentQuery.type`、`EnvironmentQuery.type`，只处理全文 `q`。这使 mock 行为不完全忠实于契约。 | [mswHandlers.ts](../frontend/src/mocks/mswHandlers.ts:8)、[mockClient.ts](../frontend/src/api/mockClient.ts:143)、[mockClient.ts](../frontend/src/api/mockClient.ts:288) |
| F-L1 | Low | `OperationStatus` story 缺少 `cancelled` 和 `system` resourceType 变体。 | [OperationStatus.stories.tsx](../frontend/src/ui/components/OperationStatus.stories.tsx:16) |
| F-L2 | Low | Screen story 没有单独的 light/zh 变体，当前只在 App 层覆盖。 | [Screens.stories.tsx](../frontend/src/screens/Screens.stories.tsx:1) |
| F-L3 | Low | Job、Dataset、Environment 没有独立的破坏性确认 story。 | [Screens.stories.tsx](../frontend/src/screens/Screens.stories.tsx:1) |
| F-L4 | Low | `JobsTable` story 没有 loading/error 状态。 | [JobsTable.stories.tsx](../frontend/src/ui/components/JobsTable.stories.tsx:26) |

Critical 与 High：无。

## 审计证据

- `frontend/src` 中不存在 `/api/experiments`、`/api/runs`、`/api/benchmarks` 引用。
- `app/`、生产 `screens/` 和生产 `ui/components/` 不导入 mocks；发现的 imports 均位于 Storybook story，符合 fixture 用途。
- HTTP、mock、unavailable 三个 client 均实现同一 WebUI client；API 模式网络失败返回错误，不回退 mock 成功。
- 可见写操作均返回并轮询 `Operation`；同步的系统更新检查是 `ApiResponse<UpdateCheckResultDto>` 读取，不是异步写操作。
- 审计器重新执行并确认：typecheck、lint、45 个 unit、build、Storybook smoke、10 个 e2e 均通过；`4174` 无旧监听。

## 边界

以下不是本次 Stage 2 缺口：真实 `/api/webui/v1` 后端、Operation 持久化/SSE/子进程执行、真实 Harbor Job/Dataset/Agent 行为和旧后端路由的破坏性升级。这些属于 Stage 3 及后续真实联调阶段。

## 关闭标准

修复 F-M1 至 F-L4，重新执行全部 Stage 2 门禁，并进行一次独立复审。复审无 Medium/Low 遗留时，Stage 2 才能标记为 Done。
