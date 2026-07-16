import { fireEvent, render, screen, waitFor, within } from '@testing-library/react'
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
    const listDatasets = vi.spyOn(client, 'listDatasets')
    render(<App client={client} dataMode="api" />)

    await screen.findByText('terminal-bench-smoke')
    fireEvent.click(screen.getByRole('link', { name: 'Agents' }))
    fireEvent.change(screen.getByLabelText('Search agents'), { target: { value: 'local' } })
    await waitFor(() => expect(listAgents).toHaveBeenCalledWith({ limit: 100, q: 'local' }))

    fireEvent.click(screen.getByRole('link', { name: 'Environment' }))
    fireEvent.change(screen.getByLabelText('Search environments'), { target: { value: 'docker' } })
    await waitFor(() => expect(listEnvironments).toHaveBeenCalledWith({ limit: 100, q: 'docker' }))

    fireEvent.click(screen.getByRole('link', { name: 'Leaderboard' }))
    fireEvent.click(screen.getByLabelText('Select dataset'))
    fireEvent.change(screen.getByLabelText('Search datasets'), { target: { value: 'swe' } })
    await waitFor(() => expect(listDatasets).toHaveBeenCalledWith({ limit: 100, q: 'swe' }))
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
    expect(createJob.mock.calls[0][0].config.modelName).toBe('claude-haiku-4-5')
    await waitFor(() => expect(window.location.hash).toBe('#jobs'))
    const drawer = await screen.findByRole('dialog', { name: 'Selected job' })
    expect(within(drawer).getByRole('heading', { name: 'new-job' })).toBeInTheDocument()
    expect(within(drawer).getByText('Queued')).toBeInTheDocument()
  })

  it('copies a Job into New Job without creating it and falls back for stale references', async () => {
    const client = createMockWebUiClient()
    const createJob = vi.spyOn(client, 'createJob')
    vi.spyOn(client, 'getJobCopyConfig').mockResolvedValue({
      data: {
        agentSetupTimeoutMultiplier: 1,
        agentName: 'Deleted agent',
        agentTimeoutMultiplier: 1,
        attempts: 2,
        concurrency: 3,
        datasetRef: 'deleted-dataset@1.0',
        debug: true,
        environmentPresetId: 'deleted-environment',
        environmentBuildTimeoutMultiplier: 1,
        extraInstructionPaths: ['instructions/copied.md'],
        includeInLeaderboard: false,
        jobName: 'terminal-bench-smoke-copy',
        jobsDir: 'jobs/terminal-bench-smoke',
        maxRetries: 0,
        metric: 'mean',
        modelName: 'deleted-model',
        notes: 'copied note',
        retryExclude: '',
        retryInclude: '',
        retryMaxWaitSeconds: 30,
        retryMinWaitSeconds: 2,
        retryWaitMultiplier: 1.5,
        selectedTaskNames: null,
        timeoutMultiplier: 1,
        verifierTimeoutMultiplier: 1,
        verifierMode: 'dataset-default',
      },
      error: null,
    })
    render(<App client={client} dataMode="api" />)

    fireEvent.click(await screen.findByRole('button', { name: 'terminal-bench-smoke' }))
    const drawer = screen.getByRole('dialog', { name: 'Selected job' })
    fireEvent.click(within(drawer).getByRole('button', { name: 'Copy' }))

    await waitFor(() => expect(window.location.hash).toBe('#jobs/new'))
    expect(screen.getByLabelText('job_name')).toHaveValue('terminal-bench-smoke-copy')
    expect(screen.getByLabelText('jobs_dir')).toHaveValue('jobs/terminal-bench-smoke')
    await waitFor(() => expect(screen.getByLabelText('Dataset')).toHaveTextContent('terminal-bench@2.0'))
    expect(screen.getByLabelText('Agent')).toHaveTextContent('Claude Code default')
    expect(createJob).not.toHaveBeenCalled()
  })

  it('keeps the original Job open when its saved configuration cannot be copied', async () => {
    const client = createMockWebUiClient()
    vi.spyOn(client, 'getJobCopyConfig').mockResolvedValue({
      data: null,
      error: { code: 'INVALID_REQUEST', message: 'raw config error' },
    })
    render(<App client={client} dataMode="api" />)

    fireEvent.click(await screen.findByRole('button', { name: 'terminal-bench-smoke' }))
    const drawer = screen.getByRole('dialog', { name: 'Selected job' })
    fireEvent.click(within(drawer).getByRole('button', { name: 'Copy' }))

    expect(await within(drawer).findByRole('alert')).toHaveTextContent(
      'This Job cannot be copied because its original configuration is unavailable.',
    )
    expect(window.location.hash).toBe('#jobs')
    expect(screen.queryByText('raw config error')).not.toBeInTheDocument()
  })

  it('shows a failed resume Operation in the Job drawer and refreshes Jobs', async () => {
    const client = createMockWebUiClient()
    const listJobs = vi.spyOn(client, 'listJobs')
    vi.spyOn(client, 'resumeJob').mockResolvedValue({
      data: {
        operation: {
          id: 'resume-failed',
          type: 'resume-job',
          status: 'failed',
          resourceType: 'job',
          resourceId: 'job_55e9',
          error: { code: 'OPERATION_FAILED', message: 'low-level Harbor output' },
        },
      },
      error: null,
    })
    render(<App client={client} dataMode="api" />)

    fireEvent.click(await screen.findByRole('button', { name: 'harbor-hello-world' }))
    const drawer = screen.getByRole('dialog', { name: 'Selected job' })
    fireEvent.click(within(drawer).getByRole('button', { name: 'Resume' }))

    expect(await within(drawer).findByRole('alert')).toHaveTextContent(
      'This Job could not be resumed. Check the Job event log for details.',
    )
    expect(screen.queryByText('low-level Harbor output')).not.toBeInTheDocument()
    await waitFor(() => expect(listJobs.mock.calls.length).toBeGreaterThan(1))
  })

  it('keeps an active Job status synchronized between its list row and detail drawer', async () => {
    const client = createMockWebUiClient()
    const listJobs = client.listJobs.bind(client)
    let status: 'running' | 'completed' = 'running'
    vi.spyOn(client, 'listJobs').mockImplementation(async (query) => {
      const response = await listJobs(query)
      if (!response.data) return response
      return {
        ...response,
        data: {
          ...response.data,
          items: response.data.items.map((job, index) => index === 0 ? { ...job, status } : job),
        },
      }
    })
    render(<App client={client} dataMode="api" />)

    const jobButton = await screen.findByRole('button', { name: 'terminal-bench-smoke' })
    fireEvent.click(jobButton)
    const selectedRow = jobButton.closest('tr')
    expect(selectedRow).not.toBeNull()
    const drawer = await screen.findByRole('dialog', { name: 'Selected job' })
    await waitFor(() => expect(within(selectedRow as HTMLElement).getByText('Running')).toBeInTheDocument())
    expect(within(drawer).getByText('Running')).toBeInTheDocument()

    status = 'completed'
    await waitFor(() => expect(within(selectedRow as HTMLElement).getByText('Completed')).toBeInTheDocument(), { timeout: 2_500 })
    expect(within(drawer).getByText('Completed')).toBeInTheDocument()
    expect(within(selectedRow as HTMLElement).queryByText('Running')).not.toBeInTheDocument()
    expect(within(drawer).queryByText('Running')).not.toBeInTheDocument()
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
    fireEvent.click(screen.getByLabelText('Select dataset'))
    fireEvent.click(screen.getByRole('option', { name: /^terminal-bench@2\.0 / }))
    await screen.findByText('job_91a7')
    for (const button of screen.getAllByRole('button', { name: 'Remove' })) {
      expect(button).toBeEnabled()
    }

    fireEvent.click(screen.getByRole('link', { name: 'System' }))
    expect(screen.getByRole('button', { name: 'Check update' })).toBeEnabled()
  })

  it('uses the Dataset catalog for leaderboard selection, including Datasets without rankings', async () => {
    const client = createMockWebUiClient()
    render(<App client={client} />)

    await screen.findByText('terminal-bench-smoke')
    fireEvent.click(screen.getByRole('link', { name: 'Leaderboard' }))
    await waitFor(() => expect(screen.getByLabelText('Select dataset')).toHaveTextContent('Select dataset'))
    fireEvent.click(screen.getByLabelText('Select dataset'))

    expect(screen.getByRole('option', { name: /^swebench-verified@1\.0 / })).toBeInTheDocument()
    expect(screen.getByRole('option', { name: /^terminal-bench@2\.0 / })).toBeInTheDocument()
    fireEvent.click(screen.getByRole('option', { name: /^terminal-bench-nightly@nightly / }))
    expect(await screen.findByText('No leaderboard entries are available for this Dataset.')).toBeInTheDocument()
  })

  it('renders Hub connection status from the injected client contract', async () => {
    const client = createMockWebUiClient()
    vi.spyOn(client, 'getHubConnection').mockResolvedValue({ data: { status: 'disconnected' }, error: null })
    render(<App client={client} />)

    expect(await screen.findByText('Hub disconnected')).toBeInTheDocument()
  })

  it('does not reload the Dataset catalog after a leaderboard write completes', async () => {
    const client = createMockWebUiClient()
    const listDatasets = vi.spyOn(client, 'listDatasets')
    render(<App client={client} />)

    await screen.findByText('terminal-bench-smoke')
    fireEvent.click(screen.getByRole('link', { name: 'Leaderboard' }))
    await waitFor(() => expect(listDatasets).toHaveBeenCalledTimes(1))
    fireEvent.click(screen.getByLabelText('Select dataset'))
    fireEvent.click(screen.getByRole('option', { name: /^terminal-bench@2\.0 / }))
    await screen.findByText('job_91a7')
    fireEvent.click(screen.getAllByRole('button', { name: 'Remove' })[0])

    await waitFor(() => expect(screen.queryByText('job_91a7')).not.toBeInTheDocument(), { timeout: 2_000 })
    expect(listDatasets).toHaveBeenCalledTimes(1)
  })
})
