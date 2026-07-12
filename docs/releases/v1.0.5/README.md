# OrnnLab Harbor WebUI v1.0.5

- 状态：Active
- 更新：2026-07-13

v1.0.5 将 OrnnLab 建设为基于 Harbor 的本地实验控制台。当前前端默认保持 mock 模式；真实联调通过唯一 `/api/webui/v1` 契约进入 API 模式。后续运行体验工作聚焦应用级 dev service 守护，不做系统级开机自启动。

## 权威文档

| 文档 | 负责内容 |
|---|---|
| [PRD](prd.md) | 产品定位、范围、页面职责、交互与验收口径 |
| [技术设计](technical-design.md) | 当前架构、Harbor 映射、数据边界、测试与 Storybook 治理 |
| [工程计划](engineering-plan.md) | 阶段状态、验收项、执行记录、风险与下一步 |
| [应用级守护进程](dev-daemon/README.md) | 本地前后端服务后台启动、主动关闭、崩溃重启与 System 状态设计 |
| [应用级守护进程工程设计](dev-daemon/engineering-design.md) | Stage 7 的阶段门、完整性矩阵、日志链、测试与回滚方案 |
| [WebUI API 契约](../../architecture/frontend-api-contract.md) | `/api/webui/v1` 的唯一对外接口规范 |

## 文档规则

- 产品需求只写入 PRD，技术实现与运行边界只写入技术设计，阶段进度只写入工程计划。
- API 字段、路由、响应包络和错误模型只以 API 契约为准。
- 历史专题资料已移至 [归档目录](../../archive/v1.0.5-stage-1-2-reference/)，不参与当前实现决策。
- 新增可见功能时，先更新 PRD；新增或修改接口时，先更新 API 契约和技术设计；完成状态变化时，只更新工程计划。
