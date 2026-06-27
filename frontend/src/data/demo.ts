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

export const events: EventLog[] = [
  { time: '14:18:21', level: 'success', message: 'JobConfig persisted to harbor.config.json' },
  { time: '14:18:23', level: 'info', message: 'Docker context resolved: colima' },
  { time: '14:18:26', level: 'info', message: 'Trial terminal-bench/apt-setup started' },
  { time: '14:19:04', level: 'warning', message: 'Verifier retry scheduled for flaky shell assertion' },
  { time: '14:19:12', level: 'success', message: '18 trials completed, 46 still pending' },
]

export const taskRows = [
  ['apt-setup', 'linux', 'ok', '45s'],
  ['git-rebase-conflict', 'linux', 'running', '2m'],
  ['sqlite-log-repair', 'linux', 'queued', '-'],
  ['python-env-pin', 'linux', 'queued', '-'],
]
