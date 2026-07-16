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
    state: 'running', endpoint: 'http://127.0.0.1:5173',
    logsPath: '~/.ornnlab/dev-service/logs', error: null,
    actions: ['check-update', 'restart-service'],
  },
  {
    kind: 'harbor-cli',
    state: 'installed', version: '0.13.2',
    executablePath: '~/.ornnlab/HarnessLab/bin/harbor', actions: [],
  },
  {
    kind: 'docker',
    state: 'running', context: 'colima', clientVersion: '28.1.1', serverVersion: '27.5.1', startCommand: 'colima start', executablePath: 'docker', error: null,
    actions: ['clean-docker-cache'],
  },
  {
    kind: 'storage',
    state: 'available', sizeBytes: 10_486, path: '~/.cache/harbor', error: null,
    actions: ['clean-storage-cache'],
  },
  {
    kind: 'resource-cpu',
    state: 'normal', usagePercent: 12, logicalCores: 12, actions: [],
  },
  {
    kind: 'resource-gpu',
    state: 'not-detected', usagePercent: null, deviceCount: 0, actions: [],
  },
  {
    kind: 'resource-storage',
    state: 'normal', availableBytes: 48 * 1024 ** 3, totalBytes: 1024 * 1024 ** 3,
    path: '/Volumes/XU-1TB-NPM', actions: [],
  },
]

export const degradedSystemRows: SystemRow[] = systemRows.map((row) => row.kind === 'ornnlab-service'
  ? {
      ...row,
      state: 'degraded', error: 'frontend exited',
  }
  : row)

export const startingSystemRows: SystemRow[] = systemRows.map((row) => row.kind === 'ornnlab-service'
  ? {
      ...row,
      state: 'starting',
    }
  : row)

export const restartingSystemRows: SystemRow[] = systemRows.map((row) => row.kind === 'ornnlab-service'
  ? {
      ...row,
      state: 'restarting',
    }
  : row)

export const stoppedSystemRows: SystemRow[] = systemRows.map((row) => row.kind === 'ornnlab-service'
  ? {
      ...row,
      state: 'stopped', endpoint: null,
  }
  : row)

export const errorSystemRows: SystemRow[] = systemRows.map((row) => row.kind === 'ornnlab-service'
  ? {
      ...row,
      state: 'error', error: 'frontend exceeded restart limit',
    }
  : row)
