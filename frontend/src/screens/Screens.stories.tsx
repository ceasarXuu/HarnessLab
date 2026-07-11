import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { useState } from 'react'
import { getTranslator } from '../i18n'
import { createMockWebUiClient } from '../api/mockClient'
import { jobs } from '../mocks/demo'
import { agentRows, datasetRows, environmentRows } from '../mocks/demoCatalog'
import { leaderboardRows, systemRows } from '../mocks/demoSystem'
import { AgentsPage } from './AgentsPage'
import { DatasetsPage } from './DatasetsPage'
import { EnvironmentsPage } from './EnvironmentsPage'
import { JobsPage } from './JobsPage'
import { LeaderboardPage } from './LeaderboardPage'
import { NewAgentPage } from './NewAgentPage'
import { SystemPage } from './SystemPage'

const t = getTranslator('en')
const tZh = getTranslator('zh')
const client = createMockWebUiClient()
const leaderboardDatasets = [...new Map(leaderboardRows.map((row) => {
  const [name, version = ''] = row.dataset.split('@')
  return [row.dataset, { name, ref: row.dataset, version }]
})).values()]

const meta = {
  title: 'Screens/Harbor WebUI',
  parameters: { layout: 'fullscreen' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

function DatasetsFixture() {
  const [search, setSearch] = useState('')
  return <DatasetsPage client={client} rows={datasetRows} search={search} t={t} onRefresh={async () => undefined} onSearch={setSearch} />
}

function LeaderboardFixture() {
  const [dataset, setDataset] = useState('terminal-bench@2.0')
  const rows = leaderboardRows.filter((row) => row.dataset === dataset)
  return (
    <LeaderboardPage
      dataset={dataset}
      datasetSearch=""
      leaderboardDatasets={leaderboardDatasets}
      client={client}
      jobs={jobs}
      rows={rows}
      t={t}
      onDataset={setDataset}
      onDatasetSearch={() => undefined}
      onJobAction={() => undefined}
      onLeaderboardChange={() => undefined}
      onRemove={() => undefined}
    />
  )
}

export const Jobs: Story = {
  render: () => (
    <JobsPage
      client={client}
      jobs={jobs}
      open={false}
      search=""
      selected={jobs[0]}
      t={t}
      onClose={() => undefined}
      onJobAction={() => undefined}
      onLeaderboardChange={() => undefined}
      onNewJob={() => undefined}
      onSearch={() => undefined}
      onSelect={() => undefined}
    />
  ),
}

export const JobsEmpty: Story = {
  render: () => (
    <JobsPage
      client={client}
      jobs={[]}
      open={false}
      search=""
      selected={null}
      t={t}
      onClose={() => undefined}
      onJobAction={() => undefined}
      onLeaderboardChange={() => undefined}
      onNewJob={() => undefined}
      onSearch={() => undefined}
      onSelect={() => undefined}
    />
  ),
}

export const JobsLight: Story = {
  globals: { theme: 'light' },
  render: () => (
    <JobsPage
      client={createMockWebUiClient()}
      jobs={jobs}
      open={false}
      search=""
      selected={jobs[0]}
      t={t}
      onClose={() => undefined}
      onJobAction={() => undefined}
      onLeaderboardChange={() => undefined}
      onNewJob={() => undefined}
      onSearch={() => undefined}
      onSelect={() => undefined}
    />
  ),
}

export const JobsChinese: Story = {
  globals: { locale: 'zh' },
  render: () => (
    <JobsPage
      client={createMockWebUiClient()}
      jobs={jobs}
      open={false}
      search=""
      selected={jobs[0]}
      t={tZh}
      onClose={() => undefined}
      onJobAction={() => undefined}
      onLeaderboardChange={() => undefined}
      onNewJob={() => undefined}
      onSearch={() => undefined}
      onSelect={() => undefined}
    />
  ),
}

export const JobCancelConfirm: Story = {
  render: () => (
    <JobsPage
      client={createMockWebUiClient()}
      jobs={jobs}
      open
      search=""
      selected={jobs[0]}
      t={t}
      onClose={() => undefined}
      onJobAction={() => undefined}
      onLeaderboardChange={() => undefined}
      onNewJob={() => undefined}
      onSearch={() => undefined}
      onSelect={() => undefined}
    />
  ),
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByRole('button', { name: 'Cancel' }))
    await expect(canvas.getByRole('dialog', { name: 'Cancel Job' })).toBeVisible()
  },
}

