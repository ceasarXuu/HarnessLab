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

  return (
    <aside className="detail-rail agent-detail">
      <section className="surface rail-card">
        <div className="rail-heading">
          <div>
            <h2>{agent.agentName}</h2>
            <p>{agent.harness}</p>
          </div>
          <span className={`status-dot ${statusClass}`}>{agent.status}</span>
        </div>
        <div className="metric-grid">
          <Metric label={t('agentName')} value={agent.agentName} />
          <Metric label={t('harness')} value={agent.harness} />
          <Metric label={t('agentType')} value={agent.type} />
          <Metric label={t('models')} value={agent.models} />
          <Metric label={t('sourceRef')} value={agent.source} />
          <Metric label={t('updated')} value={agent.updated} />
          <Metric label="env readiness" value={agent.env ?? '-'} />
          <Metric label="kwargs" value={agent.kwargs ?? '-'} />
          <Metric label="runtime" value={agent.runtime ?? '-'} />
          <Metric label="setup timeout" value={agent.setupTimeout ?? '-'} />
          <Metric label="max timeout" value={agent.maxTimeout ?? '-'} />
          <Metric label="allowed hosts" value={agent.allowedHosts ?? '-'} />
          <Metric label="compatible models" value={agent.compatibleModels ?? '-'} />
        </div>
      </section>
      <section className="surface rail-card">
        <div className="rail-title">
          <Bot aria-hidden="true" />
          <h3>{t('adapter')}</h3>
        </div>
        <div className="path-list">
          <code>{agent.adapter}</code>
          <code>{agent.skills ?? 'skills: none'}</code>
          <code>{agent.mcp ?? 'mcp: none'}</code>
          <code>{agent.adapterReview ?? 'adapter review: none'}</code>
        </div>
        <div className="button-row tight">
          <button className="secondary-button">Adapter init</button>
          <button className="secondary-button">Adapter review</button>
        </div>
      </section>
      <section className="surface rail-card">
        <div className="rail-title">
          <Bot aria-hidden="true" />
          <h3>{t('adapterTools')}</h3>
        </div>
        <div className="path-list">
          <code>harbor adapter init --agent {agent.harness}</code>
          <code>harbor adapter review {agent.adapter}</code>
          <code>max_timeout_sec: {agent.maxTimeout ?? '-'}</code>
          <code>override_setup_timeout_sec: {agent.setupTimeout ?? '-'}</code>
          <code>extra_allowed_hosts: {agent.allowedHosts ?? '-'}</code>
        </div>
      </section>
    </aside>
  )
}
