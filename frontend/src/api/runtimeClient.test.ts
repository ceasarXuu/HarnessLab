import { describe, expect, it, vi } from 'vitest'
import { resolveWebUiDataMode } from './dataMode'
import { createRuntimeWebUiClient } from './runtimeClient'

describe('createRuntimeWebUiClient', () => {
  it('uses the HTTP client in API mode without falling back to mock data', async () => {
    const request = vi.fn<typeof fetch>().mockRejectedValue(new Error('offline'))
    const client = createRuntimeWebUiClient('api', request)

    const response = await client.listJobs()

    expect(request).toHaveBeenCalledWith('/api/webui/v1/jobs', undefined)
    expect(response.data).toBeNull()
    expect(response.error?.code).toBe('NETWORK_REQUEST_FAILED')
  })

  it('uses the offline mock client only when mock mode is selected', async () => {
    const client = createRuntimeWebUiClient('mock')

    const response = await client.listDatasets()

    expect(response.data?.items).toHaveLength(4)
    expect(response.error).toBeNull()
  })

  it('rejects explicit unsupported data modes instead of silently selecting mock', () => {
    expect(() => resolveWebUiDataMode('preview', 'mock')).toThrow('VITE_ORNNLAB_DATA_MODE')
    expect(resolveWebUiDataMode(undefined, 'mock')).toBe('mock')
    expect(resolveWebUiDataMode(undefined, 'api')).toBe('api')
  })
})
