# Harbor WebUI 前端治理说明

- 状态：执行中
- 适用版本：v1.0.5
- 范围：`frontend/`，暂时保持 mock 数据，不接入后端

## 目录边界

`frontend/src/app/` 放应用装配、路由状态和跨页面状态流。

`frontend/src/screens/` 放页面级组件，对应 WebUI 一级页面或二级页面。

`frontend/src/ui/components/` 放可复用 UI 组件，必须优先注册 Storybook。

`frontend/src/mocks/` 放 Harbor WebUI mock 数据和类型，后续接后端前仍是唯一数据夹具来源。

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

## 后续治理原则

样式修改优先落在对应层级文件中，不再向单个巨型 CSS 文件追加规则。

组件进入复用前应先有清晰 props 边界和 Storybook 注册。

接入后端前，mock 数据字段要继续保持与 Harbor WebUI 可见能力一致，避免 demo-only 字段扩散。

后续新增 API client、data hook、MSW mock 或后端接口时，必须先对齐 `frontend-api-contract.md`。如果页面新增可见操作，而契约没有对应接口，应先更新接口规范，再实现页面与服务端对接。
