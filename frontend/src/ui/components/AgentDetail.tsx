import { Save } from 'lucide-react'
import { useEffect, useState } from 'react'
import type { AgentRow } from '../../domain/harbor'
import type { Translate } from '../../i18n'
import { AgentIdentityEditor, AgentProfileEditor, getAgentStatusLabel } from './AgentProfileEditor'

interface AgentDetailProps {
  agent: AgentRow
  canSave?: boolean
  t: Translate
  onSave: (agent: AgentRow) => void
}

export function AgentDetail({ agent, canSave = true, t, onSave }: AgentDetailProps) {
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
          <div className="rail-heading-actions">
            <span className={`status-dot ${statusClass}`}>{statusLabel}</span>
            {draft.type === 'custom' && (
              <button className="primary-button compact-action" disabled={!canSave} onClick={() => onSave(draft)}>
                <Save aria-hidden="true" />
                {t('save')}
              </button>
            )}
          </div>
        </div>
        <AgentIdentityEditor readOnly={draft.type === 'built-in'} value={draft} t={t} onChange={setDraft} />
      </section>
      <AgentProfileEditor readOnly={draft.type === 'built-in'} value={draft} t={t} onChange={setDraft} />
    </aside>
  )
}
