import { describe, expect, it } from 'vitest'
import { createMockWebUiClient } from './mockClient'
import type { AgentDto, EnvironmentDto } from './contract'

describe('Stage 3 Operation write boundary', () => {
  it('matches the backend lifecycle for synchronous and asynchronous writes', async () => {
    const client = createMockWebUiClient()
    const agents = await client.listAgents()
    const environments = await client.listEnvironments()
    const agent = agents.data?.items.find((item) => item.type === 'custom') as AgentDto
    const environment = environments.data?.items.find((item) => item.profileType === 'custom') as EnvironmentDto

    const [job, datasetDownload, datasetMove, datasetRelocate, datasetRegistration, agentUpdate, environmentUpdate, leaderboard, system, update] = await Promise.all([
      client.createJob({
        config: {
          agentSetupTimeoutMultiplier: 1,
          agentName: agent.agentName,
          agentTimeoutMultiplier: 1,
          attempts: 1,
          concurrency: 1,
          datasetRef: 'terminal-bench@2.0',
          debug: false,
          environmentPresetId: 'docker-default',
          environmentBuildTimeoutMultiplier: 1,
          extraInstructionPaths: [],
          includeInLeaderboard: true,
          jobName: 'operation-test',
          jobsDir: 'jobs/operation-test',
          maxRetries: 0,
          metric: 'mean',
          notes: '',
          retryExclude: '',
          retryInclude: '',
          retryMaxWaitSeconds: 30,
          retryMinWaitSeconds: 2,
          retryWaitMultiplier: 1.5,
          selectedTaskNames: null,
          timeoutMultiplier: 1,
          verifierTimeoutMultiplier: 1,
          verifierMode: 'dataset-default',
        },
        runImmediately: true,
      }),
      client.downloadDataset('swebench-verified@1.0', { parentPath: '/tmp/datasets' }),
      client.moveDataset('terminal-bench@2.0', { parentPath: '/tmp/relocated' }),
      client.relocateDataset('terminal-bench@2.0', { path: '/tmp/relocated/terminal-bench@2.0' }),
      client.removeDatasetRegistration('harbor/hello-world@latest'),
      client.updateAgent(agent.id, agent),
      client.updateEnvironment(environment.id, environment),
      client.updateJobLeaderboard('job_91a7', { includeInLeaderboard: false }),
      client.cleanStorageCache(),
      client.installSystemUpdate(),
    ])

    expect(job.data?.operation).toMatchObject({ status: 'completed', type: 'run-job' })
    expect(datasetDownload.data?.operation.status).toBe('queued')
    expect(datasetMove.data?.operation.status).toBe('queued')
    expect(datasetRelocate.data?.operation.status).toBe('completed')
    expect(datasetRegistration.data?.operation.status).toBe('completed')
    expect(agentUpdate.data?.operation.status).toBe('completed')
    expect(environmentUpdate.data?.operation.status).toBe('completed')
    expect(leaderboard.data?.operation.status).toBe('completed')
    expect(system.data?.operation.status).toBe('queued')
    expect(update.data?.operation.status).toBe('queued')
  })

  it('exposes Operation polling for every submitted mock write', async () => {
    const client = createMockWebUiClient()
    const submitted = await client.cleanStorageCache()
    const operationId = submitted.data?.operation.id

    expect(operationId).toBeTruthy()
    expect((await client.getOperation(operationId ?? '')).data?.status).toBe('running')
    expect((await client.getOperation(operationId ?? '')).data?.status).toBe('completed')
  })

  it('reports supervisor availability instead of simulating a service restart', async () => {
    const client = createMockWebUiClient()
    const submitted = await client.restartSystemService()

    expect(submitted.data?.operation).toMatchObject({
      status: 'failed',
      error: { code: 'SERVICE_RESTART_UNAVAILABLE' },
    })
  })

  it('cancels an in-flight Operation through the shared Operation contract', async () => {
    const client = createMockWebUiClient()
    const submitted = await client.cleanDockerCache()
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
