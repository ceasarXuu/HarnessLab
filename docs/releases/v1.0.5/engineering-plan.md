# v1.0.5 工程计划与进度

- Status: Active
- Created: 2026-07-09
- Updated: 2026-07-09
- Scope: Harbor WebUI 产品化实施进度、阶段计划、联调门禁

## 1. 文档定位

本文只维护实施计划和进度，不重复定义产品范围和技术架构。

- 产品范围见 [PRD](prd.md)。
- 技术边界见 [技术设计](technical-design.md)。
- API 契约见 [前后端接口规范](../../architecture/frontend-api-contract.md)。
- 前端治理规则见 [前端治理说明](../../architecture/frontend-webui-governance.md)。

## 2. 当前进度

截至 2026-07-09，v1.0.5 处于“前端 mock 产品化基本成型，准备进入契约层和联调基建”的阶段。

| 工作项 | 状态 | 当前证据 | 下一步 |
|---|---|---|---|
| 产品范围收敛 | Done | [PRD](prd.md) 已明确六个一级页面、New Job 二级页面、Job/Dataset/Agent/Environment/Leaderboard/System 职责 | 后续需求变化继续更新 PRD |
| 版本文档治理 | Done | [README](README.md) 已收敛为版本文档入口；新增 [技术设计](technical-design.md) 与本文 | 后续只在对应权威文档更新主题 |
| 前端重建 | Done | `frontend/` 已采用 React/Vite/Storybook；旧 Vue demo 不作为开发基础 | 继续按正式前端治理推进 |
| 主页面 mock | Done | Jobs、Datasets、Agents、Environment、Leaderboard、System、New Job、New Agent 已具备 mock 页面与主要交互 | 继续修 UI 细节和状态覆盖 |
| 样式拆分 | Done | `frontend/src/styles/` 已按 token/base/layout/controls/tables/surfaces/screens/run-builder 拆分 | 后续禁止回到巨型样式文件 |
| Storybook 基线 | Partial | 已有 theme/locale globals、MSW handlers、a11y 参数和主要 story | 补 loading/empty/error/operation-running 状态矩阵 |
| i18n 基线 | Partial | 已有 `i18n.zh.ts`、`i18n.en.ts`、`i18n.ts` | 继续清除组件硬编码文案和翻译字符串判断 |
| 领域类型治理 | Partial | 已有 `frontend/src/domain/harbor.ts` | 继续把生产 UI 类型从 `frontend/src/mocks/` 剥离 |
| API 契约规范 | Draft | [前后端接口规范](../../architecture/frontend-api-contract.md) 已定义 `/api/webui/v1`、`ApiResponse<T>`、`Operation` | 建立 `frontend/src/api/` typed client 和 legacy adapter |
| 后端 WebUI facade | Not started | 现有后端仍是 `/api/experiments`、`/api/runs`、`/api/benchmarks` 等旧路由 | 设计并实现 `/api/webui/v1` 或明确 adapter 过渡期 |
| 联调门禁 | Not ready | 测试脚本存在，但 contract client、状态矩阵和真实 smoke 未完成 | 补齐 API 层后跑完整门禁 |

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

状态：Mostly done

已完成：

- React/Vite 前端已建立。
- 主要页面与抽屉交互已覆盖。
- New Job 参数已多轮收敛：基础、Tasks、验证器、运行策略；输出 tab 已移除。
- Environment、Agent 详情已切分子 tab。
- Jobs、Datasets、Agents 详情已使用右侧抽屉。
- Storybook 已作为组件注册入口。
- 样式文件已拆分。

剩余：

- 补齐 Storybook 页面状态矩阵。
- 继续清理组件中剩余硬编码多语言文案。
- 继续压缩相似组件的重复实现，特别是 select、key-value、路径选择、add/delete 列表、抽屉表格。

### Stage 2: 前端契约层建设

状态：Not started

目标：

- 新增 `frontend/src/api/`。
- 定义 typed WebUI client。
- 定义 `ApiResponse<T>`、`Operation`、DTO。
- 建立 mock adapter 和 legacy adapter。
- 页面通过 data hook 消费 domain model，不直接读取 mock seed data。

验收：

- screen/component 不再从 `frontend/src/mocks/` 导入生产类型。
- mock 只用于 fixture、MSW、Storybook 和测试。
- Job、Dataset、Agent、Environment、Leaderboard、System 资源都能通过统一 data hook 读取。
- 所有写操作返回 Operation 状态，不在页面内伪造完成。

### Stage 3: 后端 WebUI facade

状态：Not started

目标：

- 新增或规划 `/api/webui/v1`。
- 将旧路由转换为 WebUI 资源语义。
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
- `GET /leaderboards`
- `GET /system/health`

### Stage 4: 只读联调

状态：Not started

目标：

- 先接只读列表和详情，不接写操作。
- Jobs、Datasets、Agents、Environment、Leaderboard、System 从真实 API 读取。
- 保持 mock adapter 可切换，用于 Storybook 和离线开发。

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
| 前端缺 `src/api` 层 | 页面继续绑定 mock 或旧路由会放大联调返工 | Stage 2 优先处理 |
| 后端缺 `/api/webui/v1` facade | React 页面无法稳定消费契约 | Stage 3 建 facade，或短期前端 adapter 过渡 |
| Storybook 状态矩阵不足 | 复杂交互容易回归，例如抽屉溢出、select 样式不一致 | Stage 1/6 持续补齐 |
| Harbor 能力边界仍需核验 | UI 可能重新出现 fake-only action | 功能清单只做证据，PRD 和技术设计只收敛确认后的能力 |
| Environment / Agent / MCP 语义复杂 | 容易把 Harbor、OrnnLab-local 和 harness 责任混在一起 | PRD 保持产品语义，技术设计保持契约边界 |

## 5. 下一轮建议

建议下一轮先做 Stage 2：

1. 新增 `frontend/src/api/contract.ts`，定义 `ApiResponse<T>`、`ApiError`、`Operation` 和 DTO。
2. 新增 `frontend/src/api/webuiClient.ts`，封装读取和写操作。
3. 新增 `frontend/src/api/mockClient.ts`，把当前 fixture 转成 contract response。
4. 新增 `frontend/src/api/hooks.ts` 或等价资源 hook，让 screens 不直接读 mock。
5. 迁移 Jobs 和 Datasets 作为第一批样板，再迁移 Agents、Environment、Leaderboard、System。

完成 Stage 2 后，再进入后端 `/api/webui/v1` facade 设计和只读联调。
