import type { Meta, StoryObj } from '@storybook/react-vite'
import { useState } from 'react'
import { getTranslator } from '../../i18n'
import { agentRows, datasetRows, environmentRows, taskRows } from '../../mocks/demoCatalog'
import { AgentDetail } from './AgentDetail'
import { DatasetDetail } from './DatasetDetail'
import { EnvironmentProfileEditor } from './EnvironmentProfileEditor'

const t = getTranslator('en')

const meta = {
  title: 'Components/DrawerContent',
  parameters: { layout: 'fullscreen' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const BuiltInAgent: Story = {
  render: () => (
    <main className="workspace single-page">
      <AgentDetail agent={agentRows[0]} t={t} />
    </main>
  ),
}

export const CustomAgent: Story = {
  render: () => (
    <main className="workspace single-page">
      <AgentDetail agent={agentRows.find((row) => row.type === 'custom') ?? agentRows[0]} t={t} />
    </main>
  ),
}

export const Dataset: Story = {
  render: () => {
    const dataset = datasetRows[0]
    return (
      <main className="workspace single-page">
        <DatasetDetail
          downloadState={{ path: dataset.downloadPath ?? '', size: dataset.size ?? '', status: 'downloaded' }}
          expandedTaskName={null}
          isRegistryDataset
          selected={dataset}
          splitOptions={[{ label: 'All splits', value: 'all' }, { label: 'test', value: 'test' }]}
          taskSearch=""
          taskSplit="all"
          tasks={taskRows.filter((row) => row.dataset === dataset.name)}
          t={t}
          onCancelDownload={() => undefined}
          onDelete={() => undefined}
          onExpandedTaskName={() => undefined}
          onStartDownload={() => undefined}
          onTaskSearch={() => undefined}
          onTaskSplit={() => undefined}
        />
      </main>
    )
  },
}

function EnvironmentEditorFixture() {
  const [environment, setEnvironment] = useState(environmentRows[0])

  return (
    <main className="workspace single-page">
      <section className="surface rail-card">
        <EnvironmentProfileEditor value={environment} onChange={setEnvironment} />
      </section>
    </main>
  )
}

export const EnvironmentEditor: StoryObj<typeof EnvironmentEditorFixture> = {
  render: () => <EnvironmentEditorFixture />,
}
