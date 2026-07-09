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
        trial: { completed: 18, total: 64 },
      }),
    ]))
  })

  it('uses the nested Dataset Task contract route', async () => {
    const response = await fetch('http://localhost/api/webui/v1/datasets/terminal-bench%402.0/tasks?split=test')
    const body = await response.json()

    expect(response.ok).toBe(true)
    expect(body.data.items.map((task: { name: string }) => task.name)).toEqual(['apt-setup', 'git-rebase-conflict'])
    expect(body.error).toBeNull()
  })
})
