import { Copy, Play, RotateCcw, X } from 'lucide-react'
import { useState } from 'react'
import type { AgentRow, DatasetRow, DatasetTask, EnvironmentRow, RunDraft } from '../../domain/harbor'
import { agentModelNames, reconcileAgentModel } from '../../domain/agentModels'
import type { Translate } from '../../i18n'
import { datasetRef, datasetSelectOptions } from '../datasetSelectOptions'
import { CustomSelect } from './CustomSelect'
import { FolderPathInput, type FolderPathSelection } from './FolderPathInput'
import { Field, TabPanel } from './RunBuilderChrome'
import { RunBuilderRuntimePanel } from './RunBuilderRuntimePanel'
import { FieldError, FormValidationSummary, issuesByField, type FormIssue } from './FormValidationSummary'

interface RunBuilderProps {
  canLaunch?: boolean
  submitError?: string | null
  agents: AgentRow[]
  datasets: DatasetRow[]
  draft: RunDraft
  environments: EnvironmentRow[]
  taskRows: DatasetTask[]
  t: Translate
  onDraft: (draft: RunDraft) => void
  onCancel: () => void
  onCopyJobConfig: () => void
  onChooseDirectory: () => Promise<FolderPathSelection>
  onLaunch: () => void
  onReset: () => void
}

type RunBuilderTab = 'core' | 'tasks' | 'verifier' | 'runtime'

type VerifierMode = 'dataset-default' | 'skip'

