import { fireEvent, render, screen, within } from '@testing-library/react'
import { beforeEach, describe, expect, it } from 'vitest'
import { App } from './App'

describe('App', () => {
  beforeEach(() => {
    window.localStorage.clear()
    window.location.hash = ''
  })

  it('renders jobs as the default Harbor operating surface', () => {
    render(<App />)

    expect(screen.getByRole('heading', { name: 'Job registry' })).toBeInTheDocument()
    expect(screen.getByRole('link', { name: 'Jobs' })).toHaveClass('active')
    expect(screen.queryByRole('dialog', { name: 'Selected job' })).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'terminal-bench-smoke' }))
    expect(screen.getByRole('dialog', { name: 'Selected job' })).toBeInTheDocument()
    expect(screen.getByText('Job trials')).toBeInTheDocument()
    expect(screen.getByText('Hub actions')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Open viewer' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Analyze' })).toBeInTheDocument()
    expect(screen.getByText('harbor.capability.json')).toBeInTheDocument()
  })

  it('renders datasets as the Harbor catalog surface', () => {
    render(<App />)

    fireEvent.click(screen.getByRole('link', { name: 'Datasets' }))
    expect(screen.getByRole('heading', { name: 'Dataset catalog' })).toBeInTheDocument()
    expect(screen.getAllByText('terminal-bench').length).toBeGreaterThan(0)
    expect(screen.queryByRole('dialog', { name: 'Selected dataset' })).not.toBeInTheDocument()
    fireEvent.click(screen.getByText('terminal-bench'))
    expect(screen.getByRole('dialog', { name: 'Selected dataset' })).toBeInTheDocument()
    expect(screen.getByText('Dataset tasks')).toBeInTheDocument()
    expect(screen.getAllByRole('button', { name: 'Run single task' }).length).toBeGreaterThan(0)
    expect(screen.getByText('registry_url')).toBeInTheDocument()
    expect(screen.getAllByRole('button', { name: 'Start environment' }).length).toBeGreaterThan(0)
    expect(screen.getAllByRole('button', { name: 'Check' }).length).toBeGreaterThan(0)
    expect(screen.getAllByRole('button', { name: 'Debug' }).length).toBeGreaterThan(0)
    expect(screen.getByRole('link', { name: 'Datasets' })).toHaveClass('active')
  })

  it('renders the jobs hierarchy without flattening the run builder into it', () => {
    render(<App />)

    fireEvent.click(screen.getByRole('link', { name: 'Jobs' }))
    expect(screen.getByRole('heading', { name: 'Job registry' })).toBeInTheDocument()
    expect(screen.getAllByText('terminal-bench-smoke').length).toBeGreaterThan(0)
    fireEvent.click(screen.getByRole('button', { name: 'terminal-bench-smoke' }))
    expect(screen.getByRole('dialog', { name: 'Selected job' })).toBeInTheDocument()
    expect(screen.getByText('Job trials')).toBeInTheDocument()
    expect(screen.getByText('apt-setup')).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: 'System doctor' })).toBeInTheDocument()
    expect(screen.queryByRole('heading', { name: 'New Run' })).not.toBeInTheDocument()
    expect(screen.queryByRole('link', { name: 'New Run' })).not.toBeInTheDocument()
    expect(screen.queryByLabelText('Job creation path')).not.toBeInTheDocument()
  })

  it('filters jobs and keeps the table as the primary surface', () => {
    render(<App />)

    fireEvent.click(screen.getByRole('link', { name: 'Jobs' }))
    fireEvent.change(screen.getByLabelText('Search jobs'), { target: { value: 'swe' } })

    const jobsTable = screen.getByRole('table')
    expect(screen.getByText('swe-bench-lite-regression')).toBeInTheDocument()
    expect(within(jobsTable).queryByText('terminal-bench-smoke')).not.toBeInTheDocument()
  })

  it('keeps task and trial concepts nested under datasets and jobs', () => {
    render(<App />)

    expect(screen.queryByRole('link', { name: 'Tasks' })).not.toBeInTheDocument()
    expect(screen.queryByRole('link', { name: 'Trials' })).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('link', { name: 'Datasets' }))
    fireEvent.click(screen.getByText('terminal-bench'))
    expect(screen.getByText('Dataset tasks')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('link', { name: 'Jobs' }))
    fireEvent.click(screen.getByRole('button', { name: 'terminal-bench-smoke' }))
    expect(screen.getByText('Job trials')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('link', { name: 'System' }))
    expect(screen.getByRole('heading', { name: 'System health' })).toBeInTheDocument()
  })

  it('renders agents and dataset-scoped leaderboard pages', () => {
    render(<App />)

    fireEvent.click(screen.getByRole('link', { name: 'Agents' }))
    expect(screen.getByRole('heading', { name: 'Agent catalog' })).toBeInTheDocument()
    expect(screen.getByText('claude-code')).toBeInTheDocument()
    expect(screen.getByText('local-repair-agent')).toBeInTheDocument()
    expect(screen.queryByRole('dialog', { name: 'Selected agent' })).not.toBeInTheDocument()
    fireEvent.click(screen.getByText('local-repair-agent'))
    expect(screen.getByRole('dialog', { name: 'Selected agent' })).toBeInTheDocument()
    expect(screen.getByText('env readiness')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Validate' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Compile' })).toBeInTheDocument()

    fireEvent.click(screen.getByRole('link', { name: 'Leaderboard' }))
    expect(screen.getByRole('heading', { name: 'Leaderboard' })).toBeInTheDocument()
    expect(screen.getByText('Metric')).toBeInTheDocument()
    expect(screen.getByText('Submission')).toBeInTheDocument()
    expect(screen.getByLabelText('Select dataset')).toHaveTextContent('terminal-bench@2.0')
    expect(screen.getByText('claude-code')).toBeInTheDocument()
    expect(screen.getAllByRole('button', { name: 'Submit' }).length).toBeGreaterThan(0)

    fireEvent.change(screen.getByLabelText('Search datasets'), { target: { value: 'swe' } })
    fireEvent.click(screen.getByLabelText('Select dataset'))
    fireEvent.click(screen.getByRole('option', { name: 'swe-bench-lite@2026.06' }))
    expect(screen.getByText('job_74c1')).toBeInTheDocument()
    expect(screen.getByText('claude-sonnet-4-5')).toBeInTheDocument()
    expect(screen.getByText('gpt-5.1')).toBeInTheDocument()
    expect(screen.queryByLabelText('Search leaderboard')).not.toBeInTheDocument()
  })

  it('switches language and theme from the header', () => {
    render(<App />)

    fireEvent.click(screen.getByLabelText('Language'))
    fireEvent.click(screen.getByRole('option', { name: '中' }))
    expect(screen.getByRole('heading', { name: 'Job 管理' })).toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: '深色' }))
    expect(document.documentElement.dataset.theme).toBe('dark')
  })

  it('adds a queued job from the dedicated new run flow', () => {
    render(<App />)

    fireEvent.click(screen.getByRole('link', { name: 'Jobs' }))
    fireEvent.click(screen.getByRole('button', { name: 'Run job' }))
    expect(window.location.hash).toBe('#jobs/new')
    expect(screen.getByRole('link', { name: 'Jobs' })).toHaveClass('active')
    expect(screen.getByRole('navigation', { name: 'Job creation path' })).toBeInTheDocument()
    fireEvent.click(within(screen.getByRole('navigation', { name: 'Job creation path' })).getByRole('button', { name: 'Jobs' }))
    expect(screen.getByRole('heading', { name: 'Job registry' })).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Run job' }))
    const runBuilder = screen.getByRole('heading', { name: 'New Job' }).closest('section')
    expect(runBuilder).not.toBeNull()
    expect(screen.getByLabelText('job_name')).toHaveValue('terminal-bench-smoke')
    expect(screen.getByLabelText('include_task_name')).toHaveValue('apt-*')
    expect(screen.getByLabelText('agent env')).toHaveValue('ANTHROPIC_API_KEY')
    expect(screen.getByLabelText('verifier env')).toHaveValue('PYTEST_ADDOPTS=-q')
    expect(screen.getByLabelText('upload to Hub')).toHaveTextContent('disabled')
    expect(screen.getByText(/--include-task-name apt-\*/)).toBeInTheDocument()
    const runButton = within(runBuilder as HTMLElement).getByRole('button', { name: 'Run job' })
    fireEvent.click(runButton)

    expect(screen.getAllByText('terminal-bench-draft').length).toBeGreaterThan(0)
    expect(screen.getByRole('heading', { name: 'Job registry' })).toBeInTheDocument()
  })

  it('shows Harbor maintenance operations in system', () => {
    render(<App />)

    fireEvent.click(screen.getByRole('link', { name: 'System' }))
    expect(screen.getByRole('heading', { name: 'System health' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Auth' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Cache' })).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Plugins' })).toBeInTheDocument()
    expect(screen.getByText('harbor auth status')).toBeInTheDocument()
    expect(screen.getByText('harbor cache clean --dry-run')).toBeInTheDocument()
  })
})
