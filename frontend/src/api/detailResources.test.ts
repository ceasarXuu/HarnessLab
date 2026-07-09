import { describe, expect, it } from 'vitest'
import { createMockWebUiClient } from './mockClient'

describe('WebUI detail resources', () => {
  it('returns Job events through the contract client', async () => {
    const client = createMockWebUiClient()

    const [terminalBenchEvents, helloWorldEvents] = await Promise.all([
      client.listJobEvents('job_91a7'),
      client.listJobEvents('job_55e9'),
    ])

    expect(terminalBenchEvents.error).toBeNull()
    expect(terminalBenchEvents.data?.[0]).toMatchObject({
      level: 'success',
      message: 'JobConfig persisted to harbor.config.json',
    })
    expect(helloWorldEvents.error).toBeNull()
    expect(helloWorldEvents.data).not.toEqual(terminalBenchEvents.data)
  })

  it('returns Job trials through the contract client', async () => {
    const client = createMockWebUiClient()

    const response = await client.listJobTrials('job_91a7')

    expect(response.error).toBeNull()
    expect(response.data?.map((trial) => trial.taskName)).toEqual(['apt-setup', 'git-rebase-conflict'])
  })
})
