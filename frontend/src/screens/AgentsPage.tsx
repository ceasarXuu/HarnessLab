import { Bot, Search } from 'lucide-react'
import { useMemo, useState } from 'react'
import { DetailDrawer } from '../ui/components/DetailDrawer'
import { ConfirmDialog } from '../ui/components/ConfirmDialog'
import { AgentDetail } from '../ui/components/AgentDetail'
import type { AgentRow } from '../mocks/demo'
import type { Translate } from '../i18n'

interface AgentsPageProps {
  rows: AgentRow[]
  t: Translate
}

export function AgentsPage({ rows, t }: AgentsPageProps) {
  const [agentRows, setAgentRows] = useState(rows)
  const [selected, setSelected] = useState<AgentRow | null>(null)
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<AgentRow | null>(null)
  const [search, setSearch] = useState('')
  const filteredRows = useMemo(() => {
    const query = search.trim().toLowerCase()
    if (!query) return agentRows
    return agentRows.filter((row) =>
      [row.agentName, row.harness, row.type, row.adapter, row.models, row.status, row.source].some((value) =>
        value.toLowerCase().includes(query),
      ),
    )
  }, [agentRows, search])

  const confirmDelete = () => {
    if (!deleteTarget) return
    setAgentRows((current) => current.filter((row) => row.harness !== deleteTarget.harness))
    if (selected?.harness === deleteTarget.harness) {
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
            <button className="primary-button">{t('newAgent')}</button>
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
                  key={row.harness}
                  className={selected?.harness === row.harness ? 'selected-row' : undefined}
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
                        <button className="secondary-button compact-action" onClick={(event) => {
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
