import { fireEvent, render, screen } from '@testing-library/react'
import userEvent from '@testing-library/user-event'
import { describe, expect, it, vi } from 'vitest'
import { createMockWebUiClient } from '../api/mockClient'
import type { DatasetRow } from '../domain/harbor'
import { getTranslator } from '../i18n'
import { DatasetsPage } from './DatasetsPage'

function makeRow(overrides: Partial<DatasetRow> = {}): DatasetRow {
  return {
    name: 'terminal-bench',
    version: '2.0',
    visibility: 'public',
    tasks: 89,
    source: 'registry',
    downloadStatus: 'not-downloaded',
    ...overrides,
  }
}

describe('DatasetsPage', () => {
  it('shows the operation error when a Dataset download is rejected', async () => {
    const user = userEvent.setup()
    const client = createMockWebUiClient()
    vi.spyOn(client, 'downloadDataset').mockResolvedValue({
      data: null,
      error: { code: 'INVALID_REQUEST', message: 'dataset destination already exists' },
    })
    const row = makeRow()

    render(
      <DatasetsPage
        client={client}
        rows={[row]}
        search=""
        t={getTranslator('en')}
        onRefresh={async () => undefined}
        onSearch={() => undefined}
      />,
    )

    await user.click(screen.getByRole('button', { name: 'Download' }))
    fireEvent.change(screen.getByLabelText('Dataset parent directory'), {
      target: { value: '/tmp/parent' },
    })
    await user.click(screen.getByRole('button', { name: 'Start download' }))

    expect(await screen.findByText('dataset destination already exists')).toBeInTheDocument()
  })
})
