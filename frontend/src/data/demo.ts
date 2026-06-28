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
  updated: string
  jobDir?: string
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
  taskFilter: string
  excludeFilter: string
  taskLimit: number
  extraInstructions: string
  debug: boolean
  quiet: boolean
  yes: boolean
  envFile: string
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
  shareTargets: string
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

export interface SystemRow {
  component: string
  status: JobStatus | 'healthy'
  value: string
  evidence: string
}

export interface DatasetRow {
  name: string
  version: string
  visibility: 'public' | 'private'
  tasks: number
  source: string
  digest: string
  updated: string
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
  name: string
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
  adapterReview?: string
}

export interface LeaderboardRow {
  dataset: string
  rank: number
  agent: string
  model: string
  score: string
  trials: string
  cost: string
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

export const initialDraft: RunDraft = {
  jobName: 'terminal-bench-smoke',
  jobsDir: 'jobs/terminal-bench-smoke',
  source: 'terminal-bench@2.0',
  taskFilter: 'apt-*',
  excludeFilter: 'flaky-*',
  taskLimit: 64,
  extraInstructions: 'instructions/hardening.md',
  debug: false,
  quiet: false,
  yes: true,
  envFile: '.env.harbor',
  agent: 'claude-code',
  model: 'anthropic/claude-haiku-4-5',
  agentImportPath: '',
  agentEnv: 'ANTHROPIC_API_KEY',
  agentKwargs: 'temperature=0',
  allowAgentHosts: 'api.anthropic.com,hub.harborframework.com',
  skills: '~/.ornnlab/skills/terminal-bench',
  mcpConfig: '~/.ornnlab/mcp/claude.mcp.json',
  environment: 'docker',
  environmentImportPath: '',
  environmentEnv: 'HTTP_PROXY=',
  environmentKwargs: 'reuse_layers=true',
  allowEnvironmentHosts: 'pypi.org,github.com',
  forceBuild: false,
  deleteEnvironment: true,
  suppressOverrideWarnings: false,
  cpus: 'auto',
  cpuOverride: '4',
  memoryMb: '4096',
  storageMb: '20480',
  gpus: '0',
  tpu: '',
  mounts: '[{"source":"./cache","target":"/cache"}]',
  dockerCompose: 'compose.gpu.yaml',
  verifierImportPath: '',
  verifierEnv: 'PYTEST_ADDOPTS=-q',
  verifierKwargs: 'max_failures=1',
  disableVerifier: false,
  verifierMaxTimeoutSec: '900',
  concurrency: 4,
  attempts: 1,
  timeoutMultiplier: 1,
  agentTimeoutMultiplier: '',
  verifierTimeoutMultiplier: '',
  agentSetupTimeoutMultiplier: '',
  environmentBuildTimeoutMultiplier: '',
  maxRetries: 1,
  retryInclude: 'TimeoutError',
  retryExclude: 'ValidationError',
  retryWaitMultiplier: '1.5',
  retryMinWaitSec: '2',
  retryMaxWaitSec: '30',
  artifacts: '/workspace/result.json,/workspace/logs',
  metric: 'mean',
  plugins: 'harbor.plugins.cost:CostPlugin',
  upload: false,
  visibility: 'private',
  shareTargets: '@ornn',
}

export const jobs: HarborJob[] = [
  {
    id: 'job_91a7',
    name: 'terminal-bench-smoke',
    status: 'running',
    dataset: 'terminal-bench@2.0',
    agent: 'claude-code',
    model: 'claude-haiku-4-5',
    environment: 'docker',
    trials: '18 / 64',
    score: '0.72',
    cost: '$3.42',
    tokens: '18.4k',
    updated: '2m ago',
    jobDir: 'jobs/terminal-bench-smoke',
    split: 'test',
  },
  {
    id: 'job_74c1',
    name: 'swe-bench-lite-regression',
    status: 'completed',
    dataset: 'swe-bench-lite@2026.06',
    agent: 'codex-cli',
    model: 'gpt-5.1',
    environment: 'docker',
    trials: '300 / 300',
    score: '0.41',
    cost: '$92.18',
    tokens: '1.8M',
    updated: '1h ago',
    jobDir: 'jobs/swe-bench-lite-regression',
    split: 'verified',
  },
  {
    id: 'job_55e9',
    name: 'harbor-hello-world',
    status: 'failed',
    dataset: 'harbor/hello-world',
    agent: 'claude-code',
    model: 'claude-sonnet-4-5',
    environment: 'docker',
    trials: '3 / 8',
    score: '-',
    cost: '$0.63',
    tokens: '3.2k',
    updated: '3h ago',
    jobDir: 'jobs/harbor-hello-world',
    split: 'smoke',
    failureCode: 'verifier_assertion_failed',
  },
  {
    id: 'job_118b',
    name: 'terminal-bench-nightly',
    status: 'queued',
    dataset: 'terminal-bench@2.0',
    agent: 'oracle',
    model: 'local-sim',
    environment: 'docker',
    trials: '0 / 128',
    score: '-',
    cost: '$0.00',
    tokens: '-',
    updated: 'queued',
    jobDir: 'jobs/terminal-bench-nightly',
    split: 'nightly',
  },
]

export const events: EventLog[] = [
  { time: '14:18:21', level: 'success', message: 'JobConfig persisted to harbor.config.json' },
  { time: '14:18:23', level: 'info', message: 'Docker context resolved: colima' },
  { time: '14:18:26', level: 'info', message: 'Trial terminal-bench/apt-setup started' },
  { time: '14:19:04', level: 'warning', message: 'Verifier retry scheduled for flaky shell assertion' },
  { time: '14:19:12', level: 'success', message: '18 trials completed, 46 still pending' },
]

export const trialRows: TrialRow[] = [
  {
    id: 'trial_001',
    jobId: 'job_91a7',
    task: 'apt-setup',
    result: 'passed',
    score: '1.00',
    retries: 0,
    duration: '45s',
    cost: '$0.11',
    tokens: '620',
    progress: 'completed',
    logPath: 'trials/job_91a7/apt-setup.log',
    analysisPath: 'trials/job_91a7/apt-setup.analysis.json',
    verifierEvidence: 'pytest passed',
    artifactPath: 'trials/job_91a7/apt-setup/result.json',
  },
  {
    id: 'trial_002',
    jobId: 'job_91a7',
    task: 'git-rebase-conflict',
    result: 'running',
    score: '-',
    retries: 1,
    duration: '2m',
    cost: '$0.34',
    tokens: '1.9k',
    progress: 'running 62%',
    logPath: 'trials/job_91a7/git-rebase-conflict.log',
    analysisPath: 'pending',
    verifierEvidence: 'verifier pending',
    artifactPath: 'trials/job_91a7/git-rebase-conflict/',
  },
  {
    id: 'trial_003',
    jobId: 'job_55e9',
    task: 'hello-world',
    result: 'failed',
    score: '0.00',
    retries: 2,
    duration: '38s',
    cost: '$0.63',
    tokens: '3.2k',
    progress: 'failed',
    logPath: 'trials/job_55e9/hello-world.log',
    analysisPath: 'trials/job_55e9/hello-world.analysis.json',
    verifierEvidence: 'assertion failed',
    artifactPath: 'trials/job_55e9/hello-world/result.json',
  },
  {
    id: 'trial_004',
    jobId: 'job_74c1',
    task: 'django-migration',
    result: 'passed',
    score: '0.41',
    retries: 0,
    duration: '6m',
    cost: '$1.92',
    tokens: '24.1k',
    progress: 'completed',
    logPath: 'trials/job_74c1/django-migration.log',
    analysisPath: 'trials/job_74c1/django-migration.analysis.json',
    verifierEvidence: 'resolved mean evidence',
    artifactPath: 'trials/job_74c1/django-migration/result.json',
  },
]
