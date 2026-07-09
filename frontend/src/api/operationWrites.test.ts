import { describe, expect, it } from 'vitest'
import { createMockWebUiClient } from './mockClient'
import type { AgentDto, EnvironmentDto } from './contract'

describe('Stage 2 Operation write boundary', () => {
  it('returns queued Operations for representative writes instead of mutating page state directly', async () => {
    const client = createMockWebUiClient()
    const agents = await client.listAgents()
    const environments = await client.listEnvironments()
    const agent = agents.data?.items.find((item) => item.type === 'custom') as AgentDto
    const environment = environments.data?.items.find((item) => item.profileType === 'custom') as EnvironmentDto

    const [job, dataset, agentUpdate, environmentUpdate, leaderboard, system, update] = await Promise.all([
      client.createJob({
        config: {
          agentEnv: [],
          agentKwargs: '',
          agentName: 'claude-code',
          attempts: 1,
          concurrency: 1,
          datasetRef: 'terminal-bench@2.0',
          debug: false,
          environmentPresetId: 'docker-default',
          includeInLeaderboard: true,
          jobName: 'operation-test',
          jobsDir: 'jobs/operation-test',
          maxRetries: 0,
          metric: 'pass@1 mean',
          model: 'claude-haiku-4-5',
          notes: '',
          retryExclude: '',
          retryInclude: '',
          retryIntervalPolicy: 'standard',
          retryMaxWaitSeconds: 30,
          retryMinWaitSeconds: 2,
          retryWaitMultiplier: 1.5,
          selectedTaskNames: null,
          split: 'test',
          timeoutMultiplier: 1,
          timeoutPolicy: 'standard',
          verifierMode: 'dataset-default',
        },
        runImmediately: true,
      }),
      client.downloadDataset('swe-bench-lite@2026.06'),
      client.updateAgent(agent.id, agent),
      client.updateEnvironment(environment.id, environment),
      client.updateJobLeaderboard('job_91a7', { includeInLeaderboard: false }),
      client.cleanStorageCache(),
      client.installSystemUpdate(),
    ])

    expect(job.data?.operation.status).toBe('queued')
    expect(dataset.data?.operation.status).toBe('queued')
    expect(agentUpdate.data?.operation.status).toBe('queued')
    expect(environmentUpdate.data?.operation.status).toBe('queued')
    expect(leaderboard.data?.operation.status).toBe('queued')
    expect(system.data?.operation.status).toBe('queued')
    expect(update.data?.operation.status).toBe('queued')
  })

  it('exposes Operation polling for every submitted mock write', async () => {
    const client = createMockWebUiClient()
    const submitted = await client.restartSystemService()
    const operationId = submitted.data?.operation.id

    expect(operationId).toBeTruthy()
    expect((await client.getOperation(operationId ?? '')).data?.status).toBe('running')
    expect((await client.getOperation(operationId ?? '')).data?.status).toBe('completed')
  })

  it('cancels an in-flight Operation through the shared Operation contract', async () => {
    const client = createMockWebUiClient()
    const submitted = await client.restartSystemService()
    const operationId = submitted.data?.operation.id

    const cancelled = await client.cancelOperation(operationId ?? '')

    expect(cancelled.error).toBeNull()
    expect(cancelled.data?.operation.status).toBe('cancelled')
    expect((await client.getOperation(operationId ?? '')).data?.status).toBe('cancelled')
  })

  it('derives selectable leaderboard Datasets from current leaderboard state', async () => {
    const client = createMockWebUiClient()

    expect((await client.listLeaderboardDatasets()).data?.items.map((item) => item.ref)).toContain('harbor/hello-world@latest')
    await client.updateJobLeaderboard('job_99ab', { includeInLeaderboard: false })

    expect((await client.listLeaderboardDatasets()).data?.items.map((item) => item.ref)).not.toContain('harbor/hello-world@latest')
  })
})
