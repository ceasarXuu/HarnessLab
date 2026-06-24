import { describe, it, expect, beforeEach, vi, afterEach } from 'vitest'
import { ApiError, createApiClient, ornnLabApi } from './client'

const mockFetch = vi.fn()
const originalFetch = globalThis.fetch

beforeEach(() => {
  globalThis.fetch = mockFetch as unknown as typeof fetch
  mockFetch.mockReset()
})

afterEach(() => {
  globalThis.fetch = originalFetch
})

const okJson = (data: unknown) =>
  new Response(JSON.stringify(data), { status: 200, headers: { 'Content-Type': 'application/json' } })

const okEmpty = () => new Response('', { status: 200 })

const errResponse = (status: number, body = '') =>
  new Response(body, { status })

// ─── createApiClient: basePath joining ─────────────────────

describe('createApiClient — path joining', () => {
  it('joins basePath + relative path', async () => {
    mockFetch.mockResolvedValueOnce(okJson({ ok: true }))
    const client = createApiClient('/api')
    await client.get('/system/status')
    expect(mockFetch).toHaveBeenCalledWith('/api/system/status', expect.any(Object))
  })

  it('normalizes trailing slash in basePath', async () => {
    mockFetch.mockResolvedValueOnce(okJson({}))
    const client = createApiClient('/api/')
    await client.get('/system/status')
    expect(mockFetch.mock.calls[0][0]).toBe('/api/system/status')
  })

  it('normalizes missing leading slash in path', async () => {
    mockFetch.mockResolvedValueOnce(okJson({}))
    const client = createApiClient('/api')
    await client.get('system/status')
    expect(mockFetch.mock.calls[0][0]).toBe('/api/system/status')
  })
})

// ─── headers + JSON ────────────────────────────────────────

describe('createApiClient — headers and body', () => {
  it('injects Content-Type: application/json on GET', async () => {
    mockFetch.mockResolvedValueOnce(okJson({}))
    const client = createApiClient('/api')
    await client.get('/x')
    const init = mockFetch.mock.calls[0][1] as RequestInit
    expect((init.headers as Record<string, string>)['Content-Type']).toBe('application/json')
  })

  it('serializes payload as JSON for POST', async () => {
    mockFetch.mockResolvedValueOnce(okJson({}))
    const client = createApiClient('/api')
    await client.post('/x', { a: 1, b: 'two' })
    const init = mockFetch.mock.calls[0][1] as RequestInit
    expect(init.method).toBe('POST')
    expect(init.body).toBe('{"a":1,"b":"two"}')
  })

  it('uses PUT method when called', async () => {
    mockFetch.mockResolvedValueOnce(okJson({}))
    const client = createApiClient('/api')
    await client.put('/x', { v: 1 })
    expect((mockFetch.mock.calls[0][1] as RequestInit).method).toBe('PUT')
  })

  it('uses DELETE method when called', async () => {
    mockFetch.mockResolvedValueOnce(okEmpty())
    const client = createApiClient('/api')
    await client.delete('/x')
    expect((mockFetch.mock.calls[0][1] as RequestInit).method).toBe('DELETE')
  })
})

// ─── query params ──────────────────────────────────────────

describe('createApiClient — query params (F4)', () => {
  it('appends query params to GET URL', async () => {
    mockFetch.mockResolvedValueOnce(okJson([]))
    const client = createApiClient('/api')
    await client.get('/leaderboard', { benchmark: 'terminal-bench' })
    expect(mockFetch.mock.calls[0][0]).toBe('/api/leaderboard?benchmark=terminal-bench')
  })

  it('appends query params to POST URL', async () => {
    mockFetch.mockResolvedValueOnce(okJson({}))
    const client = createApiClient('/api')
    await client.post('/system/doctor', {}, { logs: true })
    expect(mockFetch.mock.calls[0][0]).toBe('/api/system/doctor?logs=true')
  })

  it('handles multiple query params', async () => {
    mockFetch.mockResolvedValueOnce(okJson([]))
    const client = createApiClient('/api')
    await client.get('/runs/r1/events', { after: 100 })
    expect(mockFetch.mock.calls[0][0]).toBe('/api/runs/r1/events?after=100')
  })

  it('skips undefined/null query values', async () => {
    mockFetch.mockResolvedValueOnce(okJson([]))
    const client = createApiClient('/api')
    await client.get('/leaderboard', { benchmark: undefined, source: null })
    expect(mockFetch.mock.calls[0][0]).toBe('/api/leaderboard')
  })

  it('omits "?" when query is empty', async () => {
    mockFetch.mockResolvedValueOnce(okJson([]))
    const client = createApiClient('/api')
    await client.get('/agents')
    expect(mockFetch.mock.calls[0][0]).toBe('/api/agents')
  })

  it('URL-encodes special characters', async () => {
    mockFetch.mockResolvedValueOnce(okJson([]))
    const client = createApiClient('/api')
    await client.get('/leaderboard', { benchmark: 'a b/c&d' })
    // URLSearchParams encodes space as '+', slash and & as %2F %26
    expect(mockFetch.mock.calls[0][0]).toContain('benchmark=a+b%2Fc%26d')
  })
})

// ─── error handling ────────────────────────────────────────

