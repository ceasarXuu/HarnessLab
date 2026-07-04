import { Bot } from 'lucide-react'
import type { AgentRow } from '../../mocks/demo'
import type { Translate } from '../../i18n'
import { Metric } from './Metric'

interface AgentDetailProps {
  agent: AgentRow
  t: Translate
}

export function AgentDetail({ agent, t }: AgentDetailProps) {
  const statusClass = agent.status === 'needs-token' ? 'warning' : 'success'
  const statusLabel = getAgentStatusLabel(agent.status, t)

  return (
    <aside className="detail-rail agent-detail">
      <section className="surface rail-card">
        <div className="rail-heading">
          <div>
            <h2>{agent.agentName}</h2>
            <p>{agent.harness}</p>
          </div>
          <span className={`status-dot ${statusClass}`}>{statusLabel}</span>
        </div>
        <div className="metric-grid">
          <Metric label={t('agentName')} value={agent.agentName} />
          <Metric label={t('harness')} value={agent.harness} />
          <Metric label={t('agentType')} value={agent.type} />
          <Metric label={t('supportedModels')} value={agent.compatibleModels ?? agent.models} />
        </div>
      </section>
      <section className="surface rail-card">
        <div className="rail-title">
          <Bot aria-hidden="true" />
          <h3>{t('credentialsAndParams')}</h3>
        </div>
        <div className="metric-grid">
          <Metric label={t('credentialStatus')} value={agent.env ?? '-'} />
          <Metric label={t('agentKwargs')} value={agent.kwargs ?? '-'} />
        </div>
      </section>
      <section className="surface rail-card">
        <div className="rail-title">
          <Bot aria-hidden="true" />
          <h3>{t('capabilityConfig')}</h3>
        </div>
        <div className="metric-grid">
          <Metric label={t('skills')} value={agent.skills ?? '-'} />
          <Metric label={t('mcpConfig')} value={agent.mcp ?? '-'} />
        </div>
      </section>
      <details className="surface rail-card agent-advanced">
        <summary>{t('advancedDefaults')}</summary>
        <div className="metric-grid">
          <Metric label={t('entrypoint')} value={agent.adapter} />
          <Metric label={t('runtimeDefaults')} value={agent.runtime ?? '-'} />
          <Metric label={t('setupTimeout')} value={agent.setupTimeout ?? '-'} />
          <Metric label={t('maxTimeout')} value={agent.maxTimeout ?? '-'} />
          <Metric label={t('agentAllowedHosts')} value={agent.allowedHosts ?? '-'} />
          <Metric label={t('sourceRef')} value={agent.source} />
          <Metric label={t('updated')} value={agent.updated} />
        </div>
      </details>
    </aside>
  )
}

function getAgentStatusLabel(status: AgentRow['status'], t: Translate) {
  if (status === 'needs-token') return t('agentStatusNeedsToken')
  if (status === 'configured') return t('agentStatusConfigured')
  return t('agentStatusAvailable')
}
