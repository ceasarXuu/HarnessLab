import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, within } from 'storybook/test'
import { useState } from 'react'
import { getTranslator } from '../../i18n'
import { CustomSelect } from './CustomSelect'
import { Field, Toggle } from './RunBuilderChrome'

function ControlsFixture() {
  const [dataset, setDataset] = useState('terminal-bench@2.0')
  const [enabled, setEnabled] = useState(true)
  const t = getTranslator('en')

  return (
    <main className="workspace single-page">
      <section className="surface rail-card">
        <div className="run-grid">
          <Field label={t('dataset')}>
            <CustomSelect
              ariaLabel={t('dataset')}
              value={dataset}
              options={[
                { label: 'terminal-bench@2.0', value: 'terminal-bench@2.0' },
                { label: 'swe-bench-lite@2026.06', value: 'swe-bench-lite@2026.06' },
                { label: 'harbor/hello-world@latest', value: 'harbor/hello-world@latest' },
              ]}
              onChange={setDataset}
            />
          </Field>
          <Field label={t('includeInLeaderboard')}>
            <Toggle checked={enabled} onChange={setEnabled} />
          </Field>
        </div>
      </section>
      <section className="surface rail-card">
        <div className="drawer-task-toolbar">
          <label className="search-field drawer-search">
            <input aria-label="Search tasks" placeholder="Search tasks" />
          </label>
          <CustomSelect
            ariaLabel="Split"
            className="toolbar-select"
            value="all"
            options={[
              { label: 'All splits', value: 'all' },
              { label: 'test', value: 'test' },
              { label: 'nightly', value: 'nightly' },
            ]}
            onChange={() => undefined}
          />
        </div>
      </section>
    </main>
  )
}

const meta = {
  title: 'Components/Controls',
  component: ControlsFixture,
  parameters: { layout: 'fullscreen' },
} satisfies Meta<typeof ControlsFixture>

export default meta
type Story = StoryObj<typeof meta>

export const FormControls: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    const split = canvas.getByLabelText('Split')
    await expect(split).toHaveTextContent('All splits')
    const bounds = split.getBoundingClientRect()
    await expect(bounds.width).toBeLessThanOrEqual(180)
  },
}
