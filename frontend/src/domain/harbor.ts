export type JobStatus = 'draft' | 'running' | 'queued' | 'completed' | 'failed' | 'cancelled' | 'interrupted'

export interface JobTrialProgress {
  completed: number
  errored: number
  notPassed: number
  passed: number
  total: number
}

export interface HarborJob {
  id: string
  name: string
  status: JobStatus
  dataset: string
  agent: string
  model: string
  environment: string
  trial: JobTrialProgress
  trials: string
  score: string
  cost: string
  tokens: string
  tokenUsage: string
  runtimeSeconds: number | null
  runtimeDuration: string
  createdAt: string
  includeInLeaderboard: boolean
  canResume: boolean
  jobDir?: string
  eventLogPath?: string
  artifactPaths?: string[]
  failureCode?: string
}

export interface EventLog {
  time: string
  level: 'info' | 'success' | 'warning' | 'error'
  message: string
}

export interface DatasetTask {
  datasetRef: string
  description: string
  environment: import('../api/contract').DatasetTaskEnvironmentDto | null
  name: string
}

export interface RunDraft {
  jobName: string
  jobsDir: string
  source: string
  selectedTaskNames: string[] | null
  extraInstructions: string
  debug: boolean
  notes: string
  agent: string
  model: string
  environment: string
  verifierMode: 'dataset-default' | 'skip'
  concurrency: number
  attempts: number
  timeoutPolicy: 'standard' | 'strict' | 'relaxed' | 'custom'
  timeoutMultiplier: number
  agentTimeoutMultiplier: string
  verifierTimeoutMultiplier: string
  agentSetupTimeoutMultiplier: string
  environmentBuildTimeoutMultiplier: string
  maxRetries: number
  retryIntervalPolicy: 'standard' | 'fast' | 'slow' | 'custom'
  retryInclude: string
  retryExclude: string
  retryWaitMultiplier: string
  retryMinWaitSec: string
  retryMaxWaitSec: string
  metric: 'sum' | 'min' | 'max' | 'mean' | 'uv-script'
  includeInLeaderboard: boolean
}

export interface TaskRow {
  name: string
  dataset: string
  description: string
  jobId: string
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

export interface TrialRow {
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

export type SystemAction = 'check-update' | 'restart-service' | 'clean-docker-cache' | 'clean-storage-cache'

interface SystemRowBase {
  actions: SystemAction[]
}

export type SystemRow =
  | (SystemRowBase & { kind: 'ornnlab-service'; state: 'running' | 'starting' | 'restarting' | 'degraded' | 'stopped' | 'error'; endpoint: string | null; logsPath: string; error: string | null })
  | (SystemRowBase & { kind: 'harbor-cli'; state: 'installed' | 'not-installed'; version: string | null; executablePath: string })
  | (SystemRowBase & { kind: 'docker'; state: 'running' | 'not-running' | 'not-installed' | 'error'; context: string | null; clientVersion: string | null; serverVersion: string | null; startCommand: string; executablePath: string; error: string | null })
  | (SystemRowBase & { kind: 'storage'; state: 'available' | 'unavailable'; sizeBytes: number | null; path: string; error: string | null })
  | (SystemRowBase & { kind: 'resource-cpu'; state: 'normal' | 'elevated' | 'high' | 'unavailable'; usagePercent: number | null; logicalCores: number | null })
  | (SystemRowBase & { kind: 'resource-gpu'; state: 'normal' | 'elevated' | 'high' | 'not-detected' | 'error'; usagePercent: number | null; deviceCount: number })
  | (SystemRowBase & { kind: 'resource-storage'; state: 'normal' | 'low' | 'critical' | 'unavailable'; availableBytes: number | null; totalBytes: number | null; path: string })

export interface DatasetRow {
  name: string
  version: string
  visibility: 'public' | 'private'
  tasks: number
  source: string
  digest?: string
  updated?: string
  downloadStatus: 'downloaded' | 'downloading' | 'not-downloaded' | 'path-unavailable'
  downloadProgress?: number
  downloadPath?: string
  downloadedAt?: string
  storageKind?: 'external' | 'managed'
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
}

export interface AgentRow {
  id: string
  agentName: string
  authenticationMode?: string
  harness: string
  adapter: string
  models: string
  status: 'configured' | 'needs-token'
  source: string
  updated: string
  env?: string
  kwargs?: string
  skills?: string
  mcp?: string
  runtime?: string
  setupTimeout?: string
  timeout?: string
  maxTimeout?: string
  modelPricing: ModelPricing[]
  capabilities?: AgentCapabilities
}

export type ModelPricingSource = 'reported' | 'litellm' | 'custom'

export interface ModelPricing {
  modelName: string
  source: ModelPricingSource
  inputCacheMissUsdPerMillion?: number
  inputCacheHitUsdPerMillion?: number
  outputUsdPerMillion?: number
}

export interface HarnessTemplate {
  capabilities: AgentCapabilities
  name: string
  source: 'harbor-built-in'
}

export type AgentCapabilityField =
  | 'customKwargs'
  | 'env'
  | 'harnessParameters'
  | 'mcpServers'
  | 'modelName'
  | 'skills'
  | 'timeouts'

export interface AgentParameter {
  choices?: string[]
  defaultValue?: boolean | number | string
  key: string
  kind: 'boolean' | 'number' | 'select' | 'text'
  label: string
  source: 'env' | 'kwarg'
}

export interface AgentCapabilities {
  authenticationModes: AgentAuthenticationMode[]
  environmentVariables: string[]
  parameters: AgentParameter[]
  supportedFields: AgentCapabilityField[]
}

export interface AgentAuthenticationMode {
  environmentVariables: string[]
  label: string
  value: string
}

export interface EnvironmentRow {
  id: string
  name: string
  profileType: 'built-in' | 'custom'
  environmentType: string
  importPath: string
  mounts: string
  env: string
  kwargs: string
  allowedHosts: string
  forceBuild: boolean
  deleteAfterRun: boolean
  cpuPolicy: string
  memoryPolicy: string
  overrideCpus: string
  overrideMemoryMb: string
  overrideStorageMb: string
  overrideGpus: string
  overrideTpu: string
  dockerCompose: string
}

export interface LeaderboardRow {
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
  metric: string
  submitted: string
  reportPath: string
  comparabilityKey: string
}

export interface LeaderboardDataset {
  name: string
  ref: string
  version: string
}
