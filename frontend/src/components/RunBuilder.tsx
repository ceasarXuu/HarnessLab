import { Copy, Play, Plus, RotateCcw } from 'lucide-react'
import type { ReactNode } from 'react'
import type { RunDraft } from '../data/demo'
import type { Translate } from '../i18n'
import { CustomSelect } from './CustomSelect'

interface RunBuilderProps {
  draft: RunDraft
  t: Translate
  onDraft: (draft: RunDraft) => void
  onLaunch: () => void
}

export function RunBuilder({ draft, t, onDraft, onLaunch }: RunBuilderProps) {
  const command = [
    'harbor run',
    `--job-name ${draft.jobName}`,
    `--jobs-dir ${draft.jobsDir}`,
    `--dataset ${draft.source}`,
    draft.taskFilter ? `--include-task-name ${draft.taskFilter}` : '',
    draft.excludeFilter ? `--exclude-task-name ${draft.excludeFilter}` : '',
    `--n-tasks ${draft.taskLimit}`,
    `--agent ${draft.agent}`,
    draft.agentImportPath ? `--agent-import-path ${draft.agentImportPath}` : '',
    `--model ${draft.model}`,
    `--env ${draft.environment}`,
    `--n-concurrent ${draft.concurrency}`,
    `--n-attempts ${draft.attempts}`,
    `--max-retries ${draft.maxRetries}`,
    draft.upload ? `--upload --${draft.visibility}` : '',
  ]
    .filter(Boolean)
    .join(' ')

  return (
    <section className="surface run-builder" id="new-job">
      <div className="section-header compact">
        <div>
          <h1>{t('newJob')}</h1>
          <p>{t('newJobDesc')}</p>
        </div>
        <button className="secondary-button">
          <Copy aria-hidden="true" />
          {t('jobConfig')}
        </button>
      </div>
      <div className="run-grid">
        <Field label={t('jobName')}>
          <input value={draft.jobName} onChange={(event) => onDraft({ ...draft, jobName: event.target.value })} />
        </Field>
        <Field label={t('jobsDir')}>
          <input value={draft.jobsDir} onChange={(event) => onDraft({ ...draft, jobsDir: event.target.value })} />
        </Field>
        <label>
          {t('source')}
          <CustomSelect
            ariaLabel={t('source')}
            value={draft.source}
            options={[
              { label: 'terminal-bench@2.0', value: 'terminal-bench@2.0' },
              { label: 'swe-bench-lite', value: 'swe-bench-lite' },
              { label: 'harbor/hello-world', value: 'harbor/hello-world' },
            ]}
            onChange={(value) => onDraft({ ...draft, source: value })}
          />
        </label>
        <Field label={t('taskInclude')}>
          <input value={draft.taskFilter} onChange={(event) => onDraft({ ...draft, taskFilter: event.target.value })} />
        </Field>
        <Field label={t('taskExclude')}>
          <input value={draft.excludeFilter} onChange={(event) => onDraft({ ...draft, excludeFilter: event.target.value })} />
        </Field>
        <Field label={t('taskLimit')}>
          <input
            type="number"
            min="1"
            value={draft.taskLimit}
            onChange={(event) => onDraft({ ...draft, taskLimit: Number(event.target.value) })}
          />
        </Field>
        <Field label={t('extraInstructions')}>
          <input
            value={draft.extraInstructions}
            onChange={(event) => onDraft({ ...draft, extraInstructions: event.target.value })}
          />
        </Field>
        <label>
          {t('agent')}
          <CustomSelect
            ariaLabel={t('agent')}
            value={draft.agent}
            options={[
              { label: 'claude-code', value: 'claude-code' },
              { label: 'codex-cli', value: 'codex-cli' },
              { label: 'oracle', value: 'oracle' },
            ]}
            onChange={(value) => onDraft({ ...draft, agent: value })}
          />
        </label>
        <Field label={t('model')}>
          <input value={draft.model} onChange={(event) => onDraft({ ...draft, model: event.target.value })} />
        </Field>
        <Field label={t('agentImportPath')}>
          <input
            value={draft.agentImportPath}
            onChange={(event) => onDraft({ ...draft, agentImportPath: event.target.value })}
          />
        </Field>
        <Field label={t('agentEnv')}>
          <input value={draft.agentEnv} onChange={(event) => onDraft({ ...draft, agentEnv: event.target.value })} />
        </Field>
        <Field label={t('agentKwargs')}>
          <input value={draft.agentKwargs} onChange={(event) => onDraft({ ...draft, agentKwargs: event.target.value })} />
        </Field>
        <Field label={t('skills')}>
          <input value={draft.skills} onChange={(event) => onDraft({ ...draft, skills: event.target.value })} />
        </Field>
        <Field label={t('mcpConfig')}>
          <input value={draft.mcpConfig} onChange={(event) => onDraft({ ...draft, mcpConfig: event.target.value })} />
        </Field>
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
              { label: 'custom import path', value: 'custom' },
            ]}
            onChange={(value) => onDraft({ ...draft, environment: value })}
          />
        </label>
        <Field label={t('forceBuild')}>
          <Toggle checked={draft.forceBuild} onChange={(value) => onDraft({ ...draft, forceBuild: value })} />
        </Field>
        <Field label={t('deleteEnvironment')}>
          <Toggle checked={draft.deleteEnvironment} onChange={(value) => onDraft({ ...draft, deleteEnvironment: value })} />
        </Field>
        <Field label={t('resourcePolicy')}>
          <input value={draft.cpus} onChange={(event) => onDraft({ ...draft, cpus: event.target.value })} />
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
        <Field label={t('mounts')}>
          <input value={draft.mounts} onChange={(event) => onDraft({ ...draft, mounts: event.target.value })} />
        </Field>
        <Field label={t('dockerCompose')}>
          <input value={draft.dockerCompose} onChange={(event) => onDraft({ ...draft, dockerCompose: event.target.value })} />
        </Field>
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
        <Field label={t('artifacts')}>
          <input value={draft.artifacts} onChange={(event) => onDraft({ ...draft, artifacts: event.target.value })} />
        </Field>
        <Field label={t('metric')}>
          <input value={draft.metric} onChange={(event) => onDraft({ ...draft, metric: event.target.value })} />
        </Field>
        <Field label={t('plugins')}>
          <input value={draft.plugins} onChange={(event) => onDraft({ ...draft, plugins: event.target.value })} />
        </Field>
        <Field label={t('uploadToHub')}>
          <Toggle checked={draft.upload} onChange={(value) => onDraft({ ...draft, upload: value })} />
        </Field>
        <label>
          {t('visibility')}
          <CustomSelect
            ariaLabel={t('visibility')}
            value={draft.visibility}
            options={[
              { label: 'private', value: 'private' },
              { label: 'public', value: 'public' },
            ]}
            onChange={(value) => onDraft({ ...draft, visibility: value as 'private' | 'public' })}
          />
        </label>
        <Field label={t('shareTargets')}>
          <input value={draft.shareTargets} onChange={(event) => onDraft({ ...draft, shareTargets: event.target.value })} />
        </Field>
      </div>
      <div className="config-preview">
        <code>{command}</code>
      </div>
      <div className="button-row">
        <button className="secondary-button">
          <RotateCcw aria-hidden="true" />
          {t('reset')}
        </button>
        <button className="secondary-button">
          <Plus aria-hidden="true" />
          {t('saveTemplate')}
        </button>
        <button className="primary-button" onClick={onLaunch}>
          <Play aria-hidden="true" />
          {t('runJob')}
        </button>
      </div>
    </section>
  )
}

function Field({ children, label }: { children: ReactNode; label: string }) {
  return (
    <label>
      {label}
      {children}
    </label>
  )
}

function Toggle({ checked, onChange }: { checked: boolean; onChange: (value: boolean) => void }) {
  return (
    <button type="button" className={checked ? 'toggle active' : 'toggle'} onClick={() => onChange(!checked)}>
      {checked ? 'enabled' : 'disabled'}
    </button>
  )
}
