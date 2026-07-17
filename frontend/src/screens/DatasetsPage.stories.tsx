import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { useState } from 'react'
import { createMockWebUiClient } from '../api/mockClient'
import type { DatasetRow } from '../domain/harbor'
import { datasetRows } from '../mocks/demoCatalog'
import { getTranslator } from '../i18n'
import { DatasetsPage } from './DatasetsPage'

const t = getTranslator('en')
const client = createMockWebUiClient()

const meta = {
  title: 'Screens/Datasets',
  parameters: { layout: 'fullscreen' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

function Fixture({ rows = datasetRows }: { rows?: DatasetRow[] }) {
  const [search, setSearch] = useState('')
  return (
    <DatasetsPage
      client={client}
      rows={rows}
      search={search}
      t={t}
      onRefresh={async () => undefined}
      onSearch={setSearch}
    />
  )
}

export const ActiveDownload: Story = {
  render: () => (
    <Fixture rows={datasetRows.map((row) => row.name === 'swebench-verified'
      ? { ...row, downloadProgress: 42, downloadStatus: 'downloading' }
      : row)} />
  ),
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    const row = canvas.getByText('swebench-verified').closest('tr')
    if (!row) throw new Error('Dataset row was not found')
    await expect(within(row).getByText('42%')).toBeVisible()
    await expect(within(row).getByRole('button', { name: 'Cancel download' })).toBeVisible()
  },
}

export const DownloadDialogAboveDrawer: Story = {
  render: () => <Fixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByText('swebench-verified'))
    const drawer = canvas.getByRole('dialog', { name: 'Selected dataset' })
    await userEvent.click(within(drawer).getByRole('button', { name: 'Download' }))
    const dialog = canvas.getByRole('dialog', { name: 'Download Dataset' })
    await expect(dialog).toBeVisible()
    const modalLayer = dialog.closest('.confirm-overlay')
    const drawerLayer = drawer.closest('.drawer-layer')
    if (!modalLayer || !drawerLayer) throw new Error('Overlay layers were not found')
    expect(Number(window.getComputedStyle(modalLayer).zIndex)).toBeGreaterThan(
      Number(window.getComputedStyle(drawerLayer).zIndex),
    )
  },
}
