import type { Meta, StoryObj } from '@storybook/react-vite'
import { useState } from 'react'
import { getTranslator } from '../../i18n'
import { initialDraft } from '../../mocks/demo'
import { datasetRows, environmentRows, taskRows } from '../../mocks/demoCatalog'
import { RunBuilder } from './RunBuilder'

function RunBuilderFixture({ initial = initialDraft }: { initial?: typeof initialDraft }) {
  const [draft, setDraft] = useState(initial)

  return (
    <main className="workspace single-page">
      <div className="content-column">
        <RunBuilder
          datasets={datasetRows}
          draft={draft}
          environments={environmentRows}
          taskRows={taskRows}
          t={getTranslator('en')}
          onDraft={setDraft}
          onCancel={() => undefined}
          onLaunch={() => undefined}
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
