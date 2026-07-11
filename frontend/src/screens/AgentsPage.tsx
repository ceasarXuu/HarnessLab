import { Bot, Search } from 'lucide-react'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { useAgent, useCachedServerSearch, useOperation } from '../api/hooks'
import { agentRowToDto } from '../api/requestMappers'
import { agentDtoToRow } from '../api/viewModels'
import type { WebUiClient } from '../api/webUiClient'
import { DetailDrawer } from '../ui/components/DetailDrawer'
import { ConfirmDialog } from '../ui/components/ConfirmDialog'
import { AgentDetail } from '../ui/components/AgentDetail'
import { OperationStatus } from '../ui/components/OperationStatus'
import { ResourceStatus } from '../ui/components/ResourceStatus'
import type { AgentRow } from '../domain/harbor'
import type { Translate } from '../i18n'

interface AgentsPageProps {
  writesEnabled?: boolean
  client: WebUiClient
  rows: AgentRow[]
  t: Translate
  onNewAgent: () => void
  onRefresh: () => Promise<void>
}

export function AgentsPage({ writesEnabled = true, client, rows, t, onNewAgent, onRefresh }: AgentsPageProps) {
  const [selected, setSelected] = useState<AgentRow | null>(null)
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<AgentRow | null>(null)
  const [search, setSearch] = useState('')
  const searchQuery = search.trim() || undefined
  const loadSearch = useCallback((query: string) => client.listAgents({ limit: 100, q: query }), [client])
  const searchResource = useCachedServerSearch('agents', searchQuery, loadSearch)
  const detailResource = useAgent(client, selected?.id)
  const agentOperation = useOperation(client)
  const detailAgent = detailResource.data ? agentDtoToRow(detailResource.data) : selected
  const filteredRows = useMemo(() => {
    if (!searchQuery) return rows
    if (searchResource.data) return searchResource.data.items.map(agentDtoToRow)
    const query = searchQuery.toLowerCase()
    return rows.filter((row) =>
      [row.agentName, row.harness, row.type, row.adapter, row.models, row.status, row.source].some((value) =>
        value.toLowerCase().includes(query),
      ),
    )
  }, [rows, searchQuery, searchResource.data])

  useEffect(() => {
    if (agentOperation.operation?.status !== 'completed') return
    void onRefresh()
    void detailResource.refresh()
  }, [agentOperation.operation?.id, agentOperation.operation?.status, detailResource.refresh, onRefresh])

  const confirmDelete = async () => {
    if (!writesEnabled) return
    if (!deleteTarget) return
    await agentOperation.submit(() => client.deleteAgent(deleteTarget.id), ({ operation }) => operation)
    if (selected && getAgentKey(selected) === getAgentKey(deleteTarget)) {
      setDrawerOpen(false)
      setSelected(null)
    }
    setDeleteTarget(null)
  }

  const saveAgent = async (agent: AgentRow) => {
    if (!writesEnabled || agent.type !== 'custom') return
    await agentOperation.submit(() => client.updateAgent(agent.id, agentRowToDto(agent)), ({ operation }) => operation)
  }

  return (
    <main className="workspace single-page">
      <section className="surface">
        <div className="section-header">
          <div>
            <h1>{t('agentsCatalog')}</h1>
          </div>
          <div className="toolbar">
            <label className="search-field">
              <Search aria-hidden="true" />
              <input
                aria-label={t('searchAgents')}
                value={search}
                onChange={(event) => setSearch(event.target.value)}
                placeholder={t('searchAgentsPlaceholder')}
              />
            </label>
            <button className="primary-button" disabled={!writesEnabled} onClick={onNewAgent}>{t('newAgent')}</button>
          </div>
        </div>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t('agentName')}</th>
                <th>{t('harness')}</th>
                <th>{t('agentType')}</th>
                <th>{t('models')}</th>
                <th>{t('status')}</th>
                <th>{t('actions')}</th>
              </tr>
            </thead>
            <tbody>
              {filteredRows.map((row) => (
                <tr
                  key={getAgentKey(row)}
                  className={selected && getAgentKey(selected) === getAgentKey(row) ? 'selected-row' : undefined}
                  onClick={() => {
                    setSelected(row)
                    setDrawerOpen(true)
                  }}
                >
                  <td>
                    <span className="cell-title">
                      <Bot aria-hidden="true" />
                      {row.agentName}
                    </span>
                  </td>
                  <td>{row.harness}</td>
                  <td>{row.type}</td>
                  <td>{row.models || '-'}</td>
                  <td>
                    <span className={`status-dot ${row.status === 'needs-token' ? 'warning' : 'success'}`}>
                      {row.status}
                    </span>
                  </td>
                  <td>
                    <div className="row-actions">
                      {row.type === 'custom' && (
                        <button className="secondary-button compact-action" disabled={!writesEnabled} onClick={(event) => {
                          event.stopPropagation()
                          setDeleteTarget(row)
                        }}>
                          {t('delete')}
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
              {filteredRows.length === 0 && (
                <tr>
                  <td className="empty-row" colSpan={6}>{t('noAgentsAvailable')}</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </section>
      <ResourceStatus
        error={searchResource.error?.message ?? null}
        loading={searchResource.loading}
        loadingLabel={t('loadingAgents')}
      />
      {detailAgent && (
        <DetailDrawer label={t('selectedAgent')} open={drawerOpen} onClose={() => setDrawerOpen(false)}>
          <>
            <AgentDetail
              agent={detailAgent}
              canSave={writesEnabled && !isOperationRunning(agentOperation.operation?.status)}
              t={t}
              onSave={saveAgent}
            />
            <ResourceStatus
              error={detailResource.error?.message ?? null}
              loading={detailResource.loading}
              loadingLabel={t('loadingAgents')}
            />
          </>
        </DetailDrawer>
      )}
      {deleteTarget && (
        <ConfirmDialog
          cancelLabel={t('cancel')}
          confirmLabel={t('confirmDelete')}
          impacts={[t('deleteAgentLocalImpact'), deleteTarget.agentName]}
          title={t('deleteAgentTitle')}
          onCancel={() => setDeleteTarget(null)}
          onConfirm={confirmDelete}
        />
      )}
      <OperationStatus error={agentOperation.error?.message} operation={agentOperation.operation} t={t} />
    </main>
  )
}

function getAgentKey(row: AgentRow) {
  return row.id
}

function isOperationRunning(status: string | undefined) {
  return status === 'queued' || status === 'running'
}
