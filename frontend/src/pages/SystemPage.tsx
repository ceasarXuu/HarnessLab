import { ServerCog } from 'lucide-react'
import { useState } from 'react'
import type { SystemRow } from '../data/demo'
import type { Translate } from '../i18n'

interface SystemPageProps {
  rows: SystemRow[]
  t: Translate
}

export function SystemPage({ rows, t }: SystemPageProps) {
  const actions = [t('cleanCache')]
  const [showCacheDialog, setShowCacheDialog] = useState(false)
  const [dryRun, setDryRun] = useState(true)
  const [skipDocker, setSkipDocker] = useState(false)
  const [skipCacheDir, setSkipCacheDir] = useState(false)
  const [force, setForce] = useState(false)
  const cacheCommand = [
    'harbor cache clean',
    dryRun ? '--dry' : '',
    skipDocker ? '--no-docker' : '',
    skipCacheDir ? '--no-cache-dir' : '',
    force ? '--force' : '',
  ]
    .filter(Boolean)
    .join(' ')

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
              <p>{t('cacheCleanupDesc')}</p>
            </div>
            <div className="cache-option-grid">
              <label>
                <input type="checkbox" checked={dryRun} onChange={(event) => setDryRun(event.target.checked)} />
                <span>{t('cacheDryRun')}</span>
              </label>
              <label>
                <input type="checkbox" checked={skipDocker} onChange={(event) => setSkipDocker(event.target.checked)} />
                <span>{t('cacheNoDocker')}</span>
              </label>
              <label>
                <input type="checkbox" checked={skipCacheDir} onChange={(event) => setSkipCacheDir(event.target.checked)} />
                <span>{t('cacheNoCacheDir')}</span>
              </label>
              <label>
                <input type="checkbox" checked={force} onChange={(event) => setForce(event.target.checked)} />
                <span>{t('cacheForce')}</span>
              </label>
            </div>
            <div className="config-preview cache-command-preview">
              <code>{cacheCommand}</code>
            </div>
            {!dryRun && !force && <p className="confirm-warning">{t('cacheConfirmWarning')}</p>}
            <div className="button-row confirm-actions">
              <button className="secondary-button" onClick={() => setShowCacheDialog(false)}>{t('cancel')}</button>
              <button className="primary-button">{dryRun ? t('runDryRun') : t('confirmCleanup')}</button>
            </div>
          </section>
        </div>
      )}
    </main>
  )
}
