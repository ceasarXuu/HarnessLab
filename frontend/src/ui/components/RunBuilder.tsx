import { Copy, Play, RotateCcw } from 'lucide-react'
import { useState } from 'react'
import type { DatasetRow, RunDraft } from '../../mocks/demo'
import type { Translate } from '../../i18n'
import { CustomSelect } from './CustomSelect'
import { Field, TabPanel, Toggle } from './RunBuilderChrome'
import { RunBuilderHubPanel } from './RunBuilderHubPanel'

interface RunBuilderProps {
  datasets: DatasetRow[]
  draft: RunDraft
  t: Translate
  onDraft: (draft: RunDraft) => void
  onLaunch: () => void
}

type RunBuilderTab = 'core' | 'agent' | 'environment' | 'verifier' | 'runtime' | 'hub'

const datasetValue = (row: DatasetRow) => `${row.name}@${row.version}`

export function RunBuilder({ datasets, draft, t, onDraft, onLaunch }: RunBuilderProps) {
  const [activeTab, setActiveTab] = useState<RunBuilderTab>('core')
  const datasetOptions = datasets.map((row) => ({ label: datasetValue(row), value: datasetValue(row) }))
  const selectedDataset = datasets.find((row) => datasetValue(row) === draft.source)
  const availableSplits = selectedDataset?.splits ?? []
  const tabs: Array<{ key: RunBuilderTab; label: string }> = [
    { key: 'core', label: t('runTabCore') },
    { key: 'agent', label: t('runTabAgent') },
    { key: 'environment', label: t('runTabEnvironment') },
    { key: 'verifier', label: t('runTabVerifier') },
    { key: 'runtime', label: t('runTabRuntime') },
    { key: 'hub', label: t('runTabHub') },
  ]
  const command = [
    'harbor run',
    `--job-name ${draft.jobName}`,
    `--jobs-dir ${draft.jobsDir}`,
    `--dataset ${draft.source}`,
    draft.split ? `--split ${draft.split}` : '',
    draft.taskFilter ? `--include-task-name ${draft.taskFilter}` : '',
    draft.excludeFilter ? `--exclude-task-name ${draft.excludeFilter}` : '',
    `--n-tasks ${draft.taskLimit}`,
    draft.debug ? '--debug' : '',
    draft.quiet ? '--quiet' : '',
    draft.yes ? '--yes' : '',
    draft.envFile ? `--env-file ${draft.envFile}` : '',
    `--agent ${draft.agent}`,
    draft.agentImportPath ? `--agent-import-path ${draft.agentImportPath}` : '',
    `--model ${draft.model}`,
    draft.agentEnv ? `--agent-env ${draft.agentEnv}` : '',
    draft.agentKwargs ? `--agent-kwarg ${draft.agentKwargs}` : '',
    draft.allowAgentHosts ? `--allow-agent-host ${draft.allowAgentHosts}` : '',
    draft.skills ? `--skills ${draft.skills}` : '',
    draft.mcpConfig ? `--mcp-config ${draft.mcpConfig}` : '',
    `--env ${draft.environment}`,
    draft.environmentImportPath ? `--environment-import-path ${draft.environmentImportPath}` : '',
    draft.environmentEnv ? `--environment-env ${draft.environmentEnv}` : '',
    draft.environmentKwargs ? `--environment-kwarg ${draft.environmentKwargs}` : '',
    draft.allowEnvironmentHosts ? `--allow-environment-host ${draft.allowEnvironmentHosts}` : '',
    draft.forceBuild ? '--force-build' : '--no-force-build',
    draft.deleteEnvironment ? '--delete' : '--no-delete',
    draft.suppressOverrideWarnings ? '--suppress-override-warnings' : '',
    `--cpus ${draft.cpus}`,
    draft.cpuOverride ? `--override-cpus ${draft.cpuOverride}` : '',
    draft.memoryMb ? `--override-memory-mb ${draft.memoryMb}` : '',
    draft.storageMb ? `--override-storage-mb ${draft.storageMb}` : '',
    draft.gpus ? `--override-gpus ${draft.gpus}` : '',
    draft.tpu ? `--override-tpu ${draft.tpu}` : '',
    draft.mounts ? `--mount ${draft.mounts}` : '',
    draft.dockerCompose ? `--extra-docker-compose ${draft.dockerCompose}` : '',
    draft.verifierImportPath ? `--verifier-import-path ${draft.verifierImportPath}` : '',
    draft.verifierEnv ? `--verifier-env ${draft.verifierEnv}` : '',
    draft.verifierKwargs ? `--verifier-kwarg ${draft.verifierKwargs}` : '',
    draft.disableVerifier ? '--disable-verification' : '--enable-verification',
    draft.verifierMaxTimeoutSec ? `--verifier-max-timeout-sec ${draft.verifierMaxTimeoutSec}` : '',
    `--n-concurrent ${draft.concurrency}`,
    `--n-attempts ${draft.attempts}`,
    `--timeout-multiplier ${draft.timeoutMultiplier}`,
    draft.agentTimeoutMultiplier ? `--agent-timeout-multiplier ${draft.agentTimeoutMultiplier}` : '',
    draft.verifierTimeoutMultiplier ? `--verifier-timeout-multiplier ${draft.verifierTimeoutMultiplier}` : '',
    draft.agentSetupTimeoutMultiplier ? `--agent-setup-timeout-multiplier ${draft.agentSetupTimeoutMultiplier}` : '',
    draft.environmentBuildTimeoutMultiplier
      ? `--environment-build-timeout-multiplier ${draft.environmentBuildTimeoutMultiplier}`
      : '',
    `--max-retries ${draft.maxRetries}`,
    draft.retryInclude ? `--retry-include ${draft.retryInclude}` : '',
    draft.retryExclude ? `--retry-exclude ${draft.retryExclude}` : '',
    draft.retryWaitMultiplier ? `--retry-wait-multiplier ${draft.retryWaitMultiplier}` : '',
    draft.retryMinWaitSec ? `--retry-min-wait-sec ${draft.retryMinWaitSec}` : '',
    draft.retryMaxWaitSec ? `--retry-max-wait-sec ${draft.retryMaxWaitSec}` : '',
    draft.artifacts ? `--artifact ${draft.artifacts}` : '',
    draft.metric ? `--metric ${draft.metric}` : '',
    draft.plugins ? `--plugin ${draft.plugins}` : '',
    draft.upload ? `--upload --${draft.visibility}` : '',
    draft.shareTargets ? `--share-user ${draft.shareTargets}` : '',
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
            {t('source')}
            <CustomSelect
              ariaLabel={t('source')}
              value={draft.source}
              options={datasetOptions}
              onChange={(value) => {
                const nextDataset = datasets.find((row) => datasetValue(row) === value)
                onDraft({ ...draft, source: value, split: nextDataset?.splits?.[0] ?? '' })
              }}
            />
          </label>
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
          <Field label="debug">
            <Toggle checked={draft.debug} onChange={(value) => onDraft({ ...draft, debug: value })} />
          </Field>
          <Field label="yes">
            <Toggle checked={draft.yes} onChange={(value) => onDraft({ ...draft, yes: value })} />
          </Field>
          <Field label="env_file">
            <input value={draft.envFile} onChange={(event) => onDraft({ ...draft, envFile: event.target.value })} />
          </Field>
        </div>
      </TabPanel>
      <TabPanel active={activeTab === 'agent'} title={t('runTabAgent')}>
        <div className="run-grid">
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
        <Field label="allow_agent_host">
          <input
            value={draft.allowAgentHosts}
            onChange={(event) => onDraft({ ...draft, allowAgentHosts: event.target.value })}
          />
        </Field>
        <Field label={t('skills')}>
          <input value={draft.skills} onChange={(event) => onDraft({ ...draft, skills: event.target.value })} />
        </Field>
        <Field label={t('mcpConfig')}>
          <input value={draft.mcpConfig} onChange={(event) => onDraft({ ...draft, mcpConfig: event.target.value })} />
        </Field>
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
      <div className="config-preview">
        <code>{command}</code>
      </div>
      <div className="button-row">
        <button className="secondary-button">
          <RotateCcw aria-hidden="true" />
          {t('reset')}
        </button>
        <button className="primary-button" onClick={onLaunch}>
          <Play aria-hidden="true" />
          {t('runJob')}
        </button>
      </div>
    </section>
  )
}
