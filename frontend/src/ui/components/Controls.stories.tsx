import type { Meta, StoryObj } from '@storybook/react-vite'
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

export const FormControls: Story = {}
