import { Copy, Play, RotateCcw, X } from 'lucide-react'
import { useState } from 'react'
import type { DatasetRow, EnvironmentRow, RunDraft, TaskRow } from '../../mocks/demo'
import type { Translate } from '../../i18n'
import { CustomSelect } from './CustomSelect'
import { FolderPathInput } from './FolderPathInput'
import { KeyValueControl } from './KeyValueControl'
import { Field, TabPanel } from './RunBuilderChrome'
import { RunBuilderHubPanel } from './RunBuilderHubPanel'
import { RunBuilderRuntimePanel } from './RunBuilderRuntimePanel'

interface RunBuilderProps {
  datasets: DatasetRow[]
  draft: RunDraft
  environments: EnvironmentRow[]
  taskRows: TaskRow[]
  t: Translate
  onDraft: (draft: RunDraft) => void
  onCancel: () => void
  onLaunch: () => void
}

type RunBuilderTab = 'core' | 'tasks' | 'verifier' | 'runtime' | 'hub'

const datasetValue = (row: DatasetRow) => `${row.name}@${row.version}`
const agentOptions = [
  { label: 'claude-code', value: 'claude-code', model: 'anthropic/claude-haiku-4-5' },
  { label: 'codex-cli', value: 'codex-cli', model: 'gpt-5.1' },
  { label: 'oracle', value: 'oracle', model: 'local-sim' },
]
type VerifierMode = 'dataset-default' | 'custom' | 'skip'

