import { describe, expect, it } from 'vitest'
import type { DatasetDto, JobDto } from './contract'
import { datasetDtoToRow, jobDtoToHarborJob } from './viewModels'

describe('WebUI view model mappers', () => {
  it('formats structured Job fields only at the UI boundary', () => {
    const job: JobDto = {
      id: 'job_001',
      name: 'benchmark-run',
      status: 'completed',
      datasetRef: 'terminal-bench@2.0',
      agentName: 'Claude Code default',
      harness: 'claude-code',
      model: 'claude-haiku-4-5',
      environmentName: 'docker',
      trial: { completed: 64, total: 64 },
      score: { kind: 'percentage', value: 72.5 },
      costUsd: 3.42,
      tokenUsageM: 0.0184,
      runtimeSeconds: 2538,
      createdAt: '2026-07-10T01:02:03Z',
      includeInLeaderboard: true,
    }

    expect(jobDtoToHarborJob(job)).toMatchObject({
      dataset: 'terminal-bench@2.0',
      trials: '64 / 64',
      score: '72.5%',
      cost: '$3.42',
      tokenUsage: '0.0184M',
      runtimeDuration: '00:42:18',
    })
  })

  it('maps Dataset download data without exposing mock-only detail fields', () => {
    const dataset: DatasetDto = {
      ref: 'terminal-bench@2.0',
      name: 'terminal-bench',
      version: '2.0',
      visibility: 'public',
      taskCount: 64,
      source: 'harbor registry',
      download: {
        status: 'downloaded',
        path: '~/.cache/harbor/datasets/terminal-bench',
        sizeBytes: 1288490188,
      },
      registryUrl: 'https://hub.harborframework.com',
      splits: ['test', 'nightly'],
    }

    expect(datasetDtoToRow(dataset)).toMatchObject({
      name: 'terminal-bench',
      downloadStatus: 'downloaded',
      size: '1.2 GB',
      splits: ['test', 'nightly'],
    })
  })
})
