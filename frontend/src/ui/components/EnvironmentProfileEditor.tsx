import { useState } from 'react'
import { Plus, Trash2 } from 'lucide-react'
import type { EnvironmentRow } from '../../mocks/demo'
import type { MessageKey, Translate } from '../../i18n'
import { CustomSelect } from './CustomSelect'
import { KeyValueControl } from './KeyValueControl'
import { NetworkAccessControl } from './NetworkAccessControl'
import { TpuSpecControl } from './TpuSpecControl'

type EnvironmentTab = 'base' | 'network' | 'advanced'
type EnvironmentFieldKind = 'text' | 'select' | 'number' | 'tags' | 'keyValue' | 'json' | 'path' | 'pathList' | 'switch' | 'tpu' | 'healthcheck'
type EnvironmentFieldLayout = 'short' | 'medium' | 'wide' | 'full'

interface EnvironmentField {
  key: keyof EnvironmentRow
  labelKey: MessageKey
  kind: EnvironmentFieldKind
  layout?: EnvironmentFieldLayout
  options?: string[]
  placeholder?: string
}

interface EnvironmentFieldGroup {
  tab: EnvironmentTab
  title: string
  fields: EnvironmentField[]
}

const environmentTypes = ['docker', 'daytona', 'e2b', 'modal', 'runloop', 'langsmith', 'gke', 'novita', 'apple-container', 'singularity', 'islo', 'tensorlake', 'cwsandbox', 'wandb', 'use-computer']
const operatingSystems = ['linux', 'windows']
const resourcePolicies = ['auto', 'limit', 'request', 'guarantee', 'ignore']

const environmentFieldGroups: EnvironmentFieldGroup[] = [
  {
    tab: 'base',
    title: 'OrnnLab template',
    fields: [
      { key: 'name', labelKey: 'environmentName', kind: 'text', layout: 'medium' },
      { key: 'environmentType', labelKey: 'agentType', kind: 'select', layout: 'medium', options: environmentTypes },
      { key: 'importPath', labelKey: 'environmentImportPath', kind: 'path', layout: 'full', placeholder: 'module.path:ClassName' },
    ],
  },
  {
    tab: 'base',
    title: 'Task environment baseline',
    fields: [
      { key: 'dockerImage', labelKey: 'environmentDockerImage', kind: 'text', layout: 'full', placeholder: 'python:3.13-slim or ghcr.io/org/image:tag' },
      { key: 'os', labelKey: 'os', kind: 'select', layout: 'short', options: operatingSystems },
      { key: 'cpus', labelKey: 'cpuCores', kind: 'number', layout: 'short' },
      { key: 'memoryMb', labelKey: 'memoryMb', kind: 'number', layout: 'short' },
      { key: 'storageMb', labelKey: 'storageMb', kind: 'number', layout: 'short' },
      { key: 'gpus', labelKey: 'gpus', kind: 'number', layout: 'short' },
      { key: 'gpuTypes', labelKey: 'gpuTypes', kind: 'tags', layout: 'wide', placeholder: 'A100, H100' },
      { key: 'tpu', labelKey: 'tpu', kind: 'tpu', layout: 'full' },
      { key: 'env', labelKey: 'environmentVariables', kind: 'keyValue', layout: 'full', placeholder: 'KEY=value' },
      { key: 'healthcheck', labelKey: 'healthcheck', kind: 'healthcheck', layout: 'full' },
    ],
  },
  {
    tab: 'advanced',
    title: 'Runtime overrides',
    fields: [
      { key: 'forceBuild', labelKey: 'forceBuild', kind: 'switch', layout: 'medium' },
      { key: 'deleteAfterRun', labelKey: 'deleteAfterRun', kind: 'switch', layout: 'medium' },
      { key: 'cpuPolicy', labelKey: 'resourcePolicy', kind: 'select', layout: 'medium', options: resourcePolicies },
      { key: 'memoryPolicy', labelKey: 'memoryPolicy', kind: 'select', layout: 'medium', options: resourcePolicies },
      { key: 'overrideCpus', labelKey: 'overrideCpus', kind: 'number', layout: 'short' },
      { key: 'overrideMemoryMb', labelKey: 'overrideMemoryMb', kind: 'number', layout: 'short' },
      { key: 'overrideStorageMb', labelKey: 'overrideStorageMb', kind: 'number', layout: 'short' },
      { key: 'overrideGpus', labelKey: 'overrideGpus', kind: 'number', layout: 'short' },
      { key: 'mounts', labelKey: 'mounts', kind: 'json', layout: 'full' },
      { key: 'dockerCompose', labelKey: 'dockerCompose', kind: 'pathList', layout: 'full' },
      { key: 'extraAllowedHosts', labelKey: 'extraAllowedHosts', kind: 'tags', layout: 'wide', placeholder: 'model.internal' },
      { key: 'workdir', labelKey: 'workdir', kind: 'path', layout: 'full', placeholder: '/workspace' },
      { key: 'kwargs', labelKey: 'environmentKwargs', kind: 'keyValue', layout: 'full' },
    ],
  },
]

