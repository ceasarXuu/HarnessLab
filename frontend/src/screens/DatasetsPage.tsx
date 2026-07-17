import { Database, Download, FolderInput, MapPin, Plus, Search, Trash2, Unlink, X } from 'lucide-react'
import { useCallback, useEffect, useMemo, useState } from 'react'
import { useCachedServerSearch, useDataset, useDatasetTasks, useOperation } from '../api/hooks'
import { datasetDtoToRow, datasetTaskDtoToDatasetTask } from '../api/viewModels'
import type { WebUiClient } from '../api/webUiClient'
import { orderDatasetCatalog } from '../domain/datasetOrdering'
import { DetailDrawer } from '../ui/components/DetailDrawer'
import { ConfirmDialog } from '../ui/components/ConfirmDialog'
import { DatasetDetail } from '../ui/components/DatasetDetail'
import { FolderPathInput, type FolderPathSelection } from '../ui/components/FolderPathInput'
import { Pagination } from '../ui/components/Pagination'
import { ResourceStatus } from '../ui/components/ResourceStatus'
import { usePaginatedItems } from '../ui/pagination'
import type { DatasetRow, DatasetTask } from '../domain/harbor'
import type { Translate } from '../i18n'

interface DatasetsPageProps {
  writesEnabled?: boolean
  client: WebUiClient
  rows: DatasetRow[]
  search: string
  t: Translate
  onRefresh: () => Promise<void>
  onPrepareTaskRun?: (datasetRef: string, taskName: string) => void
  onSearch: (value: string) => void
}

type DatasetDownloadState =
  | { status: 'not-downloaded' }
  | { progress: number; status: 'downloading' }
  | { path: string; size: string; status: 'downloaded' }
  | { path: string; status: 'path-unavailable' }

type LocationAction = {
  mode: 'download' | 'move' | 'relocate'
  row: DatasetRow
}

const datasetKey = (row: DatasetRow) => `${row.name}@${row.version}`
const initialDownloadState = (row: DatasetRow): DatasetDownloadState =>
  row.downloadStatus === 'downloaded'
    ? {
        path: row.downloadPath ?? row.path ?? row.downloadDir ?? 'local dataset path',
        size: row.size ?? 'unknown',
        status: 'downloaded',
      }
    : row.downloadStatus === 'path-unavailable'
      ? { path: row.downloadPath ?? row.path ?? '', status: 'path-unavailable' }
    : { status: 'not-downloaded' }
const defaultImportDraft = {
  name: 'local/custom-dataset',
  path: './datasets/custom-dataset',
  tasks: '12',
  version: 'local',
}

