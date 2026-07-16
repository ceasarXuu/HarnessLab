import { useEffect, useState } from 'react'
import { useOperation } from '../api/hooks'
import type { WebUiClient } from '../api/webUiClient'
import type { SystemRow } from '../domain/harbor'
import type { Translate } from '../i18n'
import { ConfirmDialog } from '../ui/components/ConfirmDialog'
import { ResourceStatus } from '../ui/components/ResourceStatus'
import { SystemDashboard } from '../ui/components/SystemDashboard'
import { Toast } from '../ui/components/Toast'

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
        <SystemDashboard
          disabled={!writesEnabled || isOperationRunning(systemOperation.operation?.status)}
          rows={rows}
          t={t}
          onCheckUpdate={handleCheckUpdate}
          onCleanDockerCache={() => setConfirmAction('docker-cache')}
          onCleanStorageCache={() => setConfirmAction('local-cache')}
          onRestartService={() => setConfirmAction('service-restart')}
        />
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
