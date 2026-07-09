import { describe, expect, it } from 'vitest'
import { createMockWebUiClient } from './mockClient'

interface Stage2ReadClient {
  getAgent: (id: string) => Promise<{ data: { id: string } | null; error: unknown }>
  getEnvironment: (id: string) => Promise<{ data: { id: string } | null; error: unknown }>
  getHubConnection: () => Promise<{ data: { status: string } | null; error: unknown }>
  listAgents: () => Promise<{ data: { items: Array<{ agentName: string; id: string }> } | null; error: unknown }>
  listEnvironments: () => Promise<{ data: { items: Array<{ id: string; name: string }> } | null; error: unknown }>
  listLeaderboard: (query: { dataset: string }) => Promise<{ data: { items: Array<{ jobId: string }> } | null; error: unknown }>
  listLeaderboardDatasets: () => Promise<{ data: { items: Array<{ ref: string }> } | null; error: unknown }>
  listSystemHealth: () => Promise<{ data: { items: Array<{ component: string }> } | null; error: unknown }>
}

describe('Stage 2 remaining resource reads', () => {
  it('returns the four remaining resource domains through the mock contract client', async () => {
    const client = createMockWebUiClient() as unknown as Stage2ReadClient

    const [agents, agent, environments, environment, hubConnection, leaderboardDatasets, leaderboard, system] = await Promise.all([
      client.listAgents(),
      client.getAgent('claude-code-default'),
      client.listEnvironments(),
      client.getEnvironment('docker-default'),
      client.getHubConnection(),
      client.listLeaderboardDatasets(),
      client.listLeaderboard({ dataset: 'terminal-bench@2.0' }),
      client.listSystemHealth(),
    ])

    expect(agents.data?.items[0]).toMatchObject({ agentName: 'Claude Code default', id: 'claude-code-default' })
    expect(agent.data).toMatchObject({ id: 'claude-code-default' })
    expect(environments.data?.items[0]).toMatchObject({ id: 'docker-default', name: 'Docker default' })
    expect(environment.data).toMatchObject({ id: 'docker-default' })
    expect(hubConnection.data).toMatchObject({ status: 'connected' })
    expect(leaderboardDatasets.data?.items.map((item) => item.ref)).toContain('terminal-bench@2.0')
    expect(leaderboard.data?.items[0]).toMatchObject({ jobId: 'job_91a7' })
    expect(system.data?.items[0]).toMatchObject({ component: 'OrnnLab Service' })
  })
})
