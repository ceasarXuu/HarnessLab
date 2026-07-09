import type { Meta, StoryObj } from '@storybook/react-vite'
import { getTranslator } from '../../i18n'
import { OperationStatus } from './OperationStatus'

const t = getTranslator('en')

const meta = {
  title: 'Components/OperationStatus',
  component: OperationStatus,
  parameters: { layout: 'centered' },
} satisfies Meta<typeof OperationStatus>

export default meta
type Story = StoryObj<typeof meta>

export const Queued: Story = {
  args: { operation: { id: 'operation_1', progress: 0, resourceType: 'job', status: 'queued', type: 'create-job' }, t },
}

export const Running: Story = {
  args: { operation: { id: 'operation_1', progress: 50, resourceType: 'dataset', status: 'running', type: 'download-dataset' }, t },
}

export const Completed: Story = {
  args: { operation: { id: 'operation_1', progress: 100, resourceType: 'environment', status: 'completed', type: 'create-environment' }, t },
}

export const Failed: Story = {
  args: { error: 'The requested operation could not be completed.', t },
}