function environmentTabs(t: Translate): Array<{ key: EnvironmentTab; label: string }> {
  return [
    { key: 'base', label: t('runTabCore') },
    { key: 'network', label: t('environmentNetwork') },
    { key: 'advanced', label: t('agentAdvancedTab') },
  ]
}

export function EnvironmentProfileEditor({
  value,
  t,
  onChange,
}: {
  value: EnvironmentRow
  t: Translate
  onChange: (value: EnvironmentRow) => void
}) {
  const [activeTab, setActiveTab] = useState<EnvironmentTab>('base')
  const setNetworkAccess = (nextValue: string) => {
    if (nextValue === 'none') {
      onChange({ ...value, networkMode: 'no-network' })
      return
    }
    onChange({
      ...value,
      allowedHosts: nextValue,
      networkMode: nextValue.trim() === '*' ? 'public' : 'allowlist',
    })
  }

  return (
    <div className="environment-editor">
      <div className="run-tabs environment-detail-tabs" role="tablist" aria-label={t('selectedEnvironment')}>
        {environmentTabs(t).map((tab) => (
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

      {getEnvironmentGroupsForTab(activeTab).map((group) => (
        <section className="run-config-group" key={group.title}>
          {activeTab !== 'base' && (
            <div className="run-config-group-heading">
              <h3>{getEnvironmentGroupTitle(group.title, t)}</h3>
            </div>
          )}
          <div className="run-grid">
            {group.fields.filter((field) => isEnvironmentFieldVisible(field, value)).map((field) => (
              <EnvironmentFieldControl field={field} key={field.key} t={t} value={value} onChange={onChange} />
            ))}
          </div>
        </section>
      ))}

      {activeTab === 'network' && (
        <section className="run-config-group">
          <div className="run-config-group-heading">
            <h3>{t('environmentNetworkAccess')}</h3>
          </div>
          <div className="run-grid">
            <NetworkAccessControl
              enabledLabel={t('environmentNetworkAccess')}
              hostsLabel={t('environmentAllowedHosts')}
              addLabel={t('add')}
              deleteLabel={t('delete')}
              value={value.networkMode === 'no-network' ? 'none' : value.allowedHosts || '*'}
              onChange={setNetworkAccess}
            />
          </div>
        </section>
      )}
    </div>
  )
}

function getEnvironmentGroupsForTab(activeTab: EnvironmentTab): EnvironmentFieldGroup[] {
  const groups = environmentFieldGroups.filter((group) => group.tab === activeTab)
  if (activeTab !== 'base') return groups
  return [{ tab: 'base', title: 'Base fields', fields: groups.flatMap((group) => group.fields) }]
}

function getEnvironmentGroupTitle(title: string, t: Translate) {
  return t('agentAdvancedTab')
}

function EnvironmentFieldControl({
  field,
  t,
  value,
  onChange,
}: {
  field: EnvironmentField
  t: Translate
  value: EnvironmentRow
  onChange: (value: EnvironmentRow) => void
}) {
  const currentValue = value[field.key]
  const label = t(field.labelKey)
  const className = getEnvironmentFieldClassName(field)
  const setValue = (nextValue: string | boolean) => onChange({ ...value, [field.key]: nextValue })
  if (field.kind === 'switch') {
    return (
      <label className={`switch-control environment-switch ${className}`}>
        <span>{label}</span>
        <input checked={Boolean(currentValue)} onChange={(event) => setValue(event.target.checked)} type="checkbox" />
      </label>
    )
  }
  if (field.kind === 'select') {
    return (
      <label className={className}>
        {label}
        <CustomSelect
          ariaLabel={label}
          value={String(currentValue)}
          options={field.options?.map((option) => ({ label: option, value: option })) ?? []}
          onChange={setValue}
        />
      </label>
    )
  }
  if (field.kind === 'keyValue') {
    return (
      <KeyValueControl
        className={className}
        label={label}
        labels={{ add: t('add'), delete: t('delete'), key: t('formKey'), value: t('value') }}
        value={String(currentValue)}
        onChange={setValue}
      />
    )
  }
  if (field.kind === 'json') {
    return (
      <label className={className}>
        {label}
        <textarea placeholder={field.placeholder} value={String(currentValue)} onChange={(event) => setValue(event.target.value)} />
      </label>
    )
  }
  if (field.kind === 'tpu') {
    return (
      <TpuSpecControl
        className={className}
        label={label}
        labels={{
          notConfigured: t('notConfigured'),
          topologyX: t('topologyX'),
          topologyY: t('topologyY'),
          topologyZ: t('topologyZ'),
          type: t('tpuType'),
        }}
        value={String(currentValue)}
        onChange={setValue}
      />
    )
  }
  if (field.kind === 'healthcheck') {
    return (
      <HealthcheckControl
        className={className}
        label={label}
        labels={{
          command: t('healthcheckCommand'),
          intervalSec: t('healthcheckIntervalSec'),
          retries: t('healthcheckRetries'),
          startIntervalSec: t('healthcheckStartIntervalSec'),
          startPeriodSec: t('healthcheckStartPeriodSec'),
          timeoutSec: t('healthcheckTimeoutSec'),
        }}
        value={String(currentValue)}
        onChange={setValue}
      />
    )
  }
  if (field.kind === 'pathList') {
    return (
      <PathListControl
        className={className}
        label={label}
        addLabel={t('add')}
        deleteLabel={t('delete')}
        value={String(currentValue)}
        onChange={setValue}
      />
    )
  }
  const displayValue = field.kind === 'number' ? normalizeNumberValue(currentValue) : normalizeInputValue(currentValue)
  return (
    <label className={className}>
      {label}
      <input
        inputMode={field.kind === 'number' ? 'numeric' : undefined}
        placeholder={field.placeholder}
        type={field.kind === 'number' ? 'number' : 'text'}
        value={displayValue}
        onChange={(event) => setValue(event.target.value)}
      />
    </label>
  )
}

function getEnvironmentFieldClassName(field: EnvironmentField) {
  return `environment-field environment-field--${field.layout ?? getDefaultFieldLayout(field.kind)}`
}

function getDefaultFieldLayout(kind: EnvironmentFieldKind): EnvironmentFieldLayout {
  if (kind === 'number' || kind === 'select' || kind === 'switch') return 'short'
  if (kind === 'keyValue' || kind === 'json' || kind === 'path' || kind === 'pathList' || kind === 'tpu' || kind === 'healthcheck') return 'full'
  return 'medium'
}

function isEnvironmentFieldVisible(field: EnvironmentField, value: EnvironmentRow) {
  if (field.key === 'allowedHosts') return value.networkMode === 'allowlist'
  return true
}

function normalizeInputValue(value: EnvironmentRow[keyof EnvironmentRow]) {
  return typeof value === 'boolean' ? String(value) : String(value).replace(/^none$/, '')
}

function normalizeNumberValue(value: EnvironmentRow[keyof EnvironmentRow]) {
  const text = normalizeInputValue(value)
  return /^-?\d+(\.\d+)?$/.test(text) ? text : ''
}

interface HealthcheckValue {
  command: string
  interval_sec: string
  timeout_sec: string
  start_period_sec: string
  start_interval_sec: string
  retries: string
}

const defaultHealthcheck: HealthcheckValue = {
  command: '',
  interval_sec: '5',
  timeout_sec: '30',
  start_period_sec: '0',
  start_interval_sec: '5',
  retries: '3',
}

function HealthcheckControl({
  className,
  label,
  labels,
  value,
  onChange,
}: {
  className: string
  label: string
  labels: {
    command: string
    intervalSec: string
    retries: string
    startIntervalSec: string
    startPeriodSec: string
    timeoutSec: string
  }
  value: string
  onChange: (value: string) => void
}) {
  const enabled = isHealthcheckEnabled(value)
  const parsed = parseHealthcheck(value)
  const commit = (nextValue: HealthcheckValue) => onChange(formatHealthcheck(nextValue))
  return (
    <fieldset className={`healthcheck-control ${className}`}>
      <label className="switch-control healthcheck-switch">
        <span>{label}</span>
        <input checked={enabled} onChange={(event) => onChange(event.target.checked ? formatHealthcheck(parsed) : 'none')} type="checkbox" />
      </label>
      {enabled && (
        <div className="healthcheck-grid">
          <label className="environment-field environment-field--full">
            {labels.command}
            <input value={parsed.command} onChange={(event) => commit({ ...parsed, command: event.target.value })} />
          </label>
          <HealthcheckNumberField label={labels.intervalSec} value={parsed.interval_sec} onChange={(interval_sec) => commit({ ...parsed, interval_sec })} />
          <HealthcheckNumberField label={labels.timeoutSec} value={parsed.timeout_sec} onChange={(timeout_sec) => commit({ ...parsed, timeout_sec })} />
          <HealthcheckNumberField label={labels.retries} value={parsed.retries} onChange={(retries) => commit({ ...parsed, retries })} />
          <HealthcheckNumberField label={labels.startPeriodSec} value={parsed.start_period_sec} onChange={(start_period_sec) => commit({ ...parsed, start_period_sec })} />
          <HealthcheckNumberField label={labels.startIntervalSec} value={parsed.start_interval_sec} onChange={(start_interval_sec) => commit({ ...parsed, start_interval_sec })} />
        </div>
      )}
    </fieldset>
  )
}

function HealthcheckNumberField({ label, value, onChange }: { label: string; value: string; onChange: (value: string) => void }) {
  return (
    <label className="environment-field environment-field--short">
      {label}
      <input inputMode="numeric" min="0" type="number" value={value} onChange={(event) => onChange(event.target.value)} />
    </label>
  )
}

function isHealthcheckEnabled(value: string) {
  return value.trim() !== '' && value.trim() !== 'none' && value.trim() !== 'task default'
}

function parseHealthcheck(value: string): HealthcheckValue {
  if (!isHealthcheckEnabled(value)) return defaultHealthcheck
  try {
    const parsed = JSON.parse(value) as Partial<Record<keyof HealthcheckValue, unknown>>
    return {
      command: String(parsed.command ?? ''),
      interval_sec: String(parsed.interval_sec ?? defaultHealthcheck.interval_sec),
      timeout_sec: String(parsed.timeout_sec ?? defaultHealthcheck.timeout_sec),
      start_period_sec: String(parsed.start_period_sec ?? defaultHealthcheck.start_period_sec),
      start_interval_sec: String(parsed.start_interval_sec ?? defaultHealthcheck.start_interval_sec),
      retries: String(parsed.retries ?? defaultHealthcheck.retries),
    }
  } catch {
    return { ...defaultHealthcheck, command: value }
  }
}

function formatHealthcheck(value: HealthcheckValue) {
  return JSON.stringify({
    command: value.command,
    interval_sec: Number(value.interval_sec || defaultHealthcheck.interval_sec),
    timeout_sec: Number(value.timeout_sec || defaultHealthcheck.timeout_sec),
    start_period_sec: Number(value.start_period_sec || defaultHealthcheck.start_period_sec),
    start_interval_sec: Number(value.start_interval_sec || defaultHealthcheck.start_interval_sec),
    retries: Number(value.retries || defaultHealthcheck.retries),
  })
}

function PathListControl({
  addLabel,
  className,
  deleteLabel,
  label,
  value,
  onChange,
}: {
  addLabel: string
  className: string
  deleteLabel: string
  label: string
  value: string
  onChange: (value: string) => void
}) {
  const paths = parsePathList(value)
  const commit = (nextPaths: string[]) => onChange(formatPathList(nextPaths))
  return (
    <div className={`path-list-control ${className}`}>
      <div className="path-list-header">
        <span>{label}</span>
        <button className="secondary-button compact-action" type="button" onClick={() => commit([...paths, ''])}>
          <Plus aria-hidden="true" />
          {addLabel}
        </button>
      </div>
      <div className="path-list-rows">
        {paths.map((path, index) => (
          <div className="path-list-row" key={index}>
            <input
              aria-label={`${label} ${index + 1}`}
              value={path}
              onChange={(event) => commit(paths.map((item, itemIndex) => (itemIndex === index ? event.target.value : item)))}
            />
            <button
              aria-label={`${deleteLabel} ${label} ${index + 1}`}
              className="icon-button"
              type="button"
              onClick={() => commit(paths.filter((_, itemIndex) => itemIndex !== index))}
            >
              <Trash2 aria-hidden="true" />
            </button>
          </div>
        ))}
      </div>
    </div>
  )
}

function parsePathList(value: string) {
  if (!value || value === 'none') return ['']
  return value.split('\n').map((path) => path.trim()).filter(Boolean)
}

function formatPathList(paths: string[]) {
  const cleanPaths = paths.map((path) => path.trim()).filter(Boolean)
  return cleanPaths.length ? cleanPaths.join('\n') : 'none'
}
