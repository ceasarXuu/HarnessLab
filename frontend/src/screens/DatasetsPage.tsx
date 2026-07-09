import { Database, Download, Plus, Search, Trash2, X } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { useDataset, useDatasetTasks, useOperation } from '../api/hooks'
import { datasetDtoToRow, datasetTaskDtoToDatasetTask } from '../api/viewModels'
import type { WebUiClient } from '../api/webUiClient'
import { DetailDrawer } from '../ui/components/DetailDrawer'
import { ConfirmDialog } from '../ui/components/ConfirmDialog'
import { DatasetDetail } from '../ui/components/DatasetDetail'
import { OperationStatus } from '../ui/components/OperationStatus'
import { ResourceStatus } from '../ui/components/ResourceStatus'
import type { DatasetRow } from '../domain/harbor'
import type { Translate } from '../i18n'

interface DatasetsPageProps {
  writesEnabled?: boolean
  client: WebUiClient
  rows: DatasetRow[]
  search: string
  t: Translate
  onRefresh: () => Promise<void>
  onSearch: (value: string) => void
}

type DatasetDownloadState =
  | { status: 'not-downloaded' }
  | { progress: number; status: 'downloading' }
  | { path: string; size: string; status: 'downloaded' }

const datasetKey = (row: DatasetRow) => `${row.name}@${row.version}`
const initialDownloadState = (row: DatasetRow): DatasetDownloadState =>
  row.downloadStatus === 'downloaded'
    ? {
        path: row.downloadPath ?? row.path ?? row.downloadDir ?? 'local dataset path',
        size: row.size ?? 'unknown',
        status: 'downloaded',
      }
    : { status: 'not-downloaded' }
const defaultImportDraft = {
  name: 'local/custom-dataset',
  path: './datasets/custom-dataset',
  tasks: '12',
  version: 'local',
}

