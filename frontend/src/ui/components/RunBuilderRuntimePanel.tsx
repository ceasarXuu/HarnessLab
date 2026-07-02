import type { RunDraft } from '../../mocks/demo'
import type { Translate } from '../../i18n'
import { useState } from 'react'
import { CustomSelect } from './CustomSelect'
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
  const labels = runtimeLabels(t('runTabRuntime') === '运行策略')
  const [advancedOpen, setAdvancedOpen] = useState(false)
  const timeoutPolicy = draft.timeoutPolicy
  const retryIntervalPolicy = draft.retryIntervalPolicy
  const selectedRetryScenarios = new Set(splitRules(draft.retryInclude))

  const setTimeoutPolicy = (policy: TimeoutPolicy) => {
    if (policy === 'standard') {
      onDraft({ ...draft, timeoutPolicy: policy, timeoutMultiplier: 1, agentTimeoutMultiplier: '', verifierTimeoutMultiplier: '' })
    } else if (policy === 'strict') {
      onDraft({ ...draft, timeoutPolicy: policy, timeoutMultiplier: 0.5, agentTimeoutMultiplier: '', verifierTimeoutMultiplier: '' })
    } else if (policy === 'relaxed') {
      onDraft({ ...draft, timeoutPolicy: policy, timeoutMultiplier: 2, agentTimeoutMultiplier: '', verifierTimeoutMultiplier: '' })
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
        <div className="run-config-group-heading">
          <h3>{labels.retryGroup}</h3>
        </div>
        <div className="run-grid">
          <Field label={labels.maxRetries}>
            <input
              type="number"
              min="0"
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
    const rules = splitRules(value)
    return rules.length ? rules : ['']
  })
  const commit = (nextRows: string[]) => {
    setRows(nextRows.length ? nextRows : [''])
    onChange(formatRules(nextRows))
  }

  return (
    <div className="rule-list-control field-wide">
      <div className="rule-list-header">
        <span>{label}</span>
        <button className="secondary-button compact-action" type="button" onClick={() => setRows([...rows, ''])}>
          {addLabel}
        </button>
      </div>
      <div className="rule-list-rows">
        {rows.map((rule, index) => (
          <div className="rule-list-row" key={index}>
            <input
              aria-label={`${label} ${index + 1}`}
              value={rule}
              onChange={(event) => commit(rows.map((item, rowIndex) => (rowIndex === index ? event.target.value : item)))}
            />
            <button className="secondary-button compact-action" type="button" onClick={() => commit(rows.filter((_, rowIndex) => rowIndex !== index))}>
              {deleteLabel}
            </button>
          </div>
        ))}
      </div>
    </div>
  )
}

function runtimeLabels(zh: boolean) {
  if (zh) {
    return {
      addRule: '添加',
      advancedGroup: '高级参数',
      agentSetupTimeoutMultiplier: 'Agent 初始化超时倍率',
      agentTimeoutMultiplier: 'Agent 执行超时倍率',
      custom: '自定义',
      collapseAdvanced: '收起高级参数',
      deleteRule: '删除',
      environmentBuildTimeoutMultiplier: '环境构建超时倍率',
      expandAdvanced: '展开高级参数',
      fast: '快速',
      maxRetries: '失败重试次数',
      relaxed: '放宽',
      retryCustom: '自定义间隔',
      retryExclude: '不重试的原始错误（命中规则）',
      retryFast: '快速（立即重试，最多 5s）',
      retryGroup: '失败重试',
      retryInterval: '重试间隔',
      retryMaxWaitSec: '最长等待秒数',
      retryMinWaitSec: '最短等待秒数',
      retryScenarios: '重试场景',
      retrySlow: '慢速（10s 起，最多 120s）',
      retryStandard: '标准（2s 起，最多 30s）',
      retryWaitMultiplier: '等待递增倍率',
      scenarios: { environment: '环境启动失败', network: '网络错误', timeout: '任务执行超时', verifier: '验证器临时失败' },
      slow: '慢速',
      standard: '标准',
      strict: '严格',
      timeoutGroup: '执行时长',
      timeoutCustom: '自定义倍率',
      timeoutMultiplier: '整体超时倍率',
      timeoutPolicy: '超时策略',
      timeoutRelaxed: '放宽（2x，允许更久）',
      timeoutStandard: '标准（1x）',
      timeoutStrict: '严格（0.5x，更快超时）',
      verifierTimeoutMultiplier: '验证器超时倍率',
    }
  }
  return {
    addRule: 'Add',
    advancedGroup: 'Advanced parameters',
    agentSetupTimeoutMultiplier: 'Agent setup timeout multiplier',
    agentTimeoutMultiplier: 'Agent execution timeout multiplier',
    custom: 'Custom',
    collapseAdvanced: 'Collapse advanced parameters',
    deleteRule: 'Delete',
    environmentBuildTimeoutMultiplier: 'Environment build timeout multiplier',
    expandAdvanced: 'Expand advanced parameters',
    fast: 'Fast',
    maxRetries: 'Failure retries',
    relaxed: 'Relaxed',
    retryCustom: 'Custom interval',
    retryExclude: 'Raw errors that should not retry (match rule)',
    retryFast: 'Fast (retry immediately, max 5s)',
    retryGroup: 'Failure retry',
    retryInterval: 'Retry interval',
    retryMaxWaitSec: 'Max wait seconds',
    retryMinWaitSec: 'Min wait seconds',
    retryScenarios: 'Retry scenarios',
    retrySlow: 'Slow (starts at 10s, max 120s)',
    retryStandard: 'Standard (starts at 2s, max 30s)',
    retryWaitMultiplier: 'Wait growth multiplier',
    scenarios: { environment: 'Environment startup failure', network: 'Network error', timeout: 'Task execution timeout', verifier: 'Temporary verifier failure' },
    slow: 'Slow',
    standard: 'Standard',
    strict: 'Strict',
    timeoutGroup: 'Execution duration',
    timeoutCustom: 'Custom multiplier',
    timeoutMultiplier: 'Overall timeout multiplier',
    timeoutPolicy: 'Timeout policy',
    timeoutRelaxed: 'Relaxed (2x, allow longer runs)',
    timeoutStandard: 'Standard (1x)',
    timeoutStrict: 'Strict (0.5x, fail faster)',
    verifierTimeoutMultiplier: 'Verifier timeout multiplier',
  }
}
