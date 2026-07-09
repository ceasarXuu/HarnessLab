import type { Operation } from '../../api/contract'
import type { Translate } from '../../i18n'

interface OperationStatusProps {
  error?: string | null
  operation?: Operation | null
  t: Translate
}

export function OperationStatus({ error = null, operation = null, t }: OperationStatusProps) {
  if (error) return <div className="resource-state error" role="alert">{error}</div>
  if (!operation) return null
  const label = operationLabel(operation.status, t)
  return (
    <div className={`resource-state operation-${operation.status}`} role="status">
      {label}{operation.progress === undefined ? '' : ` ${operation.progress}%`}
    </div>
  )
}

function operationLabel(status: Operation['status'], t: Translate) {
  if (status === 'completed') return t('operationCompleted')
  if (status === 'failed') return t('operationFailed')
  if (status === 'cancelled') return t('operationCancelled')
  return status === 'queued' ? t('operationQueued') : t('operationRunning')
}
