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
}

export interface EventLog {
  time: string
  level: 'info' | 'success' | 'warning' | 'error'
  message: string
}

export interface RunDraft {
  source: string
  agent: string
  model: string
  environment: string
  concurrency: number
  attempts: number
}

export interface TaskRow {
  name: string
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
}

export const initialDraft: RunDraft = {
  source: 'terminal-bench@2.0',
  agent: 'claude-code',
  model: 'anthropic/claude-haiku-4-5',
  environment: 'docker',
  concurrency: 4,
  attempts: 1,
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
  },
  {
    id: 'job_74c1',
    name: 'swe-bench-lite-regression',
    status: 'completed',
    dataset: 'swe-bench-lite',
    agent: 'codex-cli',
    model: 'gpt-5.1',
    environment: 'docker',
    trials: '300 / 300',
    score: '0.41',
    cost: '$92.18',
    updated: '1h ago',
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
  },
  {
    name: 'swe-bench-lite',
    version: '2026.06',
    visibility: 'private',
    tasks: 300,
    source: 'ScaleAI mirror',
    digest: 'sha256:72aa...0f44',
    updated: '1h ago',
  },
  {
    name: 'harbor/hello-world',
    version: 'latest',
    visibility: 'public',
    tasks: 8,
    source: 'local package',
    digest: 'sha256:100f...d6a0',
    updated: '3h ago',
  },
  {
    name: 'terminal-bench-nightly',
    version: 'nightly',
    visibility: 'private',
    tasks: 128,
    source: 'local cache',
    digest: 'sha256:f91b...aa02',
    updated: 'queued',
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
    jobId: 'job_91a7',
    os: 'linux',
    state: 'ok',
    duration: '45s',
    owner: 'terminal-bench-smoke',
    verifier: 'passed',
  },
  {
    name: 'git-rebase-conflict',
    jobId: 'job_91a7',
    os: 'linux',
    state: 'running',
    duration: '2m',
    owner: 'terminal-bench-smoke',
    verifier: 'pending',
  },
  {
    name: 'sqlite-log-repair',
    jobId: 'job_118b',
    os: 'linux',
    state: 'queued',
    duration: '-',
    owner: 'terminal-bench-nightly',
    verifier: 'waiting',
  },
  {
    name: 'python-env-pin',
    jobId: 'job_118b',
    os: 'linux',
    state: 'queued',
    duration: '-',
    owner: 'terminal-bench-nightly',
    verifier: 'waiting',
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
    logPath: 'trials/job_74c1/django-migration.log',
  },
]

export const systemRows: SystemRow[] = [
  {
    component: 'Harbor CLI',
    status: 'healthy',
    value: '0.13.x available',
    evidence: '~/.ornnlab/HarnessLab/bin/harbor',
  },
  {
    component: 'Docker',
    status: 'running',
    value: 'context colima',
    evidence: 'docker context inspect colima',
  },
  {
    component: 'Storage',
    status: 'completed',
    value: '~/.ornnlab/HarnessLab',
    evidence: 'artifact store writable',
  },
  {
    component: 'Verifier',
    status: 'queued',
    value: '1 retry pending',
    evidence: 'event log warning',
  },
]
