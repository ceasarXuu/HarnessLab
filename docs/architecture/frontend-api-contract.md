# Harbor WebUI 前后端接口规范

- 状态：草案
- 适用版本：v1.0.5
- 范围：当前 Harbor WebUI 前端功能与后续后端对接契约
- 当前实现：仍使用 `frontend/src/mocks/` mock 数据，不直接接入后端

## 目标

这份规范用于把当前 WebUI 可见功能固化为前后端接口边界。后续接入后端时，前端页面、Storybook mock、后端 API、Harbor CLI/Hub 能力映射都应以这里的资源模型和接口分组为基准。

规范遵循两个约束：

- Harbor 有的能力，WebUI 应有对应入口或状态展示。
- WebUI 有的按钮、字段、状态，后端必须能落到真实 Harbor、OrnnLab 服务或本机系统能力，不能保留 fake-only 功能。

本规范不是后端已实现声明。凡是列入“待确认问题”的能力，在后端实现前只能作为 UI 需求和接口预留，不能在产品文案中表述为已可用。

## 通用约定

### API 根路径

建议使用版本化根路径：

```text
/api/webui/v1
```

后端可以内部再分 Harbor CLI、Harbor Hub、OrnnLab Service、本机系统探针，但对前端暴露统一 WebUI API。

### 响应包络

所有 JSON 接口返回统一包络：

```ts
interface ApiResponse<T> {
  data: T
  error: ApiError | null
  meta?: ApiMeta
}

interface ApiError {
  code: string
  message: string
  details?: Record<string, unknown>
}

interface ApiMeta {
  cursor?: string
  nextCursor?: string
  total?: number
  requestId: string
}
```

列表接口统一支持：

- `q`：搜索关键字。
- `cursor`：分页游标。
- `limit`：页大小，默认 50。

写操作建议支持 `Idempotency-Key` 请求头，避免用户重复点击导致重复 Job、重复下载或重复清理。

### 异步操作

下载 dataset、清理缓存、重启服务、检查更新、运行 Job 等可能耗时操作统一返回 Operation：

```ts
type OperationStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled'

interface Operation {
  id: string
  type: string
  status: OperationStatus
  resourceType: 'job' | 'dataset' | 'agent' | 'system'
  resourceId?: string
  progress?: number
  message?: string
  startedAt?: string
  completedAt?: string
  error?: ApiError
}
```

查询与取消：

- `GET /operations/{operationId}`
- `POST /operations/{operationId}/cancel`

前端需要实时进度时，后端优先提供 SSE：

- `GET /events?cursor={cursor}`

首期也可以用轮询 `GET /operations/{operationId}`。

### 当前 mock 字段迁移

`frontend/src/mocks/` 是当前离线演示和 Storybook 的夹具来源，但后端接口以本规范为准。接入 API 前需要完成以下字段收敛：

- Job 与 Leaderboard 统一使用 `agentName` 和 `harness`；旧 `agent` 字段只允许作为临时 mock 兼容字段。
- Dataset 列表不再向页面暴露 `digest` 和 `updated`，如后端需要保留，应只放在详情或调试信息中。
- Agent 列表不再暴露 `adapter`、`source`、`updated` 三列；harness/adapter 的技术检查只进入详情或 review 操作。
- Leaderboard 不再暴露 `submissionId`、`uploadedUrl`、可复现性和 Actions 列；提交与上传只通过明确操作触发。

## 数据模型

### Job

```ts
type JobStatus = 'running' | 'queued' | 'completed' | 'failed'

interface Job {
  id: string
  name: string
  status: JobStatus
  dataset: string
  agentName: string
  harness: string
  model: string
  environment: string
  trials: string
  score: string
  cost: string
  tokens: string
  runtimeDuration: string
  createdAt: string
  includeInLeaderboard: boolean
  jobDir?: string
  eventLogPath?: string
  artifactPaths?: string[]
  split?: string
  metric?: string
  failureCode?: string
}
```

`score` 允许两种展示格式：

- 百分比：`72.5%`
- 分数：`87/100`

### JobConfig

`JobConfig` 对应新建 Job 页面当前所有可配置项。字段按 UI 子 tab 分组传输，但后端可以存为一个配置文件。

