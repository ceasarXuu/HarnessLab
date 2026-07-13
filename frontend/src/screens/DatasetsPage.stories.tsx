import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { useState } from 'react'
import { createMockWebUiClient } from '../api/mockClient'
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

function Fixture() {
  const [search, setSearch] = useState('')
  return (
    <DatasetsPage
      client={client}
      rows={datasetRows}
      search={search}
      t={t}
      onRefresh={async () => undefined}
      onSearch={setSearch}
    />
  )
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
