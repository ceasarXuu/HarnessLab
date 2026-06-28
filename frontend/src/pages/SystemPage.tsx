import { useState } from 'react'
import type { SystemRow } from '../data/demo'
import type { Translate } from '../i18n'

interface SystemPageProps {
  rows: SystemRow[]
  t: Translate
}

type ConfirmAction = 'docker-cache' | 'local-cache' | 'service-restart' | 'service-update'

const ornnlabVersion = {
  current: '0.1.3',
  latest: '0.1.3',
}

export function SystemPage({ rows, t }: SystemPageProps) {
  const [confirmAction, setConfirmAction] = useState<ConfirmAction | null>(null)
  const [toast, setToast] = useState<string | null>(null)
  const confirmContent = confirmAction === 'docker-cache'
    ? {
        title: t('dockerCacheCleanupTitle'),
        body: t('dockerCacheCleanupBody'),
        impact: [t('dockerCacheImpactImages'), t('dockerCacheImpactRebuild')],
        confirm: t('confirmCleanup'),
      }
    : confirmAction === 'service-restart'
      ? {
        title: t('restartServiceTitle'),
        body: t('restartServiceBody'),
        impact: [t('restartServiceImpactFrontend'), t('restartServiceImpactBackend')],
        confirm: t('confirmRestart'),
      }
      : confirmAction === 'service-update'
        ? {
          title: t('updateServiceTitle'),
          body: t('updateServiceBody'),
          impact: [t('updateServiceImpactNpm'), t('updateServiceImpactRestart')],
          confirm: t('confirmUpdate'),
        }
    : {
        title: t('localCacheCleanupTitle'),
        body: t('localCacheCleanupBody'),
        impact: [t('localCacheImpactDirectory'), t('localCacheImpactRecreate')],
        confirm: t('confirmCleanup'),
      }

  const handleCheckUpdate = () => {
    if (ornnlabVersion.current !== ornnlabVersion.latest) {
      setConfirmAction('service-update')
      return
    }
    setToast(t('ornnlabAlreadyLatest'))
  }

  const closeConfirm = () => setConfirmAction(null)
  const confirmAndClose = () => {
    setConfirmAction(null)
    if (confirmAction === 'service-restart') {
      setToast(t('restartRequestQueued'))
    }
    if (confirmAction === 'service-update') {
      setToast(t('updateRequestQueued'))
    }
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
                <th>{t('path')}</th>
                <th>{t('actions')}</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={row.component}>
                  <td>
                    <span className="cell-title">
                      {row.component}
                    </span>
                  </td>
                  <td>
                    <span className={`status-dot ${row.status}`}>{row.status}</span>
                  </td>
                  <td>{row.value}</td>
                  <td>
                    <code>{row.path}</code>
                  </td>
                  <td>
                    <div className="row-actions">
                      {row.kind === 'ornnlab-service' && (
                        <>
                          <button className="secondary-button compact-action" onClick={handleCheckUpdate}>
                            {t('checkUpdate')}
                          </button>
                          <button className="secondary-button compact-action" onClick={() => setConfirmAction('service-restart')}>
                            {t('restart')}
                          </button>
                        </>
                      )}
                      {row.kind === 'docker' && (
                        <button className="secondary-button compact-action" onClick={() => setConfirmAction('docker-cache')}>
                          {t('cleanCache')}
                        </button>
                      )}
                      {row.kind === 'storage' && (
                        <button className="secondary-button compact-action" onClick={() => setConfirmAction('local-cache')}>
                          {t('cleanCache')}
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
      {toast && (
        <div className="toast" role="status">
          <span>{toast}</span>
          <button className="icon-button" aria-label={t('dismiss')} onClick={() => setToast(null)}>x</button>
        </div>
      )}
      {confirmAction && (
        <div className="confirm-overlay">
          <section className="surface confirm-dialog" role="dialog" aria-modal="true" aria-label={confirmContent.title}>
            <div className="confirm-heading">
              <h2>{confirmContent.title}</h2>
              <p>{confirmContent.body}</p>
            </div>
            <ul className="cleanup-impact-list">
              {confirmContent.impact.map((item) => (
                <li key={item}>{item}</li>
              ))}
            </ul>
            <div className="button-row confirm-actions">
              <button className="secondary-button" onClick={closeConfirm}>{t('cancel')}</button>
              <button className="primary-button" onClick={confirmAndClose}>{confirmContent.confirm}</button>
            </div>
          </section>
        </div>
      )}
    </main>
  )
}
