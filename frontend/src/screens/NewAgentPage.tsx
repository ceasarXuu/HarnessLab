import { Save, X } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { useOperation } from '../api/hooks'
import { agentRowToDto } from '../api/requestMappers'
import { agentCapabilitiesByHarness, agentCapabilitiesForHarness } from '../domain/agentCapabilities'
import type { WebUiClient } from '../api/webUiClient'
import type { AgentRow } from '../domain/harbor'
import type { Translate } from '../i18n'
import { AgentIdentityEditor, AgentProfileEditor } from '../ui/components/AgentProfileEditor'
import { ResourceStatus } from '../ui/components/ResourceStatus'

interface NewAgentPageProps {
  canSave?: boolean
  client: WebUiClient
  initialHarness?: string
  rows: AgentRow[]
  t: Translate
  onAgents: () => void
  onRefresh: () => Promise<void>
}

export function NewAgentPage({ canSave = true, client, initialHarness, rows, t, onAgents, onRefresh }: NewAgentPageProps) {
  const capabilitiesByHarness = useMemo(() => agentCapabilitiesByHarness(rows), [rows])
  const [draft, setDraft] = useState(() => buildNewAgent(rows, capabilitiesByHarness, initialHarness))
  const agentOperation = useOperation(client)

  useEffect(() => {
    const capabilities = capabilitiesByHarness[draft.harness]
    if (!capabilities) return
    setDraft((current) => current.capabilities === capabilities ? current : { ...current, capabilities })
  }, [capabilitiesByHarness, draft.harness])

  useEffect(() => {
    if (agentOperation.operation?.status !== 'completed') return
    void onRefresh().then(onAgents)
  }, [agentOperation.operation?.id, agentOperation.operation?.status, onAgents, onRefresh])

  const save = async () => {
    if (!canSave) return
    const agent = normalizeNewAgent(rows, draft)
    await agentOperation.submit(() => client.createAgent(agentRowToDto(agent)), ({ operation }) => operation)
  }

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
              <button className="primary-button" disabled={!canSave || isOperationRunning(agentOperation.operation?.status)} onClick={save}>
                <Save aria-hidden="true" />
                {t('save')}
              </button>
            </div>
          </div>
          <section className="surface rail-card">
            <AgentIdentityEditor capabilitiesByHarness={capabilitiesByHarness} value={draft} t={t} onChange={setDraft} />
          </section>
          <AgentProfileEditor capabilitiesByHarness={capabilitiesByHarness} value={draft} t={t} onChange={setDraft} />
        </section>
      </div>
      <ResourceStatus error={agentOperation.error?.message ?? null} />
    </main>
  )
}

function isOperationRunning(status: string | undefined) {
  return status === 'queued' || status === 'running'
}

function buildNewAgent(
  rows: AgentRow[],
  capabilitiesByHarness: ReturnType<typeof agentCapabilitiesByHarness>,
  initialHarness?: string,
): AgentRow {
  const harness = initialHarness || 'custom-harness'
  const baseName = harness === 'custom-harness' ? 'Custom Agent' : `${formatHarnessName(harness)} Agent`
  return {
    id: buildAgentId(rows, baseName),
    agentName: buildUniqueAgentName(rows, baseName),
    harness,
    type: 'custom',
    adapter: harness === 'custom-harness' ? 'agents.custom:Agent' : 'none',
    models: '',
    status: 'configured',
    source: '~/.ornnlab/agents/custom-agent.toml',
    updated: 'just now',
    env: 'none',
    kwargs: 'none',
    skills: 'none',
    mcp: 'none',
    setupTimeout: '300s',
    timeout: '1800s',
    maxTimeout: '3600s',
    capabilities: agentCapabilitiesForHarness(harness, capabilitiesByHarness),
  }
}

function formatHarnessName(harness: string) {
  return harness.split('-').map((part) => part ? `${part[0].toUpperCase()}${part.slice(1)}` : part).join(' ')
}

function normalizeNewAgent(rows: AgentRow[], draft: AgentRow): AgentRow {
  const agentName = draft.agentName.trim() || buildUniqueAgentName(rows, 'Custom Agent')
  return {
    ...draft,
    id: buildAgentId(rows, agentName),
    agentName,
    type: 'custom',
    source: buildAgentSource(agentName),
    updated: 'just now',
  }
}

function buildAgentId(rows: AgentRow[], agentName: string) {
  const base = agentName.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'custom-agent'
  const existing = new Set(rows.map((row) => row.id))
  if (!existing.has(base)) return base
  let index = 2
  let candidate = `${base}-${index}`
  while (existing.has(candidate)) {
    index += 1
    candidate = `${base}-${index}`
  }
  return candidate
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
