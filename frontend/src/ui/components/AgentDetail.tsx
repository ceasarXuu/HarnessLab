import { useMemo, useState } from 'react'
import { Plus, Trash2 } from 'lucide-react'
import type { AgentRow } from '../../mocks/demo'
import type { Translate } from '../../i18n'
import { KeyValueControl } from './KeyValueControl'
import { Metric } from './Metric'

interface AgentDetailProps {
  agent: AgentRow
  t: Translate
}

interface HarnessConfig {
  reasoning?: string[]
  reasoningSummary?: string[]
  temperature?: boolean
  contextLength?: boolean
  tools?: boolean
  permissions?: boolean
  credentials?: 'anthropic' | 'openai' | 'custom' | 'none'
}

const harnessConfigs: Record<string, HarnessConfig> = {
  'claude-code': {
    reasoning: ['low', 'medium', 'high', 'xhigh', 'max'],
    tools: true,
    permissions: true,
    credentials: 'anthropic',
  },
  codex: {
    reasoning: ['minimal', 'low', 'medium', 'high'],
    reasoningSummary: ['auto', 'concise', 'detailed', 'none'],
    credentials: 'openai',
  },
  'codex-cli': {
    reasoning: ['minimal', 'low', 'medium', 'high'],
    reasoningSummary: ['auto', 'concise', 'detailed', 'none'],
    credentials: 'openai',
  },
  'qwen-coder': {
    credentials: 'openai',
  },
  'custom-harness': {
    temperature: true,
    contextLength: true,
    tools: true,
    permissions: true,
    credentials: 'custom',
  },
  oracle: {
    credentials: 'none',
  },
}

export function AgentDetail({ agent, t }: AgentDetailProps) {
  const [draft, setDraft] = useState(agent)
  const statusClass = draft.status === 'needs-token' ? 'warning' : 'success'
  const statusLabel = getAgentStatusLabel(draft.status, t)
  const config = useMemo(() => harnessConfigs[draft.harness] ?? harnessConfigs['custom-harness'], [draft.harness])
  const setField = (field: keyof AgentRow, value: string) => setDraft((current) => ({ ...current, [field]: value }))
  const isCustomAgent = draft.type === 'custom' || draft.harness === 'custom-harness'

  return (
    <aside className="detail-rail agent-detail">
      <section className="surface rail-card">
        <div className="rail-heading">
          <div className="rail-title-copy">
            <h2>{draft.agentName}</h2>
            <p>{draft.harness}</p>
          </div>
          <span className={`status-dot ${statusClass}`}>{statusLabel}</span>
        </div>
        <div className="agent-form-grid">
          <label>
            {t('agentName')}
            <input value={draft.agentName} onChange={(event) => setField('agentName', event.target.value)} />
          </label>
          <label>
            {t('harness')}
            <select value={draft.harness} onChange={(event) => setField('harness', event.target.value)}>
              {Object.keys(harnessConfigs).map((harness) => (
                <option key={harness} value={harness}>{harness}</option>
              ))}
            </select>
          </label>
          <label>
            {t('agentType')}
            <input readOnly value={draft.type} />
          </label>
          {isCustomAgent && (
            <label>
              {t('customImportPath')}
              <input value={draft.adapter} onChange={(event) => setField('adapter', event.target.value)} />
            </label>
          )}
        </div>
      </section>

      <section className="surface rail-card">
        <SectionTitle>{t('modelSettings')}</SectionTitle>
        <div className="agent-form-grid">
          <ModelListControl
            label={t('supportedModels')}
            value={draft.models}
            onChange={(value) => setField('models', value)}
          />
          {config.reasoning && (
            <TagGroupControl
              label={t('reasoningEffort')}
              options={config.reasoning}
              value={draft.reasoningEffort ?? ''}
              onChange={(value) => setField('reasoningEffort', value)}
            />
          )}
          {config.reasoningSummary && (
            <label>
              {t('reasoningSummary')}
              <select value={draft.reasoningSummary ?? ''} onChange={(event) => setField('reasoningSummary', event.target.value)}>
                <option value="">-</option>
                {config.reasoningSummary.map((option) => <option key={option} value={option}>{option}</option>)}
              </select>
            </label>
          )}
          {config.temperature && (
            <label>
              {t('temperature')}
              <input inputMode="decimal" value={draft.temperature ?? ''} onChange={(event) => setField('temperature', event.target.value)} />
            </label>
          )}
          {config.contextLength && (
            <label>
              {t('contextLength')}
              <input inputMode="numeric" value={draft.contextLength ?? ''} onChange={(event) => setField('contextLength', event.target.value)} />
            </label>
          )}
        </div>
      </section>

      <section className="surface rail-card">
        <SectionTitle>{t('credentialsAndParams')}</SectionTitle>
        <div className="agent-form-grid">
          {config.credentials !== 'none' && (
            <>
              <label>
                {t('apiKeyEnv')}
                <input value={draft.apiKeyEnv ?? ''} onChange={(event) => setField('apiKeyEnv', event.target.value)} />
              </label>
              <label>
                {t('baseUrlEnv')}
                <input value={draft.baseUrlEnv ?? ''} onChange={(event) => setField('baseUrlEnv', event.target.value)} />
              </label>
            </>
          )}
          <div className="field-wide">
            <KeyValueControl compact label={t('genericAgentEnv')} value={draft.env ?? 'none'} onChange={(value) => setField('env', value)} />
          </div>
        </div>
      </section>

      {(config.permissions || config.tools) && (
        <section className="surface rail-card">
          <SectionTitle>{t('permissionsAndTools')}</SectionTitle>
          <div className="agent-form-grid">
            {config.permissions && (
              <label>
                {t('permissionMode')}
                <select value={draft.permissionMode ?? ''} onChange={(event) => setField('permissionMode', event.target.value)}>
                  <option value="">-</option>
                  <option value="tool allowlist">tool allowlist</option>
                  <option value="allow">allow</option>
                  <option value="deny">deny</option>
                  <option value="custom harness policy">custom harness policy</option>
                </select>
              </label>
            )}
            {config.tools && (
              <>
                <label>
                  {t('allowedTools')}
                  <textarea value={draft.allowedTools ?? ''} onChange={(event) => setField('allowedTools', event.target.value)} />
                </label>
                <label>
                  {t('disallowedTools')}
                  <textarea value={draft.disallowedTools ?? ''} onChange={(event) => setField('disallowedTools', event.target.value)} />
                </label>
              </>
            )}
          </div>
        </section>
      )}

      <section className="surface rail-card">
        <SectionTitle>{t('networkAccess')}</SectionTitle>
        <div className="agent-form-grid">
          <label className="field-wide">
            {t('agentAllowedHosts')}
            <textarea value={draft.allowedHosts ?? ''} onChange={(event) => setField('allowedHosts', event.target.value)} />
          </label>
        </div>
      </section>

      <section className="surface rail-card">
        <SectionTitle>{t('capabilityConfig')}</SectionTitle>
        <div className="agent-form-grid">
          <label>
            {t('skills')}
            <textarea value={draft.skills ?? ''} onChange={(event) => setField('skills', event.target.value)} />
          </label>
          <label>
            {t('mcpConfig')}
            <textarea value={draft.mcp ?? ''} onChange={(event) => setField('mcp', event.target.value)} />
          </label>
        </div>
      </section>

      <details className="surface rail-card agent-advanced">
        <summary>{t('advancedAgentParams')}</summary>
        <div className="agent-form-grid">
          <KeyValueControl label={t('genericAgentKwargs')} value={draft.kwargs ?? 'none'} onChange={(value) => setField('kwargs', value)} />
          <Metric label={t('runtimeDefaults')} value={draft.runtime ?? '-'} />
          <Metric label={t('setupTimeout')} value={draft.setupTimeout ?? '-'} />
          <Metric label={t('maxTimeout')} value={draft.maxTimeout ?? '-'} />
          <Metric label={t('sourceRef')} value={draft.source} />
          <Metric label={t('updated')} value={draft.updated} />
        </div>
      </details>
    </aside>
  )
}

