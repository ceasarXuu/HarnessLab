import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, fn, userEvent, waitFor, within } from 'storybook/test'
import { useState } from 'react'
import { getTranslator } from '../../i18n'
import { agentRows, datasetRows, datasetTaskRows, environmentRows } from '../../mocks/demoCatalog'
import { AgentDetail } from './AgentDetail'
import { DatasetDetail } from './DatasetDetail'
import { EnvironmentProfileEditor } from './EnvironmentProfileEditor'

const t = getTranslator('en')
const saveAgent = fn(async () => true)

const meta = {
  title: 'Components/DrawerContent',
  parameters: { layout: 'fullscreen' },
} satisfies Meta

export default meta
type Story = StoryObj<typeof meta>

export const BuiltInAgent: Story = {
  render: () => (
    <main className="workspace single-page">
      <AgentDetail agent={agentRows[0]} t={t} onSave={saveAgent} />
    </main>
  ),
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    saveAgent.mockClear()
    await expect(canvas.getByText('Claude Code default')).toBeVisible()
    await expect(canvas.queryByRole('button', { name: 'Save' })).not.toBeInTheDocument()
    await expect(canvas.getByLabelText('Agent Name')).toBeEnabled()
    const modelInputs = canvas.getAllByLabelText('Model name')
    await expect(modelInputs).toHaveLength(2)
    await expect(modelInputs[0]).toHaveValue('claude-haiku-4-5')
    await expect(modelInputs[1]).toHaveValue('claude-sonnet-4-5')
    await userEvent.click(canvas.getByRole('button', { name: 'Add' }))
    await expect(canvas.getAllByLabelText('Model name')).toHaveLength(3)
    await expect(canvas.getByRole('tab', { name: 'Basic' })).toBeVisible()
    await expect(canvas.getByRole('tab', { name: 'Skills' })).toBeVisible()
    await expect(canvas.getByRole('tab', { name: 'MCPs' })).toBeVisible()
    await expect(canvas.getByRole('tab', { name: 'Advanced' })).toBeVisible()
    const resourceType = canvas.getByText('Harbor built-in Harness').closest('.metric')
    await expect(resourceType).toHaveClass('metric--plain')
    await expect(resourceType).not.toHaveClass('metric--card')
    await userEvent.clear(canvas.getByLabelText('Agent Name'))
    await userEvent.type(canvas.getByLabelText('Agent Name'), 'Autosaved Agent')
    await waitFor(() => expect(saveAgent).toHaveBeenCalledTimes(1))
    await expect(saveAgent).toHaveBeenCalledWith(expect.objectContaining({ agentName: 'Autosaved Agent' }))
  },
}

export const CustomAgent: Story = {
  render: () => (
    <main className="workspace single-page">
      <AgentDetail agent={agentRows.find((row) => row.type === 'custom') ?? agentRows[0]} t={t} onSave={async () => true} />
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
          taskSearch=""
          tasks={datasetTaskRows.filter((row) => row.datasetRef === dataset.ref)}
          t={t}
          onCancelDownload={() => undefined}
          onDelete={() => undefined}
          onExpandedTaskName={() => undefined}
          onMove={() => undefined}
          onRelocate={() => undefined}
          onRemoveRegistration={() => undefined}
          onStartDownload={() => undefined}
          onSync={() => undefined}
          onTaskSearch={() => undefined}
          onRunTask={() => undefined}
        />
      </main>
    )
  },
}

export const ExternalDataset: Story = {
  render: () => {
    const dataset = {
      ...datasetRows[2],
      downloadPath: '/Users/demo/datasets/hello-world',
      size: '18.0 MB',
      storageKind: 'external' as const,
    }
    return (
      <main className="workspace single-page">
        <DatasetDetail
          downloadState={{ path: dataset.downloadPath, size: dataset.size, status: 'downloaded' }}
          expandedTaskName={null}
          isRegistryDataset={false}
          selected={dataset}
          taskSearch=""
          tasks={[]}
          t={t}
          onCancelDownload={() => undefined}
          onDelete={() => undefined}
          onExpandedTaskName={() => undefined}
          onMove={() => undefined}
          onRelocate={() => undefined}
          onRemoveRegistration={() => undefined}
          onStartDownload={() => undefined}
          onSync={() => undefined}
          onTaskSearch={() => undefined}
          onRunTask={() => undefined}
        />
      </main>
    )
  },
}

export const DatasetPathUnavailable: Story = {
  render: () => {
    const dataset = {
      ...datasetRows[0],
      downloadPath: '/Volumes/archive/terminal-bench@2.0',
      storageKind: 'managed' as const,
    }
    return (
      <main className="workspace single-page">
        <DatasetDetail
          downloadState={{ path: dataset.downloadPath, status: 'path-unavailable' }}
          expandedTaskName={null}
          isRegistryDataset
          selected={dataset}
          taskSearch=""
          tasks={[]}
          t={t}
          onCancelDownload={() => undefined}
          onDelete={() => undefined}
          onExpandedTaskName={() => undefined}
          onMove={() => undefined}
          onRelocate={() => undefined}
          onRemoveRegistration={() => undefined}
          onStartDownload={() => undefined}
          onSync={() => undefined}
          onTaskSearch={() => undefined}
          onRunTask={() => undefined}
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
        <EnvironmentProfileEditor value={environment} t={t} onChange={setEnvironment} />
      </section>
    </main>
  )
}

export const EnvironmentEditor: StoryObj<typeof EnvironmentEditorFixture> = {
  render: () => <EnvironmentEditorFixture />,
}
