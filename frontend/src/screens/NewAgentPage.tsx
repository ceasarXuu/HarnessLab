import { Save, X } from 'lucide-react'
import { useState } from 'react'
import type { AgentRow } from '../domain/harbor'
import type { Translate } from '../i18n'
import { AgentIdentityEditor, AgentProfileEditor } from '../ui/components/AgentProfileEditor'

interface NewAgentPageProps {
  canSave?: boolean
  rows: AgentRow[]
  t: Translate
  onAgents: () => void
  onSave: (agent: AgentRow) => void
}

export function NewAgentPage({ canSave = true, rows, t, onAgents, onSave }: NewAgentPageProps) {
  const [draft, setDraft] = useState(() => buildNewAgent(rows))

  return (
    <main className="workspace single-page">
      <div className="content-column">
        <nav className="breadcrumb-nav" aria-label="Agent path">
          <button type="button" onClick={onAgents}>
            {t('agents')}
          </button>
          <span aria-current="page">{t('newAgent')}</span>
        </nav>
        <section className="surface run-builder agent-create-page">
          <div className="section-header compact">
            <div>
              <h1>{t('newAgent')}</h1>
            </div>
            <div className="run-builder-actions">
              <button className="secondary-button" onClick={onAgents}>
                <X aria-hidden="true" />
                {t('cancel')}
              </button>
              <button className="primary-button" disabled={!canSave} onClick={() => onSave(normalizeNewAgent(rows, draft))}>
                <Save aria-hidden="true" />
                {t('save')}
              </button>
            </div>
          </div>
          <section className="surface rail-card">
            <AgentIdentityEditor value={draft} t={t} onChange={setDraft} />
          </section>
          <AgentProfileEditor value={draft} t={t} onChange={setDraft} />
        </section>
      </div>
    </main>
  )
}

function buildNewAgent(rows: AgentRow[]): AgentRow {
  return {
    agentName: buildUniqueAgentName(rows, 'Custom Agent'),
    harness: 'custom-harness',
    type: 'custom',
    adapter: 'agents.custom:Agent',
    models: 'custom-model',
    status: 'needs-token',
    source: '~/.ornnlab/agents/custom-agent.toml',
    updated: 'just now',
    env: 'CUSTOM_API_KEY=${CUSTOM_API_KEY}',
    kwargs: 'none',
    skills: 'none',
    mcp: 'none',
    runtime: 'docker / 1800s',
    setupTimeout: '300s',
    maxTimeout: '3600s',
    allowedHosts: '*',
    compatibleModels: 'custom-model',
    temperature: '0.2',
    contextLength: '131072',
    apiKeyEnv: 'CUSTOM_API_KEY',
    baseUrlEnv: 'CUSTOM_BASE_URL',
  }
}

function normalizeNewAgent(rows: AgentRow[], draft: AgentRow): AgentRow {
  const agentName = draft.agentName.trim() || buildUniqueAgentName(rows, 'Custom Agent')
  return {
    ...draft,
    agentName,
    type: 'custom',
    source: buildAgentSource(agentName),
    updated: 'just now',
  }
}

function buildUniqueAgentName(rows: AgentRow[], baseName: string) {
  const existing = new Set(rows.map((row) => row.agentName))
  if (!existing.has(baseName)) return baseName
  let index = 2
  let candidate = `${baseName} ${index}`
  while (existing.has(candidate)) {
    index += 1
    candidate = `${baseName} ${index}`
  }
  return candidate
}

function buildAgentSource(agentName: string) {
  const slug = agentName.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'custom-agent'
  return `~/.ornnlab/agents/${slug}.toml`
}
