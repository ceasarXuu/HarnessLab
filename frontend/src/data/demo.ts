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
  agent: string
  model: string
  agentImportPath: string
  agentEnv: string
  agentKwargs: string
  skills: string
  mcpConfig: string
  environment: string
  forceBuild: boolean
  deleteEnvironment: boolean
  cpus: string
  memoryMb: string
  storageMb: string
  gpus: string
  mounts: string
  dockerCompose: string
  verifierImportPath: string
  verifierEnv: string
  verifierKwargs: string
  disableVerifier: boolean
  concurrency: number
  attempts: number
  timeoutMultiplier: number
  agentTimeoutMultiplier: string
  verifierTimeoutMultiplier: string
  maxRetries: number
  retryInclude: string
  retryExclude: string
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
  logPath: string
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
}

export const initialDraft: RunDraft = {
  jobName: 'terminal-bench-smoke',
  jobsDir: 'jobs/terminal-bench-smoke',
  source: 'terminal-bench@2.0',
  taskFilter: 'apt-*',
  excludeFilter: 'flaky-*',
  taskLimit: 64,
  extraInstructions: 'instructions/hardening.md',
  agent: 'claude-code',
  model: 'anthropic/claude-haiku-4-5',
  agentImportPath: '',
  agentEnv: 'ANTHROPIC_API_KEY',
  agentKwargs: 'temperature=0',
  skills: '~/.ornnlab/skills/terminal-bench',
  mcpConfig: '~/.ornnlab/mcp/claude.mcp.json',
  environment: 'docker',
  forceBuild: false,
  deleteEnvironment: true,
  cpus: 'auto',
  memoryMb: '4096',
  storageMb: '20480',
  gpus: '0',
  mounts: '[{"source":"./cache","target":"/cache"}]',
  dockerCompose: 'compose.gpu.yaml',
  verifierImportPath: '',
  verifierEnv: 'PYTEST_ADDOPTS=-q',
  verifierKwargs: 'max_failures=1',
  disableVerifier: false,
  concurrency: 4,
  attempts: 1,
  timeoutMultiplier: 1,
  agentTimeoutMultiplier: '',
  verifierTimeoutMultiplier: '',
  maxRetries: 1,
  retryInclude: 'TimeoutError',
  retryExclude: 'ValidationError',
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
    updated: 'queued',
    jobDir: 'jobs/terminal-bench-nightly',
    split: 'nightly',
  },
]

export const datasetRows: DatasetRow[] = [
  {
    name: 'terminal-bench',
    version: '2.0',
    visibility: 'public',
    tasks: 64,
    source: 'harbor registry',
    digest: 'sha256:8f3a...b91c',
    updated: '12m ago',
    registryUrl: 'https://hub.harborframework.com',
    registryPath: 'registry/datasets/terminal-bench',
    downloadDir: '~/.cache/harbor/datasets/terminal-bench',
    manifestPath: 'dataset.toml',
    taskInclude: 'apt-*',
    taskExclude: 'flaky-*',
  },
  {
    name: 'swe-bench-lite',
    version: '2026.06',
    visibility: 'private',
    tasks: 300,
    source: 'ScaleAI mirror',
    digest: 'sha256:72aa...0f44',
    updated: '1h ago',
    registryUrl: 'https://hub.harborframework.com',
    registryPath: 'registry/datasets/swe-bench-lite',
    downloadDir: '~/.cache/harbor/datasets/swe-bench-lite',
    manifestPath: 'dataset.toml',
    taskInclude: 'django-*',
    taskExclude: 'large-*',
  },
  {
    name: 'harbor/hello-world',
    version: 'latest',
    visibility: 'public',
    tasks: 8,
    source: 'local package',
    digest: 'sha256:100f...d6a0',
    updated: '3h ago',
    registryUrl: 'local',
    registryPath: './examples/hello-world',
    downloadDir: './datasets/hello-world',
    manifestPath: 'dataset.toml',
  },
  {
    name: 'terminal-bench-nightly',
    version: 'nightly',
    visibility: 'private',
    tasks: 128,
    source: 'local cache',
    digest: 'sha256:f91b...aa02',
    updated: 'queued',
    registryUrl: 'local',
    registryPath: './nightly/terminal-bench',
    downloadDir: '~/.cache/harbor/datasets/nightly',
    manifestPath: 'dataset.toml',
    taskExclude: 'unstable-*',
  },
]

