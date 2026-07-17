import { Box, Download, FolderInput, MapPin, Search, Trash2, Unlink, X } from 'lucide-react'
import type { DatasetRow, DatasetTask } from '../../domain/harbor'
import type { Translate } from '../../i18n'
import { DatasetTaskEnvironment } from './DatasetTaskEnvironment'
import { Metric } from './Metric'

type DatasetDownloadState =
  | { status: 'not-downloaded' }
  | { progress: number; status: 'downloading' }
  | { path: string; size: string; status: 'downloaded' }
  | { path: string; status: 'path-unavailable' }

interface DatasetDetailProps {
  downloadState: DatasetDownloadState
  expandedTaskName: string | null
  isRegistryDataset: boolean
  selected: DatasetRow
  taskSearch: string
  tasks: DatasetTask[]
  t: Translate
  writeDisabled?: boolean
  onCancelDownload: (row: DatasetRow) => void
  onDelete: (row: DatasetRow) => void
  onExpandedTaskName: (taskName: string) => void
  onMove: (row: DatasetRow) => void
  onRelocate: (row: DatasetRow) => void
  onRemoveRegistration: (row: DatasetRow) => void
  onStartDownload: (row: DatasetRow) => void
  onSync: (row: DatasetRow) => void
  onRunTask: (row: DatasetRow, taskName: string) => void
  onTaskSearch: (value: string) => void
}

export function DatasetDetail({
  downloadState,
  expandedTaskName,
  isRegistryDataset,
  onCancelDownload,
  onDelete,
  onExpandedTaskName,
  onMove,
  onRelocate,
  onRemoveRegistration,
  onStartDownload,
  onSync,
  onTaskSearch,
  onRunTask,
  selected,
  taskSearch,
  tasks,
  t,
  writeDisabled = false,
}: DatasetDetailProps) {
  return (
    <aside className="detail-rail dataset-detail">
      <section className="surface rail-card">
        <div className="rail-heading">
          <div>
            <h2>{selected.name}</h2>
            <p>{selected.version}</p>
          </div>
          <span className={`status-dot ${selected.visibility === 'public' ? 'success' : 'queued'}`}>
            {selected.visibility}
          </span>
        </div>
        <div className="metric-grid">
          <Metric label={t('tasksCount')} value={String(selected.tasks)} />
          <Metric label={t('sourceRef')} value={selected.source} />
          <Metric label={t('path')} value={downloadState.status === 'downloaded' || downloadState.status === 'path-unavailable' ? downloadState.path : t('notDownloaded')} />
          <Metric label={t('size')} value={downloadState.status === 'downloaded' ? downloadState.size : t('notDownloaded')} />
          <Metric label={t('registry')} value={selected.registryUrl ?? '-'} />
        </div>
        <div className="button-row tight">
          {downloadState.status === 'not-downloaded' && (
            <button className="secondary-button" disabled={writeDisabled} onClick={() => onStartDownload(selected)}>
              <Download aria-hidden="true" />
              {t('download')}
            </button>
          )}
          {downloadState.status === 'downloading' && (
            <>
              <span className="progress-label">{downloadState.progress}%</span>
              <button className="secondary-button" disabled={writeDisabled} onClick={() => onCancelDownload(selected)}>
                <X aria-hidden="true" />
                {t('cancelDownload')}
              </button>
            </>
          )}
          {downloadState.status === 'downloaded' && isRegistryDataset && (
            <button className="secondary-button" disabled={writeDisabled} onClick={() => onSync(selected)}>{t('pullUpdates')}</button>
          )}
          {downloadState.status === 'downloaded' && selected.storageKind === 'managed' && (
            <button className="secondary-button" disabled={writeDisabled} onClick={() => onMove(selected)}>
              <FolderInput aria-hidden="true" />
              {t('moveDataset')}
            </button>
          )}
          {downloadState.status === 'downloaded' && selected.storageKind === 'managed' && (
            <button className="secondary-button" disabled={writeDisabled} onClick={() => onDelete(selected)}>
              <Trash2 aria-hidden="true" />
              {t('delete')}
            </button>
          )}
          {downloadState.status === 'downloaded' && selected.storageKind === 'external' && (
            <button className="secondary-button" disabled={writeDisabled} onClick={() => onRemoveRegistration(selected)}>
              <Unlink aria-hidden="true" />
              {t('removeRegistration')}
            </button>
          )}
          {downloadState.status === 'path-unavailable' && (
            <>
              <button className="secondary-button" disabled={writeDisabled} onClick={() => onRelocate(selected)}>
                <MapPin aria-hidden="true" />
                {t('relocateDataset')}
              </button>
              <button className="secondary-button" disabled={writeDisabled} onClick={() => onRemoveRegistration(selected)}>
                <Unlink aria-hidden="true" />
                {t('removeRegistration')}
              </button>
            </>
          )}
        </div>
      </section>
      <section className="surface rail-card">
        <div className="rail-title">
          <Box aria-hidden="true" />
          <h3>{t('datasetTasks')}</h3>
        </div>
        <div className="drawer-task-toolbar">
          <label className="search-field drawer-search">
            <Search aria-hidden="true" />
            <input aria-label={t('searchTasks')} value={taskSearch} onChange={(event) => onTaskSearch(event.target.value)} placeholder={t('searchTasks')} />
          </label>
        </div>
        <div className="mini-table">
          <div className="mini-row task-row mini-header" role="row">
            <span>{t('taskName')}</span>
            <span>{t('actions')}</span>
          </div>
          {tasks.map((row) => (
            <div key={row.name} className="task-entry">
              <div
                className="mini-row task-row task-toggle"
                role="button"
                tabIndex={0}
                aria-expanded={expandedTaskName === row.name}
                onClick={() => onExpandedTaskName(row.name)}
                onKeyDown={(event) => {
                  if (event.key !== 'Enter' && event.key !== ' ') return
                  event.preventDefault()
                  onExpandedTaskName(row.name)
                }}
              >
                <span>{row.name}</span>
                <div className="row-actions">
                  <button
                    className="row-action"
                    disabled={writeDisabled}
                    onClick={(event) => {
                      event.stopPropagation()
                      onRunTask(selected, row.name)
                    }}
                  >
                    {t('runSingleTask')}
                  </button>
                </div>
              </div>
              {expandedTaskName === row.name && (
                <div className="task-expanded">
                  {row.description && (
                    <div className="task-description">
                      <span>{t('description')}</span>
                      <p>{row.description}</p>
                    </div>
                  )}
                  <DatasetTaskEnvironment environment={row.environment} t={t} />
                </div>
              )}
            </div>
          ))}
          {tasks.length === 0 && <div className="empty-row">{t('noTasksAvailable')}</div>}
        </div>
      </section>
    </aside>
  )
}
