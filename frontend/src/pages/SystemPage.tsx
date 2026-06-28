import { ServerCog } from 'lucide-react'
import { useState } from 'react'
import type { SystemRow } from '../data/demo'
import type { Translate } from '../i18n'

interface SystemPageProps {
  rows: SystemRow[]
  t: Translate
}

type CacheCleanupMode = 'all' | 'local' | 'docker'

export function SystemPage({ rows, t }: SystemPageProps) {
  const actions = [t('cleanCache')]
  const [showCacheDialog, setShowCacheDialog] = useState(false)
  const [cacheMode, setCacheMode] = useState<CacheCleanupMode>('all')

  return (
    <main className="workspace single-page">
      <section className="surface">
        <div className="section-header">
          <div>
            <h1>{t('systemHealth')}</h1>
          </div>
          <div className="system-header-actions" aria-label={t('systemActions')}>
            {actions.map((action) => (
              <button key={action} className="secondary-button" onClick={() => setShowCacheDialog(true)}>{action}</button>
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
      {showCacheDialog && (
        <div className="confirm-overlay">
          <section className="surface confirm-dialog" role="dialog" aria-modal="true" aria-label={t('cacheCleanupDialog')}>
            <div className="confirm-heading">
              <h2>{t('cacheCleanupDialog')}</h2>
            </div>
            <div className="cache-option-grid">
              <label>
                <input
                  type="radio"
                  name="cache-cleanup-mode"
                  checked={cacheMode === 'all'}
                  onChange={() => setCacheMode('all')}
                />
                <span>{t('cacheCleanAll')}</span>
              </label>
              <label>
                <input
                  type="radio"
                  name="cache-cleanup-mode"
                  checked={cacheMode === 'local'}
                  onChange={() => setCacheMode('local')}
                />
                <span>{t('cacheCleanLocal')}</span>
              </label>
              <label>
                <input
                  type="radio"
                  name="cache-cleanup-mode"
                  checked={cacheMode === 'docker'}
                  onChange={() => setCacheMode('docker')}
                />
                <span>{t('cacheCleanDocker')}</span>
              </label>
            </div>
            <div className="button-row confirm-actions">
              <button className="secondary-button" onClick={() => setShowCacheDialog(false)}>{t('cancel')}</button>
              <button className="primary-button">{t('confirmCleanup')}</button>
            </div>
          </section>
        </div>
      )}
    </main>
  )
}
