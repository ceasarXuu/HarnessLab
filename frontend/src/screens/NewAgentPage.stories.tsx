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
    await expect(canvas.getByLabelText('Agent Name')).toHaveValue('Acp Agent')
    await userEvent.click(canvas.getByLabelText('Harness'))
    await userEvent.type(canvas.getByLabelText('Search Harnesses'), 'claude')
    await userEvent.click(canvas.getByRole('option', { name: 'claude-code' }))
    await expect(canvas.getByLabelText('Harness')).toHaveTextContent('claude-code')
    await expect(canvas.getByLabelText('Agent Name')).toHaveValue('Claude Code Agent')
    await expect(canvas.getByRole('tab', { name: 'Basic' })).toHaveAttribute('aria-selected', 'true')
  },
}

export const PrefilledHarness: Story = {
  args: { ...baseArgs, initialHarness: 'claude-code' },
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await expect(canvas.getByLabelText('Harness')).toHaveTextContent('claude-code')
    await expect(canvas.getByLabelText('Agent Name')).toHaveValue('Claude Code Agent')
    await expect(canvas.getByRole('tab', { name: 'Skills' })).toBeVisible()
    await expect(canvas.getByRole('tab', { name: 'MCPs' })).toBeVisible()
    await expect(canvas.getByRole('tab', { name: 'Advanced' })).toBeVisible()
  },
}
