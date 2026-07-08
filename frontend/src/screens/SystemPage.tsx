import { useEffect, useState } from 'react'
import type { SystemRow } from '../domain/harbor'
import type { Translate } from '../i18n'
import { ConfirmDialog } from '../ui/components/ConfirmDialog'
import { Toast } from '../ui/components/Toast'

interface SystemPageProps {
  rows: SystemRow[]
  t: Translate
}

type ConfirmAction = 'docker-cache' | 'local-cache' | 'service-restart' | 'service-update'

interface ToastState {
  message: string
  remaining: number
}

const ornnlabVersion = {
  current: '0.1.3',
  latest: '0.1.3',
}

export function SystemPage({ rows, t }: SystemPageProps) {
  const [confirmAction, setConfirmAction] = useState<ConfirmAction | null>(null)
  const [toast, setToast] = useState<ToastState | null>(null)
  const confirmContent = confirmAction === 'docker-cache'
    ? {
        title: t('dockerCacheCleanupTitle'),
        impact: [t('dockerCacheImpactImages'), t('dockerCacheImpactRebuild')],
        confirm: t('confirmCleanup'),
      }
    : confirmAction === 'service-restart'
      ? {
        title: t('restartServiceTitle'),
        impact: [t('restartServiceImpactFrontend'), t('restartServiceImpactBackend')],
        confirm: t('confirmRestart'),
      }
      : confirmAction === 'service-update'
        ? {
          title: t('updateServiceTitle'),
          impact: [t('updateServiceImpactNpm'), t('updateServiceImpactRestart')],
          confirm: t('confirmUpdate'),
        }
    : {
        title: t('localCacheCleanupTitle'),
        impact: [t('localCacheImpactDirectory'), t('localCacheImpactRecreate')],
        confirm: t('confirmCleanup'),
      }

  const handleCheckUpdate = () => {
    if (ornnlabVersion.current !== ornnlabVersion.latest) {
      setConfirmAction('service-update')
      return
    }
    showToast(t('ornnlabAlreadyLatest'))
  }

  useEffect(() => {
    if (!toast) return undefined
    const timeout = window.setTimeout(() => {
      setToast((currentToast) => {
        if (!currentToast) return null
        if (currentToast.remaining <= 1) return null
        return { ...currentToast, remaining: currentToast.remaining - 1 }
      })
    }, 1000)

    return () => window.clearTimeout(timeout)
  }, [toast])

  const showToast = (message: string) => {
    setToast({ message, remaining: 3 })
  }

  const closeConfirm = () => setConfirmAction(null)
  const confirmAndClose = () => {
    setConfirmAction(null)
    if (confirmAction === 'service-restart') {
      showToast(t('restartRequestQueued'))
    }
    if (confirmAction === 'service-update') {
      showToast(t('updateRequestQueued'))
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
          <table className="system-health-table">
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
        <Toast dismissLabel={t('dismiss')} message={toast.message} remaining={toast.remaining} onDismiss={() => setToast(null)} />
      )}
      {confirmAction && (
        <ConfirmDialog
          cancelLabel={t('cancel')}
          confirmLabel={confirmContent.confirm}
          impacts={confirmContent.impact}
          title={confirmContent.title}
          onCancel={closeConfirm}
          onConfirm={confirmAndClose}
        />
      )}
    </main>
  )
}
