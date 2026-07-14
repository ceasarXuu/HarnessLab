import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, within } from 'storybook/test'
import { Metric } from './Metric'

const meta = {
  title: 'Components/Metric',
  component: Metric,
  args: {
    label: 'Harness',
    value: 'claude-code',
  },
} satisfies Meta<typeof Metric>

export default meta
type Story = StoryObj<typeof meta>

export const Card: Story = {}

export const PlainReadOnly: Story = {
  args: { variant: 'plain' },
  play: async ({ canvasElement }) => {
    const metric = within(canvasElement).getByText('claude-code').closest('.metric')
    await expect(metric).toHaveClass('metric--plain')
    await expect(metric).not.toHaveClass('metric--card')
    await expect(within(canvasElement).getByText('claude-code')).toHaveStyle({ minHeight: '34px' })
  },
}
