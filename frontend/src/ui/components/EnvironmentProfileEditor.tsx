import { useState } from 'react'
import type { EnvironmentRow } from '../../mocks/demo'
import type { Translate } from '../../i18n'
import { CustomSelect } from './CustomSelect'
import { KeyValueControl } from './KeyValueControl'
import { NetworkAccessControl } from './NetworkAccessControl'
import { TpuSpecControl } from './TpuSpecControl'

type EnvironmentTab = 'base' | 'network' | 'advanced'
type EnvironmentFieldKind = 'text' | 'select' | 'number' | 'tags' | 'keyValue' | 'json' | 'path' | 'switch' | 'tpu'

interface EnvironmentField {
  key: keyof EnvironmentRow
  label: string
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
      { key: 'name', label: 'Environment Name', kind: 'text' },
      { key: 'environmentType', label: 'type', kind: 'select', options: environmentTypes },
      { key: 'importPath', label: 'import_path', kind: 'path', placeholder: 'module.path:ClassName' },
    ],
  },
  {
    tab: 'base',
    title: 'Task environment baseline',
    fields: [
      { key: 'dockerImage', label: 'docker_image', kind: 'text', placeholder: 'python:3.13-slim' },
      { key: 'os', label: 'os', kind: 'select', options: operatingSystems },
      { key: 'cpus', label: 'cpus', kind: 'number' },
      { key: 'memoryMb', label: 'memory_mb', kind: 'number' },
      { key: 'storageMb', label: 'storage_mb', kind: 'number' },
      { key: 'gpus', label: 'gpus', kind: 'number' },
      { key: 'gpuTypes', label: 'gpu_types', kind: 'tags', placeholder: 'A100, H100' },
      { key: 'tpu', label: 'tpu', kind: 'tpu' },
      { key: 'env', label: 'env', kind: 'keyValue', placeholder: 'KEY=value' },
      { key: 'healthcheck', label: 'healthcheck', kind: 'json' },
      { key: 'workdir', label: 'workdir', kind: 'path' },
    ],
  },
  {
    tab: 'advanced',
    title: 'Runtime overrides',
    fields: [
      { key: 'forceBuild', label: 'force_build', kind: 'switch' },
      { key: 'deleteAfterRun', label: 'delete', kind: 'switch' },
      { key: 'cpuPolicy', label: 'cpu_enforcement_policy', kind: 'select', options: resourcePolicies },
      { key: 'memoryPolicy', label: 'memory_enforcement_policy', kind: 'select', options: resourcePolicies },
      { key: 'overrideCpus', label: 'override_cpus', kind: 'number' },
      { key: 'overrideMemoryMb', label: 'override_memory_mb', kind: 'number' },
      { key: 'overrideStorageMb', label: 'override_storage_mb', kind: 'number' },
      { key: 'overrideGpus', label: 'override_gpus', kind: 'number' },
      { key: 'overrideTpu', label: 'override_tpu', kind: 'tpu' },
      { key: 'mounts', label: 'mounts', kind: 'json' },
      { key: 'dockerCompose', label: 'extra_docker_compose', kind: 'path' },
      { key: 'extraAllowedHosts', label: 'extra_allowed_hosts', kind: 'tags', placeholder: 'model.internal' },
      { key: 'kwargs', label: 'kwargs', kind: 'keyValue' },
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
              <EnvironmentFieldControl field={field} key={field.key} value={value} onChange={onChange} />
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
  value,
  onChange,
}: {
  field: EnvironmentField
  value: EnvironmentRow
  onChange: (value: EnvironmentRow) => void
}) {
  const currentValue = value[field.key]
  const setValue = (nextValue: string | boolean) => onChange({ ...value, [field.key]: nextValue })
  if (field.kind === 'switch') {
    return (
      <label className="switch-control environment-switch">
        <span>{field.label}</span>
        <input checked={Boolean(currentValue)} onChange={(event) => setValue(event.target.checked)} type="checkbox" />
      </label>
    )
  }
  if (field.kind === 'select') {
    return (
      <label>
        {field.label}
        <CustomSelect
          ariaLabel={field.label}
          value={String(currentValue)}
          options={field.options?.map((option) => ({ label: option, value: option })) ?? []}
          onChange={setValue}
        />
      </label>
    )
  }
  if (field.kind === 'keyValue') {
    return <KeyValueControl label={field.label} value={String(currentValue)} onChange={setValue} />
  }
  if (field.kind === 'json') {
    return (
      <label className="field-wide">
        {field.label}
        <textarea placeholder={field.placeholder} value={String(currentValue)} onChange={(event) => setValue(event.target.value)} />
      </label>
    )
  }
  if (field.kind === 'tpu') {
    return <TpuSpecControl label={field.label} value={String(currentValue)} onChange={setValue} />
  }
  const displayValue = field.kind === 'number' ? normalizeNumberValue(currentValue) : normalizeInputValue(currentValue)
  return (
    <label>
      {field.label}
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
