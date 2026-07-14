import { useState } from 'react'
import type { AgentCapabilities, AgentCapabilityField, AgentParameter } from '../../domain/harbor'
import type { Translate } from '../../i18n'
import { Metric } from './Metric'

type CapabilityTab = 'base' | 'skills' | 'mcps' | 'advanced'

interface AgentCapabilityPreviewProps {
  capabilities: AgentCapabilities
  t: Translate
}

export function AgentCapabilityPreview({ capabilities, t }: AgentCapabilityPreviewProps) {
  const [activeTab, setActiveTab] = useState<CapabilityTab>('base')
  const supports = (field: AgentCapabilityField) => capabilities.supportedFields.includes(field)
  const tabs = buildTabs(capabilities, t)
  const resolvedTab = tabs.some((tab) => tab.key === activeTab) ? activeTab : tabs[0]?.key ?? 'base'

  if (!tabs.length) return null

  return (
    <div className="agent-profile-editor agent-capability-preview">
      <div className="run-tabs agent-detail-tabs" role="tablist" aria-label={t('supportedAgentCapabilities')}>
        {tabs.map((tab) => (
          <button
            key={tab.key}
            type="button"
            role="tab"
            aria-selected={resolvedTab === tab.key}
            className={resolvedTab === tab.key ? 'active' : undefined}
            onClick={() => setActiveTab(tab.key)}
          >
            {tab.label}
          </button>
        ))}
      </div>

      {resolvedTab === 'base' && (
        <section className="surface rail-card">
          <h3>{t('supportedAgentCapabilities')}</h3>
          <div className="agent-form-grid">
            {supports('modelName') && <Metric label={t('supportedModels')} value={t('agentModelListConfig')} />}
            {supports('env') && <Metric label={t('genericAgentEnv')} value={t('agentKeyValueConfig')} />}
          </div>
        </section>
      )}

      {resolvedTab === 'skills' && (
        <section className="surface rail-card">
          <h3>{t('skillsConfig')}</h3>
          <div className="agent-form-grid">
            <Metric label={t('skills')} value={t('agentSkillsConfig')} />
          </div>
        </section>
      )}

      {resolvedTab === 'mcps' && (
        <section className="surface rail-card">
          <h3>{t('mcpConfigSection')}</h3>
          <div className="agent-form-grid">
            <Metric label={t('mcpTransport')} value="stdio / SSE / Streamable HTTP" />
          </div>
        </section>
      )}

      {resolvedTab === 'advanced' && (
        <section className="surface rail-card">
          <h3>{t('advancedAgentParams')}</h3>
          <div className="agent-form-grid">
            {supports('timeouts') && (
              <>
                <Metric label={t('agentExecutionTimeout')} value={t('agentNumberConfig')} />
                <Metric label={t('setupTimeout')} value={t('agentNumberConfig')} />
                <Metric label={t('maxTimeout')} value={t('agentNumberConfig')} />
              </>
            )}
            {supports('customKwargs') && <Metric label={t('genericAgentKwargs')} value={t('agentKeyValueConfig')} />}
            {supports('harnessParameters') && capabilities.parameters.map((parameter) => (
              <Metric key={`${parameter.source}-${parameter.key}`} label={parameter.label} value={parameterSummary(parameter, t)} />
            ))}
          </div>
        </section>
      )}
    </div>
  )
}

function buildTabs(capabilities: AgentCapabilities, t: Translate) {
  const supports = (field: AgentCapabilityField) => capabilities.supportedFields.includes(field)
  const hasBase = supports('modelName') || supports('env')
  const hasAdvanced = supports('timeouts') || supports('customKwargs') || supports('harnessParameters')
  return [
    hasBase ? { key: 'base' as const, label: t('runTabCore') } : null,
    supports('skills') ? { key: 'skills' as const, label: t('skillsTab') } : null,
    supports('mcpServers') ? { key: 'mcps' as const, label: t('mcpsTab') } : null,
    hasAdvanced ? { key: 'advanced' as const, label: t('agentAdvancedTab') } : null,
  ].filter((tab): tab is { key: CapabilityTab; label: string } => tab !== null)
}

function parameterSummary(parameter: AgentParameter, t: Translate) {
  if (parameter.choices?.length) return parameter.choices.join(' / ')
  if (parameter.defaultValue !== undefined) return `${t('defaultValue')}: ${String(parameter.defaultValue)}`
  if (parameter.kind === 'boolean') return t('agentBooleanConfig')
  if (parameter.kind === 'number') return t('agentNumberConfig')
  return t('agentTextConfig')
}