function SectionTitle({ children }: { children: string }) {
  return (
    <div className="rail-title">
      <h3>{children}</h3>
    </div>
  )
}

function ModelListControl({ label, value, onChange }: { label: string; value: string; onChange: (value: string) => void }) {
  const [models, setModels] = useState(() => parseModelNames(value))

  const commit = (nextModels: string[]) => {
    setModels(nextModels.length ? nextModels : [''])
    onChange(formatModelNames(nextModels))
  }

  return (
    <div className="model-list-control field-wide">
      <div className="rule-list-header">
        <span>{label}</span>
        <button className="secondary-button compact-action" type="button" onClick={() => commit([...models, ''])}>
          <Plus aria-hidden="true" />
          Add
        </button>
      </div>
      <div className="rule-list-rows">
        {models.map((modelName, index) => (
          <div className="rule-list-row" key={index}>
            <input
              aria-label="Model name"
              value={modelName}
              onChange={(event) => commit(models.map((item, rowIndex) => rowIndex === index ? event.target.value : item))}
            />
            <button
              aria-label={`Delete model ${modelName || index + 1}`}
              className="icon-button"
              type="button"
              onClick={() => commit(models.filter((_, rowIndex) => rowIndex !== index))}
            >
              <Trash2 aria-hidden="true" />
            </button>
          </div>
        ))}
      </div>
    </div>
  )
}

function TagGroupControl({ label, options, value, onChange }: { label: string; options: string[]; value: string; onChange: (value: string) => void }) {
  const selected = new Set(parseList(value))
  const toggle = (option: string) => {
    const next = new Set(selected)
    if (next.has(option)) {
      next.delete(option)
    } else {
      next.add(option)
    }
    onChange(options.filter((item) => next.has(item)).join(', '))
  }

  return (
    <div className="tag-group-control field-wide">
      <span>{label}</span>
      <div className="tag-group" role="group" aria-label={label}>
        {options.map((option) => (
          <button
            aria-pressed={selected.has(option)}
            className="tag-option"
            key={option}
            type="button"
            onClick={() => toggle(option)}
          >
            {option}
          </button>
        ))}
      </div>
    </div>
  )
}

function parseModelNames(value: string) {
  const names = parseList(value)
  return names.length ? names : ['']
}

function parseList(value: string) {
  return value.split(',').map((item) => item.trim()).filter(Boolean)
}

function formatModelNames(models: string[]) {
  const names = models.map((item) => item.trim()).filter(Boolean)
  return names.join(', ')
}

function getAgentStatusLabel(status: AgentRow['status'], t: Translate) {
  if (status === 'needs-token') return t('agentStatusNeedsToken')
  if (status === 'configured') return t('agentStatusConfigured')
  return t('agentStatusAvailable')
}
