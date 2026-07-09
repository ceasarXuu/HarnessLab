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

export const NewJobFlow: Story = {}

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
