import type { JobStatus } from '../../domain/harbor'
import type { Translate } from '../../i18n'

interface JobStatusBadgeProps {
  status: JobStatus
  t: Translate
}

const statusTranslationKeys: Record<JobStatus, Parameters<Translate>[0]> = {
  cancelled: 'statusCancelled',
  completed: 'statusCompleted',
  draft: 'statusDraft',
  failed: 'statusFailed',
  interrupted: 'statusInterrupted',
  queued: 'statusQueued',
  running: 'statusRunning',
}

export function JobStatusBadge({ status, t }: JobStatusBadgeProps) {
  return <span className={`status-dot ${status}`}>{t(statusTranslationKeys[status])}</span>
}