export const JobOperationRunning: Story = {
  render: () => (
    <JobsPage
      client={client}
      jobs={jobs}
      open
      search=""
      selected={jobs[0]}
      t={t}
      onClose={() => undefined}
      onJobAction={() => undefined}
      onLeaderboardChange={() => undefined}
      onNewJob={() => undefined}
      onSearch={() => undefined}
      onSelect={() => undefined}
    />
  ),
}

export const Datasets: Story = {
  render: () => <DatasetsFixture />,
}

export const DatasetsEmpty: Story = {
  render: () => <DatasetsPage client={client} rows={[]} search="" t={t} onRefresh={async () => undefined} onSearch={() => undefined} />,
}

export const DatasetDownloading: Story = {
  render: () => <DatasetsFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getAllByRole('button', { name: 'Download' })[0])
    await expect(canvas.getByText('0%')).toBeVisible()
    await expect(canvas.getByRole('button', { name: 'Cancel download' })).toBeVisible()
  },
}

export const DatasetDeleteConfirm: Story = {
  render: () => <DatasetsPage client={createMockWebUiClient()} rows={datasetRows} search="" t={t} onRefresh={async () => undefined} onSearch={() => undefined} />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getAllByRole('button', { name: 'Delete' })[0])
    await expect(canvas.getByRole('dialog', { name: 'Delete local dataset' })).toBeVisible()
  },
}

export const DatasetDrawer: Story = {
  render: () => <DatasetsFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByText('terminal-bench'))
    await expect(canvas.getByText('Dataset tasks')).toBeVisible()
    await expect(canvas.queryByText('Manifest tools')).not.toBeInTheDocument()
    await expect(canvas.queryByLabelText('Split')).not.toBeInTheDocument()
    await userEvent.type(canvas.getByLabelText('Search tasks'), 'sqlite')
    await expect(canvas.getByText('sqlite-log-repair')).toBeVisible()
    await expect(canvas.queryByText('apt-setup')).not.toBeInTheDocument()
  },
}

export const DatasetEmptyState: Story = {
  render: () => <DatasetsFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByText('swebench-verified'))
    await expect(canvas.getByText('No Tasks are available for this Dataset.')).toBeVisible()
  },
}

export const Agents: Story = {
  render: () => <AgentsFixture />,
}

export const AgentsEmpty: Story = {
  render: () => <AgentsPage client={client} rows={[]} t={t} onNewAgent={() => undefined} onRefresh={async () => undefined} />,
}

export const AgentDeleteConfirm: Story = {
  render: () => <AgentsFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    const customRow = canvas.getByText('Local repair agent').closest('tr')
    if (!customRow) throw new Error('Custom agent row not found')
    await userEvent.click(within(customRow as HTMLElement).getByRole('button', { name: 'Delete' }))
    await expect(canvas.getByRole('dialog', { name: 'Delete custom agent' })).toBeVisible()
  },
}

