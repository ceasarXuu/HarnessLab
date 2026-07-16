import type { Meta, StoryObj } from '@storybook/react-vite'
import { getTranslator } from '../../i18n'
import { JobStatusBadge } from './JobStatusBadge'

const meta = {
  title: 'Components/JobStatusBadge',
  component: JobStatusBadge,
  args: {
    status: 'running',
    t: getTranslator('en'),
  },
} satisfies Meta<typeof JobStatusBadge>

export default meta
type Story = StoryObj<typeof meta>

export const Running: Story = {}

export const QueuedChinese: Story = {
  args: { status: 'queued', t: getTranslator('zh') },
}

export const TerminalStates: Story = {
  render: () => (
    <div className="button-row">
      {(['completed', 'failed', 'cancelled', 'interrupted'] as const).map((status) => (
        <JobStatusBadge key={status} status={status} t={getTranslator('en')} />
      ))}
    </div>
  ),
}
