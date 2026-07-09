import { useDatasetTasks } from '../api/hooks'
import { datasetTaskDtoToDatasetTask } from '../api/viewModels'
import type { WebUiClient } from '../api/webUiClient'
import { RunBuilder } from '../ui/components/RunBuilder'
import type { DatasetRow, EnvironmentRow, RunDraft } from '../domain/harbor'
import type { Translate } from '../i18n'

interface NewRunPageProps {
  canSimulateWrites: boolean
  client: WebUiClient
  datasets: DatasetRow[]
  draft: RunDraft
  environments: EnvironmentRow[]
  t: Translate
  onDraft: (draft: RunDraft) => void
  onJobs: () => void
  onLaunch: () => void
}

export function NewRunPage({ canSimulateWrites, client, datasets, draft, environments, t, onDraft, onJobs, onLaunch }: NewRunPageProps) {
  const tasksResource = useDatasetTasks(client, draft.source)
  const taskRows = tasksResource.data?.items.map(datasetTaskDtoToDatasetTask) ?? []
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
          canLaunch={canSimulateWrites}
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
