import { Bot, Search } from 'lucide-react'
import { useMemo, useState } from 'react'
import { DetailDrawer } from '../ui/components/DetailDrawer'
import { ConfirmDialog } from '../ui/components/ConfirmDialog'
import { AgentDetail } from '../ui/components/AgentDetail'
import type { AgentRow } from '../domain/harbor'
import type { Translate } from '../i18n'

interface AgentsPageProps {
  allowMockWrites?: boolean
  rows: AgentRow[]
  t: Translate
  onNewAgent: () => void
  onRowsChange: (rows: AgentRow[]) => void
}

export function AgentsPage({ allowMockWrites = true, rows, t, onNewAgent, onRowsChange }: AgentsPageProps) {
  const [selected, setSelected] = useState<AgentRow | null>(null)
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<AgentRow | null>(null)
  const [search, setSearch] = useState('')
  const filteredRows = useMemo(() => {
    const query = search.trim().toLowerCase()
    if (!query) return rows
    return rows.filter((row) =>
      [row.agentName, row.harness, row.type, row.adapter, row.models, row.status, row.source].some((value) =>
        value.toLowerCase().includes(query),
      ),
    )
  }, [rows, search])

  const confirmDelete = () => {
    if (!allowMockWrites) return
    if (!deleteTarget) return
    onRowsChange(rows.filter((row) => getAgentKey(row) !== getAgentKey(deleteTarget)))
    if (selected && getAgentKey(selected) === getAgentKey(deleteTarget)) {
      setDrawerOpen(false)
      setSelected(null)
    }
    setDeleteTarget(null)
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
            <button className="primary-button" disabled={!allowMockWrites} onClick={onNewAgent}>{t('newAgent')}</button>
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
                  <td>{row.models}</td>
                  <td>
                    <span className={`status-dot ${row.status === 'needs-token' ? 'warning' : 'success'}`}>
                      {row.status}
                    </span>
                  </td>
                  <td>
                    <div className="row-actions">
                      {row.type === 'custom' && (
                        <button className="secondary-button compact-action" disabled={!allowMockWrites} onClick={(event) => {
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
            </tbody>
          </table>
        </div>
      </section>
      {selected && (
        <DetailDrawer label={t('selectedAgent')} open={drawerOpen} onClose={() => setDrawerOpen(false)}>
          <AgentDetail agent={selected} t={t} />
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
    </main>
  )
}

function getAgentKey(row: AgentRow) {
  return `${row.agentName}:${row.harness}`
}
