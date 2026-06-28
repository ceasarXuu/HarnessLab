# PRD: Harbor WebUI 正式前端治理

- Status: Ready for implementation
- Created: 2026-06-29
- Updated: 2026-06-29
- Owner / requester: OrnnLab
- Source request: 将 Harbor WebUI demo 转为正式前端工程，并保持 mock 数据状态继续优化。

## 1. 背景和产品意图

v1.0.5 阶段 OrnnLab 仍是基于 Harbor 的本地实验控制台，但当前交付重点是 Harbor WebUI 产品化。现有前端 demo 已覆盖 Jobs、Datasets、Agents、Leaderboard、System 等主要页面，但代码结构、样式层级和组件注册还停留在快速 demo 状态。

本阶段目标是把前端当作正式产品工程治理：建立清晰目录边界、拆分样式层级、把关键组件和页面纳入 Storybook，继续使用 mock 数据，不接入后端。

## 2. 目标

- 建立可持续维护的前端目录结构。
- 保持 mock 数据为唯一前端夹具来源。
- 拆解巨型样式文件，按 tokens、base、layout、controls、tables、surfaces、run-builder、screens 分层。
- 使用 Storybook 管理关键组件和页面状态。
- 保持现有 WebUI 交互和测试通过。

## 3. 范围

### In Scope

- `frontend/src/app/` 应用装配。
- `frontend/src/screens/` 页面级组件。
- `frontend/src/ui/components/` 可复用组件。
- `frontend/src/mocks/` mock 数据与类型。
- `frontend/src/styles/` 分层样式入口。
- Storybook 全局样式、组件 story、页面 story。

### Out Of Scope

- 后端 API 接入。
- Harbor 实际运行链路改造。
- 登录、权限、真实持久化。
- 大规模视觉重设计。

## 4. 验收标准

- 前端仍可通过 Vite 启动。
- `npm run lint`、`npm test`、`npm run build`、`npm run e2e` 通过。
- `npm run storybook:test` 通过。
- 单个代码或样式文件不超过 500 行。
- Storybook 中能看到 App、核心组件和主要页面状态。