```ts
interface JobConfig {
  jobName: string
  jobsDir: string
  source: string
  split: string
  taskFilter: string
  excludeFilter: string
  taskLimit: number
  extraInstructions: string
  debug: boolean
  quiet: boolean
  yes: boolean
  envFile: string

  agentName: string
  harness: string
  model: string
  agentImportPath: string
  agentEnv: string
  agentKwargs: string
  allowAgentHosts: string
  skills: string
  mcpConfig: string

  environment: string
  environmentImportPath: string
  environmentEnv: string
  environmentKwargs: string
  allowEnvironmentHosts: string
  forceBuild: boolean
  deleteEnvironment: boolean
  suppressOverrideWarnings: boolean
  cpus: string
  cpuOverride: string
  memoryMb: string
  storageMb: string
  gpus: string
  tpu: string
  mounts: string
  dockerCompose: string

  verifierImportPath: string
  verifierEnv: string
  verifierKwargs: string
  disableVerifier: boolean
  verifierMaxTimeoutSec: string

  concurrency: number
  attempts: number
  timeoutMultiplier: number
  agentTimeoutMultiplier: string
  verifierTimeoutMultiplier: string
  agentSetupTimeoutMultiplier: string
  environmentBuildTimeoutMultiplier: string
  maxRetries: number
  retryInclude: string
  retryExclude: string
  retryWaitMultiplier: string
  retryMinWaitSec: string
  retryMaxWaitSec: string

  artifacts: string
  metric: string
  plugins: string
  upload: boolean
  visibility: 'private' | 'public'
  includeInLeaderboard: boolean
  shareTargets: string
}
```

### Trial

```ts
interface Trial {
  id: string
  jobId: string
  task: string
  result: string
  score: string
  retries: number
  duration: string
  cost: string
  tokens: string
  progress: string
  logPath: string
  analysisPath: string
  verifierEvidence: string
  artifactPath: string
}
```

### EventLog

`EventLog` 表示前端日志窗口中的滚动条目；日志文件的绝对路径由 `Job.eventLogPath` 提供，避免每条日志重复携带路径。
Job 详情中的产物路径列表由 `Job.artifactPaths` 提供，所有条目必须是绝对路径，避免 WebUI 混用相对路径和裸文件名。

```ts
interface EventLog {
  time: string
  level: 'info' | 'success' | 'warning' | 'error'
  message: string
}
```

### Dataset

```ts
interface Dataset {
  name: string
  version: string
  visibility: 'public' | 'private'
  tasks: number
  source: string
  downloadStatus: 'downloaded' | 'not-downloaded' | 'downloading'
  downloadProgress?: number
  downloadPath?: string
  size?: string
  registryUrl?: string
  registryPath?: string
  downloadDir?: string
  manifestPath?: string
  taskInclude?: string
  taskExclude?: string
  ref?: string
  path?: string
  overwrite?: boolean
  splits?: string[]
}
```

未下载 dataset 的 `downloadPath` 与 `size` 可以为空，前端展示为“未下载”。如果 Harbor 官方列表不能稳定返回远端体积，后端不应伪造远端大小。

### Task

```ts
interface Task {
  name: string
  dataset: string
  description: string
  os: string
  state: string
  duration: string
  owner: string
  verifier: string
  path: string
  gitUrl: string
  gitCommitId: string
  ref: string
  source: string
  schemaVersion: string
  packageInfo: string
  environment: string
  solution: string
  steps: string
  artifacts: string
}
```

Task 是 Dataset 的子资源，不作为一级页面接口暴露给导航。

### Agent

```ts
interface Agent {
  id: string
  agentName: string
  harness: string
  type: 'built-in' | 'custom'
  models: string
  status: 'available' | 'configured' | 'needs-token'
  env?: string
  kwargs?: string
  skills?: string
  mcp?: string
  runtime?: string
  setupTimeout?: string
  maxTimeout?: string
  allowedHosts?: string
  compatibleModels?: string
  adapterReview?: string
}
```

`agentName` 是用户自定义名称；`harness` 是 Harbor/OrnnLab 执行适配层，例如 `claude-code`、`codex-cli`、`custom-harness`。只有 `type = custom` 的 Agent 可以删除。

### LeaderboardEntry

```ts
interface LeaderboardEntry {
  dataset: string
  rank: number
  agentName: string
  harness: string
  model: string
  score: string
  trials: string
  cost: string
  tokens: string
  duration: string
  jobId: string
  split: string
  metric: string
  submitted: string
  reportPath: string
  comparabilityKey: string
  configHash: string
  agentSnapshotHash: string
}
```

Leaderboard 不展示 `submissionId`、`uploadedUrl`、可复现性、Actions。点击 `jobId` 打开同一套 Job 详情抽屉。

### SystemComponent

