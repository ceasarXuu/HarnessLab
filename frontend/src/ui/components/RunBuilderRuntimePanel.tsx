import type { RunDraft } from '../../domain/harbor'
import type { Translate } from '../../i18n'
import { useEffect, useState } from 'react'
import { CustomSelect } from './CustomSelect'
import { EditableStringList } from './EditableStringList'
import { Field } from './RunBuilderChrome'

interface RuntimePanelProps {
  draft: RunDraft
  t: Translate
  onDraft: (draft: RunDraft) => void
}

type TimeoutPolicy = 'standard' | 'strict' | 'relaxed' | 'custom'
type RetryIntervalPolicy = 'standard' | 'fast' | 'slow' | 'custom'

const retryScenarios = [
  { key: 'timeout', value: 'TimeoutError' },
  { key: 'network', value: 'NetworkError' },
  { key: 'environment', value: 'EnvironmentError' },
  { key: 'verifier', value: 'VerifierTransientError' },
] as const

export function RunBuilderRuntimePanel({ draft, t, onDraft }: RuntimePanelProps) {
  const labels = {
    addRule: t('runtimeAddRule'),
    advancedGroup: t('runtimeAdvancedGroup'),
    agentSetupTimeoutMultiplier: t('runtimeAgentSetupTimeoutMultiplier'),
    agentTimeoutMultiplier: t('runtimeAgentTimeoutMultiplier'),
    collapseAdvanced: t('runtimeCollapseAdvanced'),
    deleteRule: t('runtimeDeleteRule'),
    environmentBuildTimeoutMultiplier: t('runtimeEnvironmentBuildTimeoutMultiplier'),
    expandAdvanced: t('runtimeExpandAdvanced'),
    maxRetries: t('runtimeMaxRetries'),
    retryCustom: t('runtimeRetryCustom'),
    retryEnabled: t('runtimeRetryEnabled'),
    retryExclude: t('runtimeRetryExclude'),
    retryFast: t('runtimeRetryFast'),
    retryGroup: t('runtimeRetryGroup'),
    retryInterval: t('runtimeRetryInterval'),
    retryMaxWaitSec: t('runtimeRetryMaxWaitSec'),
    retryMinWaitSec: t('runtimeRetryMinWaitSec'),
    retryScenarios: t('runtimeRetryScenarios'),
    retrySlow: t('runtimeRetrySlow'),
    retryStandard: t('runtimeRetryStandard'),
    retryWaitMultiplier: t('runtimeRetryWaitMultiplier'),
    scenarios: {
      environment: t('runtimeRetryScenarioEnvironment'),
      network: t('runtimeRetryScenarioNetwork'),
      timeout: t('runtimeRetryScenarioTimeout'),
      verifier: t('runtimeRetryScenarioVerifier'),
    },
    timeoutCustom: t('runtimeTimeoutCustom'),
    timeoutGroup: t('runtimeTimeoutGroup'),
    timeoutMultiplier: t('runtimeTimeoutMultiplier'),
    timeoutPolicy: t('runtimeTimeoutPolicy'),
    timeoutRelaxed: t('runtimeTimeoutRelaxed'),
    timeoutStandard: t('runtimeTimeoutStandard'),
    timeoutStrict: t('runtimeTimeoutStrict'),
    verifierTimeoutMultiplier: t('runtimeVerifierTimeoutMultiplier'),
  }
  const [advancedOpen, setAdvancedOpen] = useState(false)
  const timeoutPolicy = draft.timeoutPolicy
  const retryEnabled = draft.maxRetries > 0
  const retryIntervalPolicy = draft.retryIntervalPolicy
  const selectedRetryScenarios = new Set(splitRules(draft.retryInclude))

  const setTimeoutPolicy = (policy: TimeoutPolicy) => {
    if (policy === 'standard') {
      onDraft({ ...draft, timeoutPolicy: policy, timeoutMultiplier: 1, agentTimeoutMultiplier: '1', verifierTimeoutMultiplier: '1' })
    } else if (policy === 'strict') {
      onDraft({ ...draft, timeoutPolicy: policy, timeoutMultiplier: 0.5, agentTimeoutMultiplier: '1', verifierTimeoutMultiplier: '1' })
    } else if (policy === 'relaxed') {
      onDraft({ ...draft, timeoutPolicy: policy, timeoutMultiplier: 2, agentTimeoutMultiplier: '1', verifierTimeoutMultiplier: '1' })
    } else {
      onDraft({ ...draft, timeoutPolicy: policy, timeoutMultiplier: draft.timeoutMultiplier || 1 })
    }
  }

  const setRetryIntervalPolicy = (policy: RetryIntervalPolicy) => {
    if (policy === 'standard') {
      onDraft({ ...draft, retryIntervalPolicy: policy, retryWaitMultiplier: '1.5', retryMinWaitSec: '2', retryMaxWaitSec: '30' })
    } else if (policy === 'fast') {
      onDraft({ ...draft, retryIntervalPolicy: policy, retryWaitMultiplier: '1', retryMinWaitSec: '0', retryMaxWaitSec: '5' })
    } else if (policy === 'slow') {
      onDraft({ ...draft, retryIntervalPolicy: policy, retryWaitMultiplier: '2', retryMinWaitSec: '10', retryMaxWaitSec: '120' })
    } else {
      onDraft({ ...draft, retryIntervalPolicy: policy })
    }
  }

  const setRetryScenario = (value: string, enabled: boolean) => {
    const next = new Set(selectedRetryScenarios)
    if (enabled) {
      next.add(value)
    } else {
      next.delete(value)
    }
    onDraft({ ...draft, retryInclude: Array.from(next).join(',') })
  }

  const setRetryEnabled = (enabled: boolean) => {
    if (enabled) {
      onDraft({ ...draft, maxRetries: Math.max(draft.maxRetries, 1), retryInclude: draft.retryInclude || 'TimeoutError' })
    } else {
      onDraft({ ...draft, maxRetries: 0, retryInclude: '' })
    }
  }

  return (
    <div className="run-config-groups">
      <section className="run-config-group">
        <div className="run-config-group-heading">
          <h3>{labels.timeoutGroup}</h3>
        </div>
        <div className="run-grid">
          <label>
            {labels.timeoutPolicy}
            <CustomSelect
              ariaLabel={labels.timeoutPolicy}
              value={timeoutPolicy}
              options={[
                { label: labels.timeoutStandard, value: 'standard' },
                { label: labels.timeoutStrict, value: 'strict' },
                { label: labels.timeoutRelaxed, value: 'relaxed' },
                { label: labels.timeoutCustom, value: 'custom' },
              ]}
              onChange={(value) => setTimeoutPolicy(value as TimeoutPolicy)}
            />
          </label>
          {timeoutPolicy === 'custom' && (
            <>
              <Field label={labels.timeoutMultiplier}>
                <input
                  type="number"
                  min="0.1"
                  step="0.1"
                  value={draft.timeoutMultiplier}
                  onChange={(event) => onDraft({ ...draft, timeoutMultiplier: Number(event.target.value) })}
                />
              </Field>
              <Field label={labels.agentTimeoutMultiplier}>
                <input
                  type="number"
                  min="0.1"
                  step="0.1"
                  value={draft.agentTimeoutMultiplier}
                  onChange={(event) => onDraft({ ...draft, agentTimeoutMultiplier: event.target.value })}
                />
              </Field>
              <Field label={labels.verifierTimeoutMultiplier}>
                <input
                  type="number"
                  min="0.1"
                  step="0.1"
                  value={draft.verifierTimeoutMultiplier}
                  onChange={(event) => onDraft({ ...draft, verifierTimeoutMultiplier: event.target.value })}
                />
              </Field>
            </>
          )}
        </div>
      </section>

      <section className="run-config-group">
        <div className="run-config-group-heading runtime-section-heading">
          <h3>{labels.retryGroup}</h3>
          <label className="switch-control runtime-section-switch">
            <span>{labels.retryEnabled}</span>
            <input
              aria-label={labels.retryEnabled}
              checked={retryEnabled}
              onChange={(event) => setRetryEnabled(event.target.checked)}
              type="checkbox"
            />
          </label>
        </div>
        {retryEnabled && (
          <div className="run-grid">
            <Field label={labels.maxRetries}>
              <input
                type="number"
                min="1"
                value={draft.maxRetries}
                onChange={(event) => onDraft({ ...draft, maxRetries: Number(event.target.value) })}
              />
            </Field>
            <label>
              {labels.retryInterval}
              <CustomSelect
                ariaLabel={labels.retryInterval}
                value={retryIntervalPolicy}
                options={[
                  { label: labels.retryStandard, value: 'standard' },
                  { label: labels.retryFast, value: 'fast' },
                  { label: labels.retrySlow, value: 'slow' },
                  { label: labels.retryCustom, value: 'custom' },
                ]}
                onChange={(value) => setRetryIntervalPolicy(value as RetryIntervalPolicy)}
              />
            </label>
            <fieldset className="runtime-checklist field-wide">
              <legend>{labels.retryScenarios}</legend>
              {retryScenarios.map((scenario) => (
                <label key={scenario.value}>
                  <input
                    type="checkbox"
                    checked={selectedRetryScenarios.has(scenario.value)}
                    onChange={(event) => setRetryScenario(scenario.value, event.target.checked)}
                  />
                  {labels.scenarios[scenario.key]}
                </label>
              ))}
            </fieldset>
            {retryIntervalPolicy === 'custom' && (
              <>
                <Field label={labels.retryWaitMultiplier}>
                  <input
                    type="number"
                    min="0"
                    step="0.1"
                    value={draft.retryWaitMultiplier}
                    onChange={(event) => onDraft({ ...draft, retryWaitMultiplier: event.target.value })}
                  />
                </Field>
                <Field label={labels.retryMinWaitSec}>
                  <input
                    type="number"
                    min="0"
                    value={draft.retryMinWaitSec}
                    onChange={(event) => onDraft({ ...draft, retryMinWaitSec: event.target.value })}
                  />
                </Field>
                <Field label={labels.retryMaxWaitSec}>
                  <input
                    type="number"
                    min="0"
                    value={draft.retryMaxWaitSec}
                    onChange={(event) => onDraft({ ...draft, retryMaxWaitSec: event.target.value })}
                  />
                </Field>
              </>
            )}
          </div>
        )}
      </section>

      <section className="run-config-group">
        <div className="run-config-group-heading">
          <button
            type="button"
            className="runtime-collapsible-trigger"
            aria-expanded={advancedOpen}
            aria-label={advancedOpen ? labels.collapseAdvanced : labels.expandAdvanced}
            onClick={() => setAdvancedOpen((current) => !current)}
          >
            <span className="runtime-collapsible-title">{labels.advancedGroup}</span>
            <span className="runtime-collapsible-icon" aria-hidden="true" />
          </button>
        </div>
        {advancedOpen && (
          <div className="run-grid">
            <Field label={labels.agentSetupTimeoutMultiplier}>
              <input
                type="number"
                min="0.1"
                step="0.1"
                value={draft.agentSetupTimeoutMultiplier}
                onChange={(event) => onDraft({ ...draft, agentSetupTimeoutMultiplier: event.target.value })}
              />
            </Field>
            <Field label={labels.environmentBuildTimeoutMultiplier}>
              <input
                type="number"
                min="0.1"
                step="0.1"
                value={draft.environmentBuildTimeoutMultiplier}
                onChange={(event) => onDraft({ ...draft, environmentBuildTimeoutMultiplier: event.target.value })}
              />
            </Field>
            <RuleListControl
              label={labels.retryExclude}
              addLabel={labels.addRule}
              deleteLabel={labels.deleteRule}
              value={draft.retryExclude}
              onChange={(value) => onDraft({ ...draft, retryExclude: value })}
            />
          </div>
        )}
      </section>
    </div>
  )
}

function splitRules(value: string) {
  return value.split(',').map((item) => item.trim()).filter(Boolean)
}

function formatRules(rules: string[]) {
  return rules.map((rule) => rule.trim()).filter(Boolean).join(',')
}

function RuleListControl({
  addLabel,
  deleteLabel,
  label,
  onChange,
  value,
}: {
  addLabel: string
  deleteLabel: string
  label: string
  onChange: (value: string) => void
  value: string
}) {
  const [rows, setRows] = useState(() => {
    const initialRows = splitRules(value)
    return initialRows.length ? initialRows : ['']
  })

  useEffect(() => {
    const nextRows = splitRules(value)
    setRows(nextRows.length ? nextRows : [''])
  }, [value])

  const commit = (nextRows: string[]) => {
    setRows(nextRows)
    onChange(formatRules(nextRows))
  }

  return (
    <EditableStringList
      addLabel={addLabel}
      className="rule-list-control field-wide"
      deleteLabel={deleteLabel}
      itemAriaLabel={(_, index) => `${label} ${index + 1}`}
      label={label}
      values={rows}
      onChange={commit}
    />
  )
}
