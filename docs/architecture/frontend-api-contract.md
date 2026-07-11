# Harbor WebUI 前后端接口规范

- 状态：Active
- 适用版本：v1.0.5
- 更新：2026-07-12
- 实现入口：`ornnlab/api/webui.py`、`frontend/src/api/contract.ts`

本文是 OrnnLab WebUI 的唯一对外 API 契约。产品语义见 [v1.0.5 PRD](../releases/v1.0.5/prd.md)，技术结构见 [技术设计](../releases/v1.0.5/technical-design.md)，实施状态见 [工程计划](../releases/v1.0.5/engineering-plan.md)。

## 1. 基础约定

### 根路径与弃用路由

所有端点位于：

```text
/api/webui/v1
```

`/api/experiments`、`/api/runs`、`/api/benchmarks`、`/api/leaderboard`、`/api/system`、`/api/agents`、`/api/templates` 已移除并返回 404。不得新建 legacy adapter、兼容别名或 API-to-mock 成功 fallback。

### 响应包络

```ts
interface ApiResponse<T> {
  data: T | null
  error: ApiError | null
  meta: { requestId: string; total?: number; nextCursor?: string }
}

interface ApiError {
  code: string
  message: string
  details?: Record<string, unknown>
}
```

每个响应带 `meta.requestId`，HTTP 响应同时带 `X-Request-Id`。列表数据为：

```ts
interface Page<T> {
  items: T[]
  total: number
  nextCursor?: string
}
```

列表端点可用 `q`、`cursor`、`limit`；Agents 额外支持 `type`、`status`，Environment 额外支持 `type`，Leaderboard 需要 `dataset` 并可选 `metric`。除契约明确列出的参数外，所有 query 参数一律返回 `422 INVALID_REQUEST`。

### 错误模型

| HTTP | code | 含义 |
|---|---|---|
| 404 | `ROUTE_NOT_FOUND` | 路由不存在 |
| 404 | `RESOURCE_NOT_FOUND` | 资源不存在 |
| 403 | `RESOURCE_IMMUTABLE` | built-in 资源不可变更 |
| 409 | `OPERATION_CONFLICT` | 操作已终态或与当前状态冲突 |
| 422 | `VALIDATION_ERROR` | Pydantic 请求字段校验失败，包括额外字段 |
| 422 | `INVALID_REQUEST` | 资源状态、Harbor 配置或 query 参数无效 |
| 500 | `INTERNAL_ERROR` | 未预期服务端错误 |

## 2. 异步 Operation

```ts
type OperationStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled'

interface Operation {
  id: string
  type: string
  status: OperationStatus
  resourceType: 'job' | 'dataset' | 'agent' | 'environment' | 'system'
  resourceId?: string
  progress?: number
  message?: string
  startedAt?: string
  completedAt?: string
  error?: ApiError
}
```

- `GET /operations/{operationId}` 返回 Operation。
- `POST /operations/{operationId}/cancel` 取消 queued/running Operation。
- 运行时间较长的 Dataset 导入、下载、同步和 Job resume 走后台 task；其他写操作也返回 completed Operation，保持前端统一状态流。
- 前端使用轮询，不实现 SSE：Operation 进行中时轮询；Job 日志通过 Jobs events 端点拉取。

## 3. 端点目录

| 资源 | 方法与路径 | 返回 |
|---|---|---|
| Operation | `GET /operations/{id}` | `Operation` |
| Operation | `POST /operations/{id}/cancel` | `{ operation }` |
| Agent | `GET /agents`、`GET /agents/{id}` | `Page<Agent>`、`Agent` |
| Agent | `POST /agents`、`PATCH /agents/{id}`、`DELETE /agents/{id}` | `{ operation }` |
| Environment | `GET /environments`、`GET /environments/{id}` | `Page<Environment>`、`Environment` |
| Environment | `POST /environments`、`PATCH /environments/{id}`、`DELETE /environments/{id}`、`POST /environments/{id}/copy` | `{ operation }` |
| Job | `GET /jobs`、`GET /jobs/{id}` | `Page<Job>`、`Job` |
| Job | `POST /jobs` | `{ job, operation }` |
| Job | `POST /jobs/{id}/cancel`、`resume` | `{ operation }` |
| Job | `GET /jobs/{id}/events`、`GET /jobs/{id}/trials` | `JobEvent[]`、`Trial[]` |
| Job | `PATCH /jobs/{id}/leaderboard` | `{ job, leaderboard, operation }` |
| Dataset | `GET /datasets`、`GET /datasets/{ref}`、`GET /datasets/{ref}/tasks` | `Page<Dataset>`、`Dataset`、`Page<DatasetTask>` |
| Dataset | `GET /datasets/storage/default-parent` | `{ parentPath }` |
| Dataset | `POST /datasets/import`、`POST /datasets/{ref}/download`、`POST /datasets/{ref}/download/cancel`、`POST /datasets/{ref}/move`、`POST /datasets/{ref}/relocate`、`POST /datasets/{ref}/sync`、`DELETE /datasets/{ref}/local`、`DELETE /datasets/{ref}/registration` | `{ operation }` |
| Leaderboard | `GET /leaderboard/datasets`、`GET /leaderboard` | `Page<LeaderboardDataset>`、`Page<LeaderboardEntry>` |
| System | `GET /system/health`、`GET /system/hub-connection` | `Page<SystemComponent>`、`HubConnection` |
| System | `POST /system/service/update/check` | `UpdateCheckResult` |
| System | `POST /system/service/update`、`restart`、`POST /system/cache/docker/clean`、`POST /system/cache/storage/clean` | `{ operation }` |