export function RunBuilder({ canLaunch = true, submitError, agents, datasets, draft, environments, taskRows, t, onDraft, onCancel, onChooseDirectory, onCopyJobConfig, onLaunch, onReset }: RunBuilderProps) {
  const [activeTab, setActiveTab] = useState<RunBuilderTab>('core')
  const [validationAttempted, setValidationAttempted] = useState(false)
  const [taskSearch, setTaskSearch] = useState('')
  const datasetOptions = datasetSelectOptions(datasets, t)
  const agentOptions = agents
    .map((agent) => ({ label: agent.agentName, value: agent.agentName }))
  const selectedAgent = agents.find((agent) => agent.agentName === draft.agent)
  const modelOptions = agentModelNames(selectedAgent).map((model) => ({ label: model, value: model }))
  const environmentOptions = environments.map((row) => ({ label: row.name, value: row.id }))
  const selectedDataset = datasets.find((row) => datasetRef(row) === draft.source)
  const selectedDatasetKey = selectedDataset ? datasetRef(selectedDataset) : draft.source
  const selectedDatasetTasks = taskRows.filter((row) => row.datasetRef === selectedDatasetKey)
  const searchedTasks = selectedDatasetTasks.filter((row) => {
    const query = taskSearch.trim().toLowerCase()
    if (!query) return true
    return [row.name, row.description].some((value) => value.toLowerCase().includes(query))
  })
  const selectedTaskNames = draft.selectedTaskNames ?? selectedDatasetTasks.map((task) => task.name)
  const selectedTaskNameSet = new Set(selectedTaskNames)
  const selectedTaskCount = selectedTaskNames.length
  const verifierMode: VerifierMode = draft.verifierMode
  const leaderboardLockedByVerifier = verifierMode === 'skip'
  const allIssues = validateRunDraft(draft, t)
  const issues = validationAttempted ? allIssues : []
  const fieldErrors = issuesByField(issues)
  const launch = () => {
    setValidationAttempted(true)
    if (allIssues.length) {
      setActiveTab('core')
      window.requestAnimationFrame(() => document.querySelector<HTMLElement>('.form-validation-summary')?.focus())
      return
    }
    onLaunch()
  }
  const setVerifierMode = (mode: VerifierMode) => {
    if (mode === 'dataset-default') {
      onDraft({ ...draft, verifierMode: mode })
    } else if (mode === 'skip') {
      onDraft({ ...draft, verifierMode: mode, includeInLeaderboard: false })
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
  ]
  return (
    <section className="surface run-builder" id="new-job">
      <div className="section-header compact">
        <div>
          <h1>{t('newJob')}</h1>
        </div>
        <div className="run-builder-actions">
          <button className="secondary-button" onClick={onCancel}>
            <X aria-hidden="true" />
            {t('cancel')}
          </button>
          <button className="secondary-button" onClick={onReset}>
            <RotateCcw aria-hidden="true" />
            {t('reset')}
          </button>
          <button className="primary-button" disabled={!canLaunch} onClick={launch}>
            <Play aria-hidden="true" />
            {t('runJob')}
          </button>
          <button className="secondary-button" onClick={onCopyJobConfig}>
            <Copy aria-hidden="true" />
            {t('jobConfig')}
          </button>
        </div>
      </div>
      <FormValidationSummary
        issues={issues}
        serverError={submitError}
        title={t('formValidationTitle')}
        onIssue={(field) => {
          setActiveTab('core')
          window.requestAnimationFrame(() => document.getElementById(`job-${field}`)?.focus())
        }}
      />
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
          <Field error={fieldErrors.jobName} errorId="job-jobName-error" label={t('jobName')}>
            <input id="job-jobName" aria-describedby={fieldErrors.jobName ? 'job-jobName-error' : undefined} aria-invalid={Boolean(fieldErrors.jobName) || undefined} value={draft.jobName} onChange={(event) => onDraft({ ...draft, jobName: event.target.value })} />
          </Field>
          <Field error={fieldErrors.jobsDir} errorId="job-jobsDir-error" label={t('jobsDir')}>
            <div id="job-jobsDir" tabIndex={-1} aria-describedby={fieldErrors.jobsDir ? 'job-jobsDir-error' : undefined}>
              <FolderPathInput chooseLabel={t('chooseFolder')} label={t('jobsDirFolderPicker')} value={draft.jobsDir} onChange={(value) => onDraft({ ...draft, jobsDir: value })} onChoose={onChooseDirectory} />
            </div>
          </Field>
          <label>
            {t('jobDataset')}
            <CustomSelect
              ariaLabel={t('jobDataset')}
              describedBy={fieldErrors.source ? 'job-source-error' : undefined}
              id="job-source"
              invalid={Boolean(fieldErrors.source)}
              value={draft.source}
              options={datasetOptions}
              onChange={(value) => {
                onDraft({ ...draft, selectedTaskNames: null, source: value })
              }}
            />
            <FieldError id="job-source-error" message={fieldErrors.source} />
          </label>
          <label>
            {t('agent')}
            <CustomSelect
              ariaLabel={t('agent')}
              describedBy={fieldErrors.agent ? 'job-agent-error' : undefined}
              id="job-agent"
              invalid={Boolean(fieldErrors.agent)}
              value={draft.agent}
              options={agentOptions}
              onChange={(value) => {
                const agent = agents.find((candidate) => candidate.agentName === value)
                onDraft({ ...draft, agent: value, model: reconcileAgentModel(draft.model, agent) })
              }}
            />
            <FieldError id="job-agent-error" message={fieldErrors.agent} />
          </label>
          <label>
            {t('jobModel')}
            <CustomSelect
              ariaLabel={t('jobModel')}
              describedBy={fieldErrors.model ? 'job-model-error' : undefined}
              id="job-model"
              invalid={Boolean(fieldErrors.model)}
              value={draft.model}
              options={modelOptions}
              disabled={modelOptions.length === 0}
              placeholder={t('agentModelsRequired')}
              searchable
              searchAriaLabel={t('searchModels')}
              searchPlaceholder={t('searchModels')}
              onChange={(value) => onDraft({ ...draft, model: value })}
            />
            <FieldError id="job-model-error" message={fieldErrors.model} />
          </label>
          <label>
            {t('environment')}
            <CustomSelect
              ariaLabel={t('environment')}
              describedBy={fieldErrors.environment ? 'job-environment-error' : undefined}
              id="job-environment"
              invalid={Boolean(fieldErrors.environment)}
              value={draft.environment}
              options={environmentOptions}
              onChange={(value) => onDraft({ ...draft, environment: value })}
            />
            <FieldError id="job-environment-error" message={fieldErrors.environment} />
          </label>
          <Field error={fieldErrors.concurrency} errorId="job-concurrency-error" label={t('concurrency')}>
            <input
              id="job-concurrency"
              aria-describedby={fieldErrors.concurrency ? 'job-concurrency-error' : undefined}
              aria-invalid={Boolean(fieldErrors.concurrency) || undefined}
              type="number"
              min="1"
              value={draft.concurrency}
              onChange={(event) => onDraft({ ...draft, concurrency: Number(event.target.value) })}
            />
          </Field>
          <Field error={fieldErrors.attempts} errorId="job-attempts-error" label={t('attempts')}>
            <input
              id="job-attempts"
              aria-describedby={fieldErrors.attempts ? 'job-attempts-error' : undefined}
              aria-invalid={Boolean(fieldErrors.attempts) || undefined}
              type="number"
              min="1"
              value={draft.attempts}
              onChange={(event) => onDraft({ ...draft, attempts: Number(event.target.value) })}
            />
          </Field>
          <label>
            {t('debugMode')}
            <CustomSelect
              ariaLabel={t('debugMode')}
              value={draft.debug ? 'enabled' : 'disabled'}
              options={[
                { label: t('disabled'), value: 'disabled' },
                { label: t('enabled'), value: 'enabled' },
              ]}
              onChange={(value) => onDraft({ ...draft, debug: value === 'enabled' })}
            />
          </label>
          <label>
            {t('includeInLeaderboard')}
            <CustomSelect
              ariaLabel={t('includeInLeaderboard')}
              value={!leaderboardLockedByVerifier && draft.includeInLeaderboard ? 'enabled' : 'disabled'}
              disabled={leaderboardLockedByVerifier}
              options={[
                { label: t('enabled'), value: 'enabled' },
                { label: t('disabled'), value: 'disabled' },
              ]}
              onChange={(value) => {
                if (!leaderboardLockedByVerifier) {
                  onDraft({ ...draft, includeInLeaderboard: value === 'enabled' })
                }
              }}
            />
          </label>
          <Field label={t('notes')} wide>
            <textarea value={draft.notes} onChange={(event) => onDraft({ ...draft, notes: event.target.value })} />
          </Field>
        </div>
      </TabPanel>
      <TabPanel active={activeTab === 'tasks'} title={t('runTabTasks')}>
        <div className="run-grid">
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
                { label: t('skipVerifier'), value: 'skip' },
              ]}
              onChange={(value) => setVerifierMode(value as VerifierMode)}
            />
          </label>
        </div>
      </TabPanel>
      <TabPanel active={activeTab === 'runtime'} title={t('runTabRuntime')}>
        <RunBuilderRuntimePanel draft={draft} t={t} onDraft={onDraft} />
      </TabPanel>
    </section>
  )
}

function validateRunDraft(draft: RunDraft, t: Translate): FormIssue[] {
  const issues: FormIssue[] = []
  if (!draft.jobName.trim()) issues.push({ field: 'jobName', message: t('jobNameRequired') })
  if (!draft.jobsDir.trim()) issues.push({ field: 'jobsDir', message: t('jobsDirRequired') })
  if (!draft.source) issues.push({ field: 'source', message: t('jobDatasetRequired') })
  if (!draft.agent) issues.push({ field: 'agent', message: t('jobAgentRequired') })
  if (!draft.model) issues.push({ field: 'model', message: t('jobModelRequired') })
  if (!draft.environment) issues.push({ field: 'environment', message: t('jobEnvironmentRequired') })
  if (!Number.isInteger(draft.concurrency) || draft.concurrency < 1) issues.push({ field: 'concurrency', message: t('concurrencyInvalid') })
  if (!Number.isInteger(draft.attempts) || draft.attempts < 1) issues.push({ field: 'attempts', message: t('attemptsInvalid') })
  return issues
}
