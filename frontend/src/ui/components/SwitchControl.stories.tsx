import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { useState } from 'react'
import { SwitchControl } from './SwitchControl'

const meta = {
  title: 'Components/SwitchControl',
  component: SwitchControl,
  args: {
    checked: true,
    label: 'Cache prompts',
    onChange: () => undefined,
  },
} satisfies Meta<typeof SwitchControl>

export default meta
type Story = StoryObj<typeof meta>

export const Interactive: Story = {
  render: (args) => {
    const [checked, setChecked] = useState(args.checked)
    return <SwitchControl {...args} checked={checked} onChange={setChecked} />
  },
  play: async ({ canvasElement }) => {
    const control = within(canvasElement).getByRole('switch', { name: 'Cache prompts' })
    await expect(control).toBeChecked()
    await expect(control).toHaveStyle({ height: '18px', width: '32px' })
    await userEvent.click(control)
    await expect(control).not.toBeChecked()
  },
}

export const Disabled: Story = {
  args: { disabled: true },
  play: async ({ canvasElement }) => {
    await expect(within(canvasElement).getByRole('switch', { name: 'Cache prompts' })).toBeDisabled()
  },
}