export function DatasetsPage({ writesEnabled = true, client, rows, search, t, onRefresh, onSearch }: DatasetsPageProps) {
  const [selected, setSelected] = useState<DatasetRow | null>(null)
  const [expandedTaskName, setExpandedTaskName] = useState<string | null>(null)
  const [taskSearch, setTaskSearch] = useState('')
  const [taskSplit, setTaskSplit] = useState('all')
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<DatasetRow | null>(null)
  const [importDialogOpen, setImportDialogOpen] = useState(false)
  const [importDraft, setImportDraft] = useState(defaultImportDraft)
  const datasetOperation = useOperation(client)
  const selectedRef = selected?.ref ?? (selected ? datasetKey(selected) : undefined)
  const detailResource = useDataset(client, selectedRef)
  const tasksResource = useDatasetTasks(client, selectedRef)
  const detailRow = detailResource.data ? datasetDtoToRow(detailResource.data) : selected
  const selectedTasks = tasksResource.data?.items.map(datasetTaskDtoToDatasetTask) ?? []
  const visibleSelectedTasks = useMemo(() => {
    const query = taskSearch.trim().toLowerCase()
    const splitFilteredTasks = taskSplit === 'all'
      ? selectedTasks
      : selectedTasks.filter((row) => row.splits.includes(taskSplit))
    if (!query) return splitFilteredTasks
    return splitFilteredTasks.filter((row) =>
      [row.name, row.description].some((value) => value.toLowerCase().includes(query)),
    )
  }, [selectedTasks, taskSearch, taskSplit])
  const splitOptions = [
    { label: t('allSplits'), value: 'all' },
    ...(detailRow?.splits ?? []).map((split) => ({ label: split, value: split })),
  ]

  useEffect(() => {
    if (datasetOperation.operation?.status !== 'completed') return
    void onRefresh()
    void detailResource.refresh()
    void tasksResource.refresh()
  }, [datasetOperation.operation?.id, datasetOperation.operation?.status, detailResource.refresh, onRefresh, tasksResource.refresh])

  const downloadStateFor = (row: DatasetRow): DatasetDownloadState => {
    const operation = datasetOperation.operation
    if (operation && operation.resourceId === row.ref && operation.type === 'download-dataset' && (operation.status === 'queued' || operation.status === 'running')) {
      return { progress: operation.progress ?? 0, status: 'downloading' }
    }
    return initialDownloadState(row)
  }
  const startDownload = async (row: DatasetRow) => {
    if (!writesEnabled) return
    await datasetOperation.submit(() => client.downloadDataset(datasetKey(row)), ({ operation }) => operation)
  }
  const cancelDownload = async (row: DatasetRow) => {
    if (!writesEnabled) return
    await datasetOperation.submit(() => client.cancelDatasetDownload(datasetKey(row)), ({ operation }) => operation)
  }
  const syncDataset = async (row: DatasetRow) => {
    if (!writesEnabled) return
    await datasetOperation.submit(() => client.syncDataset(datasetKey(row)), ({ operation }) => operation)
  }
  const runTask = async (row: DatasetRow, taskName: string) => {
    if (!writesEnabled) return
    await datasetOperation.submit(() => client.runDatasetTask(datasetKey(row), taskName), ({ operation }) => operation)
  }
  const confirmDelete = async () => {
    if (!writesEnabled) return
    if (!deleteTarget) return
    await datasetOperation.submit(() => client.deleteLocalDataset(datasetKey(deleteTarget)), ({ operation }) => operation)
    setDeleteTarget(null)
  }
  const confirmImportDataset = async () => {
    if (!writesEnabled) return
    const taskCount = Number.parseInt(importDraft.tasks, 10)
    await datasetOperation.submit(
      () => client.importDataset({
        name: importDraft.name.trim() || defaultImportDraft.name,
        path: importDraft.path.trim() || defaultImportDraft.path,
        taskCount: Number.isFinite(taskCount) && taskCount > 0 ? taskCount : 0,
        version: importDraft.version.trim() || defaultImportDraft.version,
      }),
      ({ operation }) => operation,
    )
    setImportDialogOpen(false)
    setImportDraft(defaultImportDraft)
  }
  const selectedDownloadState = detailRow ? downloadStateFor(detailRow) : { status: 'not-downloaded' as const }
  const selectedIsRegistryDataset = detailRow?.registryUrl !== 'local'

  return (
    <main className="workspace single-page">
      <section className="surface dataset-catalog">
        <div className="section-header">
          <div>
            <h1>{t('datasetCatalog')}</h1>
          </div>
          <div className="toolbar">
            <label className="search-field">
              <Search aria-hidden="true" />
              <input
                aria-label={t('searchDatasets')}
                value={search}
                onChange={(event) => onSearch(event.target.value)}
                placeholder={t('searchDatasetsPlaceholder')}
              />
            </label>
            <button className="secondary-button" disabled={!writesEnabled} onClick={() => setImportDialogOpen(true)}>
              <Plus aria-hidden="true" />
              {t('importLocalDataset')}
            </button>
          </div>
        </div>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t('dataset')}</th>
                <th>{t('version')}</th>
                <th>{t('visibility')}</th>
                <th>{t('tasksCount')}</th>
                <th>{t('sourceRef')}</th>
                <th>{t('path')}</th>
                <th>{t('size')}</th>
                <th>{t('actions')}</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => {
                const downloadState = downloadStateFor(row)
                return (
                  <tr
                    key={`${row.name}-${row.version}`}
                    className={selected?.name === row.name && selected.version === row.version ? 'selected-row' : undefined}
                    onClick={() => {
                      setSelected(row)
                      setExpandedTaskName(null)
                      setTaskSearch('')
                      setTaskSplit('all')
                      setDrawerOpen(true)
                    }}
                  >
                    <td>
                      <span className="cell-title">
                        <Database aria-hidden="true" />
                        {row.name}
                      </span>
                    </td>
                    <td>{row.version}</td>
                    <td>
                      <span className={`status-dot ${row.visibility === 'public' ? 'success' : 'queued'}`}>
                        {row.visibility}
                      </span>
                    </td>
                    <td>{row.tasks}</td>
                    <td>{row.source}</td>
                    <td>{downloadState.status === 'downloaded' ? <code>{downloadState.path}</code> : t('notDownloaded')}</td>
                    <td>{downloadState.status === 'downloaded' ? downloadState.size : t('notDownloaded')}</td>
                    <td>
                      <div className="row-actions">
                        {downloadState.status === 'not-downloaded' && (
                          <button className="secondary-button compact-action" disabled={!writesEnabled} onClick={(event) => {
                            event.stopPropagation()
                            startDownload(row)
                          }}>
                            <Download aria-hidden="true" />
                            {t('download')}
                          </button>
                        )}
                        {downloadState.status === 'downloading' && (
                          <>
                            <span className="progress-label">{downloadState.progress}%</span>
                            <button className="secondary-button compact-action" disabled={!writesEnabled} onClick={(event) => {
                              event.stopPropagation()
                              cancelDownload(row)
                            }}>
                              <X aria-hidden="true" />
                              {t('cancelDownload')}
                            </button>
                          </>
                        )}
                        {downloadState.status === 'downloaded' && (
                          <button className="secondary-button compact-action" disabled={!writesEnabled} onClick={(event) => {
                            event.stopPropagation()
                            setDeleteTarget(row)
                          }}>
                            <Trash2 aria-hidden="true" />
                            {t('delete')}
                          </button>
                        )}
                      </div>
                    </td>
                  </tr>
                )
              })}
              {rows.length === 0 && (
                <tr>
                  <td className="empty-row" colSpan={8}>{t('noDatasetsAvailable')}</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </section>
      {detailRow && selected && (
        <DetailDrawer label={t('selectedDataset')} open={drawerOpen} onClose={() => setDrawerOpen(false)}>
          <>
            <DatasetDetail
              downloadState={selectedDownloadState}
              expandedTaskName={expandedTaskName}
              isRegistryDataset={selectedIsRegistryDataset}
              selected={detailRow}
              splitOptions={splitOptions}
              taskSearch={taskSearch}
              taskSplit={taskSplit}
              tasks={visibleSelectedTasks}
              t={t}
              writeDisabled={!writesEnabled || isOperationRunning(datasetOperation.operation?.status)}
              onCancelDownload={cancelDownload}
              onDelete={setDeleteTarget}
              onExpandedTaskName={setExpandedTaskName}
              onStartDownload={startDownload}
              onSync={syncDataset}
              onTaskSearch={setTaskSearch}
              onTaskSplit={setTaskSplit}
              onRunTask={runTask}
            />
            <ResourceStatus
              error={detailResource.error?.message ?? tasksResource.error?.message ?? null}
              loading={detailResource.loading || tasksResource.loading}
              loadingLabel={t('loadingDatasets')}
            />
          </>
        </DetailDrawer>
      )}
      {deleteTarget && (
        <ConfirmDialog
          cancelLabel={t('cancel')}
          confirmLabel={t('confirmDelete')}
          impacts={[t('deleteDatasetLocalImpact'), datasetKey(deleteTarget)]}
          title={t('deleteDatasetTitle')}
          onCancel={() => setDeleteTarget(null)}
          onConfirm={confirmDelete}
        />
      )}
      {importDialogOpen && (
        <ConfirmDialog
          cancelLabel={t('cancel')}
          confirmLabel={t('confirmImport')}
          impacts={[t('importLocalDatasetImpact')]}
          title={t('importLocalDataset')}
          onCancel={() => setImportDialogOpen(false)}
          onConfirm={confirmImportDataset}
        >
            <div className="import-dataset-form">
              <label>
                {t('datasetName')}
                <input
                  value={importDraft.name}
                  onChange={(event) => setImportDraft((current) => ({ ...current, name: event.target.value }))}
                />
              </label>
              <label>
                {t('version')}
                <input
                  value={importDraft.version}
                  onChange={(event) => setImportDraft((current) => ({ ...current, version: event.target.value }))}
                />
              </label>
              <label>
                {t('localPath')}
                <input
                  value={importDraft.path}
                  onChange={(event) => setImportDraft((current) => ({ ...current, path: event.target.value }))}
                />
              </label>
              <label>
                {t('tasksCount')}
                <input
                  min="0"
                  type="number"
                  value={importDraft.tasks}
                  onChange={(event) => setImportDraft((current) => ({ ...current, tasks: event.target.value }))}
                />
              </label>
            </div>
        </ConfirmDialog>
      )}
      <OperationStatus error={datasetOperation.error?.message} operation={datasetOperation.operation} t={t} />
    </main>
  )
}

function isOperationRunning(status: string | undefined) {
  return status === 'queued' || status === 'running'
}
