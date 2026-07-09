import { fireEvent, render, screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createMockWebUiClient } from '../api/mockClient'
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
    const client: WebUiClient = {
      getDataset: vi.fn(),
      getJob: vi.fn(),
      listDatasetTasks: vi.fn(),
      listDatasets: vi.fn().mockResolvedValue({ data: null, error: null }),
      listJobEvents: vi.fn().mockResolvedValue({ data: null, error: null }),
      listJobTrials: vi.fn().mockResolvedValue({ data: null, error: null }),
      listJobs,
    }

    render(<App client={client} />)

    await waitFor(() => expect(listJobs).toHaveBeenCalledOnce())
    expect(screen.queryByText('terminal-bench-smoke')).not.toBeInTheDocument()
    expect(screen.getByRole('alert')).toHaveTextContent('Unable to load Jobs.')
  })

  it('disables simulated Job creation in API mode', async () => {
    render(<App client={createMockWebUiClient()} dataMode="api" />)

    await screen.findByText('terminal-bench-smoke')
    fireEvent.click(screen.getByRole('button', { name: 'New Job' }))

    expect(screen.getByRole('button', { name: 'Run job' })).toBeDisabled()
  })

  it('disables remaining simulated write actions in API mode', async () => {
    render(<App client={createMockWebUiClient()} dataMode="api" />)

    await screen.findByText('terminal-bench-smoke')
    fireEvent.click(screen.getByRole('link', { name: 'Datasets' }))
    await screen.findByText('terminal-bench')
    expect(screen.getByRole('button', { name: 'Import local Dataset' })).toBeDisabled()
    expect(screen.getAllByRole('button', { name: 'Download' })[0]).toBeDisabled()
    for (const button of screen.getAllByRole('button', { name: 'Delete' })) {
      expect(button).toBeDisabled()
    }

    fireEvent.click(screen.getByRole('link', { name: 'Agents' }))
    expect(screen.getByRole('button', { name: 'New Agent' })).toBeDisabled()
    expect(screen.queryByText('Claude Code default')).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('link', { name: 'Environment' }))
    expect(screen.getByRole('button', { name: 'New Environment' })).toBeDisabled()
    expect(screen.queryByText('Docker default')).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('link', { name: 'Leaderboard' }))
    expect(screen.queryByRole('button', { name: 'Remove' })).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('link', { name: 'System' }))
    expect(screen.queryByRole('button', { name: 'Check update' })).not.toBeInTheDocument()
  })
})
