import { describe, expect, it, vi } from 'vitest'
import type { AgentDto, CreateJobRequestDto, DatasetImportRequestDto, EnvironmentDto } from './contract'
import { createWebUiHttpClient } from './webUiClient'

describe('WebUiHttpClient', () => {
  it('turns a malformed response envelope into a contract error', async () => {
    const request = vi.fn<typeof fetch>().mockResolvedValue(
      new Response(JSON.stringify({ items: [] }), { status: 200 }),
    )
    const client = createWebUiHttpClient('/api/webui/v1', request)

    const response = await client.listJobs()

    expect(response.data).toBeNull()
    expect(response.error).toEqual({
      code: 'INVALID_API_RESPONSE',
      message: 'The server returned an invalid API response.',
    })
  })

  it('turns a transport failure into a contract error', async () => {
    const request = vi.fn<typeof fetch>().mockRejectedValue(new Error('offline'))
    const client = createWebUiHttpClient('/api/webui/v1', request)

    const response = await client.listDatasets()

    expect(response.data).toBeNull()
    expect(response.error).toEqual({
      code: 'NETWORK_REQUEST_FAILED',
      message: 'The API request could not be completed.',
    })
  })

  it('maps every visible write to a WebUI contract route rather than a legacy route', async () => {
    const request = vi.fn<typeof fetch>().mockResolvedValue(new Response(JSON.stringify({ data: { operation: {} }, error: null })))
    const client = createWebUiHttpClient('/api/webui/v1', request)
    const agent = {} as AgentDto
    const environment = {} as EnvironmentDto
    const job = {} as CreateJobRequestDto
    const dataset = {} as DatasetImportRequestDto

    await Promise.all([
      client.cancelJob('job_1'),
      client.cancelDatasetDownload('dataset@1'),
      client.cancelOperation('operation_1'),
      client.checkForSystemUpdate(),
      client.cleanDockerCache(),
      client.cleanStorageCache(),
      client.copyEnvironment('environment_1'),
      client.createAgent(agent),
      client.createEnvironment(environment),
      client.createJob(job),
      client.deleteAgent('agent_1'),
      client.deleteEnvironment('environment_1'),
      client.deleteLocalDataset('dataset@1'),
      client.downloadDataset('dataset@1'),
      client.importDataset(dataset),
      client.installSystemUpdate(),
      client.restartSystemService(),
      client.retryJob('job_1'),
      client.resumeJob('job_1'),
      client.syncDataset('dataset@1'),
      client.updateAgent('agent_1', agent),
      client.updateEnvironment('environment_1', environment),
      client.updateJobLeaderboard('job_1', { includeInLeaderboard: false }),
    ])

    const urls = request.mock.calls.map(([url]) => String(url))
    expect(urls).toEqual(expect.arrayContaining([
      '/api/webui/v1/jobs/job_1/cancel',
      '/api/webui/v1/operations/operation_1/cancel',
      '/api/webui/v1/datasets/dataset%401/download',
      '/api/webui/v1/agents',
      '/api/webui/v1/environments',
      '/api/webui/v1/system/service/update',
      '/api/webui/v1/system/cache/storage/clean',
    ]))
    expect(urls.every((url) => url.startsWith('/api/webui/v1/'))).toBe(true)
  })
})