export function DatasetsPage({ writesEnabled = true, client, rows, search, t, onRefresh, onPrepareTaskRun, onSearch }: DatasetsPageProps) {
  const [selected, setSelected] = useState<DatasetRow | null>(null)
  const [expandedTaskName, setExpandedTaskName] = useState<string | null>(null)
  const [taskEnvironmentOverrides, setTaskEnvironmentOverrides] = useState<Record<string, DatasetTask['environment']>>({})
  const [taskSearch, setTaskSearch] = useState('')
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<DatasetRow | null>(null)
  const [locationAction, setLocationAction] = useState<LocationAction | null>(null)
  const [locationPath, setLocationPath] = useState('')
  const [registrationTarget, setRegistrationTarget] = useState<DatasetRow | null>(null)
  const [importDialogOpen, setImportDialogOpen] = useState(false)
  const [importDraft, setImportDraft] = useState(defaultImportDraft)
  const datasetOperation = useOperation(client)
  const selectedRef = selected?.ref ?? (selected ? datasetKey(selected) : undefined)
  const detailResource = useDataset(client, selectedRef)
  const tasksResource = useDatasetTasks(client, selectedRef, { limit: 100 })
  const taskSearchQuery = taskSearch.trim() || undefined
  const loadTaskSearch = useCallback(
    (query: string) => client.listDatasetTasks(selectedRef ?? '', { limit: 100, q: query }),
    [client, selectedRef],
  )
  const taskSearchResource = useCachedServerSearch(selectedRef, taskSearchQuery, loadTaskSearch)
  const detailRow = detailResource.data ? datasetDtoToRow(detailResource.data) : selected
  const applyTaskEnvironmentOverride = useCallback((task: DatasetTask) => {
    const key = `${task.datasetRef}:${task.name}`
    return Object.hasOwn(taskEnvironmentOverrides, key)
      ? { ...task, environment: taskEnvironmentOverrides[key] }
      : task
  }, [taskEnvironmentOverrides])
  const selectedTasks = useMemo(
    () => (tasksResource.data?.items.map(datasetTaskDtoToDatasetTask) ?? []).map(applyTaskEnvironmentOverride),
    [applyTaskEnvironmentOverride, tasksResource.data],
  )
  const visibleSelectedTasks = useMemo(() => {
    if (!taskSearchQuery) return selectedTasks
    if (taskSearchResource.data) {
      return taskSearchResource.data.items
        .map(datasetTaskDtoToDatasetTask)
        .map(applyTaskEnvironmentOverride)
    }
    const query = taskSearchQuery.toLowerCase()
    return selectedTasks.filter((row) =>
      [row.name, row.description].some((value) => value.toLowerCase().includes(query)),
    )
  }, [applyTaskEnvironmentOverride, selectedTasks, taskSearchQuery, taskSearchResource.data])
  const orderedRows = useMemo(() => orderDatasetCatalog(rows), [rows])
  const pagination = usePaginatedItems({ items: orderedRows, resetKey: search })

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
  const openLocationAction = async (mode: LocationAction['mode'], row: DatasetRow) => {
    if (!writesEnabled) return
    if (mode === 'relocate') {
      setLocationPath(row.downloadPath ?? row.path ?? '')
      setLocationAction({ mode, row })
      return
    }
    const preference = await client.getDatasetDefaultParent()
    setLocationPath(preference.data?.parentPath ?? '')
    setLocationAction({ mode, row })
  }
  const chooseDirectory = async (): Promise<FolderPathSelection> => {
    const response = await client.chooseDirectory()
    return { error: response.error?.message, path: response.data?.path ?? null }
  }
  const confirmLocationAction = async () => {
    if (!writesEnabled || !locationAction) return
    const { mode, row } = locationAction
    const path = locationPath.trim()
    if (mode === 'download') {
      await datasetOperation.submit(() => client.downloadDataset(datasetKey(row), { parentPath: path }), ({ operation }) => operation)
    } else if (mode === 'move') {
      await datasetOperation.submit(() => client.moveDataset(datasetKey(row), { parentPath: path }), ({ operation }) => operation)
    } else {
      await datasetOperation.submit(() => client.relocateDataset(datasetKey(row), { path }), ({ operation }) => operation)
    }
    setLocationAction(null)
    setLocationPath('')
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
    onPrepareTaskRun?.(datasetKey(row), taskName)
  }
  const expandTask = async (taskName: string) => {
    if (expandedTaskName === taskName) {
      setExpandedTaskName(null)
      return
    }
    setExpandedTaskName(taskName)
    if (!selectedRef) return
    const key = `${selectedRef}:${taskName}`
    if (Object.hasOwn(taskEnvironmentOverrides, key)) return
    const response = await client.getDatasetTaskEnvironment(selectedRef, taskName)
    const currentEnvironment = selectedTasks.find((task) => task.name === taskName)?.environment ?? null
    const resolvedEnvironment = response.data ?? (
      currentEnvironment ? { ...currentEnvironment, imagePlatforms: [] } : null
    )
    setTaskEnvironmentOverrides((current) => ({ ...current, [key]: resolvedEnvironment }))
  }
  const confirmDelete = async () => {
    if (!writesEnabled) return
    if (!deleteTarget) return
    await datasetOperation.submit(() => client.deleteLocalDataset(datasetKey(deleteTarget)), ({ operation }) => operation)
    setDeleteTarget(null)
  }
  const confirmRemoveRegistration = async () => {
    if (!writesEnabled || !registrationTarget) return
    await datasetOperation.submit(() => client.removeDatasetRegistration(datasetKey(registrationTarget)), ({ operation }) => operation)
    setRegistrationTarget(null)
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
                <th>{t('size')}</th>
                <th>{t('actions')}</th>
              </tr>
            </thead>
            <tbody>
              {pagination.items.map((row) => {
                const downloadState = downloadStateFor(row)
                return (
                  <tr
                    key={`${row.name}-${row.version}`}
                    className={selected?.name === row.name && selected.version === row.version ? 'selected-row' : undefined}
                    onClick={() => {
                      setSelected(row)
                      setExpandedTaskName(null)
                      setTaskSearch('')
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
                    <td>
                      {downloadState.status === 'downloaded'
                        ? downloadState.size
                        : downloadState.status === 'path-unavailable'
                          ? t('pathUnavailable')
                          : t('notDownloaded')}
                    </td>
                    <td>
                      <div className="row-actions">
                        {downloadState.status === 'not-downloaded' && (
                          <button className="secondary-button compact-action" disabled={!writesEnabled} onClick={(event) => {
                            event.stopPropagation()
                            void openLocationAction('download', row)
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
                          <>
                            {row.storageKind === 'managed' && (
                              <>
                                <button className="secondary-button compact-action" disabled={!writesEnabled} onClick={(event) => {
                                  event.stopPropagation()
                                  void openLocationAction('move', row)
                                }}>
                                  <FolderInput aria-hidden="true" />
                                  {t('moveDataset')}
                                </button>
                                <button className="secondary-button compact-action" disabled={!writesEnabled} onClick={(event) => {
                                  event.stopPropagation()
                                  setDeleteTarget(row)
                                }}>
                                  <Trash2 aria-hidden="true" />
                                  {t('delete')}
                                </button>
                              </>
                            )}
                            {row.storageKind === 'external' && (
                              <button className="secondary-button compact-action" disabled={!writesEnabled} onClick={(event) => {
                                event.stopPropagation()
                                setRegistrationTarget(row)
                              }}>
                                <Unlink aria-hidden="true" />
                                {t('removeRegistration')}
                              </button>
                            )}
                          </>
                        )}
                        {downloadState.status === 'path-unavailable' && (
                          <>
                            <button className="secondary-button compact-action" disabled={!writesEnabled} onClick={(event) => {
                              event.stopPropagation()
                              void openLocationAction('relocate', row)
                            }}>
                              <MapPin aria-hidden="true" />
                              {t('relocateDataset')}
                            </button>
                            <button className="secondary-button compact-action" disabled={!writesEnabled} onClick={(event) => {
                              event.stopPropagation()
                              setRegistrationTarget(row)
                            }}>
                              <Unlink aria-hidden="true" />
                              {t('removeRegistration')}
                            </button>
                          </>
                        )}
                      </div>
                    </td>
                  </tr>
                )
              })}
              {orderedRows.length === 0 && (
                <tr>
                  <td className="empty-row" colSpan={7}>{t('noDatasetsAvailable')}</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
        <Pagination {...pagination} t={t} onPage={pagination.setPage} />
      </section>
      {detailRow && selected && (
        <DetailDrawer label={t('selectedDataset')} open={drawerOpen} onClose={() => setDrawerOpen(false)}>
          <>
            <DatasetDetail
              downloadState={selectedDownloadState}
              expandedTaskName={expandedTaskName}
              isRegistryDataset={selectedIsRegistryDataset}
              selected={detailRow}
              taskSearch={taskSearch}
              tasks={visibleSelectedTasks}
              t={t}
              writeDisabled={!writesEnabled || isOperationRunning(datasetOperation.operation?.status)}
              onCancelDownload={cancelDownload}
              onDelete={setDeleteTarget}
              onExpandedTaskName={(taskName) => void expandTask(taskName)}
              onStartDownload={(row) => void openLocationAction('download', row)}
              onMove={(row) => void openLocationAction('move', row)}
              onRelocate={(row) => void openLocationAction('relocate', row)}
              onRemoveRegistration={setRegistrationTarget}
              onSync={syncDataset}
              onTaskSearch={setTaskSearch}
              onRunTask={runTask}
            />
            <ResourceStatus
              error={detailResource.error?.message ?? tasksResource.error?.message ?? taskSearchResource.error?.message ?? null}
              loading={detailResource.loading || tasksResource.loading || taskSearchResource.loading}
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
      {registrationTarget && (
        <ConfirmDialog
          cancelLabel={t('cancel')}
          confirmLabel={t('confirmRemoveRegistration')}
          impacts={[t('removeDatasetRegistrationImpact'), datasetKey(registrationTarget)]}
          title={t('removeDatasetRegistrationTitle')}
          onCancel={() => setRegistrationTarget(null)}
          onConfirm={confirmRemoveRegistration}
        />
      )}
      {locationAction && (
        <ConfirmDialog
          cancelLabel={t('cancel')}
          confirmLabel={t(locationAction.mode === 'download' ? 'confirmDownload' : locationAction.mode === 'move' ? 'confirmMoveDataset' : 'confirmRelocateDataset')}
          impacts={[t(locationAction.mode === 'relocate' ? 'relocateDatasetImpact' : 'datasetParentPathImpact')]}
          title={t(locationAction.mode === 'download' ? 'downloadDatasetTitle' : locationAction.mode === 'move' ? 'moveDatasetTitle' : 'relocateDatasetTitle')}
          onCancel={() => {
            setLocationAction(null)
            setLocationPath('')
          }}
          onConfirm={confirmLocationAction}
        >
          <div className="dataset-location-form">
            <label>
              {t(locationAction.mode === 'relocate' ? 'datasetPath' : 'datasetParentPath')}
              <FolderPathInput
                chooseLabel={t('chooseFolder')}
                label={t(locationAction.mode === 'relocate' ? 'datasetPath' : 'datasetParentPath')}
                value={locationPath}
                onChange={setLocationPath}
                onChoose={chooseDirectory}
              />
            </label>
          </div>
        </ConfirmDialog>
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
    </main>
  )
}

function isOperationRunning(status: string | undefined) {
  return status === 'queued' || status === 'running'
}
