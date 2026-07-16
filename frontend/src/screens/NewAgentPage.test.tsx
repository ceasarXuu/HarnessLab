import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { createMockWebUiClient } from '../api/mockClient'
import { getTranslator } from '../i18n'
import { harnessTemplates } from '../mocks/demoCatalog'
import { NewAgentPage } from './NewAgentPage'

describe('NewAgentPage', () => {
  it('starts empty and requires the user to choose a searchable Harbor Harness', () => {
    render(
      <NewAgentPage
        client={createMockWebUiClient()}
        harnesses={harnessTemplates}
        rows={[]}
        t={getTranslator('en')}
        onAgents={vi.fn()}
        onRefresh={vi.fn(async () => undefined)}
      />,
    )

    expect(screen.getByRole('textbox', { name: /Agent Name/ })).toHaveValue('')
    expect(screen.getByLabelText('Harness')).toHaveTextContent('Select Harness')
    expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled()

    fireEvent.click(screen.getByRole('button', { name: 'Save' }))

    expect(screen.getByRole('alert', { name: 'Check required fields' })).toBeInTheDocument()
    expect(screen.getAllByText('Enter Agent Name.')).toHaveLength(2)
    expect(screen.getAllByText('Select a Harness.')).toHaveLength(2)
    expect(screen.getByRole('textbox', { name: /Agent Name/ })).toHaveAttribute('aria-invalid', 'true')

    fireEvent.click(screen.getByLabelText('Harness'))
    fireEvent.change(screen.getByLabelText('Search Harnesses'), { target: { value: 'claude' } })
    fireEvent.click(screen.getByRole('option', { name: 'claude-code' }))

    expect(screen.getByLabelText('Harness')).toHaveTextContent('claude-code')
    expect(screen.getByRole('textbox', { name: /Agent Name/ })).toHaveValue('')
    expect(screen.getByText('Authentication method')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled()

    fireEvent.change(screen.getByRole('textbox', { name: /Agent Name/ }), { target: { value: 'Claude DeepSeek' } })
    expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled()
    expect(screen.queryByText('Enter Agent Name.')).not.toBeInTheDocument()
    expect(screen.queryByText('Select a Harness.')).not.toBeInTheDocument()
  })

  it('keeps the draft empty after the Harness catalog loads asynchronously', () => {
    const props = {
      client: createMockWebUiClient(),
      rows: [],
      t: getTranslator('en'),
      onAgents: vi.fn(),
      onRefresh: vi.fn(async () => undefined),
    }
    const { rerender } = render(<NewAgentPage {...props} harnesses={[]} />)

    expect(screen.getByLabelText('Agent Name')).toHaveValue('')
    expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled()

    rerender(<NewAgentPage {...props} harnesses={harnessTemplates} />)

    expect(screen.getByLabelText('Harness')).toHaveTextContent('Select Harness')
    expect(screen.getByLabelText('Agent Name')).toHaveValue('')
    expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled()
  })

  it('requires a model when the selected Harness supports model selection', () => {
    render(
      <NewAgentPage
        client={createMockWebUiClient()}
        harnesses={harnessTemplates}
        rows={[]}
        t={getTranslator('en')}
        onAgents={vi.fn()}
        onRefresh={vi.fn(async () => undefined)}
      />,
    )

    fireEvent.change(screen.getByLabelText('Agent Name'), { target: { value: 'Claude' } })
    fireEvent.click(screen.getByLabelText('Harness'))
    fireEvent.click(screen.getByRole('option', { name: 'claude-code' }))
    fireEvent.click(screen.getByRole('button', { name: 'Save' }))

    expect(screen.getAllByText('Add at least one available model.')).toHaveLength(2)
  })
})
