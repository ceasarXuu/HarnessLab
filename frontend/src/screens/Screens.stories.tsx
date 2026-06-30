import type { Meta, StoryObj } from '@storybook/react-vite'
import { userEvent, within } from 'storybook/test'
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
  },
}

export const Agents: Story = {
  render: () => <AgentsPage rows={agentRows} t={t} />,
}

export const Environments: Story = {
  render: () => <EnvironmentsPage rows={environmentRows} t={t} />,
}

export const Leaderboard: Story = {
  render: () => <LeaderboardFixture />,
}

export const System: Story = {
  render: () => <SystemPage rows={systemRows} t={t} />,
}
