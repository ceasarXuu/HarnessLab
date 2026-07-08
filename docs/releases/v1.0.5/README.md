# v1.0.5 文档入口

- Status: Active
- Created: 2026-06-28
- Updated: 2026-07-09
- Scope: Harbor WebUI 产品化、前端治理、前后端联调准备

## 文档治理结论

v1.0.5 文档按“权威三件套 + 支撑资料”维护，避免同一主题散落在多个文档里反复定义。

| 类型 | 文档 | 维护内容 |
|---|---|---|
| 产品需求 | [PRD](prd.md) | 产品定位、范围、用户流程、交互原则、验收标准 |
| 技术设计 | [技术设计](technical-design.md) | 前端架构、领域边界、API 契约、Storybook/测试治理、联调边界 |
| 工程计划 | [工程计划与进度](engineering-plan.md) | 当前进度、阶段计划、门禁、待办、风险 |

维护规则：

- 产品范围、导航结构、页面职责和交互原则只在 [PRD](prd.md) 定义。
- 技术边界、分层、接口契约引用和测试治理只在 [技术设计](technical-design.md) 定义。
- 实施状态、已完成事项、下一阶段计划和进入联调条件只在 [工程计划与进度](engineering-plan.md) 更新。
- 支撑资料只保留证据、清单、历史决策和专题说明，不再反向改写 PRD 或技术设计。
- 如果页面新增可见功能，先更新 PRD；如果新增后端能力或字段，先更新接口契约；如果只是进度变化，只更新工程计划。

## 当前进度摘要

截至 2026-07-09：

| 方向 | 状态 | 说明 |
|---|---|---|
| PRD 收敛 | 已完成初稿 | 已明确 Jobs、Datasets、Agents、Environment、Leaderboard、System 六个一级页面和 New Job 二级流程 |
| 前端重建 | 已进入正式前端治理 | 旧 Vue demo 已不作为开发基础；当前 `frontend/` 为 React/Vite/Storybook mock 前端 |
| UI mock | 已覆盖主链路 | Jobs、Datasets、Agents、Environment、Leaderboard、System、New Job、New Agent 等主要页面已具备 mock 交互 |
| Storybook | 已建立基础约束 | 已有 theme/locale globals、MSW handlers、a11y 参数和主要组件 stories；还需扩展状态矩阵 |
| 领域类型治理 | 部分完成 | 已有 `frontend/src/domain/harbor.ts`；生产 UI 类型仍需继续从 mock fixture 中剥离 |
| API contract client | 未开始 | 还缺 `frontend/src/api/` typed client、adapter、Operation 状态流和 legacy facade |
| 后端 WebUI facade | 未开始 | 后端仍需要新增或适配 `/api/webui/v1`，旧路由不能直接成为 React 页面契约 |
| 联调门禁 | 未达成 | 进入联调前仍需补 API 层、状态矩阵、测试口径和真实 contract smoke |

详细进度见 [工程计划与进度](engineering-plan.md)。

## 支撑资料

这些文档作为专题资料保留，但不再承担权威入口职责。

| 文档 | 角色 |
|---|---|
| [v1.0.5 前端重建架构决策](frontend-rebuild-architecture.md) | 前端重建背景、官方 Harbor Viewer 对齐依据、目标栈说明 |
| [前后端联调准备设计基线](frontend-backend-integration-readiness.md) | 联调前的架构修复基线和状态矩阵 |
| [Harbor CLI-to-UI 替代架构](harbor-cli-to-ui-architecture.md) | Harbor CLI 能力到 WebUI 工作流的映射证据 |
| [Harbor WebUI 功能覆盖清单](harbor-webui-feature-coverage-checklist.md) | Harbor 能力覆盖跟踪清单和历史审查记录 |
| [前后端接口规范](../../architecture/frontend-api-contract.md) | v1.0.5 WebUI API 契约源文件 |
| [前端治理说明](../../architecture/frontend-webui-governance.md) | 跨版本前端目录、Storybook、样式和 i18n 治理规则 |

## 主题归属

| 主题 | 权威位置 | 支撑资料 |
|---|---|---|
| v1.0.5 产品定位 | [PRD](prd.md) | [Harbor CLI-to-UI 替代架构](harbor-cli-to-ui-architecture.md) |
| 页面导航与页面职责 | [PRD](prd.md) | [功能覆盖清单](harbor-webui-feature-coverage-checklist.md) |
| New Job 参数收敛 | [PRD](prd.md) | [Harbor CLI-to-UI 替代架构](harbor-cli-to-ui-architecture.md) |
| Environment 模板定位 | [PRD](prd.md) | [功能覆盖清单](harbor-webui-feature-coverage-checklist.md) |
| 前端目录和组件治理 | [技术设计](technical-design.md) | [前端治理说明](../../architecture/frontend-webui-governance.md) |
| Storybook 状态矩阵 | [技术设计](technical-design.md) | [联调准备设计基线](frontend-backend-integration-readiness.md) |
| API 契约 | [技术设计](technical-design.md) | [前后端接口规范](../../architecture/frontend-api-contract.md) |
| 实施进度 | [工程计划与进度](engineering-plan.md) | Git commit、测试结果、联调记录 |
| Harbor 能力覆盖 | [工程计划与进度](engineering-plan.md) | [功能覆盖清单](harbor-webui-feature-coverage-checklist.md) |

## 更新流程

1. 新需求或范围变化：先改 [PRD](prd.md)，再同步技术设计和工程计划。
2. 架构或接口变化：先改 [技术设计](technical-design.md) 和 [前后端接口规范](../../architecture/frontend-api-contract.md)，再实现。
3. 实施状态变化：只改 [工程计划与进度](engineering-plan.md)，避免在 PRD 中写进度流水账。
4. Harbor 能力核验：证据写入 [功能覆盖清单](harbor-webui-feature-coverage-checklist.md)，结论再回填 PRD 或技术设计。
5. 支撑文档发现与三件套冲突时，以三件套为准，并把支撑文档改为引用。