describe('createApiClient — error handling', () => {
  it('throws ApiError on non-2xx response', async () => {
    mockFetch.mockResolvedValueOnce(errResponse(500, 'server boom'))
    const client = createApiClient('/api')
    await expect(client.get('/x')).rejects.toBeInstanceOf(ApiError)
  })

  it('ApiError carries status and payload', async () => {
    mockFetch.mockResolvedValueOnce(errResponse(404, 'not found'))
    const client = createApiClient('/api')
    try {
      await client.get('/x')
      throw new Error('should have thrown')
    } catch (err) {
      expect(err).toBeInstanceOf(ApiError)
      const apiErr = err as ApiError
      expect(apiErr.status).toBe(404)
      expect(apiErr.payload).toBe('not found')
    }
  })

  it('throws ApiError when response is not valid JSON', async () => {
    mockFetch.mockResolvedValueOnce(
      new Response('not-json-at-all', { status: 200 }),
    )
    const client = createApiClient('/api')
    await expect(client.get('/x')).rejects.toBeInstanceOf(ApiError)
  })

  it('returns undefined for empty 2xx body (DELETE)', async () => {
    mockFetch.mockResolvedValueOnce(okEmpty())
    const client = createApiClient('/api')
    const result = await client.delete<unknown>('/x')
    expect(result).toBeUndefined()
  })
})

// ─── ornnLabApi method-level URL/method correctness ────────

describe('ornnLabApi — URL & HTTP method per endpoint', () => {
  it('system.status → GET /api/system/status', async () => {
    mockFetch.mockResolvedValueOnce(okJson({}))
    await ornnLabApi.system.status()
    expect(mockFetch.mock.calls[0][0]).toBe('/api/system/status')
    expect((mockFetch.mock.calls[0][1] as RequestInit).method).toBeUndefined() // GET = no method override
  })

  it('system.doctor → POST /api/system/doctor?logs=true', async () => {
    mockFetch.mockResolvedValueOnce(okJson({}))
    await ornnLabApi.system.doctor(true)
    expect(mockFetch.mock.calls[0][0]).toBe('/api/system/doctor?logs=true')
    expect((mockFetch.mock.calls[0][1] as RequestInit).method).toBe('POST')
  })

  it('agents.list → GET /api/agents', async () => {
    mockFetch.mockResolvedValueOnce(okJson([]))
    await ornnLabApi.agents.list()
    expect(mockFetch.mock.calls[0][0]).toBe('/api/agents')
  })

  it('agents.create → POST /api/agents with payload', async () => {
    mockFetch.mockResolvedValueOnce(okJson({}))
    await ornnLabApi.agents.create({ name: 'a' })
    expect(mockFetch.mock.calls[0][0]).toBe('/api/agents')
    const init = mockFetch.mock.calls[0][1] as RequestInit
    expect(init.method).toBe('POST')
    expect(init.body).toBe('{"name":"a"}')
  })

  it('agents.compile → POST /api/agents/{id}/compile', async () => {
    mockFetch.mockResolvedValueOnce(okJson({}))
    await ornnLabApi.agents.compile('agt-1')
    expect(mockFetch.mock.calls[0][0]).toBe('/api/agents/agt-1/compile')
    expect((mockFetch.mock.calls[0][1] as RequestInit).method).toBe('POST')
  })

  it('agents.update → PUT /api/agents/{id}', async () => {
    mockFetch.mockResolvedValueOnce(okJson({}))
    await ornnLabApi.agents.update('agt-1', { name: 'b' })
    expect(mockFetch.mock.calls[0][0]).toBe('/api/agents/agt-1')
    expect((mockFetch.mock.calls[0][1] as RequestInit).method).toBe('PUT')
  })

  it('agents.delete → DELETE /api/agents/{id}', async () => {
    mockFetch.mockResolvedValueOnce(okEmpty())
    await ornnLabApi.agents.delete('agt-1')
    expect(mockFetch.mock.calls[0][0]).toBe('/api/agents/agt-1')
    expect((mockFetch.mock.calls[0][1] as RequestInit).method).toBe('DELETE')
  })

  it('runExperiment(id, wait=true) → POST with ?wait=true', async () => {
    mockFetch.mockResolvedValueOnce(okJson({}))
    await ornnLabApi.runExperiment('exp-1', true)
    expect(mockFetch.mock.calls[0][0]).toBe('/api/experiments/exp-1/run?wait=true')
  })

  it('leaderboard(benchmark) URL-encodes correctly', async () => {
    mockFetch.mockResolvedValueOnce(okJson([]))
    await ornnLabApi.leaderboard('terminal-bench')
    expect(mockFetch.mock.calls[0][0]).toBe('/api/leaderboard?benchmark=terminal-bench')
  })

  it('leaderboard() without param omits query', async () => {
    mockFetch.mockResolvedValueOnce(okJson([]))
    await ornnLabApi.leaderboard()
    expect(mockFetch.mock.calls[0][0]).toBe('/api/leaderboard')
  })

  it('experimentEvents(id, after) appends after param', async () => {
    mockFetch.mockResolvedValueOnce(okJson([]))
    await ornnLabApi.experimentEvents('exp-1', 50)
    expect(mockFetch.mock.calls[0][0]).toBe('/api/experiments/exp-1/events?after=50')
  })

  it('deleteTemplate → DELETE /api/templates/{id}', async () => {
    mockFetch.mockResolvedValueOnce(okEmpty())
    await ornnLabApi.deleteTemplate('tpl-1')
    expect(mockFetch.mock.calls[0][0]).toBe('/api/templates/tpl-1')
    expect((mockFetch.mock.calls[0][1] as RequestInit).method).toBe('DELETE')
  })
})
