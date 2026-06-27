import { Bot, Plus, Settings } from 'lucide-react'
import { useState } from 'react'
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

  return (
    <main className="workspace single-page">
      <section className="surface">
        <div className="section-header">
          <div>
            <h1>{t('agentsCatalog')}</h1>
            <p>{t('agentsCatalogDesc')}</p>
          </div>
          <div className="toolbar">
            <button className="secondary-button">
              <Settings aria-hidden="true" />
              {t('agentSettings')}
            </button>
            <button className="primary-button">
              <Plus aria-hidden="true" />
              {t('addCustomAgent')}
            </button>
          </div>
        </div>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t('agent')}</th>
                <th>{t('agentType')}</th>
                <th>{t('adapter')}</th>
                <th>{t('models')}</th>
                <th>{t('status')}</th>
                <th>{t('sourceRef')}</th>
                <th>{t('updated')}</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr
                  key={row.name}
                  className={selected?.name === row.name ? 'selected-row' : undefined}
                  onClick={() => {
                    setSelected(row)
                    setDrawerOpen(true)
                  }}
                >
                  <td>
                    <span className="cell-title">
                      <Bot aria-hidden="true" />
                      {row.name}
                    </span>
                  </td>
                  <td>{row.type}</td>
                  <td>
                    <code>{row.adapter}</code>
                  </td>
                  <td>{row.models}</td>
                  <td>
                    <span className={`status-dot ${row.status === 'needs-token' ? 'warning' : 'success'}`}>
                      {row.status}
                    </span>
                  </td>
                  <td>{row.source}</td>
                  <td>{row.updated}</td>
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
                  <h2>{selected.name}</h2>
                  <p>{selected.type}</p>
                </div>
                <span className={`status-dot ${selected.status === 'needs-token' ? 'warning' : 'success'}`}>
                  {selected.status}
                </span>
              </div>
              <div className="metric-grid">
                <Metric label={t('agentType')} value={selected.type} />
                <Metric label={t('models')} value={selected.models} />
                <Metric label={t('sourceRef')} value={selected.source} />
                <Metric label={t('updated')} value={selected.updated} />
              </div>
            </section>
            <section className="surface rail-card">
              <div className="rail-title">
                <Bot aria-hidden="true" />
                <h3>{t('adapter')}</h3>
              </div>
              <div className="path-list">
                <code>{selected.adapter}</code>
              </div>
              <div className="button-row tight">
                <button className="secondary-button">
                  <Settings aria-hidden="true" />
                  {t('agentSettings')}
                </button>
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
