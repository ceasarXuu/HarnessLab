import type { Meta, StoryObj } from '@storybook/react-vite'
import { ResourceStatus } from './ResourceStatus'

const meta = {
  title: 'Components/ResourceStatus',
  component: ResourceStatus,
  parameters: { layout: 'centered' },
} satisfies Meta<typeof ResourceStatus>

export default meta
type Story = StoryObj<typeof meta>

export const Loading: Story = {
  args: { error: null, loading: true, loadingLabel: 'Loading Jobs.' },
}

export const Error: Story = {
  args: { error: 'The API request could not be completed.', loading: false, loadingLabel: 'Loading Jobs.' },
}
