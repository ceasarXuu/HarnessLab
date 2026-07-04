import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { useState } from 'react'
import { getTranslator } from '../i18n'
import { events, jobs, trialRows } from '../mocks/demo'
import { agentRows, datasetRows, environmentRows, taskRows } from '../mocks/demoCatalog'
import { leaderboardRows, systemRows } from '../mocks/demoSystem'
import { AgentsPage } from './AgentsPage'
import { DatasetsPage } from './DatasetsPage'
import { EnvironmentsPage } from './EnvironmentsPage'
import { JobsPage } from './JobsPage'
import { LeaderboardPage } from './LeaderboardPage'
import { SystemPage } from './SystemPage'

const t = getTranslator('en')

const meta = {
  title: 'Screens/Harbor WebUI',
  parameters: { layout: 'fullscreen' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

function DatasetsFixture() {
  const [search, setSearch] = useState('')
  return <DatasetsPage rows={datasetRows} search={search} taskRows={taskRows} t={t} onSearch={setSearch} />
}

function LeaderboardFixture() {
  const [dataset, setDataset] = useState('terminal-bench@2.0')
  const rows = leaderboardRows.filter((row) => row.dataset === dataset)
  return (
    <LeaderboardPage
      dataset={dataset}
      datasetSearch=""
      datasets={datasetRows}
      events={events}
      jobs={jobs}
      rows={rows}
      t={t}
      trialRows={trialRows}
      onDataset={setDataset}
      onDatasetSearch={() => undefined}
      onLeaderboardChange={() => undefined}
      onRemove={() => undefined}
    />
  )
}

export const Jobs: Story = {
  render: () => (
    <JobsPage
      events={events}
      jobs={jobs}
      open={false}
      search=""
      selected={jobs[0]}
      trialRows={trialRows}
      t={t}
      onClose={() => undefined}
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

export const DatasetDrawer: Story = {
  render: () => <DatasetsFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByText('terminal-bench'))
    await expect(canvas.getByText('Dataset tasks')).toBeVisible()
    await expect(canvas.queryByText('Manifest tools')).not.toBeInTheDocument()
    await expect(canvas.getByLabelText('Split')).toHaveTextContent('All splits')
    await userEvent.type(canvas.getByLabelText('Search tasks'), 'sqlite')
    await expect(canvas.getByText('sqlite-log-repair')).toBeVisible()
    await expect(canvas.queryByText('apt-setup')).not.toBeInTheDocument()
  },
}

export const Agents: Story = {
  render: () => <AgentsPage rows={agentRows} t={t} />,
}

export const AgentDrawer: Story = {
  render: () => <AgentsPage rows={agentRows} t={t} />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByText('Claude Code default'))
    await expect(canvas.getByRole('dialog', { name: 'Selected agent' })).toBeVisible()
    await expect(canvas.getByLabelText('Agent Name')).toHaveValue('Claude Code default')
    await expect(canvas.getByLabelText('Harness')).toHaveValue('claude-code')
    await expect(canvas.getByLabelText('Type')).toHaveValue('built-in')
    await expect(canvas.queryByLabelText('Custom import path')).not.toBeInTheDocument()
    await expect(canvas.getByLabelText('Reasoning effort')).toHaveValue('high')
    await expect(canvas.getByLabelText('API key env')).toHaveValue('ANTHROPIC_API_KEY')
    await expect(canvas.getByLabelText('Allowed tools')).toHaveValue('Read, Glob, Grep, Bash')
    await expect(canvas.getByText('Advanced agent params')).toBeVisible()
  },
}

function EnvironmentsFixture() {
  const [rows, setRows] = useState(environmentRows)
  const [view, setView] = useState<'list' | 'new' | 'copy'>('list')
  const [environmentId, setEnvironmentId] = useState<string | undefined>()
  const onView = (nextView: 'list' | 'new' | 'copy', nextEnvironmentId?: string) => {
    setView(nextView)
    setEnvironmentId(nextEnvironmentId)
  }
  return (
    <EnvironmentsPage
      environmentId={environmentId}
      rows={rows}
      t={t}
      view={view}
      onRowsChange={setRows}
      onView={onView}
    />
  )
}

export const Environments: Story = {
  render: () => <EnvironmentsFixture />,
}

export const Leaderboard: Story = {
  render: () => <LeaderboardFixture />,
}

export const System: Story = {
  render: () => <SystemPage rows={systemRows} t={t} />,
}
