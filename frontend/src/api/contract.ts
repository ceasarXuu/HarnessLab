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

export type DatasetVisibility = 'public' | 'private'
export type DatasetDownloadStatus = 'downloaded' | 'not-downloaded' | 'path-unavailable'
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
  name: string
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

export type AgentProfileType = 'built-in' | 'custom'
export type AgentAvailability = 'available' | 'configured' | 'needs-token'

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

export interface AgentCapabilitiesDto {
  parameters: AgentParameterDto[]
  supportedFields: AgentCapabilityField[]
}

export interface AgentInputDto {
  agentName: string
  env: KeyValueDto[]
  harness: string
  id: string
  importPath?: string
  kwargs: string
  mcpServers: McpServerDto[]
  models: string[]
  setupTimeoutSeconds?: number
  timeoutSeconds?: number
  skillSources: string[]
  type: AgentProfileType
  maxTimeoutSeconds?: number
}

export interface AgentDto extends AgentInputDto {
  capabilities: AgentCapabilitiesDto
  status: AgentAvailability
}

export interface AgentQuery extends ListQuery {
  status?: AgentAvailability
  type?: AgentProfileType
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

export type SystemComponentKind =
  | 'ornnlab-service'
  | 'harbor-cli'
  | 'docker'
  | 'storage'
  | 'resource-cpu'
  | 'resource-gpu'
  | 'resource-storage'

export type SystemComponentStatus = JobStatus | 'healthy' | 'unavailable'

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
