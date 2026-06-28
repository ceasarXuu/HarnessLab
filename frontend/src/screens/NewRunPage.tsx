import { RunBuilder } from '../ui/components/RunBuilder'
import type { DatasetRow, RunDraft } from '../mocks/demo'
import type { Translate } from '../i18n'

interface NewRunPageProps {
  datasets: DatasetRow[]
  draft: RunDraft
  t: Translate
  onDraft: (draft: RunDraft) => void
  onJobs: () => void
  onLaunch: () => void
}

export function NewRunPage({ datasets, draft, t, onDraft, onJobs, onLaunch }: NewRunPageProps) {
  return (
    <main className="workspace single-page">
      <div className="content-column">
        <nav className="breadcrumb-nav" aria-label="Job creation path">
          <button type="button" onClick={onJobs}>
            {t('jobs')}
          </button>
          <span aria-current="page">{t('newJob')}</span>
        </nav>
        <RunBuilder datasets={datasets} draft={draft} t={t} onDraft={onDraft} onLaunch={onLaunch} />
      </div>
    </main>
  )
}
