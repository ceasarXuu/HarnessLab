import { fireEvent, render, screen, within } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { App } from './App'

describe('App', () => {
  it('renders the Harbor Viewer-style jobs surface', () => {
    render(<App />)

    expect(screen.getByRole('heading', { name: 'Jobs' })).toBeInTheDocument()
    expect(screen.getAllByText('terminal-bench-smoke').length).toBeGreaterThan(0)
    expect(screen.getByRole('heading', { name: 'New Run' })).toBeInTheDocument()
    expect(screen.getByRole('heading', { name: 'System doctor' })).toBeInTheDocument()
  })

  it('filters jobs and keeps the table as the primary surface', () => {
    render(<App />)

    fireEvent.change(screen.getByLabelText('Search jobs'), { target: { value: 'swe' } })

    const jobsTable = screen.getByRole('table')
    expect(screen.getByText('swe-bench-lite-regression')).toBeInTheDocument()
    expect(within(jobsTable).queryByText('terminal-bench-smoke')).not.toBeInTheDocument()
  })

  it('adds a queued job from the run builder', () => {
    render(<App />)

    const runBuilder = screen.getByRole('heading', { name: 'New Run' }).closest('section')
    expect(runBuilder).not.toBeNull()
    const runButton = within(runBuilder as HTMLElement).getByRole('button', { name: 'Run job' })
    fireEvent.click(runButton)

    expect(screen.getAllByText('terminal-bench-draft').length).toBeGreaterThan(0)
    expect(screen.getAllByText('queued').length).toBeGreaterThan(0)
  })
})
