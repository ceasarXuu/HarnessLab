import { Bot, Plus, Settings } from 'lucide-react'
import type { AgentRow } from '../data/demo'
import type { Translate } from '../i18n'

interface AgentsPageProps {
  rows: AgentRow[]
  t: Translate
}

export function AgentsPage({ rows, t }: AgentsPageProps) {
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
                <tr key={row.name}>
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
    </main>
  )
}
