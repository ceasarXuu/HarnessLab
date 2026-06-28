import { ArchiveRestore, CheckCircle2, ServerCog } from 'lucide-react'
import type { SystemRow } from '../data/demo'
import type { Translate } from '../i18n'

interface SystemPageProps {
  rows: SystemRow[]
  t: Translate
}

export function SystemPage({ rows, t }: SystemPageProps) {
  return (
    <main className="workspace two-column-page">
      <section className="surface">
        <div className="section-header">
          <div>
            <h1>{t('systemHealth')}</h1>
          </div>
          <button className="primary-button">{t('systemDoctor')}</button>
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
            <ArchiveRestore aria-hidden="true" />
            <h3>{t('recovery')}</h3>
          </div>
          <ul className="doctor-list">
            <li>
              <CheckCircle2 aria-hidden="true" />
              interrupted runs reconciled
            </li>
            <li>
              <CheckCircle2 aria-hidden="true" />
              artifact store writable
            </li>
            <li>
              <CheckCircle2 aria-hidden="true" />
              Docker orphan scan ready
            </li>
          </ul>
          <div className="button-row tight">
            <button className="secondary-button">{t('auth')}</button>
            <button className="secondary-button">{t('cache')}</button>
            <button className="secondary-button">{t('plugins')}</button>
            <button className="secondary-button">{t('sync')}</button>
          </div>
        </section>
        <section className="surface rail-card">
          <div className="rail-title">
            <ServerCog aria-hidden="true" />
            <h3>Harbor maintenance</h3>
          </div>
          <div className="path-list">
            <code>harbor auth status</code>
            <code>harbor cache clean --dry-run</code>
            <code>harbor plugins list</code>
            <code>docker orphan cleanup plan</code>
          </div>
        </section>
      </aside>
    </main>
  )
}
