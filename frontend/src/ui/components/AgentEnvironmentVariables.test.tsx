import { fireEvent, render, screen } from '@testing-library/react'
import { useState } from 'react'
import { describe, expect, it, vi } from 'vitest'
import type { AgentCapabilities, AgentRow } from '../../domain/harbor'
import { getTranslator } from '../../i18n'
import { AgentEnvironmentVariables } from './AgentEnvironmentVariables'

const capabilities: AgentCapabilities = {
  authenticationModes: [
    { environmentVariables: ['ANTHROPIC_API_KEY', 'ANTHROPIC_BASE_URL'], label: 'Anthropic API', value: 'anthropic-api' },
  ],
  environmentVariables: ['CLAUDE_CODE_MAX_OUTPUT_TOKENS'],
  parameters: [],
  supportedFields: ['env'],
}

function agentRow(overrides: Partial<AgentRow> = {}): AgentRow {
  return {
    adapter: 'none',
    agentName: 'Claude Code',
    authenticationMode: 'anthropic-api',
    capabilities,
    env: 'ANTHROPIC_API_KEY=sk-ant-secret\nANTHROPIC_BASE_URL=https://api.example.test',
    harness: 'claude-code',
    hiddenEnvKeys: [],
    id: 'claude-code-profile',
    kwargs: 'none',
    maxTimeout: '-',
    mcp: 'none',
    modelPricing: [],
    models: 'none',
    runtime: '-',
    setupTimeout: '-',
    skills: 'none',
    source: 'OrnnLab profile',
    status: 'configured',
    timeout: '-',
    updated: '-',
    ...overrides,
  }
}

function renderVariables(value: AgentRow, onChange: (next: AgentRow) => void) {
  render(<AgentEnvironmentVariables capabilities={capabilities} readOnly={false} t={getTranslator('en')} value={value} onChange={onChange} />)
}

function ControlledFixture({ initial, onChange }: { initial: AgentRow; onChange: (next: AgentRow) => void }) {
  const [agent, setAgent] = useState(initial)
  return (
    <AgentEnvironmentVariables
      capabilities={capabilities}
      readOnly={false}
      t={getTranslator('en')}
      value={agent}
      onChange={(next) => {
        setAgent(next)
        onChange(next)
      }}
    />
  )
}

describe('AgentEnvironmentVariables visibility', () => {
  it('renders hidden values as password fields with a Show button', () => {
    renderVariables(agentRow({ hiddenEnvKeys: ['ANTHROPIC_API_KEY'] }), () => undefined)

    const secret = valueInput('ANTHROPIC_API_KEY')
    expect(secret).toHaveAttribute('type', 'password')
    expect(screen.getByRole('button', { name: 'Show ANTHROPIC_API_KEY' })).toBeInTheDocument()
    expect(secret.closest('.key-value-row')?.className).toContain('key-value-row--with-visibility')
    expect(screen.getByRole('button', { name: 'Delete Variables ANTHROPIC_API_KEY' })).toBeInTheDocument()
  })

  it('toggles the hidden key in hiddenEnvKeys when the button is clicked', () => {
    const onChange = vi.fn()
    render(<ControlledFixture initial={agentRow({ hiddenEnvKeys: ['ANTHROPIC_API_KEY'] })} onChange={onChange} />)

    fireEvent.click(screen.getByRole('button', { name: 'Show ANTHROPIC_API_KEY' }))
    expect(onChange).toHaveBeenLastCalledWith(
      expect.objectContaining({ hiddenEnvKeys: [] }),
    )

    fireEvent.click(screen.getByRole('button', { name: 'Hide ANTHROPIC_BASE_URL' }))
    expect(onChange).toHaveBeenLastCalledWith(
      expect.objectContaining({ hiddenEnvKeys: ['ANTHROPIC_BASE_URL'] }),
    )
  })

  it('accumulates hidden keys from multiple rows', () => {
    const onChange = vi.fn()
    render(<ControlledFixture initial={agentRow({ hiddenEnvKeys: [] })} onChange={onChange} />)

    fireEvent.click(screen.getByRole('button', { name: 'Hide ANTHROPIC_API_KEY' }))
    fireEvent.click(screen.getByRole('button', { name: 'Hide ANTHROPIC_BASE_URL' }))

    expect(onChange).toHaveBeenLastCalledWith(
      expect.objectContaining({ hiddenEnvKeys: ['ANTHROPIC_API_KEY', 'ANTHROPIC_BASE_URL'] }),
    )
  })

  it('shows no visibility button when the value is empty', () => {
    renderVariables(agentRow({ env: 'ANTHROPIC_API_KEY=' }), () => undefined)

    expect(valueInput('ANTHROPIC_API_KEY')).toHaveAttribute('type', 'text')
    expect(screen.queryByRole('button', { name: /^(Show|Hide) ANTHROPIC_API_KEY$/ })).not.toBeInTheDocument()
  })
})

function valueInput(key: string) {
  const rows = screen.getAllByLabelText('Env value') as HTMLInputElement[]
  const row = rows.find((input) => input.closest('.key-value-row')?.textContent?.includes(key))
  if (!row) throw new Error(`no value input for ${key}`)
  return row
}
