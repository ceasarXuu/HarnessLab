export interface ApiError {
  code: string
  message: string
  details?: Record<string, unknown>
}

export interface ApiMeta {
  cursor?: string
  nextCursor?: string
  requestId: string
  total?: number
}

export interface ApiResponse<T> {
  data: T
  error: ApiError | null
  meta?: ApiMeta
}

export interface Page<T> {
  items: T[]
  nextCursor?: string
  total: number
}

export type OperationStatus = 'queued' | 'running' | 'completed' | 'failed' | 'cancelled'

export interface Operation {
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

export type HubConnectionStatus = 'connected' | 'disconnected' | 'expired'

export interface HubConnectionDto {
  status: HubConnectionStatus
}

export type JobStatus = 'running' | 'queued' | 'completed' | 'failed'

export interface TrialProgressDto {
  completed: number
  total: number
}

export type ScoreDto =
  | { kind: 'percentage'; value: number }
  | { kind: 'points'; maximum: number; value: number }

export interface JobDto {
  id: string
  name: string
  status: JobStatus
  datasetRef: string
  agentName: string
  harness: string
  model: string
  environmentName: string
  trial: TrialProgressDto
  score: ScoreDto | null
  costUsd: number
  tokenUsageM: number
  runtimeSeconds: number
  createdAt: string
  includeInLeaderboard: boolean
  jobDir?: string
  eventLogPath?: string
  artifactPaths?: string[]
  split?: string
  failureCode?: string
}

export type DatasetVisibility = 'public' | 'private'
export type DatasetDownloadStatus = 'downloaded' | 'not-downloaded'

export interface DatasetDto {
  ref: string
  name: string
  version: string
  visibility: DatasetVisibility
  taskCount: number
  source: string
  download: {
    status: DatasetDownloadStatus
    path?: string
    sizeBytes?: number
  }
  registryUrl?: string
  splits: string[]
}

export interface DatasetTaskDto {
  datasetRef: string
  description: string
  name: string
  splits: string[]
}

export type EventLevel = 'info' | 'success' | 'warning' | 'error'

export interface JobEventDto {
  level: EventLevel
  message: string
  occurredAt: string
}

export type TrialStatus = 'passed' | 'running' | 'failed'

export interface TrialDto {
  costUsd: number
  id: string
  jobId: string
  logPath: string
  retryCount: number
  runtimeSeconds: number
  score: ScoreDto | null
  status: TrialStatus
  taskName: string
  tokenUsageM: number
}

export interface ListQuery {
  cursor?: string
  limit?: number
  q?: string
}

export interface DatasetTaskQuery extends ListQuery {
  split?: string
}

export type AgentProfileType = 'built-in' | 'custom'
export type AgentAvailability = 'available' | 'configured' | 'needs-token'

export interface KeyValueDto {
  key: string
  value: string
}

export interface McpServerDto {
  args?: string[]
  command?: string
  composeYaml?: string
  deployment: 'compose-sidecar' | 'stdio' | 'external-service'
  enabled: boolean
  endpointPath?: string
  env?: KeyValueDto[]
  name: string
  port?: number
  serviceName?: string
  transport: 'stdio' | 'sse' | 'streamable-http'
  url?: string
}

export interface AgentDto {
  agentName: string
  allowedHosts: string[]
  apiKeyEnv?: string
  baseUrlEnv?: string
  contextLength?: number
  env: KeyValueDto[]
  harness: string
  id: string
  importPath?: string
  kwargs: string
  mcpServers: McpServerDto[]
  models: string[]
  reasoningEfforts: string[]
  reasoningSummary?: string
  runtime?: string
  setupTimeoutSeconds?: number
  skillSources: string[]
  status: AgentAvailability
  supportedModels: string[]
  temperature?: number
  type: AgentProfileType
  updatedAt?: string
  maxTimeoutSeconds?: number
}

export interface AgentQuery extends ListQuery {
  status?: AgentAvailability
  type?: AgentProfileType
}

export type EnvironmentProfileType = 'built-in' | 'custom'

export interface EnvironmentDto {
  allowedHosts: string[]
  cpuPolicy: string
  cpus: string
  deleteAfterRun: boolean
  dockerComposePaths: string[]
  dockerImage: string
  env: KeyValueDto[]
  environmentType: string
  extraAllowedHosts: string[]
  forceBuild: boolean
  gpus: string
  gpuTypes: string
  healthcheck: string
  id: string
  importPath?: string
  kwargs: string
  memoryMb: string
  memoryPolicy: string
  mounts: string
  name: string
  networkMode: string
  os: string
  overrideCpus: string
  overrideGpus: string
  overrideMemoryMb: string
  overrideStorageMb: string
  overrideTpu: string
  profileType: EnvironmentProfileType
  storageMb: string
  tpu: string
  workdir: string
}

export interface EnvironmentQuery extends ListQuery {
  type?: EnvironmentProfileType
}

export interface LeaderboardDatasetDto {
  name: string
  ref: string
  version: string
}

export interface LeaderboardEntryDto {
  agentName: string
  comparabilityKey: string
  costUsd: number
  datasetRef: string
  harness: string
  jobId: string
  metric: string
  model: string
  rank: number
  reportPath?: string
  runtimeSeconds: number
  score: ScoreDto | null
  split: string
  submittedAt: string
  tokenUsageM: number
  trial: TrialProgressDto
}

export interface LeaderboardQuery extends ListQuery {
  dataset: string
  metric?: string
  split?: string
}

export type SystemComponentKind =
  | 'ornnlab-service'
  | 'harbor-cli'
  | 'docker'
  | 'storage'
  | 'resource-cpu'
  | 'resource-gpu'
  | 'resource-storage'

export type SystemComponentStatus = JobStatus | 'healthy'

export type SystemAction = 'check-update' | 'restart-service' | 'clean-docker-cache' | 'clean-storage-cache'

export interface SystemComponentDto {
  actions: SystemAction[]
  component: string
  kind: SystemComponentKind
  path: string
  status: SystemComponentStatus
  value: string
}

export interface OperationResultDto {
  operation: Operation
}

export interface CreateJobRequestDto {
  config: JobConfigDto
  runImmediately: boolean
}

export interface CreateJobResponseDto {
  job: JobDto
  operation: Operation
}

export interface JobConfigDto {
  agentEnv: KeyValueDto[]
  agentImportPath?: string
  agentKwargs: string
  agentName: string
  attempts: number
  concurrency: number
  datasetRef: string
  debug: boolean
  environmentPresetId: string
  includeInLeaderboard: boolean
  jobName: string
  jobsDir: string
  maxRetries: number
  metric: string
  model: string
  notes: string
  retryExclude: string
  retryInclude: string
  retryIntervalPolicy: 'standard' | 'fast' | 'slow' | 'custom'
  retryMaxWaitSeconds: number
  retryMinWaitSeconds: number
  retryWaitMultiplier: number
  selectedTaskNames: string[] | null
  split: string
  timeoutMultiplier: number
  timeoutPolicy: 'standard' | 'strict' | 'relaxed' | 'custom'
  verifierMode: 'dataset-default' | 'custom' | 'skip'
}

export interface DatasetImportRequestDto {
  name: string
  path: string
  taskCount: number
  version: string
}

export interface UpdateJobLeaderboardRequestDto {
  includeInLeaderboard: boolean
}

export interface UpdateJobLeaderboardResponseDto {
  job: JobDto
  leaderboard: LeaderboardEntryDto[]
  operation: Operation
}

export interface UpdateCheckResultDto {
  currentVersion: string
  latestVersion: string
  operation?: Operation
  releaseNotesUrl?: string
  updateAvailable: boolean
}
