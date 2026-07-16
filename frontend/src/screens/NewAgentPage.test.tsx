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

    expect(screen.getByLabelText('Agent Name')).toHaveValue('')
    expect(screen.getByLabelText('Harness')).toHaveTextContent('Select Harness')
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled()

    fireEvent.click(screen.getByLabelText('Harness'))
    fireEvent.change(screen.getByLabelText('Search Harnesses'), { target: { value: 'claude' } })
    fireEvent.click(screen.getByRole('option', { name: 'claude-code' }))

    expect(screen.getByLabelText('Harness')).toHaveTextContent('claude-code')
    expect(screen.getByLabelText('Agent Name')).toHaveValue('')
    expect(screen.getByText('Authentication method')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled()

    fireEvent.change(screen.getByLabelText('Agent Name'), { target: { value: 'Claude DeepSeek' } })
    expect(screen.getByRole('button', { name: 'Save' })).toBeEnabled()
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
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled()

    rerender(<NewAgentPage {...props} harnesses={harnessTemplates} />)

    expect(screen.getByLabelText('Harness')).toHaveTextContent('Select Harness')
    expect(screen.getByLabelText('Agent Name')).toHaveValue('')
    expect(screen.getByRole('button', { name: 'Save' })).toBeDisabled()
  })
})
