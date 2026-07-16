import { fireEvent, render, screen } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import { createMockWebUiClient } from '../api/mockClient'
import { getTranslator } from '../i18n'
import { harnessTemplates } from '../mocks/demoCatalog'
import { NewAgentPage } from './NewAgentPage'

describe('NewAgentPage', () => {
  it('selects a searchable Harbor Harness template before creating an Agent', () => {
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

    expect(screen.getByLabelText('Agent Name')).toHaveValue('Acp Agent')
    expect(screen.getByLabelText('Harness')).toHaveTextContent('acp')

    fireEvent.click(screen.getByLabelText('Harness'))
    fireEvent.change(screen.getByLabelText('Search Harnesses'), { target: { value: 'claude' } })
    fireEvent.click(screen.getByRole('option', { name: 'claude-code' }))

    expect(screen.getByLabelText('Harness')).toHaveTextContent('claude-code')
    expect(screen.getByLabelText('Agent Name')).toHaveValue('Claude Code Agent')
    expect(screen.getByText('Authentication method')).toBeInTheDocument()
  })

  it('applies the first Harness template after the catalog loads asynchronously', () => {
    const props = {
      client: createMockWebUiClient(),
      rows: [],
      t: getTranslator('en'),
      onAgents: vi.fn(),
      onRefresh: vi.fn(async () => undefined),
    }
    const { rerender } = render(<NewAgentPage {...props} harnesses={[]} />)

    expect(screen.getByLabelText('Agent Name')).toHaveValue('New Agent')
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled()

    rerender(<NewAgentPage {...props} harnesses={harnessTemplates} />)

    expect(screen.getByLabelText('Harness')).toHaveTextContent('acp')
    expect(screen.getByLabelText('Agent Name')).toHaveValue('Acp Agent')
    expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled()
  })
})
