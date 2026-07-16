import { Save, X } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { useOperation } from '../api/hooks'
import { agentRowToDto } from '../api/requestMappers'
import { agentCapabilitiesForHarness } from '../domain/agentCapabilities'
import type { WebUiClient } from '../api/webUiClient'
import type { AgentCapabilities, AgentRow, HarnessTemplate } from '../domain/harbor'
import type { Translate } from '../i18n'
import { AgentIdentityEditor, AgentProfileEditor } from '../ui/components/AgentProfileEditor'
import { FormSubmissionError, issuesByField, type FormIssue } from '../ui/components/FormFeedback'

interface NewAgentPageProps {
  canSave?: boolean
  client: WebUiClient
  harnesses: HarnessTemplate[]
  rows: AgentRow[]
  t: Translate
  onAgents: () => void
  onRefresh: () => Promise<void>
}

export function NewAgentPage({ canSave = true, client, harnesses, rows, t, onAgents, onRefresh }: NewAgentPageProps) {
  const capabilitiesByHarness = useMemo(
    () => Object.fromEntries(harnesses.map((harness) => [harness.name, harness.capabilities])),
    [harnesses],
  )
  const [draft, setDraft] = useState(() => buildNewAgent(capabilitiesByHarness))
  const [validationAttempted, setValidationAttempted] = useState(false)
  const agentOperation = useOperation(client)
  const allIssues = validateNewAgent(draft, t)
  const issues = validationAttempted ? allIssues : []
  const fieldErrors = issuesByField(issues)

  useEffect(() => {
    if (!draft.harness) return
    if (harnesses.length && !harnesses.some((harness) => harness.name === draft.harness)) {
      setDraft((current) => ({ ...current, harness: '' }))
      return
    }
    const capabilities = capabilitiesByHarness[draft.harness]
    if (!capabilities) return
    setDraft((current) => current.capabilities === capabilities ? current : { ...current, capabilities })
  }, [capabilitiesByHarness, draft.harness, harnesses])

  useEffect(() => {
    if (agentOperation.operation?.status !== 'completed') return
    void onRefresh().then(onAgents)
  }, [agentOperation.operation?.id, agentOperation.operation?.status, onAgents, onRefresh])

  const save = async () => {
    if (!canSave || isOperationRunning(agentOperation.operation?.status)) return
    setValidationAttempted(true)
    if (allIssues.length) {
      window.requestAnimationFrame(() => focusAgentField(allIssues[0].field))
      return
    }
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
          <FormSubmissionError message={agentOperation.error?.message} />
          <section className="surface rail-card">
            <AgentIdentityEditor capabilitiesByHarness={capabilitiesByHarness} fieldErrors={fieldErrors} harnesses={harnesses} value={draft} t={t} onChange={setDraft} />
          </section>
          {draft.harness && (
            <AgentProfileEditor capabilitiesByHarness={capabilitiesByHarness} fieldErrors={fieldErrors} value={draft} t={t} onChange={setDraft} />
          )}
        </section>
      </div>
    </main>
  )
}

function validateNewAgent(draft: AgentRow, t: Translate): FormIssue[] {
  const issues: FormIssue[] = []
  if (!draft.agentName.trim()) issues.push({ field: 'agentName', message: t('agentNameRequired') })
  if (!draft.harness) issues.push({ field: 'harness', message: t('harnessRequired') })
  if (draft.harness === 'custom-harness' && (!draft.adapter || draft.adapter === 'none')) {
    issues.push({ field: 'importPath', message: t('customImportPathRequired') })
  }
  if (draft.harness && draft.capabilities?.supportedFields.includes('modelName') && !draft.models.trim()) {
    issues.push({ field: 'models', message: t('agentModelsValidationRequired') })
  }
  return issues
}

function focusAgentField(field: string) {
  const target = document.getElementById({
    agentName: 'agent-name',
    harness: 'agent-harness',
    importPath: 'agent-import-path',
    models: 'agent-models',
  }[field] ?? '')
  target?.focus()
}

function isOperationRunning(status: string | undefined) {
  return status === 'queued' || status === 'running'
}

function buildNewAgent(
  capabilitiesByHarness: Record<string, AgentCapabilities>,
): AgentRow {
  const harness = ''
  return {
    id: '',
    agentName: '',
    harness,
    adapter: 'none',
    models: '',
    status: 'configured',
    source: '',
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

function normalizeNewAgent(rows: AgentRow[], draft: AgentRow): AgentRow {
  const agentName = draft.agentName.trim()
  return {
    ...draft,
    id: buildAgentId(rows, agentName),
    agentName,
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

function buildAgentSource(agentName: string) {
  const slug = agentName.toLowerCase().replace(/[^a-z0-9]+/g, '-').replace(/^-|-$/g, '') || 'custom-agent'
  return `~/.ornnlab/agents/${slug}.toml`
}
