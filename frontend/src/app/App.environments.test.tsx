import { fireEvent, render, screen, waitFor, within } from '@testing-library/react'
import { beforeEach, describe, expect, it } from 'vitest'
import { App } from './App'

describe('Environment templates', () => {
  beforeEach(() => {
    window.localStorage.clear()
    window.location.hash = ''
  })

  it('shows field errors when a new Environment is submitted without a name', async () => {
    window.location.hash = '#environments/new'
    render(<App />)

    const name = await screen.findByLabelText('Environment Name')
    fireEvent.change(name, { target: { value: '' } })
    fireEvent.click(screen.getByRole('button', { name: 'Save' }))

    expect(screen.getByRole('alert', { name: 'Check required fields' })).toBeInTheDocument()
    expect(screen.getAllByText('Enter an Environment name.')).toHaveLength(2)
    expect(name).toHaveAttribute('aria-invalid', 'true')
  })

  it('manages OrnnLab-local templates using real Harbor EnvironmentConfig fields', async () => {
    render(<App />)

    fireEvent.click(screen.getByRole('link', { name: 'Environment' }))
    await screen.findByText('Docker default')
    expect(screen.getByRole('heading', { name: 'Environment catalog' })).toBeInTheDocument()
    expect(screen.getByRole('columnheader', { name: 'Environment Name' })).toBeInTheDocument()
    expect(screen.getByRole('columnheader', { name: 'Profile' })).toBeInTheDocument()
    expect(screen.getByRole('columnheader', { name: 'Type' })).toBeInTheDocument()
    expect(screen.getByRole('columnheader', { name: 'Actions' })).toBeInTheDocument()
    expect(screen.queryByRole('columnheader', { name: 'Docker image name / registry URL' })).not.toBeInTheDocument()

    const builtinRow = screen.getByText('Docker default').closest('tr')
    const customRow = screen.getByText('Docker GPU').closest('tr')
    expect(builtinRow).not.toBeNull()
    expect(customRow).not.toBeNull()
    expect(within(builtinRow as HTMLElement).getByRole('button', { name: 'Copy' })).toBeInTheDocument()
    expect(within(customRow as HTMLElement).getByRole('button', { name: 'Delete' })).toBeInTheDocument()

    fireEvent.click(within(builtinRow as HTMLElement).getByRole('button', { name: 'Copy' }))
    expect(await screen.findByText('Docker default copy', {}, { timeout: 2_500 })).toBeInTheDocument()

    fireEvent.click(customRow as HTMLElement)
    const drawer = screen.getByRole('dialog', { name: 'Selected environment' })
    expect(within(drawer).getByRole('tab', { name: 'Basic' })).toHaveAttribute('aria-selected', 'true')
    expect(within(drawer).getByLabelText('Environment Name')).toHaveValue('Docker GPU')
    expect(within(drawer).getByLabelText('Type')).toHaveAttribute('aria-haspopup', 'listbox')
    expect(within(drawer).getByLabelText('Import path')).toHaveValue('')
    expect(within(drawer).getByText('Environment variables')).toBeInTheDocument()
    expect(within(drawer).queryByLabelText('Working directory')).not.toBeInTheDocument()

    fireEvent.click(within(drawer).getByRole('tab', { name: 'Network' }))
    const allowedHosts = within(drawer).getAllByLabelText('Allowed hosts')
    expect(allowedHosts[0]).toHaveValue('pypi.org')
    expect(allowedHosts[1]).toHaveValue('github.com')
    expect(allowedHosts[2]).toHaveValue('huggingface.co')

    fireEvent.click(within(drawer).getByRole('tab', { name: 'Advanced' }))
    expect(within(drawer).getByLabelText('Force rebuild')).toHaveAttribute('type', 'checkbox')
    expect(within(drawer).getByLabelText('CPU policy')).toHaveAttribute('aria-haspopup', 'listbox')
    expect(within(drawer).getByLabelText('TPU type')).toBeInTheDocument()
    expect(within(drawer).getByLabelText('Topology X')).toHaveAttribute('type', 'number')
    expect(within(drawer).getByLabelText('Extra Docker Compose')).toHaveValue('compose.gpu.yml')
    expect(within(drawer).getByText('Backend params')).toBeInTheDocument()

    fireEvent.click(within(drawer).getByRole('tab', { name: 'Basic' }))
    fireEvent.change(within(drawer).getByLabelText('Environment Name'), { target: { value: 'Docker GPU tuned' } })
    fireEvent.click(within(drawer).getByRole('button', { name: 'Save' }))
    await waitFor(() => expect(screen.getAllByText('Docker GPU tuned').length).toBeGreaterThan(1), { timeout: 2_500 })
    fireEvent.click(screen.getByRole('button', { name: 'Close detail drawer' }))

    const copiedRow = screen.getByText('Docker default copy').closest('tr')
    expect(copiedRow).not.toBeNull()
    fireEvent.click(within(copiedRow as HTMLElement).getByRole('button', { name: 'Delete' }))
    fireEvent.click(screen.getByRole('button', { name: 'Confirm delete' }))
    expect(await screen.findByText('Docker GPU tuned', {}, { timeout: 2_500 })).toBeInTheDocument()

    fireEvent.click(screen.getByRole('link', { name: 'Jobs' }))
    fireEvent.click(screen.getByRole('button', { name: 'New Job' }))
    fireEvent.click(screen.getByLabelText('Environment'))
    expect(screen.getByRole('option', { name: 'Docker GPU tuned' })).toBeInTheDocument()
  })
})
