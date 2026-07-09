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
