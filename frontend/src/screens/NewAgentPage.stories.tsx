import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { createMockWebUiClient } from '../api/mockClient'
import { getTranslator } from '../i18n'
import { agentRows, harnessTemplates } from '../mocks/demoCatalog'
import { NewAgentPage } from './NewAgentPage'

const meta = {
  component: NewAgentPage,
  parameters: { layout: 'fullscreen' },
  title: 'Screens/Agents/New Agent',
} satisfies Meta<typeof NewAgentPage>

export default meta
type Story = StoryObj<typeof meta>

const baseArgs = {
  client: createMockWebUiClient(),
  harnesses: harnessTemplates,
  onAgents: () => undefined,
  onRefresh: async () => undefined,
  rows: agentRows,
  t: getTranslator('en'),
}

export const HarnessCatalog: Story = {
  args: baseArgs,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await expect(canvas.getByRole('heading', { name: 'New Agent' })).toBeVisible()
    await expect(canvas.getByLabelText('Agent Name')).toHaveValue('')
    await expect(canvas.getByLabelText('Harness')).toHaveTextContent('Select Harness')
    await userEvent.click(canvas.getByLabelText('Harness'))
    await userEvent.type(canvas.getByLabelText('Search Harnesses'), 'claude')
    await userEvent.click(canvas.getByRole('option', { name: 'claude-code' }))
    await expect(canvas.getByLabelText('Harness')).toHaveTextContent('claude-code')
    await expect(canvas.getByLabelText('Agent Name')).toHaveValue('')
    await userEvent.type(canvas.getByLabelText('Agent Name'), 'Claude DeepSeek')
    await expect(canvas.getByRole('tab', { name: 'Basic' })).toHaveAttribute('aria-selected', 'true')
  },
}

export const ValidationErrors: Story = {
  args: baseArgs,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByRole('button', { name: 'Save' }))
    await expect(canvas.queryByText('Check required fields')).not.toBeInTheDocument()
    await expect(canvas.getByText('Enter Agent Name.')).toBeVisible()
    await expect(canvas.getByText('Select a Harness.')).toBeVisible()
  },
}
