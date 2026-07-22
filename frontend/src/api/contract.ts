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

export type JobStatus = 'draft' | 'running' | 'queued' | 'completed' | 'failed' | 'cancelled' | 'interrupted'

export interface TrialProgressDto {
  completed: number
  total: number
}

export interface JobTrialProgressDto extends TrialProgressDto {
  errored: number
  notPassed: number
  passed: number
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
  trial: JobTrialProgressDto
  score: ScoreDto | null
  costUsd: number | null
  tokenUsageM: number | null
  runtimeSeconds: number | null
  createdAt: string
  includeInLeaderboard: boolean
  canResume: boolean
  jobDir?: string
  eventLogPath?: string
  artifactPaths?: string[]
  failureCode?: string
}

export interface DeleteJobResultDto {
  deletedJobId: string
}

export type DatasetVisibility = 'public' | 'private'
export type DatasetDownloadStatus = 'downloaded' | 'downloading' | 'not-downloaded' | 'path-unavailable'
export type DatasetStorageKind = 'external' | 'managed'

export interface DatasetDto {
  ref: string
  name: string
  version: string
  visibility: DatasetVisibility
  taskCount: number
  source: string
  download: {
    status: DatasetDownloadStatus
    progress?: number
    path?: string
    sizeBytes?: number
    storageKind?: DatasetStorageKind
    updatedAt?: string
  }
  registryUrl?: string
}

export interface DatasetTaskDto {
  datasetRef: string
  description: string
  environment: DatasetTaskEnvironmentDto | null
  name: string
}

export interface DatasetTaskEnvironmentDto {
  allowedHosts: string[]
  buildTimeoutSeconds: number
  containerImages: Array<{
    platforms: string[] | null
    reference: string
    source: 'dockerfile-base' | 'environment-config'
  }>
  definitions: Array<'docker-image' | 'dockerfile' | 'docker-compose'>
  networkMode: 'no-network' | 'public' | 'allowlist'
  os: 'linux' | 'windows'
  resources: {
    cpus: number | null
    gpuTypes: string[]
    gpus: number | null
    memoryMb: number | null
    storageMb: number | null
    tpu: { topology: string; type: string } | null
  }
  workdir: string | null
}

export interface DirectoryPickerResultDto {
  path: string | null
}

export type EventLevel = 'info' | 'success' | 'warning' | 'error'

export interface JobEventDto {
  level: EventLevel
  message: string
  occurredAt: string
}

export type TrialStatus = 'passed' | 'failed' | 'cancelled' | 'interrupted'

export interface TrialDto {
  costUsd: number | null
  id: string
  jobId: string
  logPath: string | null
  retryCount: number | null
  runtimeSeconds: number | null
  score: ScoreDto | null
  status: TrialStatus
  taskName: string
  tokenUsageM: number | null
}

export interface ListQuery {
  cursor?: string
  limit?: number
  q?: string
}

export type DatasetTaskQuery = ListQuery

export type AgentAvailability = 'configured' | 'needs-token'

export interface KeyValueDto {
  key: string
  /** null asks OrnnLab to inherit the same variable from its process environment. */
  value: string | null
}

export interface McpServerDto {
  args?: string[]
  command?: string
  name: string
  transport: 'stdio' | 'sse' | 'streamable-http'
  url?: string
}

export type AgentCapabilityField =
  | 'customKwargs'
  | 'env'
  | 'harnessParameters'
  | 'mcpServers'
  | 'modelName'
  | 'skills'
  | 'timeouts'

export interface AgentParameterDto {
  choices?: string[]
  defaultValue?: boolean | number | string
  key: string
  kind: 'boolean' | 'number' | 'select' | 'text'
  label: string
  source: 'env' | 'kwarg'
}

export interface AgentAuthenticationModeDto {
  environmentVariables: string[]
  label: string
  value: string
}

export interface AgentCapabilitiesDto {
  authenticationModes: AgentAuthenticationModeDto[]
  environmentVariables: string[]
  parameters: AgentParameterDto[]
  supportedFields: AgentCapabilityField[]
}

export interface AgentInputDto {
  agentName: string
  authenticationMode?: string
  env: KeyValueDto[]
  harness: string
  id: string
  importPath?: string
  kwargs: string
  mcpServers: McpServerDto[]
  modelPricing: ModelPricingDto[]
  models: string[]
  setupTimeoutSeconds?: number
  timeoutSeconds?: number
  skillSources: string[]
  maxTimeoutSeconds?: number
}

export interface ModelPricingDto {
  modelName: string
  source: 'reported' | 'litellm' | 'custom'
  inputCacheMissUsdPerMillion?: number
  inputCacheHitUsdPerMillion?: number
  outputUsdPerMillion?: number
}

