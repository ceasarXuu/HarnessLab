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
    expect(screen.getByText('Job trials')).toBeInTheDocument()
  })

  it('renders datasets as the Harbor catalog surface', () => {
    render(<App />)

    fireEvent.click(screen.getByRole('link', { name: 'Datasets' }))
    expect(screen.getByRole('heading', { name: 'Dataset catalog' })).toBeInTheDocument()
    expect(screen.getAllByText('terminal-bench').length).toBeGreaterThan(0)
    expect(screen.getByText('Selected dataset')).toBeInTheDocument()
    expect(screen.getByText('Dataset tasks')).toBeInTheDocument()
    expect(screen.getAllByRole('button', { name: 'Run single task' }).length).toBeGreaterThan(0)
    expect(screen.getByRole('link', { name: 'Datasets' })).toHaveClass('active')
  })

  it('renders the jobs hierarchy without flattening the run builder into it', () => {
    render(<App />)

    fireEvent.click(screen.getByRole('link', { name: 'Jobs' }))
    expect(screen.getByRole('heading', { name: 'Job registry' })).toBeInTheDocument()
    expect(screen.getAllByText('terminal-bench-smoke').length).toBeGreaterThan(0)
    expect(screen.getByText('Selected job')).toBeInTheDocument()
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
    expect(screen.getByText('Dataset tasks')).toBeInTheDocument()

    fireEvent.click(screen.getByRole('link', { name: 'Jobs' }))
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

    fireEvent.click(screen.getByRole('link', { name: 'Leaderboard' }))
    expect(screen.getByRole('heading', { name: 'Leaderboard' })).toBeInTheDocument()
    expect(screen.getByLabelText('Select dataset')).toHaveValue('terminal-bench@2.0')
    expect(screen.getByText('claude-code')).toBeInTheDocument()

    fireEvent.change(screen.getByLabelText('Search datasets'), { target: { value: 'swe' } })
    fireEvent.change(screen.getByLabelText('Select dataset'), { target: { value: 'swe-bench-lite@2026.06' } })
    expect(screen.getByText('job_74c1')).toBeInTheDocument()
    fireEvent.change(screen.getByLabelText('Search leaderboard'), { target: { value: 'sonnet' } })
    expect(screen.getByText('claude-sonnet-4-5')).toBeInTheDocument()
    expect(screen.queryByText('gpt-5.1')).not.toBeInTheDocument()
  })

  it('switches language and theme from the header', () => {
    render(<App />)

    fireEvent.change(screen.getByLabelText('Language'), { target: { value: 'zh' } })
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
    const runButton = within(runBuilder as HTMLElement).getByRole('button', { name: 'Run job' })
    fireEvent.click(runButton)

    expect(screen.getAllByText('terminal-bench-draft').length).toBeGreaterThan(0)
    expect(screen.getByRole('heading', { name: 'Job registry' })).toBeInTheDocument()
  })
})