路径中的 Dataset ref 采用 URL encoding，例如 `terminal-bench%402.0`。

## 4. 资源 DTO

### Job 与 Trial

```ts
interface Job {
  id: string
  name: string
  status: 'draft' | 'queued' | 'running' | 'completed' | 'failed' | 'cancelled' | 'interrupted'
  datasetRef: string
  agentName: string
  harness: string
  model: string
  environmentName: string
  trial: { completed: number; total: number }
  score: { kind: 'percentage'; value: number } | { kind: 'points'; value: number; maximum: number } | null
  costUsd: number | null
  tokenUsageM: number | null
  runtimeSeconds: number | null
  createdAt: string
  includeInLeaderboard: boolean
  jobDir?: string
  eventLogPath?: string
  artifactPaths?: string[]
  failureCode?: string
}

interface Trial {
  id: string
  jobId: string
  taskName: string
  status: 'passed' | 'failed' | 'cancelled' | 'interrupted'
  score: Job['score']
  costUsd: number | null
  tokenUsageM: number | null
  runtimeSeconds: number | null
  retryCount: number | null
  logPath: string | null
}
```

Trial 不包含模拟的 progress 百分比、analysis path 或 verifier 内部状态。Harbor CLI 的原生 Job `result.json` 会省略 `trial_results`，后端从 `<job_dir>/<job_name>/*/result.json` 读取每个 Trial 的真实结果；对应目录存在 `trial.log` 时返回绝对日志路径。只有 Harbor 明确提供的 `pass` 二元 reward 或 Job `pass@1` 才转换为百分比 score；任意 reward 聚合没有最大分值时不伪造为百分制或分数制。Harbor 不提供的 retry 字段返回 `null`。

### Job 创建

```ts
interface CreateJobRequest {
  runImmediately: boolean
  config: {
    jobName: string
    jobsDir: string
    datasetRef: string
    selectedTaskNames: string[] | null
    extraInstructionPaths: string[]
    agentName: string
    environmentPresetId: string
    concurrency: number
    attempts: number
    debug: boolean
    includeInLeaderboard: boolean
    verifierMode: 'dataset-default' | 'skip'
    timeoutMultiplier: number
    agentTimeoutMultiplier: number
    verifierTimeoutMultiplier: number
    agentSetupTimeoutMultiplier: number
    environmentBuildTimeoutMultiplier: number
    maxRetries: number
    retryInclude: string
    retryExclude: string
    retryWaitMultiplier: number
    retryMinWaitSeconds: number
    retryMaxWaitSeconds: number
    metric: 'sum' | 'min' | 'max' | 'mean' | 'uv-script'
    notes: string
  }
}
```

后端只接受已配置的 custom Agent profile。它把 Agent 展开为 `AgentConfig`，把 Environment 预设展开为 `EnvironmentConfig`，再写入 Harbor `JobConfig` override。`split`、`model`、`agentEnv`、`agentImportPath`、`agentKwargs`、custom verifier、`env_file`、输出/Hub/plugin 参数均不属于 Job 请求。

如果 `verifierMode` 为 `skip`，后端会禁用 verifier，并要求 `includeInLeaderboard` 为 false；后续尝试把该 Job 加回排行榜返回 `422`。

### Dataset

```ts
interface Dataset {
  ref: string
  name: string
  version: string
  visibility: 'public' | 'private'
  taskCount: number
  source: string
  registryUrl?: string
  download: {
    status: 'downloaded' | 'not-downloaded' | 'path-unavailable'
    path?: string
    sizeBytes?: number
    storageKind?: 'managed' | 'external'
  }
}

interface DatasetTask { datasetRef: string; name: string; description: string }
```

`POST /datasets/import` 接受 `{ name, version, path, taskCount }`，把现有本地目录登记为 `external` Dataset；不会上传、复制、移动或删除该目录。Dataset/Task 不接受通用 `split` 字段或 query。

