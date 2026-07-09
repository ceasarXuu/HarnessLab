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
    expect(response.data.items).toHaveLength(2)
    expect(response.data.items[0]).toMatchObject({
      id: 'job_91a7',
      datasetRef: 'terminal-bench@2.0',
      trial: { completed: 18, total: 64 },
      tokenUsageM: 0.0184,
    })
  })

  it('returns Dataset tasks filtered by split through the contract', async () => {
    const client = createMockWebUiClient()

    const response = await client.listDatasetTasks('terminal-bench@2.0', { split: 'test' })

    expect(response.error).toBeNull()
    expect(response.data.items.map((task) => task.name)).toEqual(['apt-setup', 'git-rebase-conflict'])
  })

  it('returns a not-found contract error for an unknown Job', async () => {
    const client = createMockWebUiClient()

    const response = await client.getJob('missing')

    expect(response.data).toBeNull()
    expect(response.error).toEqual({ code: 'JOB_NOT_FOUND', message: 'Job not found' })
  })
})
