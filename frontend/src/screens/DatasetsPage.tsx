import { Box, Database, Download, Plus, Search, Trash2, X } from 'lucide-react'
import { useEffect, useMemo, useState } from 'react'
import { DetailDrawer } from '../ui/components/DetailDrawer'
import type { DatasetRow, TaskRow } from '../mocks/demo'
import type { Translate } from '../i18n'

interface DatasetsPageProps {
  rows: DatasetRow[]
  search: string
  taskRows: TaskRow[]
  t: Translate
  onSearch: (value: string) => void
}

type DatasetDownloadState =
  | { status: 'not-downloaded' }
  | { progress: number; status: 'downloading' }
  | { path: string; size: string; status: 'downloaded' }

const datasetKey = (row: DatasetRow) => `${row.name}@${row.version}`
const defaultImportDraft = {
  name: 'local/custom-dataset',
  path: './datasets/custom-dataset',
  tasks: '12',
  version: 'local',
}

export function DatasetsPage({ rows, search, taskRows, t, onSearch }: DatasetsPageProps) {
  const [selected, setSelected] = useState<DatasetRow | null>(null)
  const [expandedTaskName, setExpandedTaskName] = useState<string | null>(null)
  const [taskSearch, setTaskSearch] = useState('')
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<DatasetRow | null>(null)
  const [importDialogOpen, setImportDialogOpen] = useState(false)
  const [importDraft, setImportDraft] = useState(defaultImportDraft)
  const [importedRows, setImportedRows] = useState<DatasetRow[]>([])
  const [downloads, setDownloads] = useState<Record<string, DatasetDownloadState>>(() =>
    Object.fromEntries(rows.map((row) => [
      datasetKey(row),
      row.downloadStatus === 'downloaded'
        ? {
            path: row.downloadPath ?? row.path ?? row.downloadDir ?? 'local dataset path',
            size: row.size ?? 'unknown',
            status: 'downloaded' as const,
          }
        : { status: 'not-downloaded' as const },
    ])),
  )
  const visibleRows = useMemo(() => {
    const query = search.trim().toLowerCase()
    const visibleImportedRows = query
      ? importedRows.filter((row) =>
          [row.name, row.version, row.visibility, row.source, row.digest, row.path ?? ''].some((value) =>
            value.toLowerCase().includes(query),
          ),
        )
      : importedRows

    return [...visibleImportedRows, ...rows]
  }, [importedRows, rows, search])
  const selectedTasks = useMemo(
    () =>
      selected
        ? taskRows.filter((row) => row.dataset === selected.name || row.dataset === `${selected.name}@${selected.version}`)
        : [],
    [selected, taskRows],
  )
  const visibleSelectedTasks = useMemo(() => {
    const query = taskSearch.trim().toLowerCase()
    if (!query) return selectedTasks
    return selectedTasks.filter((row) =>
      [row.name, row.description, row.state].some((value) => value.toLowerCase().includes(query)),
    )
  }, [selectedTasks, taskSearch])

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
    setDownloads((current) => ({ ...current, [datasetKey(row)]: { progress: 0, status: 'downloading' } }))
  }
  const cancelDownload = (row: DatasetRow) => {
    setDownloads((current) => ({ ...current, [datasetKey(row)]: { status: 'not-downloaded' } }))
  }
  const confirmDelete = () => {
    if (!deleteTarget) return
    setDownloads((current) => ({ ...current, [datasetKey(deleteTarget)]: { status: 'not-downloaded' } }))
    setDeleteTarget(null)
  }
  const confirmImportDataset = () => {
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
            <button className="secondary-button" onClick={() => setImportDialogOpen(true)}>
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
                          <button className="secondary-button compact-action" onClick={(event) => {
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
                            <button className="secondary-button compact-action" onClick={(event) => {
                              event.stopPropagation()
                              cancelDownload(row)
                            }}>
                              <X aria-hidden="true" />
                              {t('cancelDownload')}
                            </button>
                          </>
                        )}
                        {downloadState.status === 'downloaded' && (
                          <button className="secondary-button compact-action" onClick={(event) => {
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
      {selected && (
        <DetailDrawer label={t('selectedDataset')} open={drawerOpen} onClose={() => setDrawerOpen(false)}>
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
                <Metric label={t('path')} value={selectedDownloadState.status === 'downloaded' ? selectedDownloadState.path : t('notDownloaded')} />
                <Metric label={t('size')} value={selectedDownloadState.status === 'downloaded' ? selectedDownloadState.size : t('notDownloaded')} />
                <Metric label={t('registry')} value={selected.registryUrl ?? '-'} />
              </div>
              <div className="button-row tight">
                {selectedDownloadState.status === 'not-downloaded' && (
                  <button className="secondary-button" onClick={() => startDownload(selected)}>
                    <Download aria-hidden="true" />
                    {t('download')}
                  </button>
                )}
                {selectedDownloadState.status === 'downloading' && (
                  <>
                    <span className="progress-label">{selectedDownloadState.progress}%</span>
                    <button className="secondary-button" onClick={() => cancelDownload(selected)}>
                      <X aria-hidden="true" />
                      {t('cancelDownload')}
                    </button>
                  </>
                )}
                {selectedDownloadState.status === 'downloaded' && selectedIsRegistryDataset && (
                  <button className="secondary-button">{t('pullUpdates')}</button>
                )}
                {selectedDownloadState.status === 'downloaded' && (
                  <button className="secondary-button" onClick={() => setDeleteTarget(selected)}>
                    <Trash2 aria-hidden="true" />
                    {t('delete')}
                  </button>
                )}
              </div>
            </section>
            <section className="surface rail-card">
              <div className="rail-title">
                <Box aria-hidden="true" />
                <h3>{t('datasetTasks')}</h3>
              </div>
              <label className="search-field drawer-search">
                <Search aria-hidden="true" />
                <input
                  aria-label={t('searchTasks')}
                  value={taskSearch}
                  onChange={(event) => setTaskSearch(event.target.value)}
                  placeholder={t('searchTasks')}
                />
              </label>
              <div className="mini-table">
                <div className="mini-row task-row mini-header" role="row">
                  <span>{t('taskName')}</span>
                  <span>{t('actions')}</span>
                </div>
                {visibleSelectedTasks.map((row) => (
                  <div key={row.name} className="task-entry">
                    <div
                      className="mini-row task-row task-toggle"
                      role="button"
                      tabIndex={0}
                      aria-expanded={expandedTaskName === row.name}
                      onClick={() => setExpandedTaskName((current) => (current === row.name ? null : row.name))}
                      onKeyDown={(event) => {
                        if (event.key !== 'Enter' && event.key !== ' ') return
                        event.preventDefault()
                        setExpandedTaskName((current) => (current === row.name ? null : row.name))
                      }}
                    >
                      <span>{row.name}</span>
                      <div className="row-actions">
                        <button
                          className="row-action"
                          onClick={(event) => event.stopPropagation()}
                        >
                          {t('runSingleTask')}
                        </button>
                      </div>
                    </div>
                    {expandedTaskName === row.name && (
                      <div className="task-expanded">
                        <span>{t('description')}</span>
                        <p>{row.description}</p>
                      </div>
                    )}
                  </div>
                ))}
                {visibleSelectedTasks.length === 0 && <div className="empty-row">{t('noTasksAvailable')}</div>}
              </div>
            </section>
          </aside>
        </DetailDrawer>
      )}
      {deleteTarget && (
        <div className="confirm-overlay">
          <section className="surface confirm-dialog" role="dialog" aria-modal="true" aria-label={t('deleteDatasetTitle')}>
            <div className="confirm-heading">
              <h2>{t('deleteDatasetTitle')}</h2>
            </div>
            <ul className="cleanup-impact-list">
              <li>{t('deleteDatasetLocalImpact')}</li>
              <li>{datasetKey(deleteTarget)}</li>
            </ul>
            <div className="button-row confirm-actions">
              <button className="secondary-button" onClick={() => setDeleteTarget(null)}>{t('cancel')}</button>
              <button className="primary-button" onClick={confirmDelete}>{t('confirmDelete')}</button>
            </div>
          </section>
        </div>
      )}
      {importDialogOpen && (
        <div className="confirm-overlay">
          <section className="surface confirm-dialog" role="dialog" aria-modal="true" aria-label={t('importLocalDataset')}>
            <div className="confirm-heading">
              <h2>{t('importLocalDataset')}</h2>
            </div>
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
            <ul className="cleanup-impact-list">
              <li>{t('importLocalDatasetImpact')}</li>
            </ul>
            <div className="button-row confirm-actions">
              <button className="secondary-button" onClick={() => setImportDialogOpen(false)}>{t('cancel')}</button>
              <button className="primary-button" onClick={confirmImportDataset}>{t('confirmImport')}</button>
            </div>
          </section>
        </div>
      )}
    </main>
  )
}

function Metric({ label, value }: { label: string; value: string }) {
  return (
    <div className="metric">
      <span>{label}</span>
      <strong>{value}</strong>
    </div>
  )
}
