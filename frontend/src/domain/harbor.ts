export type JobStatus = 'running' | 'queued' | 'completed' | 'failed'

export interface HarborJob {
  id: string
  name: string
  status: JobStatus
  dataset: string
  agent: string
  model: string
  environment: string
  trials: string
  score: string
  cost: string
  tokens: string
  tokenUsage: string
  runtimeDuration: string
  createdAt: string
  includeInLeaderboard: boolean
  jobDir?: string
  eventLogPath?: string
  artifactPaths?: string[]
  split?: string
  failureCode?: string
}

export interface EventLog {
  time: string
  level: 'info' | 'success' | 'warning' | 'error'
  message: string
}

export interface RunDraft {
  jobName: string
  jobsDir: string
  source: string
  split: string
  selectedTaskNames: string[] | null
  extraInstructions: string
  debug: boolean
  notes: string
  agent: string
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
  verifierMode: 'dataset-default' | 'custom' | 'skip'
  verifierImportPath: string
  verifierEnv: string
  verifierKwargs: string
  disableVerifier: boolean
  verifierMaxTimeoutSec: string
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
  metric: string
  includeInLeaderboard: boolean
}

export interface TaskRow {
  name: string
  dataset: string
  description: string
  jobId: string
  splits?: string[]
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

export interface SystemRow {
  kind: 'ornnlab-service' | 'resource-cpu' | 'resource-gpu' | 'resource-storage' | 'harbor-cli' | 'docker' | 'storage'
  component: string
  status: JobStatus | 'healthy'
  value: string
  path: string
}

export interface DatasetRow {
  name: string
  version: string
  visibility: 'public' | 'private'
  tasks: number
  source: string
  digest?: string
  updated?: string
  downloadStatus: 'downloaded' | 'not-downloaded'
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

export interface AgentRow {
  agentName: string
  harness: string
  type: 'built-in' | 'custom'
  adapter: string
  models: string
  status: 'available' | 'configured' | 'needs-token'
  source: string
  updated: string
  env?: string
  kwargs?: string
  skills?: string
  mcp?: string
  runtime?: string
  setupTimeout?: string
  maxTimeout?: string
  allowedHosts?: string
  compatibleModels?: string
  reasoningEffort?: string
  reasoningSummary?: string
  temperature?: string
  contextLength?: string
  apiKeyEnv?: string
  baseUrlEnv?: string
}

export interface EnvironmentRow {
  id: string
  name: string
  profileType: 'built-in' | 'custom'
  environmentType: string
  importPath: string
  networkMode: string
  dockerImage: string
  os: string
  cpus: string
  memoryMb: string
  storageMb: string
  gpus: string
  gpuTypes: string
  tpu: string
  skillsDir: string
  healthcheck: string
  workdir: string
  mounts: string
  env: string
  kwargs: string
  allowedHosts: string
  extraAllowedHosts: string
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
  split: string
  metric: string
  submitted: string
  reportPath: string
  comparabilityKey: string
  uploadedUrl: string
  submissionId: string
  configHash: string
  agentSnapshotHash: string
}
