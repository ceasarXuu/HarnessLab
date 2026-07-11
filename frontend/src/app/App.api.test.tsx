import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createMockWebUiClient } from '../api/mockClient'
import { createUnavailableWebUiClient } from '../api/unavailableClient'
import type { WebUiClient } from '../api/webUiClient'
import { App } from './App'

describe('App API mode', () => {
  beforeEach(() => {
    window.location.hash = '#jobs'
  })

  it('does not fall back to seed Jobs when the injected client fails', async () => {
    const listJobs = vi.fn<WebUiClient['listJobs']>().mockResolvedValue({
      data: null,
      error: { code: 'NETWORK_REQUEST_FAILED', message: 'The API request could not be completed.' },
    })
    const client: WebUiClient = createUnavailableWebUiClient({ listJobs })

    render(<App client={client} />)

    await waitFor(() => expect(listJobs).toHaveBeenCalledOnce())
    expect(screen.queryByText('terminal-bench-smoke')).not.toBeInTheDocument()
    expect(screen.getByRole('alert')).toHaveTextContent('Unable to load Jobs.')
  })

  it('does not fall back to mock Datasets when the API catalog fails', async () => {
    const client = createMockWebUiClient()
    vi.spyOn(client, 'listDatasets').mockResolvedValue({
      data: null,
      error: { code: 'NETWORK_REQUEST_FAILED', message: 'The API request could not be completed.' },
    })
    render(<App client={client} dataMode="api" />)

    await screen.findByText('terminal-bench-smoke')
    fireEvent.click(screen.getByRole('link', { name: 'Datasets' }))

    await waitFor(() => expect(screen.getByRole('alert')).toHaveTextContent('Unable to load Datasets.'))
    expect(screen.queryByText('terminal-bench')).not.toBeInTheDocument()
  })

  it('queries the Dataset catalog on the server when searching', async () => {
    const client = createMockWebUiClient()
    const listDatasets = vi.spyOn(client, 'listDatasets')
    render(<App client={client} dataMode="api" />)

    await screen.findByText('terminal-bench-smoke')
    fireEvent.click(screen.getByRole('link', { name: 'Datasets' }))
    await screen.findByText('terminal-bench')
    fireEvent.change(screen.getByLabelText('Search datasets'), { target: { value: 'swebench' } })

    await waitFor(() => expect(listDatasets).toHaveBeenCalledWith({ limit: 100, q: 'swebench' }))
    expect(await screen.findByText('swebench-verified')).toBeInTheDocument()
    expect(screen.queryByText('terminal-bench')).not.toBeInTheDocument()
  })

  it('uses server-side search for Agents, Environments, and leaderboard Datasets', async () => {
    const client = createMockWebUiClient()
    const listAgents = vi.spyOn(client, 'listAgents')
    const listEnvironments = vi.spyOn(client, 'listEnvironments')
    const listLeaderboardDatasets = vi.spyOn(client, 'listLeaderboardDatasets')
    render(<App client={client} dataMode="api" />)

    await screen.findByText('terminal-bench-smoke')
    fireEvent.click(screen.getByRole('link', { name: 'Agents' }))
    fireEvent.change(screen.getByLabelText('Search agents'), { target: { value: 'local' } })
    await waitFor(() => expect(listAgents).toHaveBeenCalledWith({ limit: 100, q: 'local' }))

    fireEvent.click(screen.getByRole('link', { name: 'Environment' }))
    fireEvent.change(screen.getByLabelText('Search environments'), { target: { value: 'docker' } })
    await waitFor(() => expect(listEnvironments).toHaveBeenCalledWith({ limit: 100, q: 'docker' }))

    fireEvent.click(screen.getByRole('link', { name: 'Leaderboard' }))
    fireEvent.change(screen.getByLabelText('Search datasets'), { target: { value: 'swe' } })
    await waitFor(() => expect(listLeaderboardDatasets).toHaveBeenCalledWith({ limit: 100, q: 'swe' }))
  })

  it('reuses a cached Dataset search result when returning to a keyword', async () => {
    const client = createMockWebUiClient()
    const listDatasets = vi.spyOn(client, 'listDatasets')
    render(<App client={client} dataMode="api" />)

    await screen.findByText('terminal-bench-smoke')
    fireEvent.click(screen.getByRole('link', { name: 'Datasets' }))
    await screen.findByText('terminal-bench')
    fireEvent.change(screen.getByLabelText('Search datasets'), { target: { value: 'swebench' } })
    await waitFor(() => expect(listDatasets).toHaveBeenCalledWith({ limit: 100, q: 'swebench' }))
    await waitFor(() => expect(screen.queryByText('Loading datasets')).not.toBeInTheDocument())
    fireEvent.change(screen.getByLabelText('Search datasets'), { target: { value: 'terminal' } })
    await waitFor(() => expect(listDatasets).toHaveBeenCalledWith({ limit: 100, q: 'terminal' }))
    await waitFor(() => expect(screen.queryByText('Loading datasets')).not.toBeInTheDocument())
    fireEvent.change(screen.getByLabelText('Search datasets'), { target: { value: 'swebench' } })

    await screen.findByText('swebench-verified')
    expect(listDatasets.mock.calls.filter(([query]) => query?.q === 'swebench')).toHaveLength(1)
  })

  it('submits Job creation through the injected API client in API mode', async () => {
    const client = createMockWebUiClient()
    const createJob = vi.spyOn(client, 'createJob')
    render(<App client={client} dataMode="api" />)

    await screen.findByText('terminal-bench-smoke')
    fireEvent.click(screen.getByRole('button', { name: 'New Job' }))

    const runJob = screen.getByRole('button', { name: 'Run job' })
    expect(runJob).toBeEnabled()
    fireEvent.click(runJob)
    await waitFor(() => expect(createJob).toHaveBeenCalledOnce())
  })

  it('keeps defined write actions available in API mode', async () => {
    render(<App client={createMockWebUiClient()} dataMode="api" />)

    await screen.findByText('terminal-bench-smoke')
    fireEvent.click(screen.getByRole('link', { name: 'Datasets' }))
    await screen.findByText('terminal-bench')
    expect(screen.getByRole('button', { name: 'Import local Dataset' })).toBeEnabled()
    expect(screen.getAllByRole('button', { name: 'Download' })[0]).toBeEnabled()
    for (const button of screen.getAllByRole('button', { name: 'Delete' })) {
      expect(button).toBeEnabled()
    }

    fireEvent.click(screen.getByRole('link', { name: 'Agents' }))
    expect(screen.getByRole('button', { name: 'New Agent' })).toBeEnabled()
    expect(screen.getByText('Claude Code default')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('link', { name: 'Environment' }))
    expect(screen.getByRole('button', { name: 'New Environment' })).toBeEnabled()
    expect(screen.getByText('Docker default')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('link', { name: 'Leaderboard' }))
    for (const button of screen.getAllByRole('button', { name: 'Remove' })) {
      expect(button).toBeEnabled()
    }

    fireEvent.click(screen.getByRole('link', { name: 'System' }))
    expect(screen.getByRole('button', { name: 'Check update' })).toBeEnabled()
  })

  it('uses the leaderboard Dataset contract rather than the generic Dataset catalog', async () => {
    const client = createMockWebUiClient()
    vi.spyOn(client, 'listLeaderboardDatasets').mockResolvedValue({
      data: {
        items: [{ name: 'swebench-verified', ref: 'swebench-verified@1.0', version: '1.0' }],
        total: 1,
      },
      error: null,
    })
    render(<App client={client} />)

    await screen.findByText('terminal-bench-smoke')
    fireEvent.click(screen.getByRole('link', { name: 'Leaderboard' }))
    await waitFor(() => expect(screen.getByLabelText('Select dataset')).toHaveTextContent('swebench-verified@1.0'))
    fireEvent.click(screen.getByLabelText('Select dataset'))

    expect(screen.getByRole('option', { name: 'swebench-verified@1.0' })).toBeInTheDocument()
    expect(screen.queryByRole('option', { name: 'terminal-bench@2.0' })).not.toBeInTheDocument()
  })

  it('renders Hub connection status from the injected client contract', async () => {
    const client = createMockWebUiClient()
    vi.spyOn(client, 'getHubConnection').mockResolvedValue({ data: { status: 'disconnected' }, error: null })
    render(<App client={client} />)

    expect(await screen.findByText('Hub disconnected')).toBeInTheDocument()
  })

  it('refreshes selectable leaderboard Datasets after a leaderboard write completes', async () => {
    const client = createMockWebUiClient()
    const listLeaderboardDatasets = vi.spyOn(client, 'listLeaderboardDatasets')
    render(<App client={client} />)

    await screen.findByText('terminal-bench-smoke')
    fireEvent.click(screen.getByRole('link', { name: 'Leaderboard' }))
    await waitFor(() => expect(listLeaderboardDatasets).toHaveBeenCalledTimes(1))
    fireEvent.click(screen.getAllByRole('button', { name: 'Remove' })[0])

    await waitFor(() => expect(listLeaderboardDatasets.mock.calls.length).toBeGreaterThan(1), { timeout: 2_000 })
  })
})
