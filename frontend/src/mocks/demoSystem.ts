import type { LeaderboardRow, SystemRow } from '../domain/harbor'

export const leaderboardRows: LeaderboardRow[] = [
  {
    dataset: 'terminal-bench@2.0',
    rank: 1,
    agentName: 'Claude Code default',
    harness: 'claude-code',
    model: 'claude-haiku-4-5',
    score: '72%',
    trials: '64 / 64',
    cost: '$12.48',
    tokens: '0.0184M',
    duration: '42m',
    jobId: 'job_91a7',
    metric: 'pass@1 mean',
    submitted: 'not submitted',
    reportPath: 'reports/job_91a7.json',
    comparabilityKey: 'terminal-bench@2.0:test:pass@1',
  },
  {
    dataset: 'terminal-bench@2.0',
    rank: 2,
    agentName: 'Codex CLI default',
    harness: 'codex',
    model: 'gpt-5.1',
    score: '68%',
    trials: '64 / 64',
    cost: '$15.90',
    tokens: '0.0210M',
    duration: '47m',
    jobId: 'job_64f2',
    metric: 'pass@1 mean',
    submitted: 'submitted',
    reportPath: 'reports/job_64f2.json',
    comparabilityKey: 'terminal-bench@2.0:test:pass@1',
  },
  {
    dataset: 'swebench-verified@1.0',
    rank: 1,
    agentName: 'Codex CLI default',
    harness: 'codex',
    model: 'gpt-5.1',
    score: '41%',
    trials: '500 / 500',
    cost: '$92.18',
    tokens: '1.8M',
    duration: '3h 20m',
    jobId: 'job_74c1',
    metric: 'resolved mean',
    submitted: 'not submitted',
    reportPath: 'reports/job_74c1.json',
    comparabilityKey: 'swebench-verified@1.0:verified:resolved',
  },
  {
    dataset: 'swebench-verified@1.0',
    rank: 2,
    agentName: 'Claude Code default',
    harness: 'claude-code',
    model: 'claude-sonnet-4-5',
    score: '39%',
    trials: '500 / 500',
    cost: '$104.20',
    tokens: '2.1M',
    duration: '3h 45m',
    jobId: 'job_83aa',
    metric: 'resolved mean',
    submitted: 'submitted',
    reportPath: 'reports/job_83aa.json',
    comparabilityKey: 'swebench-verified@1.0:verified:resolved',
  },
  {
    dataset: 'harbor/hello-world@latest',
    rank: 1,
    agentName: 'Oracle baseline',
    harness: 'oracle',
    model: 'local-sim',
    score: '100%',
    trials: '8 / 8',
    cost: '$0.00',
    tokens: '0M',
    duration: '2m',
    jobId: 'job_99ab',
    metric: 'pass@1',
    submitted: 'local only',
    reportPath: 'reports/job_99ab.json',
    comparabilityKey: 'harbor/hello-world@latest:smoke:pass@1',
  },
]

export const systemRows: SystemRow[] = [
  {
    kind: 'ornnlab-service',
    component: 'OrnnLab Service',
    status: 'healthy',
    value: 'running http://127.0.0.1:5173',
    path: '~/.ornnlab/dev-service/logs',
  },
  {
    kind: 'harbor-cli',
    component: 'Harbor CLI',
    status: 'healthy',
    value: '0.13.x available',
    path: '~/.ornnlab/HarnessLab/bin/harbor',
  },
  {
    kind: 'docker',
    component: 'Docker',
    status: 'running',
    value: 'context colima',
    path: 'docker context: colima',
  },
  {
    kind: 'storage',
    component: 'Storage',
    status: 'completed',
    value: '0.01 MB cache',
    path: '~/.cache/harbor',
  },
  {
    kind: 'resource-cpu',
    component: 'CPU Usage',
    status: 'running',
    value: '12%',
    path: 'system monitor: cpu',
  },
  {
    kind: 'resource-gpu',
    component: 'GPU Usage',
    status: 'running',
    value: '0%',
    path: 'system monitor: gpu',
  },
  {
    kind: 'resource-storage',
    component: 'Available Storage',
    status: 'healthy',
    value: '48Gi available',
    path: '/Volumes/XU-1TB-NPM',
  },
]

export const degradedSystemRows: SystemRow[] = systemRows.map((row) => row.kind === 'ornnlab-service'
  ? {
      ...row,
      status: 'failed',
      value: 'degraded frontend exited',
      path: '~/.ornnlab/dev-service/logs',
  }
  : row)

export const startingSystemRows: SystemRow[] = systemRows.map((row) => row.kind === 'ornnlab-service'
  ? {
      ...row,
      status: 'queued',
      value: 'starting http://127.0.0.1:5173',
      path: '~/.ornnlab/dev-service/logs',
    }
  : row)

export const restartingSystemRows: SystemRow[] = systemRows.map((row) => row.kind === 'ornnlab-service'
  ? {
      ...row,
      status: 'running',
      value: 'restarting http://127.0.0.1:5173',
      path: '~/.ornnlab/dev-service/logs',
    }
  : row)

export const stoppedSystemRows: SystemRow[] = systemRows.map((row) => row.kind === 'ornnlab-service'
  ? {
      ...row,
      status: 'unavailable',
      value: 'stopped',
      path: '~/.ornnlab/dev-service/logs',
  }
  : row)

export const errorSystemRows: SystemRow[] = systemRows.map((row) => row.kind === 'ornnlab-service'
  ? {
      ...row,
      status: 'failed',
      value: 'error frontend exceeded restart limit',
      path: '~/.ornnlab/dev-service/logs',
    }
  : row)