export const AgentDrawer: Story = {
  render: () => <AgentsFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByText('Claude Code default'))
    await expect(canvas.getByRole('dialog', { name: 'Selected agent' })).toBeVisible()
    await expect(canvas.getByLabelText('Agent Name')).toHaveValue('Claude Code default')
    await expect(canvas.getByLabelText('Harness')).toHaveValue('claude-code')
    await expect(canvas.getByLabelText('Type')).toHaveValue('built-in')
    await expect(canvas.queryByLabelText('Custom import path')).not.toBeInTheDocument()
    await expect(canvas.getByLabelText('Agent Name')).toHaveAttribute('readonly')
    await expect(canvas.getByLabelText('Harness')).toBeDisabled()
    await expect(canvas.queryByText('Model settings')).not.toBeInTheDocument()
    await expect(canvas.queryByText('Credentials and parameters')).not.toBeInTheDocument()
    await expect(canvas.queryByLabelText('Permission mode')).not.toBeInTheDocument()
    await expect(canvas.queryByLabelText('Allowed tools')).not.toBeInTheDocument()
    await expect(canvas.queryByLabelText('Disallowed tools')).not.toBeInTheDocument()
    await expect(canvas.queryByText('Network access')).not.toBeInTheDocument()
    await expect(canvas.queryByRole('checkbox', { name: 'Enable network access' })).not.toBeInTheDocument()
    await expect(canvas.queryByText('Capability config')).not.toBeInTheDocument()
    await expect(canvas.queryByText('Advanced agent params')).not.toBeInTheDocument()
    await expect(canvas.queryByRole('tab', { name: 'Skills' })).not.toBeInTheDocument()
    await expect(canvas.queryByRole('tab', { name: 'MCPs' })).not.toBeInTheDocument()
    await expect(canvas.queryByRole('tab', { name: 'Advanced' })).not.toBeInTheDocument()
  },
}

function AgentsFixture() {
  const [view, setView] = useState<'list' | 'new'>('list')
  if (view === 'new') {
    return (
      <NewAgentPage
        client={client}
        rows={agentRows}
        t={t}
        onAgents={() => setView('list')}
        onRefresh={async () => undefined}
      />
    )
  }
  return <AgentsPage client={client} rows={agentRows} t={t} onNewAgent={() => setView('new')} onRefresh={async () => undefined} />
}

export const NewAgent: Story = {
  render: () => <NewAgentPage client={client} rows={agentRows} t={t} onAgents={() => undefined} onRefresh={async () => undefined} />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await expect(canvas.getByRole('heading', { name: 'New Agent' })).toBeVisible()
    await expect(canvas.getByLabelText('Agent Name')).toHaveValue('Custom Agent')
    await expect(canvas.getByLabelText('Harness')).toHaveValue('custom-harness')
    await expect(canvas.getByRole('tab', { name: 'Basic' })).toHaveAttribute('aria-selected', 'true')
    await userEvent.click(canvas.getByRole('tab', { name: 'Skills' }))
    await expect(canvas.getByText('Skills sources')).toBeVisible()
    await userEvent.click(canvas.getByRole('tab', { name: 'MCPs' }))
    await expect(canvas.getByText('MCP Servers')).toBeVisible()
    await userEvent.click(canvas.getByRole('tab', { name: 'Advanced' }))
    await expect(canvas.getByText('Advanced agent params')).toBeVisible()
  },
}

function EnvironmentsFixture() {
  const [view, setView] = useState<'list' | 'new' | 'copy'>('list')
  const [environmentId, setEnvironmentId] = useState<string | undefined>()
  const onView = (nextView: 'list' | 'new' | 'copy', nextEnvironmentId?: string) => {
    setView(nextView)
    setEnvironmentId(nextEnvironmentId)
  }
  return (
    <EnvironmentsPage
      client={client}
      environmentId={environmentId}
      rows={environmentRows}
      t={t}
      view={view}
      onRefresh={async () => undefined}
      onView={onView}
    />
  )
}

export const Environments: Story = {
  render: () => <EnvironmentsFixture />,
}

export const EnvironmentsEmpty: Story = {
  render: () => (
    <EnvironmentsPage
      client={client}
      rows={[]}
      t={t}
      view="list"
      onRefresh={async () => undefined}
      onView={() => undefined}
    />
  ),
}

