import type { Meta, StoryObj } from '@storybook/react-vite'
import { fn, userEvent, within, expect } from 'storybook/test'
import { FormValidationSummary } from './FormValidationSummary'

const meta = {
  component: FormValidationSummary,
  title: 'Components/Forms/Validation Summary',
  args: {
    issues: [
      { field: 'name', message: 'Enter a name.' },
      { field: 'type', message: 'Select a type.' },
    ],
    onIssue: fn(),
    title: 'Check required fields',
  },
} satisfies Meta<typeof FormValidationSummary>

export default meta
type Story = StoryObj<typeof meta>

export const FieldErrors: Story = {
  play: async ({ args, canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByRole('button', { name: 'Enter a name.' }))
    await expect(args.onIssue).toHaveBeenCalledWith('name')
  },
}

export const ServerError: Story = {
  args: {
    issues: [],
    serverError: 'The Agent name is already in use.',
  },
}