export function RunBuilder({ datasets, draft, environments, taskRows, t, onDraft, onCancel, onLaunch }: RunBuilderProps) {
  const [activeTab, setActiveTab] = useState<RunBuilderTab>('core')
  const [taskSearch, setTaskSearch] = useState('')
  const datasetOptions = datasets.map((row) => ({ label: datasetValue(row), value: datasetValue(row) }))
  const environmentOptions = environments.map((row) => ({ label: row.name, value: row.id }))
  const selectedDataset = datasets.find((row) => datasetValue(row) === draft.source)
  const availableSplits = selectedDataset?.splits ?? []
  const selectedDatasetKey = selectedDataset ? datasetValue(selectedDataset) : draft.source
  const selectedDatasetTasks = taskRows.filter(
    (row) => row.dataset === selectedDataset?.name || row.dataset === selectedDatasetKey,
  )
  const searchedTasks = selectedDatasetTasks.filter((row) => {
    const query = taskSearch.trim().toLowerCase()
    if (!query) return true
    return [row.name, row.description, row.state].some((value) => value.toLowerCase().includes(query))
  })
  const selectedTaskNames = draft.selectedTaskNames ?? selectedDatasetTasks.map((task) => task.name)
  const selectedTaskNameSet = new Set(selectedTaskNames)
  const selectedTaskCount = selectedTaskNames.length
  const verifierMode: VerifierMode = draft.verifierMode
  const setVerifierMode = (mode: VerifierMode) => {
    if (mode === 'dataset-default') {
      onDraft({ ...draft, verifierMode: mode, disableVerifier: false, verifierImportPath: '' })
    } else if (mode === 'skip') {
      onDraft({ ...draft, verifierMode: mode, disableVerifier: true })
    } else {
      onDraft({ ...draft, verifierMode: mode, disableVerifier: false })
    }
  }
  const toggleTask = (taskName: string, enabled: boolean) => {
    const next = new Set(selectedTaskNames)
    if (enabled) {
      next.add(taskName)
    } else {
      next.delete(taskName)
    }
    onDraft({ ...draft, selectedTaskNames: Array.from(next) })
  }
  const setFilteredTasks = (enabled: boolean) => {
    const filteredNames = searchedTasks.map((task) => task.name)
    const next = new Set(selectedTaskNames)
    for (const taskName of filteredNames) {
      if (enabled) {
        next.add(taskName)
      } else {
        next.delete(taskName)
      }
    }
    onDraft({ ...draft, selectedTaskNames: Array.from(next) })
  }
  const tabs: Array<{ key: RunBuilderTab; label: string }> = [
    { key: 'core', label: t('runTabCore') },
    { key: 'tasks', label: t('runTabTasks') },
    { key: 'verifier', label: t('runTabVerifier') },
    { key: 'runtime', label: t('runTabRuntime') },
    { key: 'hub', label: t('runTabHub') },
  ]
  return (
    <section className="surface run-builder" id="new-job">
      <div className="section-header compact">
        <div>
          <h1>{t('newJob')}</h1>
          <p>{t('newJobDesc')}</p>
        </div>
        <div className="run-builder-actions">
          <button className="secondary-button" onClick={onCancel}>
            <X aria-hidden="true" />
            {t('cancel')}
          </button>
          <button className="secondary-button">
            <RotateCcw aria-hidden="true" />
            {t('reset')}
          </button>
          <button className="primary-button" onClick={onLaunch}>
            <Play aria-hidden="true" />
            {t('runJob')}
          </button>
          <button className="secondary-button">
            <Copy aria-hidden="true" />
            {t('jobConfig')}
          </button>
        </div>
      </div>
      <div className="run-tabs" role="tablist" aria-label={t('jobConfig')}>
        {tabs.map((tab) => (
          <button
            key={tab.key}
            type="button"
            role="tab"
            aria-selected={activeTab === tab.key}
            className={activeTab === tab.key ? 'active' : undefined}
            onClick={() => setActiveTab(tab.key)}
          >
            {tab.label}
          </button>
        ))}
      </div>
      <TabPanel active={activeTab === 'core'} title={t('runTabCore')}>
        <div className="run-grid">
          <Field label={t('jobName')}>
            <input value={draft.jobName} onChange={(event) => onDraft({ ...draft, jobName: event.target.value })} />
          </Field>
          <Field label={t('jobsDir')}>
            <FolderPathInput
              chooseLabel={t('chooseFolder')}
              label={t('jobsDirFolderPicker')}
              value={draft.jobsDir}
              onChange={(value) => onDraft({ ...draft, jobsDir: value })}
            />
          </Field>
          <label>
            {t('jobDataset')}
            <CustomSelect
              ariaLabel={t('jobDataset')}
              value={draft.source}
              options={datasetOptions}
              onChange={(value) => {
                const nextDataset = datasets.find((row) => datasetValue(row) === value)
                onDraft({ ...draft, selectedTaskNames: null, source: value, split: nextDataset?.splits?.[0] ?? '' })
              }}
            />
          </label>
          <label>
            {t('agent')}
            <CustomSelect
              ariaLabel={t('agent')}
              value={draft.agent}
              options={agentOptions.map((option) => ({ label: option.label, value: option.value }))}
              onChange={(value) => {
                const nextAgent = agentOptions.find((option) => option.value === value)
                onDraft({ ...draft, agent: value, model: nextAgent?.model ?? draft.model })
              }}
            />
          </label>
          <label>
            {t('environment')}
            <CustomSelect
              ariaLabel={t('environment')}
              value={draft.environment}
              options={environmentOptions}
              onChange={(value) => onDraft({ ...draft, environment: value })}
            />
          </label>
          <Field label={t('concurrency')}>
            <input
              type="number"
              min="1"
              value={draft.concurrency}
              onChange={(event) => onDraft({ ...draft, concurrency: Number(event.target.value) })}
            />
          </Field>
          <Field label={t('attempts')}>
            <input
              type="number"
              min="1"
              value={draft.attempts}
              onChange={(event) => onDraft({ ...draft, attempts: Number(event.target.value) })}
            />
          </Field>
          <label>
            debug模式
            <CustomSelect
              ariaLabel="debug模式"
              value={draft.debug ? 'enabled' : 'disabled'}
              options={[
                { label: 'disabled', value: 'disabled' },
                { label: 'enabled', value: 'enabled' },
              ]}
              onChange={(value) => onDraft({ ...draft, debug: value === 'enabled' })}
            />
          </label>
          <label>
            {t('includeInLeaderboard')}
            <CustomSelect
              ariaLabel={t('includeInLeaderboard')}
              value={draft.includeInLeaderboard ? 'enabled' : 'disabled'}
              options={[
                { label: 'enabled', value: 'enabled' },
                { label: 'disabled', value: 'disabled' },
              ]}
              onChange={(value) => onDraft({ ...draft, includeInLeaderboard: value === 'enabled' })}
            />
          </label>
          <Field label={t('notes')} wide>
            <textarea value={draft.notes} onChange={(event) => onDraft({ ...draft, notes: event.target.value })} />
          </Field>
        </div>
      </TabPanel>
      <TabPanel active={activeTab === 'tasks'} title={t('runTabTasks')}>
        <div className="run-grid">
          {availableSplits.length > 0 && (
            <label>
              {t('split')}
              <CustomSelect
                ariaLabel={t('split')}
                value={draft.split}
                options={availableSplits.map((split) => ({ label: split, value: split }))}
                onChange={(value) => onDraft({ ...draft, split: value })}
              />
            </label>
          )}
          <Field label={t('searchTaskList')}>
            <input
              aria-label={t('searchTaskList')}
              value={taskSearch}
              onChange={(event) => setTaskSearch(event.target.value)}
            />
          </Field>
          <Field label={t('extraInstructions')} wide>
            <input
              value={draft.extraInstructions}
              onChange={(event) => onDraft({ ...draft, extraInstructions: event.target.value })}
            />
          </Field>
          <section className="task-whitelist field-wide" aria-label={t('taskWhitelist')}>
            <div className="task-whitelist-header">
              <div>
                <h3>{t('taskWhitelist')}</h3>
                <span>
                  {t('selectedTaskCount')}: {selectedTaskCount} / {selectedDatasetTasks.length}
                </span>
              </div>
              <div className="button-row tight">
                <button
                  className="secondary-button"
                  type="button"
                  disabled={searchedTasks.length === 0}
                  onClick={() => setFilteredTasks(true)}
                >
                  {t('enableAllTasks')}
                </button>
                <button
                  className="secondary-button"
                  type="button"
                  disabled={searchedTasks.length === 0}
                  onClick={() => setFilteredTasks(false)}
                >
                  {t('disableAllTasks')}
                </button>
              </div>
            </div>
            {selectedDatasetTasks.length === 0 ? (
              <div className="plugin-empty-state">{t('noTasksAvailable')}</div>
            ) : (
              <div className="task-switch-list">
                {searchedTasks.map((task) => (
                  <label className="task-switch-row" key={task.name}>
                    <div>
                      <strong>{task.name}</strong>
                      <span>{task.description}</span>
                    </div>
                    <input
                      type="checkbox"
                      checked={selectedTaskNameSet.has(task.name)}
                      onChange={(event) => toggleTask(task.name, event.target.checked)}
                    />
                  </label>
                ))}
              </div>
            )}
          </section>
        </div>
      </TabPanel>
      <TabPanel active={activeTab === 'verifier'} title={t('runTabVerifier')}>
        <div className="run-grid">
          <label className="field-wide">
            {t('verifierMode')}
            <CustomSelect
              ariaLabel={t('verifierMode')}
              value={verifierMode}
              options={[
                { label: t('datasetDefaultVerifier'), value: 'dataset-default' },
                { label: t('customVerifier'), value: 'custom' },
                { label: t('skipVerifier'), value: 'skip' },
              ]}
              onChange={(value) => setVerifierMode(value as VerifierMode)}
            />
          </label>
          {verifierMode === 'custom' && (
            <>
              <Field label={t('verifierImportPath')}>
                <input
                  value={draft.verifierImportPath}
                  onChange={(event) => onDraft({ ...draft, verifierImportPath: event.target.value })}
                />
              </Field>
              <Field label={t('verifierMaxTimeoutSec')}>
                <input
                  type="number"
                  min="1"
                  value={draft.verifierMaxTimeoutSec}
                  onChange={(event) => onDraft({ ...draft, verifierMaxTimeoutSec: event.target.value })}
                />
              </Field>
              <KeyValueControl
                label={t('verifierEnv')}
                value={draft.verifierEnv}
                onChange={(value) => onDraft({ ...draft, verifierEnv: value })}
              />
              <KeyValueControl
                label={t('verifierKwargs')}
                value={draft.verifierKwargs}
                onChange={(value) => onDraft({ ...draft, verifierKwargs: value })}
              />
            </>
          )}
        </div>
      </TabPanel>
      <TabPanel active={activeTab === 'runtime'} title={t('runTabRuntime')}>
        <RunBuilderRuntimePanel draft={draft} t={t} onDraft={onDraft} />
      </TabPanel>
      <TabPanel active={activeTab === 'hub'} title={t('runTabHub')}>
        <RunBuilderHubPanel draft={draft} t={t} onDraft={onDraft} />
      </TabPanel>
    </section>
  )
}
