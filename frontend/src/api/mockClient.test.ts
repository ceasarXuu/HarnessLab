import { describe, expect, it } from 'vitest'
import type { Operation } from './contract'
import { createMockWebUiClient } from './mockClient'

describe('MockWebUiClient', () => {
  it('uses the shared Operation contract for asynchronous actions', () => {
    const operation: Operation = {
      id: 'op_001',
      resourceType: 'dataset',
      status: 'queued',
      type: 'dataset.download',
    }

    expect(operation.status).toBe('queued')
  })

  it('returns a contract envelope with structured Job DTOs', async () => {
    const client = createMockWebUiClient()

    const response = await client.listJobs({ q: 'terminal' })

    expect(response.error).toBeNull()
    expect(response.data).not.toBeNull()
    if (!response.data) throw new Error('Expected Job page data')
    expect(response.data.items).toEqual(expect.arrayContaining([
      expect.objectContaining({
      id: 'job_91a7',
      datasetRef: 'terminal-bench@2.0',
      trial: { completed: 18, total: 64 },
      tokenUsageM: 0.0184,
      }),
      expect.objectContaining({ id: 'job_64f2', datasetRef: 'terminal-bench@2.0' }),
    ]))
  })

  it('returns Dataset tasks filtered by search through the contract', async () => {
    const client = createMockWebUiClient()

    const response = await client.listDatasetTasks('terminal-bench@2.0', { q: 'apt' })

    expect(response.error).toBeNull()
    expect(response.data).not.toBeNull()
    if (!response.data) throw new Error('Expected Dataset Task page data')
    expect(response.data.items.map((task) => task.name)).toEqual(['apt-setup'])
  })

  it('does not expose a downloaded Dataset until its asynchronous Operation completes', async () => {
    const client = createMockWebUiClient()

    const submitted = await client.downloadDataset('swebench-verified@1.0', { parentPath: '/tmp/datasets' })
    const operationId = submitted.data?.operation.id ?? ''

    expect((await client.getDataset('swebench-verified@1.0')).data?.download.status).toBe('not-downloaded')
    expect((await client.getOperation(operationId)).data?.status).toBe('running')
    expect((await client.getOperation(operationId)).data?.status).toBe('completed')
    expect((await client.getDataset('swebench-verified@1.0')).data?.download.status).toBe('downloaded')
    expect((await client.removeDatasetRegistration('swebench-verified@1.0')).error?.code).toBe(
      'DATASET_MANAGED_REGISTRATION_REQUIRED',
    )
  })

  it('filters Agents and Environments by their structured query fields', async () => {
    const client = createMockWebUiClient()

    const [agentsResponse, environmentsResponse] = await Promise.all([
      client.listAgents({ status: 'needs-token', type: 'custom' }),
      client.listEnvironments({ type: 'built-in' }),
    ])

    expect(agentsResponse.data?.items).toEqual([
      expect.objectContaining({ id: 'local-repair-agent', status: 'needs-token', type: 'custom' }),
    ])
    expect(environmentsResponse.data?.items).toEqual([
      expect.objectContaining({ id: 'docker-default', profileType: 'built-in' }),
    ])
  })

  it('returns a not-found contract error for an unknown Job', async () => {
    const client = createMockWebUiClient()

    const response = await client.getJob('missing')

    expect(response.data).toBeNull()
    expect(response.error).toEqual({ code: 'JOB_NOT_FOUND', message: 'Job not found' })
  })

  it('does not fabricate a native directory selection in mock mode', async () => {
    const response = await createMockWebUiClient().chooseDirectory()

    expect(response.data).toBeNull()
    expect(response.error?.code).toBe('NATIVE_DIRECTORY_PICKER_UNAVAILABLE')
  })

  it('rejects built-in harness entries when a Job requires a configured Agent profile', async () => {
    const client = createMockWebUiClient()

    const response = await client.createJob({
      config: {
        agentSetupTimeoutMultiplier: 1,
        agentName: 'Claude Code default',
        agentTimeoutMultiplier: 1,
        attempts: 1,
        concurrency: 1,
        datasetRef: 'terminal-bench@2.0',
        debug: false,
        environmentPresetId: 'docker-default',
        environmentBuildTimeoutMultiplier: 1,
        extraInstructionPaths: [],
        includeInLeaderboard: true,
        jobName: 'invalid-agent-job',
        jobsDir: 'jobs/invalid-agent-job',
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
    })

    expect(response.data).toBeNull()
    expect(response.error?.code).toBe('AGENT_PROFILE_REQUIRED')
  })
})
