import { Database, Download, Plus, Search, Trash2, X } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { useDataset, useDatasetTasks } from '../api/hooks'
import { datasetDtoToRow, datasetTaskDtoToDatasetTask } from '../api/viewModels'
import type { WebUiClient } from '../api/webUiClient'
import { DetailDrawer } from '../ui/components/DetailDrawer'
import { ConfirmDialog } from '../ui/components/ConfirmDialog'
import { DatasetDetail } from '../ui/components/DatasetDetail'
import type { DatasetRow } from '../domain/harbor'
import type { Translate } from '../i18n'

interface DatasetsPageProps {
  allowMockWrites?: boolean
  client: WebUiClient
  rows: DatasetRow[]
  search: string
  t: Translate
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

export function DatasetsPage({ allowMockWrites = true, client, rows, search, t, onSearch }: DatasetsPageProps) {
  const [selected, setSelected] = useState<DatasetRow | null>(null)
  const [expandedTaskName, setExpandedTaskName] = useState<string | null>(null)
  const [taskSearch, setTaskSearch] = useState('')
  const [taskSplit, setTaskSplit] = useState('all')
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<DatasetRow | null>(null)
  const [importDialogOpen, setImportDialogOpen] = useState(false)
  const [importDraft, setImportDraft] = useState(defaultImportDraft)
  const [importedRows, setImportedRows] = useState<DatasetRow[]>([])
  const [downloads, setDownloads] = useState<Record<string, DatasetDownloadState>>(() =>
    Object.fromEntries(rows.map((row) => [datasetKey(row), initialDownloadState(row)])),
  )
  const visibleRows = useMemo(() => {
    const query = search.trim().toLowerCase()
    const visibleImportedRows = query
      ? importedRows.filter((row) =>
          [row.name, row.version, row.visibility, row.source, row.digest, row.path ?? ''].some((value) =>
            (value ?? '').toLowerCase().includes(query),
          ),
        )
      : importedRows

    return [...visibleImportedRows, ...rows]
  }, [importedRows, rows, search])
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
    setDownloads((current) => {
      const next = { ...current }
      for (const row of rows) {
        const key = datasetKey(row)
        if (!next[key]) next[key] = initialDownloadState(row)
      }
      return next
    })
  }, [rows])

  useEffect(() => {
    const activeDownloads = Object.entries(downloads).filter(([, value]) => value.status === 'downloading')
    if (activeDownloads.length === 0) return undefined

    const timer = window.setInterval(() => {
      setDownloads((current) => {
        const next = { ...current }
        for (const [key, value] of Object.entries(current)) {
          if (value.status !== 'downloading') continue
          const progress = Math.min(value.progress + 20, 100)
          next[key] = progress >= 100
            ? { path: `~/.cache/harbor/datasets/${key.replace('@', '-')}`, size: 'pending scan', status: 'downloaded' }
            : { progress, status: 'downloading' }
        }
        return next
      })
    }, 800)

    return () => window.clearInterval(timer)
  }, [downloads])

  const downloadStateFor = (row: DatasetRow) => downloads[datasetKey(row)] ?? { status: 'not-downloaded' }
  const startDownload = (row: DatasetRow) => {
    if (!allowMockWrites) return
    setDownloads((current) => ({ ...current, [datasetKey(row)]: { progress: 0, status: 'downloading' } }))
  }
  const cancelDownload = (row: DatasetRow) => {
    if (!allowMockWrites) return
    setDownloads((current) => ({ ...current, [datasetKey(row)]: { status: 'not-downloaded' } }))
  }
  const confirmDelete = () => {
    if (!allowMockWrites) return
    if (!deleteTarget) return
    setDownloads((current) => ({ ...current, [datasetKey(deleteTarget)]: { status: 'not-downloaded' } }))
    setDeleteTarget(null)
  }
  const confirmImportDataset = () => {
    if (!allowMockWrites) return
    const taskCount = Number.parseInt(importDraft.tasks, 10)
    const nextRow: DatasetRow = {
      name: importDraft.name.trim() || defaultImportDraft.name,
      version: importDraft.version.trim() || defaultImportDraft.version,
      visibility: 'private',
      tasks: Number.isFinite(taskCount) && taskCount > 0 ? taskCount : 0,
      source: t('localImport'),
      digest: t('localOnly'),
      updated: t('justNow'),
      downloadStatus: 'downloaded',
      downloadPath: importDraft.path.trim() || defaultImportDraft.path,
      size: t('localDataset'),
      registryUrl: 'local',
      registryPath: importDraft.path.trim() || defaultImportDraft.path,
      downloadDir: importDraft.path.trim() || defaultImportDraft.path,
      manifestPath: 'dataset.toml',
      ref: `${importDraft.name.trim() || defaultImportDraft.name}@${importDraft.version.trim() || defaultImportDraft.version}`,
      path: importDraft.path.trim() || defaultImportDraft.path,
      overwrite: false,
      splits: ['local'],
    }
    setImportedRows((current) => [nextRow, ...current])
    setDownloads((current) => ({
      ...current,
      [datasetKey(nextRow)]: {
        path: nextRow.path ?? defaultImportDraft.path,
        size: nextRow.size ?? t('localDataset'),
        status: 'downloaded',
      },
    }))
    setSelected(nextRow)
    setExpandedTaskName(null)
    setTaskSearch('')
    setTaskSplit('all')
    setDrawerOpen(true)
    setImportDialogOpen(false)
    setImportDraft(defaultImportDraft)
  }
  const selectedDownloadState = selected ? downloadStateFor(selected) : { status: 'not-downloaded' as const }
  const selectedIsRegistryDataset = selected?.registryUrl !== 'local'

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
            <button className="secondary-button" disabled={!allowMockWrites} onClick={() => setImportDialogOpen(true)}>
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
              {visibleRows.map((row) => {
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
                          <button className="secondary-button compact-action" disabled={!allowMockWrites} onClick={(event) => {
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
                            <button className="secondary-button compact-action" disabled={!allowMockWrites} onClick={(event) => {
                              event.stopPropagation()
                              cancelDownload(row)
                            }}>
                              <X aria-hidden="true" />
                              {t('cancelDownload')}
                            </button>
                          </>
                        )}
                        {downloadState.status === 'downloaded' && (
                          <button className="secondary-button compact-action" disabled={!allowMockWrites} onClick={(event) => {
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
            </tbody>
          </table>
        </div>
      </section>
      {detailRow && selected && (
        <DetailDrawer label={t('selectedDataset')} open={drawerOpen} onClose={() => setDrawerOpen(false)}>
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
            writeDisabled={!allowMockWrites}
            onCancelDownload={cancelDownload}
            onDelete={setDeleteTarget}
            onExpandedTaskName={setExpandedTaskName}
            onStartDownload={startDownload}
            onTaskSearch={setTaskSearch}
            onTaskSplit={setTaskSplit}
          />
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
    </main>
  )
}
