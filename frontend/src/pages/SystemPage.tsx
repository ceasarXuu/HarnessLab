import { CheckCircle2, ServerCog, Wrench } from 'lucide-react'
import type { SystemRow } from '../data/demo'
import type { Translate } from '../i18n'

interface SystemPageProps {
  rows: SystemRow[]
  t: Translate
}

export function SystemPage({ rows, t }: SystemPageProps) {
  const actionGroups = [
    {
      title: t('cacheCommands'),
      actions: [t('cleanCache')],
    },
    {
      title: t('manifestCommands'),
      actions: [t('sync')],
    },
    {
      title: t('hubCommands'),
      actions: [t('upload'), t('submit'), t('share')],
    },
  ]

  return (
    <main className="workspace two-column-page">
      <section className="surface">
        <div className="section-header">
          <div>
            <h1>{t('systemHealth')}</h1>
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
      <aside className="detail-rail">
        <section className="surface rail-card">
          <div className="rail-title">
            <CheckCircle2 aria-hidden="true" />
            <h3>{t('systemChecks')}</h3>
          </div>
          <ul className="doctor-list">
            <li>
              <CheckCircle2 aria-hidden="true" />
              {t('runsReconciled')}
            </li>
            <li>
              <CheckCircle2 aria-hidden="true" />
              {t('artifactStoreWritable')}
            </li>
            <li>
              <CheckCircle2 aria-hidden="true" />
              {t('cacheCommandAvailable')}
            </li>
          </ul>
        </section>
        <section className="surface rail-card">
          <div className="rail-title">
            <Wrench aria-hidden="true" />
            <h3>{t('systemActions')}</h3>
          </div>
          <div className="action-groups">
            {actionGroups.map((group) => (
              <section key={group.title} className="action-group">
                <h4>{group.title}</h4>
                <div className="action-list">
                  {group.actions.map((action) => (
                    <button key={action} className="secondary-button">{action}</button>
                  ))}
                </div>
              </section>
            ))}
          </div>
        </section>
      </aside>
    </main>
  )
}
