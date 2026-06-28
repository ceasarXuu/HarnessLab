import { ServerCog } from 'lucide-react'
import type { SystemRow } from '../data/demo'
import type { Translate } from '../i18n'

interface SystemPageProps {
  rows: SystemRow[]
  t: Translate
}

export function SystemPage({ rows, t }: SystemPageProps) {
  const actions = [t('cleanCache')]

  return (
    <main className="workspace single-page">
      <section className="surface">
        <div className="section-header">
          <div>
            <h1>{t('systemHealth')}</h1>
          </div>
          <div className="system-header-actions" aria-label={t('systemActions')}>
            {actions.map((action) => (
              <button key={action} className="secondary-button">{action}</button>
            ))}
          </div>
        </div>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t('component')}</th>
                <th>{t('status')}</th>
                <th>{t('value')}</th>
                <th>{t('evidence')}</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={row.component}>
                  <td>
                    <span className="cell-title">
                      <ServerCog aria-hidden="true" />
                      {row.component}
                    </span>
                  </td>
                  <td>
                    <span className={`status-dot ${row.status}`}>{row.status}</span>
                  </td>
                  <td>{row.value}</td>
                  <td>
                    <code>{row.evidence}</code>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
    </main>
  )
}
