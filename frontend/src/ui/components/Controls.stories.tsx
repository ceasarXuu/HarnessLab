import type { Meta, StoryObj } from '@storybook/react-vite'
import { expect, userEvent, within } from 'storybook/test'
import { useState } from 'react'
import { getTranslator } from '../../i18n'
import { ConfirmDialog } from './ConfirmDialog'
import { CustomSelect } from './CustomSelect'
import { EditableStringList } from './EditableStringList'
import { FolderPathInput } from './FolderPathInput'
import { KeyValueControl } from './KeyValueControl'
import { Field, Toggle } from './RunBuilderChrome'
import { SwitchControl } from './SwitchControl'
import { Toast } from './Toast'
import { TpuSpecControl } from './TpuSpecControl'

function ControlsFixture() {
  const [dataset, setDataset] = useState('terminal-bench@2.0')
  const [envVars, setEnvVars] = useState('ANTHROPIC_API_KEY\nANTHROPIC_BASE_URL=https://api.anthropic.com')
  const [folder, setFolder] = useState('jobs/terminal-bench-smoke')
  const [enabled, setEnabled] = useState(true)
  const [tpu, setTpu] = useState('v6e=2x4')
  const [paths, setPaths] = useState(['compose.gpu.yml'])
  const [leaderboardDataset, setLeaderboardDataset] = useState('')
  const [leaderboardSearch, setLeaderboardSearch] = useState('')
  const t = getTranslator('en')
  const catalog = Array.from({ length: 40 }, (_, index) => {
    const value = `benchmark-${String(index + 1).padStart(2, '0')}@1.0`
    return { label: value, value }
  })
  const visibleCatalog = catalog.filter((option) => option.label.includes(leaderboardSearch))

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
                { label: 'swebench-verified@1.0', value: 'swebench-verified@1.0' },
                { label: 'harbor/hello-world@latest', value: 'harbor/hello-world@latest' },
              ]}
              onChange={setDataset}
            />
          </Field>
          <Field label={t('includeInLeaderboard')}>
            <Toggle checked={enabled} onChange={setEnabled} />
          </Field>
          <SwitchControl checked={enabled} label="Cache prompts" onChange={setEnabled} />
        </div>
      </section>
      <section className="surface rail-card">
        <CustomSelect
          ariaLabel="Select leaderboard dataset"
          className="toolbar-select"
          placeholder="Select dataset"
          searchable
          searchAriaLabel="Search leaderboard datasets"
          searchPlaceholder="Search datasets"
          searchValue={leaderboardSearch}
          value={leaderboardDataset}
          options={visibleCatalog}
          onChange={setLeaderboardDataset}
          onSearchChange={setLeaderboardSearch}
        />
      </section>
      <section className="surface rail-card">
        <div className="run-grid">
          <Field label="jobs_dir" wide>
            <FolderPathInput
              chooseLabel="Choose"
              label="Choose folder"
              value={folder}
              onChange={setFolder}
              onChoose={async () => ({ path: '/Users/demo/jobs' })}
            />
          </Field>
          <KeyValueControl
            allowInherited
            label="Environment variables"
            labels={{
              add: t('add'),
              delete: t('delete'),
              inherited: t('envSourceInherited'),
              key: t('envKey'),
              literal: t('envSourceLiteral'),
              source: t('envValueSource'),
              value: t('envValue'),
            }}
            value={envVars}
            onChange={setEnvVars}
          />
          <EditableStringList
            addLabel={t('add')}
            deleteLabel={t('delete')}
            itemAriaLabel={(_, index) => `Compose path ${index + 1}`}
            label="Compose paths"
            values={paths}
            onChange={setPaths}
          />
          <TpuSpecControl label="tpu" value={tpu} onChange={setTpu} />
        </div>
      </section>
      <section className="surface rail-card">
        <div className="drawer-task-toolbar">
          <label className="search-field drawer-search">
            <input aria-label="Search tasks" placeholder="Search tasks" />
          </label>
          <CustomSelect
            ariaLabel="Environment"
            className="toolbar-select"
            value="docker"
            options={[
              { label: 'docker', value: 'docker' },
              { label: 'daytona', value: 'daytona' },
              { label: 'e2b', value: 'e2b' },
            ]}
            onChange={() => undefined}
          />
        </div>
      </section>
    </main>
  )
}

function FeedbackFixture() {
  const [toastOpen, setToastOpen] = useState(true)

  return (
    <main className="workspace single-page">
      <ConfirmDialog
        cancelLabel="Cancel"
        confirmLabel="Confirm cleanup"
        impacts={['Deletes local Harbor cache directory ~/.cache/harbor.', 'Next dataset access may recreate files.']}
        title="Clean local cache"
        onCancel={() => undefined}
        onConfirm={() => undefined}
      />
      {toastOpen && (
        <Toast dismissLabel="Dismiss" message="OrnnLab is already up to date" remaining={3} onDismiss={() => setToastOpen(false)} />
      )}
    </main>
  )
}

