import type { AgentCapabilities, AgentRow } from '../../domain/harbor'
import type { Translate } from '../../i18n'
import { CustomSelect } from './CustomSelect'
import { KeyValueControl } from './KeyValueControl'
import { ReadonlyKeyValueList } from './ReadonlyDisplay'

interface AgentEnvironmentVariablesProps {
  capabilities: AgentCapabilities
  readOnly: boolean
  t: Translate
  value: AgentRow
  onChange: (value: AgentRow) => void
}

export function AgentEnvironmentVariables({ capabilities, readOnly, t, value, onChange }: AgentEnvironmentVariablesProps) {
  const authenticationMode = value.authenticationMode
    ?? capabilities.authenticationModes[0]?.value
  const selectedMode = capabilities.authenticationModes.find((mode) => mode.value === authenticationMode)
  const keyOptions = [...capabilities.environmentVariables, ...(selectedMode?.environmentVariables ?? [])]
  const updateAuthenticationMode = (nextMode: string) => onChange({
    ...value,
    authenticationMode: nextMode,
    env: filterEnvironmentForMode(value.env ?? 'none', capabilities, nextMode),
  })

  return (
    <section className="surface rail-card">
      <div className="rail-title"><h3>{t('environmentVariables')}</h3></div>
      <div className="agent-form-grid">
        {capabilities.authenticationModes.length > 0 && (
          <label>
            {t('agentAuthenticationMethod')}
            <CustomSelect
              ariaLabel={t('agentAuthenticationMethod')}
              disabled={readOnly}
              options={capabilities.authenticationModes.map((mode) => ({ label: mode.label, value: mode.value }))}
              value={authenticationMode ?? ''}
              onChange={updateAuthenticationMode}
            />
          </label>
        )}
        {readOnly ? (
          <ReadonlyKeyValueList label={t('genericAgentEnv')} value={value.env} emptyLabel={t('supportedByHarness')} />
        ) : (
          <div className="field-wide">
            <KeyValueControl
              allowInherited
              compact
              keyOptions={keyOptions}
              label={t('genericAgentEnv')}
              labels={envKeyValueLabels(t)}
              value={value.env ?? 'none'}
              onChange={(env) => onChange({ ...value, env })}
            />
          </div>
        )}
      </div>
    </section>
  )
}

function filterEnvironmentForMode(value: string, capabilities: AgentCapabilities, mode: string) {
  if (!value || value === 'none') return 'none'
  const modeVariables = new Set(capabilities.authenticationModes.flatMap((item) => item.environmentVariables))
  const allowed = new Set(
    capabilities.authenticationModes.find((item) => item.value === mode)?.environmentVariables ?? [],
  )
  const rows = value.split('\n').filter((line) => {
    const key = line.split('=', 1)[0]?.trim()
    return !modeVariables.has(key) || allowed.has(key)
  })
  return rows.length ? rows.join('\n') : 'none'
}

function envKeyValueLabels(t: Translate) {
  return {
    add: t('add'), customKey: t('customEnvironmentVariable'), delete: t('delete'),
    inherited: t('envSourceInherited'), key: t('envKey'), literal: t('envSourceLiteral'),
    searchKeys: t('searchEnvironmentVariables'), source: t('envValueSource'), value: t('envValue'),
  }
}
