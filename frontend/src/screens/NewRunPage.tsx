import { useDatasetTasks } from '../api/hooks'
import { datasetTaskDtoToDatasetTask } from '../api/viewModels'
import type { WebUiClient } from '../api/webUiClient'
import { RunBuilder } from '../ui/components/RunBuilder'
import type { FolderPathSelection } from '../ui/components/FolderPathInput'
import type { AgentRow, DatasetRow, EnvironmentRow, RunDraft } from '../domain/harbor'
import type { Translate } from '../i18n'

interface NewRunPageProps {
  canLaunch: boolean
  agents: AgentRow[]
  client: WebUiClient
  datasets: DatasetRow[]
  draft: RunDraft
  environments: EnvironmentRow[]
  t: Translate
  onDraft: (draft: RunDraft) => void
  onJobs: () => void
  onCopyJobConfig: () => void
  onLaunch: () => void
  onReset: () => void
}

export function NewRunPage({ canLaunch, agents, client, datasets, draft, environments, t, onDraft, onJobs, onCopyJobConfig, onLaunch, onReset }: NewRunPageProps) {
  const tasksResource = useDatasetTasks(client, draft.source)
  const taskRows = tasksResource.data?.items.map(datasetTaskDtoToDatasetTask) ?? []
  const chooseDirectory = async (): Promise<FolderPathSelection> => {
    const response = await client.chooseDirectory()
    return { error: response.error?.message, path: response.data?.path ?? null }
  }
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
          canLaunch={canLaunch}
          agents={agents}
          datasets={datasets}
          draft={draft}
          environments={environments}
          taskRows={taskRows}
          t={t}
          onDraft={onDraft}
          onCancel={onJobs}
          onCopyJobConfig={onCopyJobConfig}
          onChooseDirectory={chooseDirectory}
          onLaunch={onLaunch}
          onReset={onReset}
        />
      </div>
    </main>
  )
}
