import { fireEvent, render, screen, within } from '@testing-library/react'
import { beforeEach, describe, expect, it } from 'vitest'
import { App } from './App'

describe('App agents and leaderboard', () => {
  beforeEach(() => {
    window.localStorage.clear()
    window.location.hash = ''
  })

  it('renders configurable agents and dataset-scoped leaderboard pages', () => {
    render(<App />)

    fireEvent.click(screen.getByRole('link', { name: 'Agents' }))
    expect(screen.getByRole('heading', { name: 'Agent catalog' })).toBeInTheDocument()
    expect(screen.getByLabelText('Search agents')).toBeInTheDocument()
    expect(screen.getByRole('button', { name: 'New Agent' })).toBeInTheDocument()
    expect(screen.getByRole('columnheader', { name: 'Agent Name' })).toBeInTheDocument()
    expect(screen.getByRole('columnheader', { name: 'Harness' })).toBeInTheDocument()
    expect(screen.getByRole('columnheader', { name: 'Actions' })).toBeInTheDocument()
    expect(screen.queryByRole('columnheader', { name: 'Adapter' })).not.toBeInTheDocument()
    expect(screen.queryByRole('columnheader', { name: 'Source' })).not.toBeInTheDocument()
    expect(screen.queryByRole('columnheader', { name: 'Updated' })).not.toBeInTheDocument()
    expect(screen.getByText('Claude Code default')).toBeInTheDocument()
    expect(screen.getByText('claude-code')).toBeInTheDocument()
    expect(screen.getByText('Local repair agent')).toBeInTheDocument()

    const builtinRow = screen.getByText('Claude Code default').closest('tr')
    expect(builtinRow).not.toBeNull()
    expect(within(builtinRow as HTMLElement).queryByRole('button', { name: 'Delete' })).not.toBeInTheDocument()

    const customRow = screen.getByText('Local repair agent').closest('tr')
    expect(customRow).not.toBeNull()
    fireEvent.click(within(customRow as HTMLElement).getByRole('button', { name: 'Delete' }))
    expect(screen.getByRole('dialog', { name: 'Delete custom agent' })).toBeInTheDocument()
    expect(screen.getByText('This removes the custom agent configuration from the local WebUI list.')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Cancel' }))

    fireEvent.change(screen.getByLabelText('Search agents'), { target: { value: 'local' } })
    expect(screen.getByText('Local repair agent')).toBeInTheDocument()
    expect(screen.queryByText('claude-code')).not.toBeInTheDocument()
    fireEvent.change(screen.getByLabelText('Search agents'), { target: { value: '' } })

    fireEvent.click(screen.getByText('Local repair agent'))
    const agentDialog = screen.getByRole('dialog', { name: 'Selected agent' })
    expect(agentDialog).toBeInTheDocument()
    expect(within(agentDialog).queryByText('Selected agent')).not.toBeInTheDocument()
    expect(screen.getByText('Credentials and parameters')).toBeInTheDocument()
    const agentForm = within(agentDialog)
    ;[
      ['Agent Name', 'Local repair agent'],
      ['Harness', 'custom-harness'],
      ['Type', 'custom'],
      ['Custom import path', 'agents.local_repair:Agent'],
      ['Temperature', '0.2'],
      ['Context length', '131072'],
      ['API key env', 'LOCAL_MODEL_API_KEY'],
      ['Base URL env', 'LOCAL_MODEL_URL'],
      ['Domain allowlist', 'localhost, model.internal'],
    ].forEach(([label, value]) => expect(agentForm.getByLabelText(label)).toHaveValue(value))
    expect(agentForm.getByLabelText('Model name')).toHaveValue('qwen3-coder-local')
    expect(agentForm.queryByText('Permissions and tools')).not.toBeInTheDocument()
    expect(agentForm.queryByLabelText('Permission mode')).not.toBeInTheDocument()
    expect(agentForm.queryByLabelText('Allowed tools')).not.toBeInTheDocument()
    expect(agentForm.queryByLabelText('Disallowed tools')).not.toBeInTheDocument()
    expect(agentForm.getByText('Network access')).toBeInTheDocument()
    expect(agentForm.getByRole('checkbox', { name: 'Enable network access' })).toBeChecked()
    fireEvent.click(agentForm.getByRole('checkbox', { name: 'Enable network access' }))
    expect(agentForm.queryByLabelText('Domain allowlist')).not.toBeInTheDocument()
    fireEvent.click(agentForm.getByRole('checkbox', { name: 'Enable network access' }))
    expect(agentForm.getByLabelText('Domain allowlist')).toHaveValue('*')
    expect(screen.queryByText('Config check')).not.toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Adapter init' })).not.toBeInTheDocument()
    expect(screen.queryByText(/harbor adapter/)).not.toBeInTheDocument()
    expect(agentForm.queryByText('Capability config')).not.toBeInTheDocument()
    expect(agentForm.getByText('Skills sources')).toBeInTheDocument()
    expect(agentForm.getByText('MCP Servers')).toBeInTheDocument()
    expect(agentForm.getByText('Enter one or more skill sources: a single skill directory with SKILL.md, or a folder containing multiple skill directories.')).toBeInTheDocument()
    expect(agentForm.getByLabelText('skills')).toHaveValue('~/.ornnlab/skills/repair')
    expect(agentForm.getByRole('button', { name: 'Choose folder' })).toBeInTheDocument()
    expect(agentForm.getByText('Manage MCP templates on the Agent. OrnnLab expands compose sidecars into Harbor task environment and registers the generated connection in task.toml.')).toBeInTheDocument()
    expect(agentForm.getByRole('checkbox', { name: 'Enabled local-repair-tools' })).toBeChecked()
    expect(agentForm.getByLabelText('Name')).toHaveValue('local-repair-tools')
    expect(agentForm.getByLabelText('Deployment')).toHaveValue('stdio')
    expect(agentForm.getByLabelText('Transport')).toHaveValue('stdio')
    expect(agentForm.getByLabelText('Command')).toHaveValue('uvx')
    expect(agentForm.getByLabelText('Args')).toHaveValue('local-repair-mcp')
    expect(agentForm.getAllByLabelText('Env key').map((input) => (input as HTMLInputElement).value)).toContain('LOCAL_MODEL_URL')
    expect(screen.getByText('Advanced agent params')).toBeInTheDocument()
    const envKeyCount = agentForm.getAllByLabelText('Env key').length
    fireEvent.click(agentForm.getByRole('button', { name: 'Add agent env' }))
    expect(agentForm.getAllByLabelText('Env key')).toHaveLength(envKeyCount + 1)
    const newEnvKeys = agentForm.getAllByLabelText('Env key')
    fireEvent.change(newEnvKeys[envKeyCount], { target: { value: 'LOCAL_TIMEOUT' } })
    expect(newEnvKeys[envKeyCount]).toHaveValue('LOCAL_TIMEOUT')

    fireEvent.click(screen.getByRole('button', { name: 'Close detail drawer' }))
    fireEvent.click(within(customRow as HTMLElement).getByRole('button', { name: 'Delete' }))
    fireEvent.click(screen.getByRole('button', { name: 'Confirm delete' }))
    expect(screen.queryByText('Local repair agent')).not.toBeInTheDocument()

    fireEvent.click(screen.getByRole('link', { name: 'Leaderboard' }))
    expect(screen.getByRole('heading', { name: 'Leaderboard' })).toBeInTheDocument()
    expect(screen.queryByText('Submission')).not.toBeInTheDocument()
    expect(screen.getByLabelText('Agent filter')).toBeInTheDocument()
    expect(screen.getByLabelText('Select dataset')).toHaveTextContent('terminal-bench@2.0')
    expect(screen.getByRole('columnheader', { name: 'Agent Name' })).toBeInTheDocument()
    expect(screen.getByRole('columnheader', { name: 'Harness' })).toBeInTheDocument()
    expect(screen.getByRole('columnheader', { name: 'Tokens (M)' })).toBeInTheDocument()
    expect(screen.getByText('0.0184M')).toBeInTheDocument()
    expect(screen.queryByRole('button', { name: 'Submit' })).not.toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'job_91a7' }))
    expect(screen.getByRole('dialog', { name: 'Selected job' })).toBeInTheDocument()
    expect(screen.getByText('Job trials')).toBeInTheDocument()
    fireEvent.click(screen.getByRole('button', { name: 'Close detail drawer' }))

    const firstRankRow = screen.getByText('job_91a7').closest('tr')
    expect(firstRankRow).not.toBeNull()
    fireEvent.click(within(firstRankRow as HTMLElement).getByRole('button', { name: 'Remove' }))
    expect(screen.queryByText('job_91a7')).not.toBeInTheDocument()
    const nextRankRow = screen.getByText('job_64f2').closest('tr')
    expect(nextRankRow).not.toBeNull()
    expect(within(nextRankRow as HTMLElement).getByText('#1')).toBeInTheDocument()

    fireEvent.change(screen.getByLabelText('Search datasets'), { target: { value: 'swe' } })
    fireEvent.click(screen.getByLabelText('Select dataset'))
    fireEvent.click(screen.getByRole('option', { name: 'swe-bench-lite@2026.06' }))
    expect(screen.getByText('job_74c1')).toBeInTheDocument()
    expect(screen.queryByLabelText('Search leaderboard')).not.toBeInTheDocument()
  })
})
