import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { createMockWebUiClient } from '../api/mockClient'
import type { SystemRow } from '../domain/harbor'
import { getTranslator } from '../i18n'
import { degradedSystemRows, systemRows } from '../mocks/demoSystem'
import { SystemPage } from './SystemPage'

const client = createMockWebUiClient()

const meta = {
  title: 'Screens/System health',
  component: SystemPage,
  parameters: { layout: 'fullscreen' },
} satisfies Meta<typeof SystemPage>

export default meta
type Story = StoryObj<typeof meta>

export const Default: Story = {
  args: { client, rows: systemRows, t: getTranslator('en'), onRefresh: async () => undefined },
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await expect(canvas.getByRole('heading', { name: 'Services & dependencies' })).toBeVisible()
    await expect(canvas.queryByRole('table')).not.toBeInTheDocument()
    await expect(within(canvas.getByRole('article', { name: 'OrnnLab Service' })).getByText('Running')).toBeVisible()
  },
}

export const DockerNotRunning: Story = {
  args: {
    ...Default.args,
    rows: replaceRow('docker', (row) => ({ ...row, state: 'not-running', serverVersion: null, actions: [], error: 'Docker daemon is not running' })),
  },
  play: async ({ canvasElement }) => {
    const card = within(canvasElement).getByRole('article', { name: 'Docker' })
    await expect(within(card).getByText('Not running')).toBeVisible()
    await expect(within(card).getByText('Docker service is not running. Start your local Docker service to use Harbor.')).toBeVisible()
    await expect(within(card).queryByText('Docker daemon is not running')).not.toBeInTheDocument()
    await expect(within(card).getByText('28.1.1')).toBeVisible()
    await expect(within(card).getByText('Server version')).toBeVisible()
    await expect(within(card).queryByRole('button', { name: 'Clean cache' })).not.toBeInTheDocument()
  },
}

export const ServiceDegraded: Story = {
  args: { ...Default.args, rows: degradedSystemRows },
}

export const StorageCritical: Story = {
  args: {
    ...Default.args,
    rows: replaceRow('resource-storage', (row) => ({ ...row, state: 'critical', availableBytes: 2 * 1024 ** 3 })),
  },
}

export const Chinese: Story = {
  args: { ...Default.args, t: getTranslator('zh') },
}

export const Empty: Story = {
  args: { ...Default.args, rows: [] },
}

export const DestructiveConfirmation: Story = {
  args: Default.args,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    const docker = canvas.getByRole('article', { name: 'Docker' })
    await userEvent.click(within(docker).getByRole('button', { name: 'Clean cache' }))
    await expect(canvas.getByRole('dialog', { name: 'Clean Docker cache' })).toBeVisible()
  },
}

function replaceRow<K extends SystemRow['kind']>(kind: K, change: (row: Extract<SystemRow, { kind: K }>) => Extract<SystemRow, { kind: K }>): SystemRow[] {
  return systemRows.map((row) => row.kind === kind ? change(row as Extract<SystemRow, { kind: K }>) : row)
}
