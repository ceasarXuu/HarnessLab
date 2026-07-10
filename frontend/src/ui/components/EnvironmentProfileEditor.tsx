import { useState } from 'react'
import type { EnvironmentRow } from '../../domain/harbor'
import type { MessageKey, Translate } from '../../i18n'
import { CustomSelect } from './CustomSelect'
import { EditableStringList } from './EditableStringList'
import { KeyValueControl } from './KeyValueControl'
import { TpuSpecControl } from './TpuSpecControl'

type EnvironmentTab = 'base' | 'network' | 'advanced'
type FieldKind = 'number' | 'select' | 'switch' | 'text' | 'tpu'

interface Field {
  key: keyof EnvironmentRow
  labelKey: MessageKey
  kind: FieldKind
  options?: string[]
  wide?: boolean
}

const environmentTypes = [
  'docker', 'daytona', 'e2b', 'modal', 'runloop', 'langsmith', 'gke', 'novita',
  'apple-container', 'singularity', 'islo', 'tensorlake', 'cwsandbox', 'wandb', 'use-computer',
]
const resourcePolicies = ['auto', 'limit', 'request', 'guarantee', 'ignore']

const baseFields: Field[] = [
  { key: 'name', labelKey: 'environmentName', kind: 'text' },
  { key: 'environmentType', labelKey: 'agentType', kind: 'select', options: environmentTypes },
  { key: 'importPath', labelKey: 'environmentImportPath', kind: 'text', wide: true },
]
const advancedFields: Field[] = [
  { key: 'forceBuild', labelKey: 'forceBuild', kind: 'switch' },
  { key: 'deleteAfterRun', labelKey: 'deleteAfterRun', kind: 'switch' },
  { key: 'cpuPolicy', labelKey: 'resourcePolicy', kind: 'select', options: resourcePolicies },
  { key: 'memoryPolicy', labelKey: 'memoryPolicy', kind: 'select', options: resourcePolicies },
  { key: 'overrideCpus', labelKey: 'overrideCpus', kind: 'number' },
  { key: 'overrideMemoryMb', labelKey: 'overrideMemoryMb', kind: 'number' },
  { key: 'overrideStorageMb', labelKey: 'overrideStorageMb', kind: 'number' },
  { key: 'overrideGpus', labelKey: 'overrideGpus', kind: 'number' },
  { key: 'overrideTpu', labelKey: 'tpu', kind: 'tpu', wide: true },
]

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
  const setField = (key: keyof EnvironmentRow, nextValue: string | boolean) => onChange({ ...value, [key]: nextValue })

  return (
    <div className="environment-editor">
      <div className="run-tabs environment-detail-tabs" role="tablist" aria-label={t('selectedEnvironment')}>
        {[
          { key: 'base' as const, label: t('runTabCore') },
          { key: 'network' as const, label: t('environmentNetwork') },
          { key: 'advanced' as const, label: t('agentAdvancedTab') },
        ].map((tab) => (
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

      {activeTab === 'base' && (
        <section className="run-config-group">
          <div className="run-grid">
            {baseFields.map((field) => <EnvironmentFieldControl field={field} key={field.key} t={t} value={value} onChange={setField} />)}
            <KeyValueControl
              className="field-wide"
              label={t('environmentVariables')}
              labels={{ add: t('add'), delete: t('delete'), key: t('envKey'), value: t('envValue') }}
              value={value.env}
              onChange={(nextValue) => setField('env', nextValue)}
            />
          </div>
        </section>
      )}

      {activeTab === 'network' && (
        <section className="run-config-group">
          <div className="run-grid">
            <EditableStringList
              addLabel={t('add')}
              className="field-wide"
              deleteLabel={t('delete')}
              itemAriaLabel={() => t('environmentAllowedHosts')}
              label={t('environmentAllowedHosts')}
              values={parseList(value.allowedHosts)}
              onChange={(items) => setField('allowedHosts', formatList(items))}
            />
          </div>
        </section>
      )}

      {activeTab === 'advanced' && (
        <section className="run-config-group">
          <div className="run-grid">
            {advancedFields.map((field) => <EnvironmentFieldControl field={field} key={field.key} t={t} value={value} onChange={setField} />)}
            <label className="field-wide">
              {t('mounts')}
              <textarea value={value.mounts} onChange={(event) => setField('mounts', event.target.value)} />
            </label>
            <EditableStringList
              addLabel={t('add')}
              className="field-wide"
              deleteLabel={t('delete')}
              itemAriaLabel={() => t('dockerCompose')}
              label={t('dockerCompose')}
              values={parseList(value.dockerCompose)}
              onChange={(items) => setField('dockerCompose', formatList(items))}
            />
            <KeyValueControl
              className="field-wide"
              label={t('environmentKwargs')}
              labels={{ add: t('add'), delete: t('delete'), key: t('formKey'), value: t('value') }}
              value={value.kwargs}
              onChange={(nextValue) => setField('kwargs', nextValue)}
            />
          </div>
        </section>
      )}
    </div>
  )
}

function EnvironmentFieldControl({
  field,
  t,
  value,
  onChange,
}: {
  field: Field
  t: Translate
  value: EnvironmentRow
  onChange: (key: keyof EnvironmentRow, value: string | boolean) => void
}) {
  const current = value[field.key]
  const label = t(field.labelKey)
  const className = field.wide ? 'field-wide' : undefined
  if (field.kind === 'switch') {
    return <label className={`switch-control ${className ?? ''}`}><span>{label}</span><input checked={Boolean(current)} type="checkbox" onChange={(event) => onChange(field.key, event.target.checked)} /></label>
  }
  if (field.kind === 'select') {
    return <label className={className}>{label}<CustomSelect ariaLabel={label} value={String(current)} options={(field.options ?? []).map((item) => ({ label: item, value: item }))} onChange={(nextValue) => onChange(field.key, nextValue)} /></label>
  }
  if (field.kind === 'tpu') {
    return <TpuSpecControl className={className} label={label} labels={{ notConfigured: t('notConfigured'), topologyX: t('topologyX'), topologyY: t('topologyY'), topologyZ: t('topologyZ'), type: t('tpuType') }} value={String(current)} onChange={(nextValue) => onChange(field.key, nextValue)} />
  }
  return <label className={className}>{label}<input inputMode={field.kind === 'number' ? 'numeric' : undefined} type={field.kind === 'number' ? 'number' : 'text'} value={normaliseInput(current)} onChange={(event) => onChange(field.key, event.target.value)} /></label>
}

function parseList(value: string) {
  return value && value !== 'none' ? value.split(/\n|,/).map((item) => item.trim()).filter(Boolean) : ['']
}

function formatList(values: string[]) {
  const filtered = values.map((item) => item.trim()).filter(Boolean)
  return filtered.length ? filtered.join('\n') : 'none'
}

function normaliseInput(value: EnvironmentRow[keyof EnvironmentRow]) {
  return typeof value === 'boolean' ? String(value) : String(value).replace(/^none$/, '')
}