export const events: EventLog[] = [
  { time: '14:18:21', level: 'success', message: 'JobConfig persisted to harbor.config.json' },
  { time: '14:18:23', level: 'info', message: 'Docker context resolved: colima' },
  { time: '14:18:26', level: 'info', message: 'Trial terminal-bench/apt-setup started' },
  { time: '14:19:04', level: 'warning', message: 'Verifier retry scheduled for flaky shell assertion' },
  { time: '14:19:12', level: 'success', message: '18 trials completed, 46 still pending' },
]

export const taskRows: TaskRow[] = [
  {
    name: 'apt-setup',
    dataset: 'terminal-bench',
    description: 'Install packages and verify shell setup.',
    jobId: 'job_91a7',
    os: 'linux',
    state: 'ok',
    duration: '45s',
    owner: 'terminal-bench-smoke',
    verifier: 'passed',
  },
  {
    name: 'git-rebase-conflict',
    dataset: 'terminal-bench',
    description: 'Resolve a conflicted git rebase in a repo.',
    jobId: 'job_91a7',
    os: 'linux',
    state: 'running',
    duration: '2m',
    owner: 'terminal-bench-smoke',
    verifier: 'pending',
  },
  {
    name: 'sqlite-log-repair',
    dataset: 'terminal-bench',
    description: 'Repair corrupt logs and preserve SQLite rows.',
    jobId: 'job_118b',
    os: 'linux',
    state: 'queued',
    duration: '-',
    owner: 'terminal-bench-nightly',
    verifier: 'waiting',
  },
  {
    name: 'python-env-pin',
    dataset: 'terminal-bench',
    description: 'Pin Python dependencies for reproducible tests.',
    jobId: 'job_118b',
    os: 'linux',
    state: 'queued',
    duration: '-',
    owner: 'terminal-bench-nightly',
    verifier: 'waiting',
  },
]

export const agentRows: AgentRow[] = [
  {
    name: 'claude-code',
    type: 'built-in',
    adapter: 'harbor.adapters.claude_code',
    models: 'claude-haiku-4-5, claude-sonnet-4-5',
    status: 'available',
    source: 'Harbor built-in',
    updated: '12m ago',
    env: 'ANTHROPIC_API_KEY ready',
    kwargs: 'temperature=0',
    skills: '~/.ornnlab/skills/terminal-bench',
    mcp: '~/.ornnlab/mcp/claude.mcp.json',
    runtime: 'docker / 3600s',
  },
  {
    name: 'codex-cli',
    type: 'built-in',
    adapter: 'harbor.adapters.codex_cli',
    models: 'gpt-5.1',
    status: 'configured',
    source: 'Harbor built-in',
    updated: '1h ago',
    env: 'OPENAI_API_KEY ready',
    kwargs: 'reasoning_effort=medium',
    skills: '~/.ornnlab/skills/swe',
    mcp: '~/.ornnlab/mcp/codex.mcp.json',
    runtime: 'docker / 3600s',
  },
  {
    name: 'oracle',
    type: 'built-in',
    adapter: 'harbor.adapters.oracle',
    models: 'local-sim',
    status: 'available',
    source: 'Harbor built-in',
    updated: '3h ago',
    env: 'none',
    kwargs: 'mode=expected',
    skills: 'none',
    mcp: 'none',
    runtime: 'local / 600s',
  },
  {
    name: 'local-repair-agent',
    type: 'custom',
    adapter: 'agents.local_repair:Agent',
    models: 'qwen3-coder-local',
    status: 'needs-token',
    source: '~/.ornnlab/agents/local-repair.toml',
    updated: 'queued',
    env: 'LOCAL_MODEL_URL missing',
    kwargs: 'temperature=0.2',
    skills: '~/.ornnlab/skills/repair',
    mcp: '~/.ornnlab/mcp/local.mcp.json',
    runtime: 'docker / 1800s',
  },
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
    logPath: 'trials/job_91a7/apt-setup.log',
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
    logPath: 'trials/job_91a7/git-rebase-conflict.log',
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
    logPath: 'trials/job_55e9/hello-world.log',
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
    logPath: 'trials/job_74c1/django-migration.log',
  },
]
