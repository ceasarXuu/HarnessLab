import { useState } from 'react'
import type { AgentRow } from '../../domain/harbor'
import type { Translate } from '../../i18n'
import { useDebouncedAutosave } from '../useDebouncedAutosave'
import { AgentIdentityEditor, AgentProfileEditor, getAgentStatusLabel } from './AgentProfileEditor'

interface AgentDetailProps {
  agent: AgentRow
  canSave?: boolean
  t: Translate
  onSave: (agent: AgentRow) => boolean | Promise<boolean>
}

export function AgentDetail({ agent, canSave = true, t, onSave }: AgentDetailProps) {
  const [draft, setDraft] = useState(agent)
  const statusClass = draft.status === 'needs-token' ? 'warning' : 'success'
  const statusLabel = getAgentStatusLabel(draft.status, t)

  useDebouncedAutosave({ enabled: canSave, value: draft, onSave })

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
          </div>
        </div>
        <AgentIdentityEditor lockHarness value={draft} t={t} onChange={setDraft} />
      </section>
      <AgentProfileEditor value={draft} t={t} onChange={setDraft} />
    </aside>
  )
}
