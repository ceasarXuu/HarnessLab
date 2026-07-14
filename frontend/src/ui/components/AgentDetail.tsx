import { CopyPlus, Save } from 'lucide-react'
import { useEffect, useState } from 'react'
import type { AgentRow } from '../../domain/harbor'
import type { Translate } from '../../i18n'
import { AgentIdentityEditor, AgentProfileEditor, getAgentStatusLabel } from './AgentProfileEditor'
import { AgentCapabilityPreview } from './AgentCapabilityPreview'

interface AgentDetailProps {
  agent: AgentRow
  canSave?: boolean
  t: Translate
  onSave: (agent: AgentRow) => void
  onCreateProfile?: (harness: string) => void
}

export function AgentDetail({ agent, canSave = true, t, onSave, onCreateProfile }: AgentDetailProps) {
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
            {draft.type === 'built-in' && onCreateProfile && (
              <button className="primary-button compact-action" onClick={() => onCreateProfile(draft.harness)}>
                <CopyPlus aria-hidden="true" />
                {t('createAgentFromHarness')}
              </button>
            )}
          </div>
        </div>
        <AgentIdentityEditor readOnly={draft.type === 'built-in'} value={draft} t={t} onChange={setDraft} />
      </section>
      {draft.type === 'built-in' ? (
        <AgentCapabilityPreview capabilities={draft.capabilities ?? { parameters: [], supportedFields: [] }} t={t} />
      ) : (
        <AgentProfileEditor value={draft} t={t} onChange={setDraft} />
      )}
    </aside>
  )
}