```ts
type SystemComponentKind =
  | 'ornnlab-service'
  | 'harbor-cli'
  | 'docker'
  | 'storage'
  | 'resource-cpu'
  | 'resource-gpu'
  | 'resource-storage'

interface SystemComponent {
  kind: SystemComponentKind
  component: string
  status: JobStatus | 'healthy'
  value: string
  path: string
  actions: SystemAction[]
}

type SystemAction =
  | 'check-update'
  | 'restart-service'
  | 'clean-docker-cache'
  | 'clean-storage-cache'
```

`storage` 的清理缓存对应 `~/.cache/harbor`。Docker 缓存清理只清理 Harbor 匹配规则下的 Docker 镜像/缓存，不应清理用户其他 Docker 资源。

## 接口分组

### Jobs

| 方法 | 路径 | 用途 |
| --- | --- | --- |
| `GET` | `/jobs?q=&status=&dataset=&agent=&cursor=&limit=` | Job 列表、搜索、筛选 |
| `POST` | `/jobs` | 创建 JobConfig 并启动/排队 Harbor run |
| `GET` | `/jobs/{jobId}` | Job 详情抽屉 |
| `PATCH` | `/jobs/{jobId}` | 更新可变属性，例如 `includeInLeaderboard` |
| `POST` | `/jobs/{jobId}/cancel` | 取消运行中 Job |
| `POST` | `/jobs/{jobId}/retry` | 重试失败 Job |
| `POST` | `/jobs/{jobId}/resume` | 恢复可恢复 Job |
| `GET` | `/jobs/{jobId}/trials` | Trial 列表 |
| `GET` | `/jobs/{jobId}/events?cursor=&limit=` | 事件日志 |
| `GET` | `/jobs/{jobId}/artifacts` | 产物路径列表 |
| `GET` | `/jobs/{jobId}/artifacts/file?path=` | 读取或下载具体产物 |

`POST /jobs` 请求体：

```ts
interface CreateJobRequest {
  config: JobConfig
  runImmediately: boolean
}
```

响应：

```ts
interface CreateJobResponse {
  job: Job
  operation: Operation
}
```

### Datasets 与 Tasks

| 方法 | 路径 | 用途 |
| --- | --- | --- |
| `GET` | `/datasets?q=&source=&downloadStatus=&cursor=&limit=` | Dataset 列表、搜索 |
| `GET` | `/datasets/{datasetRef}` | Dataset 详情抽屉 |
| `GET` | `/datasets/{datasetRef}/tasks?q=&split=&cursor=&limit=` | Dataset 下的 Task 列表 |
| `GET` | `/datasets/{datasetRef}/tasks/{taskName}` | Task 详情 |
| `POST` | `/datasets/{datasetRef}/download` | 下载 dataset |
| `GET` | `/datasets/{datasetRef}/download` | 查询下载状态 |
| `POST` | `/datasets/{datasetRef}/download/cancel` | 取消下载并删除已下载部分 |
| `DELETE` | `/datasets/{datasetRef}/local` | 删除本地 dataset |
| `POST` | `/datasets/{datasetRef}/sync` | 同步 manifest/registry 元数据 |
| `POST` | `/datasets/init` | 初始化本地 dataset manifest |
| `POST` | `/datasets/{datasetRef}/tasks/{taskName}/run` | 从 task 创建单 task Job |
| `POST` | `/datasets/{datasetRef}/tasks/{taskName}/environment` | 启动或准备 task 环境 |
| `POST` | `/datasets/{datasetRef}/tasks/{taskName}/check` | 运行 task 检查 |
| `POST` | `/datasets/{datasetRef}/tasks/{taskName}/debug` | 进入 task debug 流程 |
| `GET` | `/datasets/{datasetRef}/tasks/{taskName}/download` | 下载 task 相关文件 |

`datasetRef` 建议使用 URL 编码后的 `name@version`，例如 `terminal-bench%402.0`。

### Agents

| 方法 | 路径 | 用途 |
| --- | --- | --- |
| `GET` | `/agents?q=&type=&status=&cursor=&limit=` | Agent 列表、搜索 |
| `POST` | `/agents` | 新建 custom Agent |
| `GET` | `/agents/{agentId}` | Agent 详情抽屉 |
| `PATCH` | `/agents/{agentId}` | 更新 custom Agent 配置 |
| `DELETE` | `/agents/{agentId}` | 删除 custom Agent |
| `POST` | `/agents/{agentId}/adapter/review` | 检查 harness/adapter 配置 |

`DELETE /agents/{agentId}` 对 built-in Agent 必须返回 `403` 或业务错误 `AGENT_BUILT_IN_IMMUTABLE`。

