import { fireEvent, render, screen, within } from '@testing-library/react'
import { beforeEach, describe, expect, it } from 'vitest'
import { App } from './App'

describe('App', () => {
  beforeEach(() => {
    window.localStorage.clear()
    window.location.hash = ''
  })

  it('renders the jobs hierarchy without flattening the run builder into it', () => {
    render(<App />)

    expect(screen.getByRole('heading', { name: 'Job registry' })).toBeInTheDocument()
    expect(screen.getAllByText('terminal-bench-smoke').length).toBeGreaterThan(0)
    expect(screen.getByText('Selected job')).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: 'System doctor' })).toBeInTheDocument()
    expect(screen.queryByRole('heading', { name: 'New Run' })).not.toBeInTheDocument()
    expect(screen.queryByRole('link', { name: 'New Run' })).not.toBeInTheDocument()
  })

  it('filters jobs and keeps the table as the primary surface', () => {
    render(<App />)

    fireEvent.change(screen.getByLabelText('Search jobs'), { target: { value: 'swe' } })

    const jobsTable = screen.getByRole('table')
    expect(screen.getByText('swe-bench-lite-regression')).toBeInTheDocument()
    expect(within(jobsTable).queryByText('terminal-bench-smoke')).not.toBeInTheDocument()
  })

  it('navigates to the other Harbor demo pages', () => {
    render(<App />)

    fireEvent.click(screen.getByRole('link', { name: 'Tasks' }))
    expect(screen.getByRole('heading', { name: 'Task queue' })).toBeInTheDocument()

    fireEvent.click(screen.getByRole('link', { name: 'Trials' }))
    expect(screen.getByRole('heading', { name: 'Trial matrix' })).toBeInTheDocument()

    fireEvent.click(screen.getByRole('link', { name: 'System' }))
    expect(screen.getByRole('heading', { name: 'System health' })).toBeInTheDocument()
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

    fireEvent.click(screen.getByRole('button', { name: 'Run job' }))
    expect(window.location.hash).toBe('#jobs/new')
    expect(screen.getByRole('link', { name: 'Jobs' })).toHaveClass('active')
    const runBuilder = screen.getByRole('heading', { name: 'New Job' }).closest('section')
    expect(runBuilder).not.toBeNull()
    const runButton = within(runBuilder as HTMLElement).getByRole('button', { name: 'Run job' })
    fireEvent.click(runButton)

    expect(screen.getAllByText('terminal-bench-draft').length).toBeGreaterThan(0)
    expect(screen.getByRole('heading', { name: 'Job registry' })).toBeInTheDocument()
  })
})