export const EnvironmentDrawer: Story = {
  render: () => <EnvironmentsFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByText('Docker GPU'))
    await expect(canvas.getByRole('dialog', { name: 'Selected environment' })).toBeVisible()
    await expect(canvas.getByRole('tab', { name: 'Basic' })).toHaveAttribute('aria-selected', 'true')
    await expect(canvas.queryByRole('tab', { name: 'Environment' })).not.toBeInTheDocument()
    await expect(canvas.queryByRole('heading', { name: 'Basic' })).not.toBeInTheDocument()
    await expect(canvas.queryByRole('heading', { name: 'Environment' })).not.toBeInTheDocument()
    await expect(canvas.getByLabelText('Environment Name')).toHaveValue('Docker GPU')
    await expect(canvas.getByLabelText('Import path')).toHaveValue('')
    await expect(canvas.getByLabelText('Docker image name / registry URL')).toHaveValue('nvidia/cuda:12.4-runtime')
    await expect(canvas.getByLabelText('OS')).toHaveTextContent('linux')
    await expect(canvas.queryByRole('checkbox', { name: 'Healthcheck' })).not.toBeInTheDocument()
    await expect(canvas.queryByLabelText('Command')).not.toBeInTheDocument()
    await expect(canvas.queryByLabelText('Working directory')).not.toBeInTheDocument()
    await userEvent.click(canvas.getByRole('tab', { name: 'Network' }))
    await expect(canvas.getByRole('checkbox', { name: 'Network access' })).toBeChecked()
    await expect(canvas.getByLabelText('Allowed hosts 1')).toHaveValue('pypi.org')
    await expect(canvas.getByLabelText('Allowed hosts 2')).toHaveValue('github.com')
    await expect(canvas.getByLabelText('Allowed hosts 3')).toHaveValue('huggingface.co')
    await userEvent.click(canvas.getByRole('tab', { name: 'Advanced' }))
    await expect(canvas.getByLabelText('CPU policy')).toHaveTextContent('limit')
    await expect(canvas.queryByText('Override TPU')).not.toBeInTheDocument()
    await expect(canvas.getByLabelText('Extra Docker Compose 1')).toHaveValue('compose.gpu.yml')
    await expect(canvas.getByLabelText('Extra allowed hosts')).toHaveValue('model.internal')
    await expect(canvas.getByRole('checkbox', { name: 'Healthcheck' })).toBeChecked()
    await expect(canvas.getByLabelText('Command')).toHaveValue('python --version')
    await expect(canvas.getByLabelText('Retries')).toHaveValue(3)
    await expect(canvas.getByLabelText('Working directory')).toHaveValue('/workspace')
    await expect(canvas.getByText('Backend params')).toBeVisible()
  },
}

export const EnvironmentDeleteConfirm: Story = {
  render: () => (
    <EnvironmentsPage
      client={createMockWebUiClient()}
      rows={environmentRows}
      t={t}
      view="list"
      onRefresh={async () => undefined}
      onView={() => undefined}
    />
  ),
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    const customRow = canvas.getByText('Docker GPU').closest('tr')
    if (!customRow) throw new Error('Custom environment row not found')
    await userEvent.click(within(customRow as HTMLElement).getByRole('button', { name: 'Delete' }))
    await expect(canvas.getByRole('dialog', { name: 'Delete custom environment' })).toBeVisible()
  },
}

export const Leaderboard: Story = {
  render: () => <LeaderboardFixture />,
}

export const LeaderboardEmpty: Story = {
  render: () => (
    <LeaderboardPage
      client={client}
      dataset="terminal-bench@2.0"
      datasetSearch=""
      leaderboardDatasets={leaderboardDatasets}
      jobs={jobs}
      rows={[]}
      t={t}
      onDataset={() => undefined}
      onDatasetSearch={() => undefined}
      onJobAction={() => undefined}
      onLeaderboardChange={() => undefined}
      onRemove={() => undefined}
    />
  ),
}

export const System: Story = {
  render: () => <SystemPage client={client} rows={systemRows} t={t} onRefresh={async () => undefined} />,
}

export const SystemEmpty: Story = {
  render: () => <SystemPage client={client} rows={[]} t={t} onRefresh={async () => undefined} />,
}

export const SystemDestructiveConfirm: Story = {
  render: () => <SystemPage client={client} rows={systemRows} t={t} onRefresh={async () => undefined} />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getAllByRole('button', { name: 'Clean cache' })[0])
    await expect(canvas.getByRole('dialog', { name: 'Clean Docker cache' })).toBeVisible()
  },
}
