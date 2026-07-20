import { setupServer } from 'msw/node'
import { afterAll, afterEach, beforeAll, describe, expect, it } from 'vitest'
import { webuiHandlers } from './mswHandlers'

const server = setupServer(...webuiHandlers)

beforeAll(() => server.listen({ onUnhandledRequest: 'error' }))
afterEach(() => server.resetHandlers())
afterAll(() => server.close())

describe('WebUI MSW handlers', () => {
  it('returns the Jobs contract envelope and structured DTOs', async () => {
    const response = await fetch('http://localhost/api/webui/v1/jobs?q=terminal')
    const body = await response.json()

    expect(response.ok).toBe(true)
    expect(body.error).toBeNull()
    expect(body.data.items).toEqual(expect.arrayContaining([
      expect.objectContaining({
        datasetRef: 'terminal-bench@2.0',
        id: 'job_91a7',
        trial: { completed: 18, errored: 0, notPassed: 6, passed: 12, total: 64 },
      }),
    ]))
  })

  it('uses the nested Dataset Task contract route', async () => {
    const response = await fetch('http://localhost/api/webui/v1/datasets/terminal-bench%402.0/tasks')
    const body = await response.json()

    expect(response.ok).toBe(true)
    expect(body.data.items.map((task: { name: string }) => task.name)).toEqual([
      'apt-setup',
      'git-rebase-conflict',
      'sqlite-log-repair',
      'python-env-pin',
    ])
    expect(body.error).toBeNull()
  })

  it('returns Job details through contract-shaped event and trial routes', async () => {
    const [eventsResponse, otherEventsResponse, trialsResponse] = await Promise.all([
      fetch('http://localhost/api/webui/v1/jobs/job_91a7/events'),
      fetch('http://localhost/api/webui/v1/jobs/job_55e9/events'),
      fetch('http://localhost/api/webui/v1/jobs/job_91a7/trials'),
    ])
    const [eventsBody, otherEventsBody, trialsBody] = await Promise.all([
      eventsResponse.json(),
      otherEventsResponse.json(),
      trialsResponse.json(),
    ])

    expect(eventsBody.data[0]).toEqual(expect.objectContaining({
      occurredAt: '14:18:21',
      level: 'success',
    }))
    expect(eventsBody.data[0]).not.toHaveProperty('time')
    expect(otherEventsBody.data).not.toEqual(eventsBody.data)
    expect(trialsBody.data[0]).toEqual(expect.objectContaining({
      jobId: 'job_91a7',
      retryCount: 0,
      taskName: 'apt-setup',
    }))
    expect(trialsBody.data[0]).not.toHaveProperty('task')
  })

  it('serves every remaining Stage 2 read resource through contract routes', async () => {
    const [agentsResponse, environmentsResponse, harnessesResponse, hubConnectionResponse, leaderboardDatasetsResponse, leaderboardResponse, systemResponse] = await Promise.all([
      fetch('http://localhost/api/webui/v1/agents'),
      fetch('http://localhost/api/webui/v1/environments'),
      fetch('http://localhost/api/webui/v1/harnesses?limit=100'),
      fetch('http://localhost/api/webui/v1/system/hub-connection'),
      fetch('http://localhost/api/webui/v1/leaderboard/datasets'),
      fetch('http://localhost/api/webui/v1/leaderboard?dataset=terminal-bench%402.0'),
      fetch('http://localhost/api/webui/v1/system/health'),
    ])
    const [agents, environments, harnesses, hubConnection, leaderboardDatasets, leaderboard, system] = await Promise.all([
      agentsResponse.json(),
      environmentsResponse.json(),
      harnessesResponse.json(),
      hubConnectionResponse.json(),
      leaderboardDatasetsResponse.json(),
      leaderboardResponse.json(),
      systemResponse.json(),
    ])

    expect(agents.data.items[0]).toMatchObject({ agentName: 'Claude Code default', id: 'claude-code-default' })
    expect(harnesses.data.items).toHaveLength(30)
    expect(harnesses.data.items).toEqual(expect.arrayContaining([expect.objectContaining({ name: 'claude-code', source: 'harbor-built-in' })]))
    expect(environments.data.items[0]).toMatchObject({ id: 'docker-default', name: 'Docker default' })
    expect(hubConnection.data).toMatchObject({ status: 'connected' })
    expect(leaderboardDatasets.data.items.map((item: { ref: string }) => item.ref)).toContain('terminal-bench@2.0')
    expect(leaderboard.data.items[0]).toMatchObject({ datasetRef: 'terminal-bench@2.0', jobId: 'job_91a7' })
    expect(system.data.items[0]).toMatchObject({ kind: 'ornnlab-service', state: 'running' })
  })

  it('preserves structured Agent and Environment query filters through HTTP', async () => {
    const [agentsResponse, environmentsResponse] = await Promise.all([
      fetch('http://localhost/api/webui/v1/agents?status=needs-token'),
      fetch('http://localhost/api/webui/v1/environments?type=built-in'),
    ])
    const [agents, environments] = await Promise.all([
      agentsResponse.json(),
      environmentsResponse.json(),
    ])

    expect(agents.data.items).toEqual([
      expect.objectContaining({ id: 'local-repair-agent', status: 'needs-token' }),
    ])
    expect(environments.data.items).toEqual([
      expect.objectContaining({ id: 'docker-default', profileType: 'built-in' }),
    ])
  })

  it('routes writes through contract-shaped Operations and supports polling', async () => {
    const response = await fetch('http://localhost/api/webui/v1/system/cache/storage/clean', { method: 'POST' })
    const body = await response.json()

    expect(response.ok).toBe(true)
    expect(body.error).toBeNull()
    expect(body.data.operation).toMatchObject({ resourceType: 'system', status: 'queued', type: 'clean-storage-cache' })

    const operationUrl = `http://localhost/api/webui/v1/operations/${body.data.operation.id}`
    expect((await (await fetch(operationUrl)).json()).data.status).toBe('running')
    const cancelled = await (await fetch(`${operationUrl}/cancel`, { method: 'POST' })).json()
    expect(cancelled.data.operation.status).toBe('cancelled')
    expect((await (await fetch(operationUrl)).json()).data.status).toBe('cancelled')
  })
})
