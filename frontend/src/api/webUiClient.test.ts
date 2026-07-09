import { describe, expect, it, vi } from 'vitest'
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
})
