import { fireEvent, render, screen, waitFor, within } from '@testing-library/react'
import { beforeEach, describe, expect, it } from 'vitest'
import { createMockWebUiClient } from '../api/mockClient'
import { App } from './App'

function downloadClient() {
  const client = createMockWebUiClient()
  client.chooseDirectory = async () => ({
    data: { path: '/tmp/datasets' },
    error: null,
    meta: { requestId: 'directory-picker-test' },
  })
  return client
}

describe('Dataset download lifecycle', () => {
  beforeEach(() => {
    window.localStorage.clear()
    window.location.hash = ''
  })

  it('restores an active download after the application is remounted', async () => {
    const client = downloadClient()
    const firstMount = render(<App client={client} />)
    fireEvent.click(screen.getByRole('link', { name: 'Datasets' }))
    const firstRow = (await screen.findByText('swebench-verified')).closest('tr')
    if (!firstRow) throw new Error('Expected Dataset row')
    fireEvent.click(within(firstRow).getByRole('button', { name: 'Download' }))
    const dialog = await screen.findByRole('dialog', { name: 'Download Dataset' })
    fireEvent.click(within(dialog).getByRole('button', { name: 'Choose folder' }))
    await waitFor(() => expect(within(dialog).getByLabelText('Dataset parent directory')).toHaveValue('/tmp/datasets'))
    fireEvent.click(within(dialog).getByRole('button', { name: 'Start download' }))
    await within(firstRow).findByText('0%')
    firstMount.unmount()

    render(<App client={client} />)
    fireEvent.click(screen.getByRole('link', { name: 'Datasets' }))
    const restoredRow = (await screen.findByText('swebench-verified')).closest('tr')
    if (!restoredRow) throw new Error('Expected restored Dataset row')

    expect(await within(restoredRow).findByText('50%')).toBeInTheDocument()
    expect(within(restoredRow).getByRole('button', { name: 'Cancel download' })).toBeInTheDocument()
  })
})
