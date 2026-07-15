import type { AgentParameter, AgentRow } from '../../domain/harbor'
import type { Translate } from '../../i18n'
import { CustomSelect } from './CustomSelect'
import { Metric } from './Metric'
import { SwitchControl } from './SwitchControl'

interface AgentHarnessParametersProps {
  readOnly?: boolean
  t: Translate
  value: AgentRow
  onChange: (value: AgentRow) => void
}

export function AgentHarnessParameters({ readOnly = false, t, value, onChange }: AgentHarnessParametersProps) {
  const parameters = value.capabilities?.parameters ?? []
  if (!parameters.length) return null
  const setParameter = (parameter: AgentParameter, nextValue: string | boolean) => {
    if (readOnly) return
    const field = parameter.source === 'env' ? 'env' : 'kwargs'
    onChange({ ...value, [field]: setKeyValue(value[field] ?? 'none', parameter.key, String(nextValue)) })
  }
  return (
    <div className="field-wide harness-parameter-grid">
      <h4>{t('harnessParameters')}</h4>
      {parameters.map((parameter) => (
        readOnly ? (
          <Metric
            key={`${parameter.source}-${parameter.key}`}
            label={parameter.label}
            value={getReadonlyParameterValue(value, parameter, t)}
          />
        ) : (
          <HarnessParameterInput
            key={`${parameter.source}-${parameter.key}`}
            parameter={parameter}
            readOnly={readOnly}
            value={getParameterValue(value, parameter)}
            onChange={(nextValue) => setParameter(parameter, nextValue)}
          />
        )
      ))}
    </div>
  )
}

function HarnessParameterInput({
  parameter,
  readOnly,
  value,
  onChange,
}: {
  parameter: AgentParameter
  readOnly: boolean
  value: string
  onChange: (value: string | boolean) => void
}) {
  if (parameter.kind === 'boolean') {
    return (
      <SwitchControl
        checked={value === 'true'}
        disabled={readOnly}
        label={parameter.label}
        onChange={onChange}
      />
    )
  }
  if (parameter.choices?.length) {
    return (
      <label>
        {parameter.label}
        <CustomSelect
          ariaLabel={parameter.label}
          disabled={readOnly}
          options={[{ label: '', value: '' }, ...parameter.choices.map((choice) => ({ label: choice, value: choice }))]}
          value={value}
          onChange={onChange}
        />
      </label>
    )
  }
  return (
    <label>
      {parameter.label}
      <input
        readOnly={readOnly}
        type={parameter.kind === 'number' ? 'number' : 'text'}
        value={value}
        onChange={(event) => onChange(event.target.value)}
      />
    </label>
  )
}

function getParameterValue(agent: AgentRow, parameter: AgentParameter) {
  const raw = parameter.source === 'env' ? agent.env : agent.kwargs
  const current = parseKeyValues(raw ?? 'none').get(parameter.key)
  return current ?? (parameter.defaultValue === undefined ? '' : String(parameter.defaultValue))
}

function getReadonlyParameterValue(agent: AgentRow, parameter: AgentParameter, t: Translate) {
  const value = getParameterValue(agent, parameter)
  if (value) return value
  return parameter.defaultValue === undefined ? t('configuredAtJobRun') : String(parameter.defaultValue)
}

function parseKeyValues(value: string) {
  const rows = new Map<string, string>()
  if (!value || value === 'none') return rows
  value.split('\n').forEach((line) => {
    const [key, ...rest] = line.split('=')
    const cleanKey = key.trim()
    if (cleanKey) rows.set(cleanKey, rest.join('=').trim())
  })
  return rows
}

function setKeyValue(value: string, key: string, nextValue: string) {
  const rows = parseKeyValues(value)
  if (nextValue.trim()) rows.set(key, nextValue.trim())
  else rows.delete(key)
  const formatted = [...rows.entries()].map(([rowKey, rowValue]) => `${rowKey}=${rowValue}`)
  return formatted.length ? formatted.join('\n') : 'none'
}
