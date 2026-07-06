import { useState } from 'react'
import type { EnvironmentRow } from '../../mocks/demo'
import type { MessageKey, Translate } from '../../i18n'
import { CustomSelect } from './CustomSelect'
import { KeyValueControl } from './KeyValueControl'
import { NetworkAccessControl } from './NetworkAccessControl'
import { TpuSpecControl } from './TpuSpecControl'

type EnvironmentTab = 'base' | 'network' | 'advanced'
type EnvironmentFieldKind = 'text' | 'select' | 'number' | 'tags' | 'keyValue' | 'json' | 'path' | 'switch' | 'tpu'

interface EnvironmentField {
  key: keyof EnvironmentRow
  labelKey: MessageKey
  kind: EnvironmentFieldKind
  options?: string[]
  placeholder?: string
}

interface EnvironmentFieldGroup {
  tab: EnvironmentTab
  title: string
  fields: EnvironmentField[]
}

const environmentTypes = ['docker', 'e2b', 'daytona', 'modal', 'runloop', 'langsmith', 'gke', 'novita', 'apple-container', 'singularity', 'islo', 'tensorlake', 'cwsandbox', 'wandb', 'use-computer', 'custom']
const operatingSystems = ['linux', 'windows']
const resourcePolicies = ['auto', 'limit', 'request', 'guarantee', 'ignore']

const environmentFieldGroups: EnvironmentFieldGroup[] = [
  {
    tab: 'base',
    title: 'OrnnLab template',
    fields: [
      { key: 'name', labelKey: 'environmentName', kind: 'text' },
      { key: 'environmentType', labelKey: 'agentType', kind: 'select', options: environmentTypes },
      { key: 'importPath', labelKey: 'environmentImportPath', kind: 'path', placeholder: 'module.path:ClassName' },
    ],
  },
  {
    tab: 'base',
    title: 'Task environment baseline',
    fields: [
      { key: 'dockerImage', labelKey: 'environmentDockerImage', kind: 'text', placeholder: 'python:3.13-slim' },
      { key: 'os', labelKey: 'os', kind: 'select', options: operatingSystems },
      { key: 'cpus', labelKey: 'cpuCores', kind: 'number' },
      { key: 'memoryMb', labelKey: 'memoryMb', kind: 'number' },
      { key: 'storageMb', labelKey: 'storageMb', kind: 'number' },
      { key: 'gpus', labelKey: 'gpus', kind: 'number' },
      { key: 'gpuTypes', labelKey: 'gpuTypes', kind: 'tags', placeholder: 'A100, H100' },
      { key: 'tpu', labelKey: 'tpu', kind: 'tpu' },
      { key: 'env', labelKey: 'environmentVariables', kind: 'keyValue', placeholder: 'KEY=value' },
      { key: 'healthcheck', labelKey: 'healthcheck', kind: 'json' },
      { key: 'workdir', labelKey: 'workdir', kind: 'path' },
    ],
  },
  {
    tab: 'advanced',
    title: 'Runtime overrides',
    fields: [
      { key: 'forceBuild', labelKey: 'forceBuild', kind: 'switch' },
      { key: 'deleteAfterRun', labelKey: 'deleteAfterRun', kind: 'switch' },
      { key: 'cpuPolicy', labelKey: 'resourcePolicy', kind: 'select', options: resourcePolicies },
      { key: 'memoryPolicy', labelKey: 'memoryPolicy', kind: 'select', options: resourcePolicies },
      { key: 'overrideCpus', labelKey: 'overrideCpus', kind: 'number' },
      { key: 'overrideMemoryMb', labelKey: 'overrideMemoryMb', kind: 'number' },
      { key: 'overrideStorageMb', labelKey: 'overrideStorageMb', kind: 'number' },
      { key: 'overrideGpus', labelKey: 'overrideGpus', kind: 'number' },
      { key: 'overrideTpu', labelKey: 'overrideTpu', kind: 'tpu' },
      { key: 'mounts', labelKey: 'mounts', kind: 'json' },
      { key: 'dockerCompose', labelKey: 'dockerCompose', kind: 'path' },
      { key: 'extraAllowedHosts', labelKey: 'extraAllowedHosts', kind: 'tags', placeholder: 'model.internal' },
      { key: 'kwargs', labelKey: 'environmentKwargs', kind: 'keyValue' },
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

      {environmentFieldGroups.filter((group) => group.tab === activeTab).map((group) => (
        <section className="run-config-group" key={group.title}>
          <div className="run-config-group-heading">
            <h3>{getEnvironmentGroupTitle(group.title, t)}</h3>
          </div>
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
              value={value.networkMode === 'no-network' ? 'none' : value.allowedHosts || '*'}
              onChange={setNetworkAccess}
            />
          </div>
        </section>
      )}
    </div>
  )
}

function getEnvironmentGroupTitle(title: string, t: Translate) {
  if (title === 'OrnnLab template') return t('runTabCore')
  if (title === 'Task environment baseline') return t('runTabEnvironment')
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
  const setValue = (nextValue: string | boolean) => onChange({ ...value, [field.key]: nextValue })
  if (field.kind === 'switch') {
    return (
      <label className="switch-control environment-switch">
        <span>{label}</span>
        <input checked={Boolean(currentValue)} onChange={(event) => setValue(event.target.checked)} type="checkbox" />
      </label>
    )
  }
  if (field.kind === 'select') {
    return (
      <label>
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
        label={label}
        labels={{ add: t('add'), delete: t('delete'), key: t('formKey'), value: t('value') }}
        value={String(currentValue)}
        onChange={setValue}
      />
    )
  }
  if (field.kind === 'json') {
    return (
      <label className="field-wide">
        {label}
        <textarea placeholder={field.placeholder} value={String(currentValue)} onChange={(event) => setValue(event.target.value)} />
      </label>
    )
  }
  if (field.kind === 'tpu') {
    return (
      <TpuSpecControl
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
  const displayValue = field.kind === 'number' ? normalizeNumberValue(currentValue) : normalizeInputValue(currentValue)
  return (
    <label>
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

function isEnvironmentFieldVisible(field: EnvironmentField, value: EnvironmentRow) {
  if (field.key === 'importPath') return value.environmentType === 'custom'
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
