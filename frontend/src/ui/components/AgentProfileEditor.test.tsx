import { fireEvent, render, screen } from '@testing-library/react'
import { useState } from 'react'
import { describe, expect, it } from 'vitest'
import type { AgentCapabilities, AgentRow } from '../../domain/harbor'
import { getTranslator } from '../../i18n'
import { AgentProfileEditor } from './AgentProfileEditor'

const claudeCapabilities = {
  authenticationModes: [
    {
      environmentVariables: ['ANTHROPIC_API_KEY', 'ANTHROPIC_AUTH_TOKEN', 'ANTHROPIC_BASE_URL'],
      label: 'Anthropic API',
      value: 'anthropic-api',
    },
    { environmentVariables: ['CLAUDE_CODE_OAUTH_TOKEN'], label: 'Claude OAuth', value: 'oauth' },
    {
      environmentVariables: ['AWS_ACCESS_KEY_ID', 'AWS_SECRET_ACCESS_KEY', 'AWS_REGION'],
      label: 'Amazon Bedrock',
      value: 'bedrock',
    },
  ],
  environmentVariables: ['CLAUDE_CODE_MAX_OUTPUT_TOKENS'],
  parameters: [
    { key: 'max_thinking_tokens', kind: 'number', label: 'Max thinking tokens', source: 'kwarg' },
  ],
  supportedFields: ['env', 'harnessParameters', 'skills'],
} as AgentCapabilities

describe('AgentProfileEditor', () => {
  it('filters Claude Code variables by authentication mode', () => {
    render(<ClaudeEditorFixture />)

    expect(screen.getByRole('button', { name: 'Authentication method' })).toHaveTextContent('Anthropic API')
    fireEvent.click(screen.getByRole('button', { name: 'Add Variables' }))
    fireEvent.click(screen.getByRole('button', { name: 'Env key' }))
    expect(screen.getByRole('option', { name: 'ANTHROPIC_API_KEY' })).toBeInTheDocument()
    expect(screen.getByRole('option', { name: 'CLAUDE_CODE_MAX_OUTPUT_TOKENS' })).toBeInTheDocument()
    expect(screen.queryByRole('option', { name: 'AWS_ACCESS_KEY_ID' })).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('button', { name: 'Authentication method' }))
    fireEvent.click(screen.getByRole('option', { name: 'Amazon Bedrock' }))
    fireEvent.click(screen.getByRole('button', { name: 'Add Variables' }))
    fireEvent.click(screen.getByRole('button', { name: 'Env key' }))
    expect(screen.getByRole('option', { name: 'AWS_ACCESS_KEY_ID' })).toBeInTheDocument()
    expect(screen.queryByRole('option', { name: 'ANTHROPIC_API_KEY' })).not.toBeInTheDocument()
  })

  it('renders max thinking tokens as an advanced numeric parameter', () => {
    render(<ClaudeEditorFixture />)

    fireEvent.click(screen.getByRole('tab', { name: 'Advanced' }))
    expect(screen.getByRole('spinbutton', { name: 'Max thinking tokens' })).toBeInTheDocument()
  })

  it('uses folder selection as the only way to add a Skills source', () => {
    render(<ClaudeEditorFixture />)

    fireEvent.click(screen.getByRole('tab', { name: 'Skills' }))

    expect(screen.getByRole('button', { name: 'Choose folder' })).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Add' })).not.toBeInTheDocument()
  })
})

function ClaudeEditorFixture() {
  const [value, setValue] = useState<AgentRow>({
    adapter: 'none', agentName: 'Claude Code', authenticationMode: 'anthropic-api',
    capabilities: claudeCapabilities, env: 'none', harness: 'claude-code', id: 'claude-code-profile',
    kwargs: 'none', maxTimeout: '-', mcp: 'none', modelPricing: [], models: 'none', runtime: '-', setupTimeout: '-',
    skills: 'none', source: 'OrnnLab profile', status: 'configured', timeout: '-', updated: '-',
  })
  return <AgentProfileEditor t={getTranslator('en')} value={value} onChange={setValue} />
}
