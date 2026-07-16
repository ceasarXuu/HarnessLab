import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { useState } from 'react'
import { getTranslator } from '../../i18n'
import { initialDraft } from '../../mocks/demo'
import { agentRows, datasetRows, datasetTaskRows, environmentRows } from '../../mocks/demoCatalog'
import { RunBuilder } from './RunBuilder'
import { RunBuilderRuntimePanel } from './RunBuilderRuntimePanel'

function RunBuilderFixture({ initial = initialDraft }: { initial?: typeof initialDraft }) {
  const [draft, setDraft] = useState(initial)

  return (
    <main className="workspace single-page">
      <div className="content-column">
        <RunBuilder
          datasets={datasetRows}
          agents={agentRows}
          draft={draft}
          environments={environmentRows}
          taskRows={datasetTaskRows}
          t={getTranslator('en')}
          onDraft={setDraft}
          onCancel={() => undefined}
          onCopyJobConfig={() => undefined}
          onChooseDirectory={async () => ({ path: '/Users/demo/jobs' })}
          onLaunch={() => undefined}
          onReset={() => setDraft(initialDraft)}
        />
      </div>
    </main>
  )
}

const meta = {
  title: 'Components/RunBuilder',
  component: RunBuilderFixture,
  parameters: { layout: 'fullscreen' },
} satisfies Meta<typeof RunBuilderFixture>

export default meta
type Story = StoryObj<typeof meta>

export const NewJobFlow: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await expect(canvas.getByLabelText('Model')).toHaveTextContent('qwen3-coder-local')
    await userEvent.click(canvas.getByLabelText('Agent'))
    await userEvent.click(canvas.getByRole('option', { name: 'Claude Code default' }))
    await expect(canvas.getByLabelText('Model')).toHaveTextContent('claude-haiku-4-5')
    await userEvent.click(canvas.getByLabelText('Model'))
    await expect(canvas.getByRole('option', { name: 'claude-sonnet-4-5' })).toBeVisible()
  },
}

export const DatasetDownloadStatus: Story = {
  render: () => <RunBuilderFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByLabelText('Dataset'))
    await expect(canvas.getByRole('option', { name: 'terminal-bench@2.0 Downloaded' })).toBeVisible()
    await expect(canvas.getByRole('option', { name: 'swebench-verified@1.0 Not downloaded' })).toBeVisible()
    await expect(canvas.getAllByRole('option').map((option) => option.textContent)).toEqual([
      'terminal-bench@2.0Downloaded',
      'harbor/hello-world@latestDownloaded',
      'swebench-verified@1.0Not downloaded',
      'terminal-bench-nightly@nightlyNot downloaded',
    ])
  },
}

export const VerifierSkipLocksLeaderboard: Story = {
  render: () => <RunBuilderFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByRole('tab', { name: 'Verifier' }))
    await userEvent.click(canvas.getByLabelText('Verifier mode'))
    await userEvent.click(canvas.getByRole('option', { name: 'Skip verification' }))
    await userEvent.click(canvas.getByRole('tab', { name: 'Basic' }))
    await expect(canvas.getByLabelText('Include in leaderboard')).toBeDisabled()
    await expect(canvas.getByLabelText('Include in leaderboard')).toHaveTextContent('Disabled')
  },
}

export const TaskSearchBulkSelection: Story = {
  render: () => <RunBuilderFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByRole('tab', { name: 'Tasks' }))
    await userEvent.type(canvas.getByLabelText('Search tasks'), 'sqlite')
    await expect(canvas.getByText('sqlite-log-repair')).toBeVisible()
    await expect(canvas.queryByText('apt-setup')).not.toBeInTheDocument()
    await userEvent.click(canvas.getByRole('button', { name: 'Disable all' }))
    await expect(canvas.getByText('Selected tasks: 3 / 4')).toBeVisible()
  },
}

function RuntimePanelFixture() {
  const [draft, setDraft] = useState(initialDraft)

  return (
    <main className="workspace single-page">
      <section className="surface rail-card">
        <RunBuilderRuntimePanel draft={draft} t={getTranslator('en')} onDraft={setDraft} />
      </section>
    </main>
  )
}

export const RuntimePanel: StoryObj<typeof RuntimePanelFixture> = {
  render: () => <RuntimePanelFixture />,
}
