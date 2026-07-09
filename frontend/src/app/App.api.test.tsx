import { render, screen, waitFor } from '@testing-library/react'
import { beforeEach, describe, expect, it, vi } from 'vitest'
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
      listJobs,
    }

    render(<App client={client} />)

    await waitFor(() => expect(listJobs).toHaveBeenCalledOnce())
    expect(screen.queryByText('terminal-bench-smoke')).not.toBeInTheDocument()
    expect(screen.getByRole('alert')).toHaveTextContent('Unable to load Jobs.')
  })
})
