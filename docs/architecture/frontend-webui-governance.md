# Harbor WebUI 前端治理说明

- 状态：执行中
- 适用版本：v1.0.5
- 范围：`frontend/`，暂时保持 mock 数据，不接入后端

> v1.0.5 引用关系：本文是跨版本前端治理规则。v1.0.5 的技术收敛入口见 [v1.0.5 技术设计](../releases/v1.0.5/technical-design.md)，实施进度见 [v1.0.5 工程计划与进度](../releases/v1.0.5/engineering-plan.md)。

## 目录边界

`frontend/src/app/` 放应用装配、路由状态和跨页面状态流。

`frontend/src/domain/` 放 WebUI 领域模型、资源状态枚举和 ViewModel 类型。联调前新增；生产 UI 类型不得继续从 `mocks/` 导出。

`frontend/src/api/` 放 WebUI contract client、DTO、`ApiResponse`、`Operation` 和 data hook。v1.0.5 不建设 legacy adapter；页面不得直接 `fetch`、直接读取 mock seed 或直接适配旧后端路由。

`frontend/src/screens/` 放页面级组件，对应 WebUI 一级页面或二级页面。

`frontend/src/ui/components/` 放可复用 UI 组件，必须优先注册 Storybook。

`frontend/src/mocks/` 放 Harbor WebUI mock 数据和 Storybook/test fixture。它可以模拟 contract response，但不能继续承担生产领域类型来源。

`frontend/src/styles/` 按层级拆分：

- `index.css`：唯一入口。
- `tokens.css`：主题 token、字体、颜色。
- `base.css`：基础元素和表单默认样式。
- `layout.css`：应用 shell、导航、workspace 布局。
- `controls.css`：按钮、搜索框、下拉、面包屑。
- `tables.css`：表格、行操作、小表格。
- `surfaces.css`：抽屉、toast、modal、metric、诊断块。
- `run-builder.css`：新建 Job 配置页专属结构。
- `screens.css`：页面级尺寸、系统页、响应式收口。

## Storybook 规则

Storybook 是前端组件注册和评审入口。新增或显著修改 UI 时，应同步新增或更新 story。

当前 story 分层：

- `Harbor WebUI/App`：完整应用壳。
- `Components/Controls`：基础交互控件。
- `Components/JobsTable`：Job 列表模式。
- `Components/RunBuilder`：新建 Job 配置流。
- `Screens/Harbor WebUI`：主要页面级状态。

所有 story 必须使用本地 mock 数据或 no-op 回调，不连接真实后端、真实登录或不稳定外部服务。

进入联调前，Screen 级 story 至少覆盖：

- loaded
- loading
- empty
- error
- operation-running
- destructive confirm
- dark/light
- zh/en

Pattern 级 story 至少覆盖：

- default
- disabled/read-only
- validation/error
- overflow/boundary
- one critical interaction play

## 后续治理原则

样式修改优先落在对应层级文件中，不再向单个巨型 CSS 文件追加规则。

组件进入复用前应先有清晰 props 边界和 Storybook 注册。

接入后端前，mock 数据字段要继续保持与 Harbor WebUI 可见能力一致，避免 demo-only 字段扩散。

后续新增 API client、data hook、MSW mock 或后端接口时，必须先对齐 `frontend-api-contract.md`。如果页面新增可见操作，而契约没有对应接口，应先更新接口规范，再实现页面与服务端对接。

任何新增文案必须进入 `i18n.ts` 或后续拆分后的 locale 文件，不允许在组件中硬编码中文/英文判断。组件不能通过比较翻译后的字符串判断当前语言。

联调前 e2e 必须全绿。Storybook play、Vitest 和 Playwright e2e 对同一交互的断言必须一致，不能一个测试断言旧 UI 存在，另一个测试断言旧 UI 不存在。
