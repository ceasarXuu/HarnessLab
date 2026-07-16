# System 健康看板实现设计

- 状态：Implemented
- 更新：2026-07-16
- 目标：用组件专属健康状态和分组卡片替代通用 System 表格。

## 1. 根因与设计原则

旧契约把所有组件压缩为 `status + value + path`。这会丢失业务语义，例如 Docker CLI 已安装时 `available=true`，即使 daemon 无法连接，页面仍可能显示 Healthy。新契约使用 `kind` 判别联合，每类组件返回自身事实字段；前端不得解析后端拼接的展示字符串。

页面遵循以下规则：

- 分为“服务与依赖”“存储”“主机资源”三个看板区域。
- 卡片直接展示核心状态、关键值、路径和操作，不增加详情抽屉。
- 错误原因直接显示在卡片内；路径是弱化信息。
- 状态码由后端按事实计算，展示文案和颜色由前端 i18n 映射。
- Docker 只有 daemon 可连接时才允许清理 Docker 缓存。

## 2. API 判别联合

`GET /api/webui/v1/system/health` 继续使用既有路由和 `ApiResponse<Page<SystemComponentDto>>` 包络，但直接升级 item 契约：

| kind | 专属字段 | state |
|---|---|---|
| `ornnlab-service` | `endpoint`、`logsPath`、`error` | `running/starting/restarting/degraded/stopped/error` |
| `harbor-cli` | `version`、`executablePath` | `installed/not-installed` |
| `docker` | `context`、`executablePath`、`error` | `running/not-running/not-installed/error` |
| `storage` | `sizeBytes`、`path` | `available/unavailable` |
| `resource-cpu` | `usagePercent`、`logicalCores` | `normal/elevated/high/unavailable` |
| `resource-gpu` | `usagePercent`、`deviceCount` | `normal/elevated/high/not-detected/error` |
| `resource-storage` | `availableBytes`、`totalBytes`、`path` | `normal/low/critical/unavailable` |

所有 item 保留 `actions`。删除通用 `component/status/value/path` 字段，不维护并行旧 DTO。

## 3. 状态判定

- CPU：`< 70%` Normal，`70-89.99%` Elevated，`>= 90%` High。
- GPU：检测不到 NVIDIA GPU 时为 Not detected；检测成功后使用与 CPU 相同阈值；命令执行异常为 Error。
- 可用存储：剩余比例 `< 5%` 或剩余 `< 5 GiB` 为 Critical；剩余比例 `< 15%` 或剩余 `< 20 GiB` 为 Low；否则 Normal。
- Docker：CLI 不存在为 Not installed；CLI 存在但 daemon 连接错误为 Not running；超时或非 daemon 类异常为 Error；`docker ps` 成功才是 Running。
- Harbor Cache：目录可读时 Available；无法读取时 Unavailable。缓存体积不是健康状态。

CPU 使用率通过跨平台系统监控库读取，不再把 load average 伪装为 CPU 占用率。所有探测失败均返回明确状态，不回退伪造值。

## 4. 前端组件与 Storybook

- `SystemDashboard`：负责分组、空状态和卡片网格。
- `SystemCard`：提供一致的卡片框架、标题、状态区、路径区和操作区；内部按 `kind` 分派专属渲染，不复用旧表格字段。
- 每个 `kind` 使用专属内容渲染器，不通过通用 label/value 数组拼装。
- `ResourceMeter`：CPU、GPU 和可用存储的稳定尺寸水平指标条。

Storybook 状态矩阵覆盖：正常看板、Docker 未运行、服务降级、GPU 未检测、存储不足、中文、空状态和破坏性操作确认。全局 Storybook theme toolbar 继续覆盖深色主题。故事使用 mock DTO，不连接真实系统服务。

## 5. 测试与日志

- 后端单测验证每类状态码和阈值边界，特别验证 Docker `available=true, ok=false` 不得返回 Running。
- API 测试验证判别联合，不允许旧 `status/value/path` 回归。
- 前端组件测试验证分组、专属文案、操作可用性和错误原因。
- Storybook build、前端生产 build、Python tests、lint 和类型检查全部通过。
- 后端健康接口记录组件探测失败日志，但不得记录凭证或完整敏感环境变量。
