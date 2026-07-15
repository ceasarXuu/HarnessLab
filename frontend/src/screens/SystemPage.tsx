import { useEffect, useState } from 'react'
import { useOperation } from '../api/hooks'
import type { WebUiClient } from '../api/webUiClient'
import type { SystemRow } from '../domain/harbor'
import type { Translate } from '../i18n'
import { ConfirmDialog } from '../ui/components/ConfirmDialog'
import { Pagination } from '../ui/components/Pagination'
import { ResourceStatus } from '../ui/components/ResourceStatus'
import { Toast } from '../ui/components/Toast'
import { usePaginatedItems } from '../ui/pagination'

interface SystemPageProps {
  writesEnabled?: boolean
  client: WebUiClient
  rows: SystemRow[]
  t: Translate
  onRefresh: () => Promise<void>
}

type ConfirmAction = 'docker-cache' | 'local-cache' | 'service-restart' | 'service-update'

interface ToastState {
  message: string
  remaining: number
}

export function SystemPage({ writesEnabled = true, client, rows, t, onRefresh }: SystemPageProps) {
  const [confirmAction, setConfirmAction] = useState<ConfirmAction | null>(null)
  const [toast, setToast] = useState<ToastState | null>(null)
  const [updateCheckError, setUpdateCheckError] = useState<string | null>(null)
  const systemOperation = useOperation(client)
  const pagination = usePaginatedItems({ items: rows })
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

  useEffect(() => {
    if (systemOperation.operation?.status !== 'completed') return
    void onRefresh()
  }, [onRefresh, systemOperation.operation?.id, systemOperation.operation?.status])

  const handleCheckUpdate = async () => {
    if (!writesEnabled) return
    const response = await client.checkForSystemUpdate()
    if (response.error) {
      setUpdateCheckError(response.error.message)
      return
    }
    setUpdateCheckError(null)
    if (response.data?.updateAvailable) {
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
  const confirmAndClose = async () => {
    if (!writesEnabled) return
    const action = confirmAction
    if (!action) return
    const mutation = action === 'docker-cache'
      ? () => client.cleanDockerCache()
      : action === 'local-cache'
        ? () => client.cleanStorageCache()
        : action === 'service-restart'
          ? () => client.restartSystemService()
          : () => client.installSystemUpdate()
    await systemOperation.submit(mutation, ({ operation }) => operation)
    setConfirmAction(null)
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
              {pagination.items.map((row) => (
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
                          <button className="secondary-button compact-action" disabled={!writesEnabled || isOperationRunning(systemOperation.operation?.status)} onClick={handleCheckUpdate}>
                            {t('checkUpdate')}
                          </button>
                          <button className="secondary-button compact-action" disabled={!writesEnabled || isOperationRunning(systemOperation.operation?.status)} onClick={() => setConfirmAction('service-restart')}>
                            {t('restart')}
                          </button>
                        </>
                      )}
                      {row.kind === 'docker' && (
                        <button className="secondary-button compact-action" disabled={!writesEnabled || isOperationRunning(systemOperation.operation?.status)} onClick={() => setConfirmAction('docker-cache')}>
                          {t('cleanCache')}
                        </button>
                      )}
                      {row.kind === 'storage' && (
                        <button className="secondary-button compact-action" disabled={!writesEnabled || isOperationRunning(systemOperation.operation?.status)} onClick={() => setConfirmAction('local-cache')}>
                          {t('cleanCache')}
                        </button>
                      )}
                    </div>
                  </td>
                </tr>
              ))}
              {rows.length === 0 && (
                <tr>
                  <td className="empty-row" colSpan={5}>{t('noSystemComponents')}</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
        <Pagination {...pagination} t={t} onPage={pagination.setPage} />
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
      <ResourceStatus error={updateCheckError ?? systemOperation.error?.message ?? null} />
    </main>
  )
}

function isOperationRunning(status: string | undefined) {
  return status === 'queued' || status === 'running'
}
