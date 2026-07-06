import { useEffect, useState } from 'react'
import type { AgentRow } from '../../mocks/demo'
import type { Translate } from '../../i18n'
import { AgentIdentityEditor, AgentProfileEditor, getAgentStatusLabel } from './AgentProfileEditor'

interface AgentDetailProps {
  agent: AgentRow
  t: Translate
}

export function AgentDetail({ agent, t }: AgentDetailProps) {
  const [draft, setDraft] = useState(agent)
  const statusClass = draft.status === 'needs-token' ? 'warning' : 'success'
  const statusLabel = getAgentStatusLabel(draft.status, t)

  useEffect(() => {
    setDraft(agent)
  }, [agent])

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
        <AgentIdentityEditor value={draft} t={t} onChange={setDraft} />
      </section>
      <AgentProfileEditor value={draft} t={t} onChange={setDraft} />
    </aside>
  )
}
