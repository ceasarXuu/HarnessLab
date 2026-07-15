import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { CustomSelect } from './CustomSelect'

const buildOptions = (count: number) => Array.from({ length: count }, (_, index) => ({
  label: `Option ${index + 1}`,
  value: String(index + 1),
}))

const meta = {
  title: 'Components/CustomSelect',
  component: CustomSelect,
  args: {
    ariaLabel: 'Dataset',
    onChange: () => undefined,
    options: buildOptions(11),
    value: '',
  },
} satisfies Meta<typeof CustomSelect>

export default meta
type Story = StoryObj<typeof meta>

export const AutomaticSearch: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByLabelText('Dataset'))
    const search = canvas.getByRole('textbox', { name: 'Search Dataset' })
    await expect(search).toBeVisible()
    await userEvent.type(search, '11')
    await expect(canvas.getByRole('option', { name: 'Option 11' })).toBeVisible()
    await expect(canvas.queryByRole('option', { name: 'Option 1' })).not.toBeInTheDocument()
  },
}

export const CompactAtTen: Story = {
  args: { options: buildOptions(10) },
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByLabelText('Dataset'))
    await expect(canvas.queryByRole('textbox', { name: 'Search Dataset' })).not.toBeInTheDocument()
    await expect(canvas.getAllByRole('option')).toHaveLength(10)
  },
}