function TransientListFixture() {
  const [models, setModels] = useState<string[]>([])
  const [environment, setEnvironment] = useState('none')
  return (
    <main className="workspace single-page">
      <section className="surface rail-card">
        <EditableStringList
          addLabel="Add"
          deleteLabel="Delete"
          label="Models"
          values={models}
          onChange={setModels}
        />
        <KeyValueControl
          label="Environment"
          labels={{ add: 'Add variable', delete: 'Delete', key: 'Key', value: 'Value' }}
          value={environment}
          onChange={setEnvironment}
        />
        <button type="button">Outside</button>
      </section>
    </main>
  )
}

function KnownEnvironmentVariablesFixture() {
  const [environment, setEnvironment] = useState('none')
  return (
    <main className="workspace single-page">
      <section className="surface rail-card">
        <KeyValueControl
          allowInherited
          compact
          keyOptions={['OPENAI_API_KEY', 'OPENAI_BASE_URL']}
          label="Agent environment"
          labels={{
            add: 'Add variable', customKey: 'Custom variable', delete: 'Delete',
            inherited: 'Inherit system variable', key: 'Variable name',
            literal: 'Fixed value', searchKeys: 'Search variables', source: 'Value source',
            value: 'Value',
          }}
          value={environment}
          onChange={setEnvironment}
        />
        <output data-testid="environment-output">{environment}</output>
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
    const environment = canvas.getByLabelText('Environment')
    await expect(environment).toHaveTextContent('docker')
    const bounds = environment.getBoundingClientRect()
    await expect(bounds.width).toBeLessThanOrEqual(180)
    const cachePrompts = canvas.getByRole('switch', { name: 'Cache prompts' })
    await expect(cachePrompts).toBeChecked()
    await userEvent.click(cachePrompts)
    await expect(cachePrompts).not.toBeChecked()
    await expect(canvas.getByLabelText('Allowed hosts 1')).toHaveValue('*')
  },
}

export const SearchableSelect: Story = {
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByLabelText('Select leaderboard dataset'))
    await expect(canvas.getByLabelText('Search leaderboard datasets')).toBeVisible()
    await expect(canvas.getAllByRole('option')).toHaveLength(40)
    await userEvent.type(canvas.getByLabelText('Search leaderboard datasets'), '20')
    await expect(canvas.getByRole('option', { name: 'benchmark-20@1.0' })).toBeVisible()
  },
}

export const FeedbackControls: StoryObj<typeof FeedbackFixture> = {
  render: () => <FeedbackFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await expect(canvas.getByRole('dialog', { name: 'Clean local cache' })).toBeVisible()
    await expect(canvas.getByRole('status')).toHaveTextContent('3s')
  },
}

export const TransientEmptyListRow: StoryObj<typeof TransientListFixture> = {
  render: () => <TransientListFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await expect(canvas.queryByRole('textbox', { name: 'Models 1' })).not.toBeInTheDocument()
    await userEvent.click(canvas.getByRole('button', { name: 'Add' }))
    await expect(canvas.getByRole('textbox', { name: 'Models 1' })).toBeVisible()
    await userEvent.click(canvas.getByRole('button', { name: 'Outside' }))
    await expect(canvas.queryByRole('textbox', { name: 'Models 1' })).not.toBeInTheDocument()
    await userEvent.click(canvas.getByRole('button', { name: 'Add variable' }))
    await expect(canvas.getByRole('textbox', { name: 'Key' })).toBeVisible()
    await userEvent.click(canvas.getByRole('button', { name: 'Outside' }))
    await expect(canvas.queryByRole('textbox', { name: 'Key' })).not.toBeInTheDocument()
  },
}

export const KnownEnvironmentVariables: StoryObj<typeof KnownEnvironmentVariablesFixture> = {
  render: () => <KnownEnvironmentVariablesFixture />,
  play: async ({ canvasElement }) => {
    const canvas = within(canvasElement)
    await userEvent.click(canvas.getByRole('button', { name: 'Add variable Agent environment' }))
    await expect(canvas.getByRole('button', { name: 'Value source' })).toBeVisible()
    await userEvent.click(canvas.getByRole('button', { name: 'Variable name' }))
    await expect(canvas.getByRole('option', { name: 'OPENAI_API_KEY' })).toBeVisible()
    await expect(canvas.getByRole('option', { name: 'Custom variable' })).toBeVisible()
    await userEvent.click(canvas.getByRole('option', { name: 'OPENAI_API_KEY' }))
    await expect(canvas.getByTestId('environment-output')).toHaveTextContent('OPENAI_API_KEY')
  },
}