Registry Dataset 下载和移动都接受 `{ parentPath }`：该值是父目录，OrnnLab 使用不可编辑的 `Dataset name + version` 目录名在其下创建唯一子目录，并将最近一次成功选择记为 `GET /datasets/storage/default-parent` 的 `parentPath`。目标子目录已存在时返回 `INVALID_REQUEST`，不会覆盖目录。

下载后的 Registry Dataset 为 `managed`，目录中有 `.ornnlab-dataset.json` 标记。只有 `managed` Dataset 可以通过 `POST /datasets/{ref}/move` 移动、通过 `DELETE /datasets/{ref}/local` 删除文件；移动失败时保留原目录。存在的 `managed` Dataset 不能直接移除登记，必须先删除其目录；若其路径已失效，则可移除失效记录。`external` Dataset 永远不能由 OrnnLab 删除，用户只能使用 `POST /datasets/{ref}/relocate` 提供当前实际目录，或 `DELETE /datasets/{ref}/registration` 移除 OrnnLab 注册。路径不再存在时读取返回 `download.status = 'path-unavailable'`，保留原路径以便重新定位。

### Agent 与 MCP

```ts
interface Agent {
  id: string
  agentName: string
  harness: string
  type: 'built-in' | 'custom'
  status: 'available' | 'configured' | 'needs-token'
  models: string[]
  importPath?: string
  env: Array<{ key: string; value: string }>
  kwargs: string
  skillSources: string[]
  mcpServers: Array<{
    name: string
    transport: 'stdio' | 'sse' | 'streamable-http'
    command?: string
    args?: string[]
    url?: string
  }>
  setupTimeoutSeconds?: number
  timeoutSeconds?: number
  maxTimeoutSeconds?: number
}
```

built-in Agent 从 Harbor `AgentName` 生成，只读且通常没有 custom profile 配置。custom Agent 必须使用 Harbor AgentName，或在 `importPath` 存在时使用 custom harness。`stdio` MCP 必须提供 `command`；`sse`/`streamable-http` 必须提供 `url`。协议不包含启用开关、部署配置或 compose sidecar。

Agent 的 `status` 是响应字段，不属于创建或更新请求。后端仅在 custom profile 通过 Harbor `AgentConfig` 校验并保存后返回 `configured`；未实现真实凭证可用性探针时不得由前端提交或伪造 `needs-token`。

### Environment

```ts
interface Environment {
  id: string
  name: string
  profileType: 'built-in' | 'custom'
  environmentType: string
  importPath?: string
  forceBuild: boolean
  deleteAfterRun: boolean
  cpuPolicy: string
  memoryPolicy: string
  overrideCpus: string
  overrideMemoryMb: string
  overrideStorageMb: string
  overrideGpus: string
  overrideTpu: string
  mounts: string
  dockerComposePaths: string[]
  env: Array<{ key: string; value: string }>
  kwargs: string
  allowedHosts: string[]
}
```

保存时 `environmentType` 必须是 Harbor `EnvironmentType`，除非存在 `importPath`；CPU/Memory policy 必须是 Harbor `ResourceMode`。`overrideTpu` 为空或 `type=topology`，其中 type 不由 Harbor 枚举。协议不包含 Docker image、network mode、os、workdir、healthcheck、gpuTypes 或无效的 `suppressOverrideWarnings`。

### Leaderboard 与 System

Leaderboard 请求为 `GET /leaderboard?dataset=<ref>&metric=<optional>`。`LeaderboardEntry` 包含 `rank`、`datasetRef`、`agentName`、`harness`、`model`、`score`、`trial`、`costUsd`、`tokenUsageM`、`runtimeSeconds`、`jobId`、`submittedAt`、`comparabilityKey`、可选 `reportPath`。

`SystemComponent` 包含 `component`、`kind`、`status`、`value`、`path`、`actions`；kind 只为 OrnnLab service、Harbor CLI、Docker、Storage、CPU、GPU、可用存储。Hub 返回 `connected`、`disconnected` 或 `expired`。检查更新返回当前/最新版本、是否可升级、可选 release notes URL；安装更新/重启若本机不可执行，返回 failed Operation，而不是模拟成功。

## 5. 联调规则

- `frontend/src/api/webUiClient.ts` 的方法集与本目录一一对应；新增端点必须同时补充 DTO、HTTP client、mock client、MSW、API 集成测试和 Storybook/页面状态。
- 字段新增先修改本文和 `contract.ts`，后端与 mock 同时实现；不允许仅给 mock 增加字段。
- 前端 API 模式失败必须呈现错误，不可回退 fixture。mock 模式只用于离线开发、Storybook 与测试。
- Stage 3 采用轮询；如未来添加 SSE，必须先以新版本契约定义事件顺序、断线恢复与 Operation 去重。
