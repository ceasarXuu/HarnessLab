import { Copy, Play, RotateCcw, X } from 'lucide-react'
import { useState } from 'react'
import type { DatasetRow, RunDraft, TaskRow } from '../../mocks/demo'
import type { Translate } from '../../i18n'
import { CustomSelect } from './CustomSelect'
import { Field, TabPanel, Toggle } from './RunBuilderChrome'
import { RunBuilderHubPanel } from './RunBuilderHubPanel'

interface RunBuilderProps {
  datasets: DatasetRow[]
  draft: RunDraft
  taskRows: TaskRow[]
  t: Translate
  onDraft: (draft: RunDraft) => void
  onCancel: () => void
  onLaunch: () => void
}

type RunBuilderTab = 'core' | 'tasks' | 'environment' | 'verifier' | 'runtime' | 'hub'

const datasetValue = (row: DatasetRow) => `${row.name}@${row.version}`
const agentOptions = [
  { label: 'claude-code', value: 'claude-code', model: 'anthropic/claude-haiku-4-5' },
  { label: 'codex-cli', value: 'codex-cli', model: 'gpt-5.1' },
  { label: 'oracle', value: 'oracle', model: 'local-sim' },
]

export function RunBuilder({ datasets, draft, taskRows, t, onDraft, onCancel, onLaunch }: RunBuilderProps) {
  const [activeTab, setActiveTab] = useState<RunBuilderTab>('core')
  const [taskSearch, setTaskSearch] = useState('')
  const datasetOptions = datasets.map((row) => ({ label: datasetValue(row), value: datasetValue(row) }))
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
    { key: 'environment', label: t('runTabEnvironment') },
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
            <input value={draft.jobsDir} onChange={(event) => onDraft({ ...draft, jobsDir: event.target.value })} />
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
              options={[
                { label: 'docker', value: 'docker' },
                { label: 'daytona', value: 'daytona' },
                { label: 'e2b', value: 'e2b' },
                { label: 'modal', value: 'modal' },
                { label: 'gke', value: 'gke' },
                { label: 'runloop', value: 'runloop' },
                { label: 'custom import path', value: 'custom' },
              ]}
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
            debug
            <CustomSelect
              ariaLabel="debug"
              value={draft.debug ? 'enabled' : 'disabled'}
              options={[
                { label: 'disabled', value: 'disabled' },
                { label: 'enabled', value: 'enabled' },
              ]}
              onChange={(value) => onDraft({ ...draft, debug: value === 'enabled' })}
            />
          </label>
          <Field label="env_file">
            <input value={draft.envFile} onChange={(event) => onDraft({ ...draft, envFile: event.target.value })} />
          </Field>
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
      <TabPanel active={activeTab === 'environment'} title={t('runTabEnvironment')}>
        <div className="run-grid">
        <label>
          {t('environment')}
          <CustomSelect
            ariaLabel={t('environment')}
            value={draft.environment}
            options={[
              { label: 'docker', value: 'docker' },
              { label: 'daytona', value: 'daytona' },
              { label: 'e2b', value: 'e2b' },
              { label: 'modal', value: 'modal' },
              { label: 'gke', value: 'gke' },
              { label: 'runloop', value: 'runloop' },
              { label: 'langsmith', value: 'langsmith' },
              { label: 'novita', value: 'novita' },
              { label: 'apple-container', value: 'apple-container' },
              { label: 'singularity', value: 'singularity' },
              { label: 'islo', value: 'islo' },
              { label: 'tensorlake', value: 'tensorlake' },
              { label: 'cwsandbox', value: 'cwsandbox' },
              { label: 'wandb', value: 'wandb' },
              { label: 'use-computer', value: 'use-computer' },
              { label: 'custom import path', value: 'custom' },
            ]}
            onChange={(value) => onDraft({ ...draft, environment: value })}
          />
        </label>
        <Field label="environment import_path">
          <input
            value={draft.environmentImportPath}
            onChange={(event) => onDraft({ ...draft, environmentImportPath: event.target.value })}
          />
        </Field>
        <Field label="environment env">
          <input
            value={draft.environmentEnv}
            onChange={(event) => onDraft({ ...draft, environmentEnv: event.target.value })}
          />
        </Field>
        <Field label="environment kwargs">
          <input
            value={draft.environmentKwargs}
            onChange={(event) => onDraft({ ...draft, environmentKwargs: event.target.value })}
          />
        </Field>
        <Field label="allow_environment_host">
          <input
            value={draft.allowEnvironmentHosts}
            onChange={(event) => onDraft({ ...draft, allowEnvironmentHosts: event.target.value })}
          />
        </Field>
        <Field label={t('forceBuild')}>
          <Toggle checked={draft.forceBuild} onChange={(value) => onDraft({ ...draft, forceBuild: value })} />
        </Field>
        <Field label={t('deleteEnvironment')}>
          <Toggle checked={draft.deleteEnvironment} onChange={(value) => onDraft({ ...draft, deleteEnvironment: value })} />
        </Field>
        <Field label="suppress_override_warnings">
          <Toggle
            checked={draft.suppressOverrideWarnings}
            onChange={(value) => onDraft({ ...draft, suppressOverrideWarnings: value })}
          />
        </Field>
        <Field label={t('resourcePolicy')}>
          <input value={draft.cpus} onChange={(event) => onDraft({ ...draft, cpus: event.target.value })} />
        </Field>
        <Field label="override_cpus">
          <input value={draft.cpuOverride} onChange={(event) => onDraft({ ...draft, cpuOverride: event.target.value })} />
        </Field>
        <Field label={t('memoryMb')}>
          <input value={draft.memoryMb} onChange={(event) => onDraft({ ...draft, memoryMb: event.target.value })} />
        </Field>
        <Field label={t('storageMb')}>
          <input value={draft.storageMb} onChange={(event) => onDraft({ ...draft, storageMb: event.target.value })} />
        </Field>
        <Field label={t('gpus')}>
          <input value={draft.gpus} onChange={(event) => onDraft({ ...draft, gpus: event.target.value })} />
        </Field>
        <Field label="tpu">
          <input value={draft.tpu} onChange={(event) => onDraft({ ...draft, tpu: event.target.value })} />
        </Field>
        <Field label={t('mounts')}>
          <input value={draft.mounts} onChange={(event) => onDraft({ ...draft, mounts: event.target.value })} />
        </Field>
        <Field label={t('dockerCompose')}>
          <input value={draft.dockerCompose} onChange={(event) => onDraft({ ...draft, dockerCompose: event.target.value })} />
        </Field>
        </div>
      </TabPanel>
      <TabPanel active={activeTab === 'verifier'} title={t('runTabVerifier')}>
        <div className="run-grid">
        <Field label={t('verifier')}>
          <input
            value={draft.verifierImportPath}
            onChange={(event) => onDraft({ ...draft, verifierImportPath: event.target.value })}
          />
        </Field>
        <Field label={t('verifierEnv')}>
          <input value={draft.verifierEnv} onChange={(event) => onDraft({ ...draft, verifierEnv: event.target.value })} />
        </Field>
        <Field label={t('verifierKwargs')}>
          <input
            value={draft.verifierKwargs}
            onChange={(event) => onDraft({ ...draft, verifierKwargs: event.target.value })}
          />
        </Field>
        <Field label={t('disableVerifier')}>
          <Toggle checked={draft.disableVerifier} onChange={(value) => onDraft({ ...draft, disableVerifier: value })} />
        </Field>
        <Field label="verifier max timeout sec">
          <input
            value={draft.verifierMaxTimeoutSec}
            onChange={(event) => onDraft({ ...draft, verifierMaxTimeoutSec: event.target.value })}
          />
        </Field>
        </div>
      </TabPanel>
      <TabPanel active={activeTab === 'runtime'} title={t('runTabRuntime')}>
        <div className="run-grid">
        <Field label={t('extraInstructions')}>
          <input
            value={draft.extraInstructions}
            onChange={(event) => onDraft({ ...draft, extraInstructions: event.target.value })}
          />
        </Field>
        <Field label="quiet">
          <Toggle checked={draft.quiet} onChange={(value) => onDraft({ ...draft, quiet: value })} />
        </Field>
        <Field label={t('timeoutMultiplier')}>
          <input
            type="number"
            min="0.1"
            step="0.1"
            value={draft.timeoutMultiplier}
            onChange={(event) => onDraft({ ...draft, timeoutMultiplier: Number(event.target.value) })}
          />
        </Field>
        <Field label={t('agentTimeoutMultiplier')}>
          <input
            value={draft.agentTimeoutMultiplier}
            onChange={(event) => onDraft({ ...draft, agentTimeoutMultiplier: event.target.value })}
          />
        </Field>
        <Field label={t('verifierTimeoutMultiplier')}>
          <input
            value={draft.verifierTimeoutMultiplier}
            onChange={(event) => onDraft({ ...draft, verifierTimeoutMultiplier: event.target.value })}
          />
        </Field>
        <Field label="agent setup timeout multiplier">
          <input
            value={draft.agentSetupTimeoutMultiplier}
            onChange={(event) => onDraft({ ...draft, agentSetupTimeoutMultiplier: event.target.value })}
          />
        </Field>
        <Field label="environment build timeout multiplier">
          <input
            value={draft.environmentBuildTimeoutMultiplier}
            onChange={(event) => onDraft({ ...draft, environmentBuildTimeoutMultiplier: event.target.value })}
          />
        </Field>
        <Field label={t('maxRetries')}>
          <input
            type="number"
            min="0"
            value={draft.maxRetries}
            onChange={(event) => onDraft({ ...draft, maxRetries: Number(event.target.value) })}
          />
        </Field>
        <Field label={t('retryInclude')}>
          <input value={draft.retryInclude} onChange={(event) => onDraft({ ...draft, retryInclude: event.target.value })} />
        </Field>
        <Field label={t('retryExclude')}>
          <input value={draft.retryExclude} onChange={(event) => onDraft({ ...draft, retryExclude: event.target.value })} />
        </Field>
        <Field label="retry wait multiplier">
          <input
            value={draft.retryWaitMultiplier}
            onChange={(event) => onDraft({ ...draft, retryWaitMultiplier: event.target.value })}
          />
        </Field>
        <Field label="retry min wait sec">
          <input
            value={draft.retryMinWaitSec}
            onChange={(event) => onDraft({ ...draft, retryMinWaitSec: event.target.value })}
          />
        </Field>
        <Field label="retry max wait sec">
          <input
            value={draft.retryMaxWaitSec}
            onChange={(event) => onDraft({ ...draft, retryMaxWaitSec: event.target.value })}
          />
        </Field>
        </div>
      </TabPanel>
      <TabPanel active={activeTab === 'hub'} title={t('runTabHub')}>
        <RunBuilderHubPanel draft={draft} t={t} onDraft={onDraft} />
      </TabPanel>
    </section>
  )
}
