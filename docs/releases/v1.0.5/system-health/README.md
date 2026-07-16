# System 健康看板

本专题负责 v1.0.5 System 一级页面的健康状态语义、看板布局和组件专属交互。

- [实现设计](implementation-design.md)：API DTO、状态规则、前端卡片、Storybook 与测试门禁。
- 产品范围与验收口径以 [v1.0.5 PRD](../prd.md) 为准。
- 执行进度以 [v1.0.5 工程计划](../engineering-plan.md) 为准。

## 开发联调

Python 健康探测或 DTO 变更后，Vite 的热更新不能替代后端进程重载。执行 `ornnlab dev restart` 后，再同时检查 `/api/webui/v1/system/health` 的结构化响应和 Codex Web Preview；否则新前端可能仍收到旧进程返回的 legacy DTO。
