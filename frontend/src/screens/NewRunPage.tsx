import { RunBuilder } from '../ui/components/RunBuilder'
import type { DatasetRow, EnvironmentRow, RunDraft, TaskRow } from '../domain/harbor'
import type { Translate } from '../i18n'

interface NewRunPageProps {
  datasets: DatasetRow[]
  draft: RunDraft
  environments: EnvironmentRow[]
  taskRows: TaskRow[]
  t: Translate
  onDraft: (draft: RunDraft) => void
  onJobs: () => void
  onLaunch: () => void
}

export function NewRunPage({ datasets, draft, environments, taskRows, t, onDraft, onJobs, onLaunch }: NewRunPageProps) {
  return (
    <main className="workspace single-page">
      <div className="content-column">
        <nav className="breadcrumb-nav" aria-label="Job creation path">
          <button type="button" onClick={onJobs}>
            {t('jobs')}
          </button>
          <span aria-current="page">{t('newJob')}</span>
        </nav>
        <RunBuilder
          datasets={datasets}
          draft={draft}
          environments={environments}
          taskRows={taskRows}
          t={t}
          onDraft={onDraft}
          onCancel={onJobs}
          onLaunch={onLaunch}
        />
      </div>
    </main>
  )
}
