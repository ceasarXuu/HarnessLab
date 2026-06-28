import { Bot, Search } from 'lucide-react'
import { useMemo, useState } from 'react'
import { DetailDrawer } from '../components/DetailDrawer'
import type { AgentRow } from '../data/demo'
import type { Translate } from '../i18n'

interface AgentsPageProps {
  rows: AgentRow[]
  t: Translate
}

export function AgentsPage({ rows, t }: AgentsPageProps) {
  const [selected, setSelected] = useState<AgentRow | null>(null)
  const [drawerOpen, setDrawerOpen] = useState(false)
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
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
      {selected && (
        <DetailDrawer label={t('selectedAgent')} open={drawerOpen} onClose={() => setDrawerOpen(false)}>
          <aside className="detail-rail agent-detail">
            <section className="surface rail-card">
              <p className="panel-kicker">{t('selectedAgent')}</p>
              <div className="rail-heading">
                <div>
                  <h2>{selected.agentName}</h2>
                  <p>{selected.harness}</p>
                </div>
                <span className={`status-dot ${selected.status === 'needs-token' ? 'warning' : 'success'}`}>
                  {selected.status}
                </span>
              </div>
              <div className="metric-grid">
                <Metric label={t('agentName')} value={selected.agentName} />
                <Metric label={t('harness')} value={selected.harness} />
                <Metric label={t('agentType')} value={selected.type} />
                <Metric label={t('models')} value={selected.models} />
                <Metric label={t('sourceRef')} value={selected.source} />
                <Metric label={t('updated')} value={selected.updated} />
                <Metric label="env readiness" value={selected.env ?? '-'} />
                <Metric label="kwargs" value={selected.kwargs ?? '-'} />
                <Metric label="runtime" value={selected.runtime ?? '-'} />
                <Metric label="setup timeout" value={selected.setupTimeout ?? '-'} />
                <Metric label="max timeout" value={selected.maxTimeout ?? '-'} />
                <Metric label="allowed hosts" value={selected.allowedHosts ?? '-'} />
                <Metric label="compatible models" value={selected.compatibleModels ?? '-'} />
              </div>
            </section>
            <section className="surface rail-card">
              <div className="rail-title">
                <Bot aria-hidden="true" />
                <h3>{t('adapter')}</h3>
              </div>
              <div className="path-list">
                <code>{selected.adapter}</code>
                <code>{selected.skills ?? 'skills: none'}</code>
                <code>{selected.mcp ?? 'mcp: none'}</code>
                <code>{selected.adapterReview ?? 'adapter review: none'}</code>
              </div>
              <div className="button-row tight">
                <button className="secondary-button">Adapter init</button>
                <button className="secondary-button">Adapter review</button>
              </div>
            </section>
            <section className="surface rail-card">
              <div className="rail-title">
                <Bot aria-hidden="true" />
                <h3>{t('adapterTools')}</h3>
              </div>
              <div className="path-list">
                <code>harbor adapter init --agent {selected.harness}</code>
                <code>harbor adapter review {selected.adapter}</code>
                <code>max_timeout_sec: {selected.maxTimeout ?? '-'}</code>
                <code>override_setup_timeout_sec: {selected.setupTimeout ?? '-'}</code>
                <code>extra_allowed_hosts: {selected.allowedHosts ?? '-'}</code>
              </div>
            </section>
          </aside>
        </DetailDrawer>
      )}
    </main>
  )
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  )
}