### Leaderboard

| 方法 | 路径 | 用途 |
| --- | --- | --- |
| `GET` | `/leaderboard/datasets?q=&cursor=&limit=` | 可排名 dataset 下拉搜索 |
| `GET` | `/leaderboard?dataset=&q=&metric=&split=&cursor=&limit=` | 单 dataset 排名列表 |
| `PATCH` | `/jobs/{jobId}/leaderboard` | 设置某个 Job 是否进入排名 |
| `POST` | `/leaderboard/submissions` | 提交或重新提交排名结果 |

`PATCH /jobs/{jobId}/leaderboard` 请求体：

```ts
interface UpdateJobLeaderboardRequest {
  includeInLeaderboard: boolean
}
```

后端返回更新后的 Job 与当前 dataset 的新排名：

```ts
interface UpdateJobLeaderboardResponse {
  job: Job
  leaderboard: LeaderboardEntry[]
}
```

### System

| 方法 | 路径 | 用途 |
| --- | --- | --- |
| `GET` | `/system/health` | 系统健康列表 |
| `POST` | `/system/service/update/check` | 检查 OrnnLab npm 新版本 |
| `POST` | `/system/service/restart` | 重启 OrnnLab 前后端服务 |
| `POST` | `/system/cache/docker/clean` | 清理 Harbor Docker 缓存 |
| `POST` | `/system/cache/storage/clean` | 清理 `~/.cache/harbor` |

更新检查响应：

```ts
interface UpdateCheckResult {
  currentVersion: string
  latestVersion: string
  updateAvailable: boolean
  releaseNotesUrl?: string
}
```

缓存清理响应：

```ts
interface CacheCleanResult {
  target: 'docker' | 'storage'
  removedBytes?: number
  removedItems?: number
  operation: Operation
}
```

## 当前 UI 到接口映射

| UI 区域 | 当前交互 | 后端接口 |
| --- | --- | --- |
| Jobs 列表 | 搜索、点击行打开详情 | `GET /jobs`、`GET /jobs/{jobId}` |
| 新建 Job | 保存配置并运行 | `POST /jobs` |
| Job 详情抽屉 | 事件、trials、产物、取消、重试 | `/jobs/{jobId}/events`、`/trials`、`/artifacts`、`/cancel`、`/retry` |
| Datasets 列表 | 搜索、下载、取消下载、删除本地 | `GET /datasets`、`POST /download`、`POST /download/cancel`、`DELETE /local` |
| Dataset 详情 | task 列表、task 操作、新建 Job | `/datasets/{datasetRef}/tasks`、task 子接口、`POST /jobs` |
| Agents 列表 | 搜索、新建、删除 custom | `GET /agents`、`POST /agents`、`DELETE /agents/{agentId}` |
| Agent 详情 | 查看 harness、运行配置、adapter review | `GET /agents/{agentId}`、`POST /adapter/review` |
| Leaderboard | dataset 搜索/切换、移除 Job、打开 Job 详情 | `/leaderboard/datasets`、`GET /leaderboard`、`PATCH /jobs/{jobId}/leaderboard`、`GET /jobs/{jobId}` |
| System | 健康检查、检查更新、重启、清理缓存 | `/system/health`、`/service/update/check`、`/service/restart`、cache clean 接口 |

## 前端接入顺序

1. 保留 `frontend/src/mocks/` 作为 Storybook 与离线演示夹具。
2. 新增 `frontend/src/api/`，先只放 typed client 与 contract adapter，不直接改页面。
3. 每个 screen 通过明确的 data hook 或 service 接入 API，避免组件直接 `fetch`。
4. 接入一个接口就同步补充 MSW mock 或等价测试夹具，保证 Storybook 不依赖真实后端。
5. 接入完成后，mock 字段不得再比接口模型多出 demo-only 能力；新增字段必须先改本规范。

## 待确认问题

- Harbor Hub 登录态是否只由 header 的 Hub 状态管理，后端需要明确 token 存储与失效状态。
- `plugins` 当前是 JobConfig 字段，后端需要确认 Harbor 插件发现、启用、版本锁定的真实命令边界。
- Leaderboard 的 `metric`、`split` 应从 JobConfig 与 Dataset manifest 返回，不应由前端自由编造。
- Docker 缓存清理的 Harbor 匹配规则需要后端固定白名单，避免误删用户其他镜像。
- CPU/GPU/Storage 采样频率、单位、macOS/Linux 差异需要在系统探针实现时补充。
