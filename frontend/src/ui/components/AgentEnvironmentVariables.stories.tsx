import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { useState } from 'react'
import type { AgentCapabilities, AgentRow } from '../../domain/harbor'
import { getTranslator } from '../../i18n'
import { AgentEnvironmentVariables } from './AgentEnvironmentVariables'

const capabilities: AgentCapabilities = {
  authenticationModes: [
    { environmentVariables: ['ANTHROPIC_API_KEY', 'ANTHROPIC_BASE_URL'], label: 'Anthropic API', value: 'anthropic-api' },
    { environmentVariables: ['CLAUDE_CODE_OAUTH_TOKEN'], label: 'Claude OAuth', value: 'oauth' },
    { environmentVariables: ['AWS_ACCESS_KEY_ID', 'AWS_SECRET_ACCESS_KEY', 'AWS_REGION'], label: 'Amazon Bedrock', value: 'bedrock' },
  ],
  environmentVariables: ['CLAUDE_CODE_MAX_OUTPUT_TOKENS'],
  parameters: [],
  supportedFields: ['env'],
}

function AuthenticationModesFixture() {
  const [agent, setAgent] = useState<AgentRow>({
    adapter: 'none', agentName: 'Claude Code', authenticationMode: 'anthropic-api', capabilities,
    env: 'none', harness: 'claude-code', id: 'claude-code-profile', kwargs: 'none', maxTimeout: '-',
    mcp: 'none', modelPricing: [], models: 'none', runtime: '-', setupTimeout: '-', skills: 'none', source: 'OrnnLab profile',
    status: 'configured', timeout: '-', updated: '-',
  })
  return <AgentEnvironmentVariables capabilities={capabilities} readOnly={false} t={getTranslator('en')} value={agent} onChange={setAgent} />
}

const meta = {
  title: 'Patterns/Agent/EnvironmentVariables',
  parameters: { layout: 'padded' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof AuthenticationModesFixture>

export const AuthenticationModes: Story = {
  render: () => <AuthenticationModesFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await expect(canvas.getByLabelText('Authentication method')).toHaveTextContent('Anthropic API')
    await userEvent.click(canvas.getByLabelText('Authentication method'))
    await userEvent.click(canvas.getByRole('option', { name: 'Amazon Bedrock' }))
    await userEvent.click(canvas.getByRole('button', { name: 'Add Variables' }))
    await userEvent.click(canvas.getByLabelText('Env key'))
    await expect(canvas.getByRole('option', { name: 'AWS_ACCESS_KEY_ID' })).toBeVisible()
    await expect(canvas.queryByRole('option', { name: 'ANTHROPIC_API_KEY' })).not.toBeInTheDocument()
  },
}
