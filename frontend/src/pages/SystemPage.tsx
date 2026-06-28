import { ServerCog } from 'lucide-react'
import { useState } from 'react'
import type { SystemRow } from '../data/demo'
import type { Translate } from '../i18n'

interface SystemPageProps {
  rows: SystemRow[]
  t: Translate
}

type CleanupScope = 'docker' | 'local'

export function SystemPage({ rows, t }: SystemPageProps) {
  const [cleanupScope, setCleanupScope] = useState<CleanupScope | null>(null)
  const cleanupContent = cleanupScope === 'docker'
    ? {
        title: t('dockerCacheCleanupTitle'),
        body: t('dockerCacheCleanupBody'),
        impact: [t('dockerCacheImpactImages'), t('dockerCacheImpactRebuild')],
      }
    : {
        title: t('localCacheCleanupTitle'),
        body: t('localCacheCleanupBody'),
        impact: [t('localCacheImpactDirectory'), t('localCacheImpactRecreate')],
      }

  return (
    <main className="workspace single-page">
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
                <th>{t('actions')}</th>
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
                  <td>
                    {row.component === 'Docker' && (
                      <button className="secondary-button compact-action" onClick={() => setCleanupScope('docker')}>
                        {t('cleanCache')}
                      </button>
                    )}
                    {row.component === 'Local cache' && (
                      <button className="secondary-button compact-action" onClick={() => setCleanupScope('local')}>
                        {t('clean')}
                      </button>
                    )}
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
      {cleanupScope && (
        <div className="confirm-overlay">
          <section className="surface confirm-dialog" role="dialog" aria-modal="true" aria-label={cleanupContent.title}>
            <div className="confirm-heading">
              <h2>{cleanupContent.title}</h2>
              <p>{cleanupContent.body}</p>
            </div>
            <ul className="cleanup-impact-list">
              {cleanupContent.impact.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
            <div className="button-row confirm-actions">
              <button className="secondary-button" onClick={() => setCleanupScope(null)}>{t('cancel')}</button>
              <button className="primary-button">{t('confirmCleanup')}</button>
            </div>
          </section>
        </div>
      )}
    </main>
  )
}