export interface ModelPricingPreviewDto {
  catalogModelName: string
  inputCacheHitUsdPerMillion: number
  inputCacheMissUsdPerMillion: number
  modelName: string
  outputUsdPerMillion: number
  source: 'litellm'
  sourceUrl?: string
}

export interface AgentDto extends AgentInputDto {
  capabilities: AgentCapabilitiesDto
  status: AgentAvailability
}

export interface AgentQuery extends ListQuery {
  status?: AgentAvailability
}

export interface HarnessDto {
  capabilities: AgentCapabilitiesDto
  name: string
  source: 'harbor-built-in'
}

export type EnvironmentProfileType = 'built-in' | 'custom'

export interface EnvironmentDto {
  allowedHosts: string[]
  cpuPolicy: string
  deleteAfterRun: boolean
  dockerComposePaths: string[]
  env: KeyValueDto[]
  environmentType: string
  forceBuild: boolean
  id: string
  importPath?: string
  kwargs: string
  memoryPolicy: string
  mounts: string
  name: string
  overrideCpus: string
  overrideGpus: string
  overrideMemoryMb: string
  overrideStorageMb: string
  overrideTpu: string
  profileType: EnvironmentProfileType
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
  costUsd: number | null
  datasetRef: string
  harness: string
  jobId: string
  metric: string
  model: string
  rank: number
  reportPath?: string
  runtimeSeconds: number | null
  score: ScoreDto | null
  submittedAt: string
  tokenUsageM: number | null
  trial: TrialProgressDto
}

export interface LeaderboardQuery extends ListQuery {
  dataset: string
  metric?: string
}

export interface DockerStartCommandDto {
  command: string
}

export type SystemComponentKind =
  | 'ornnlab-service'
  | 'harbor-cli'
  | 'docker'
  | 'storage'
  | 'resource-cpu'
  | 'resource-gpu'
  | 'resource-storage'

export type SystemAction = 'check-update' | 'restart-service' | 'clean-docker-cache' | 'clean-storage-cache'

interface SystemComponentBaseDto {
  actions: SystemAction[]
}

export type SystemComponentDto =
  | (SystemComponentBaseDto & {
      kind: 'ornnlab-service'
      state: 'running' | 'starting' | 'restarting' | 'degraded' | 'stopped' | 'error'
      endpoint: string | null
      logsPath: string
      error: string | null
    })
  | (SystemComponentBaseDto & {
      kind: 'harbor-cli'
      state: 'installed' | 'not-installed'
      version: string | null
      executablePath: string
    })
  | (SystemComponentBaseDto & {
      kind: 'docker'
      state: 'running' | 'not-running' | 'not-installed' | 'error'
      context: string | null
      clientVersion: string | null
      serverVersion: string | null
      startCommand: string
      executablePath: string
      error: string | null
    })
  | (SystemComponentBaseDto & {
      kind: 'storage'
      state: 'available' | 'unavailable'
      sizeBytes: number | null
      path: string
      error: string | null
    })
  | (SystemComponentBaseDto & {
      kind: 'resource-cpu'
      state: 'normal' | 'elevated' | 'high' | 'unavailable'
      usagePercent: number | null
      logicalCores: number | null
    })
  | (SystemComponentBaseDto & {
      kind: 'resource-gpu'
      state: 'normal' | 'elevated' | 'high' | 'not-detected' | 'error'
      usagePercent: number | null
      deviceCount: number
    })
  | (SystemComponentBaseDto & {
      kind: 'resource-storage'
      state: 'normal' | 'low' | 'critical' | 'unavailable'
      availableBytes: number | null
      totalBytes: number | null
      path: string
    })

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
  agentSetupTimeoutMultiplier: number
  agentName: string
  agentTimeoutMultiplier: number
  attempts: number
  concurrency: number
  datasetRef: string
  debug: boolean
  environmentPresetId: string
  environmentBuildTimeoutMultiplier: number
  extraInstructionPaths: string[]
  includeInLeaderboard: boolean
  jobName: string
  jobsDir: string
  maxRetries: number
  metric: 'sum' | 'min' | 'max' | 'mean' | 'uv-script'
  modelName: string
  notes: string
  retryExclude: string
  retryInclude: string
  retryMaxWaitSeconds: number
  retryMinWaitSeconds: number
  retryWaitMultiplier: number
  selectedTaskNames: string[] | null
  timeoutMultiplier: number
  verifierTimeoutMultiplier: number
  verifierMode: 'dataset-default' | 'skip'
}

export interface DatasetImportRequestDto {
  name: string
  path: string
  taskCount: number
  version: string
}

export interface DatasetParentPathRequestDto {
  parentPath: string
}

export interface DatasetPathRequestDto {
  path: string
}

export interface DatasetStoragePreferenceDto {
  parentPath: string
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
