import { Box, Database, Download, Search, Trash2, X } from 'lucide-react'
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

export function DatasetsPage({ rows, search, taskRows, t, onSearch }: DatasetsPageProps) {
  const [selected, setSelected] = useState<DatasetRow | null>(null)
  const [drawerOpen, setDrawerOpen] = useState(false)
  const [deleteTarget, setDeleteTarget] = useState<DatasetRow | null>(null)
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
  const selectedTasks = useMemo(
    () =>
      selected
        ? taskRows.filter((row) => row.dataset === selected.name || row.dataset === `${selected.name}@${selected.version}`)
        : [],
    [selected, taskRows],
  )

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
                <Metric label={t('splits')} value={selected.splits?.join(', ') ?? '-'} />
                <Metric label={t('digest')} value={selected.digest} />
                <Metric label={t('updated')} value={selected.updated} />
                <Metric label="registry_url" value={selected.registryUrl ?? '-'} />
                <Metric label="registry_path" value={selected.registryPath ?? '-'} />
                <Metric label="download_dir" value={selected.downloadDir ?? '-'} />
                <Metric label="manifest" value={selected.manifestPath ?? '-'} />
                <Metric label="include" value={selected.taskInclude ?? '-'} />
                <Metric label="exclude" value={selected.taskExclude ?? '-'} />
                <Metric label="ref" value={selected.ref ?? '-'} />
                <Metric label="path" value={selected.path ?? '-'} />
                <Metric label="overwrite" value={selected.overwrite ? 'true' : 'false'} />
              </div>
              <div className="button-row tight">
                <button className="secondary-button">
                  <Download aria-hidden="true" />
                  {t('download')}
                </button>
                <button className="secondary-button">{t('pullUpdates')}</button>
                <button className="secondary-button">{t('publish')}</button>
              </div>
            </section>
            <section className="surface rail-card">
              <div className="rail-title">
                <Database aria-hidden="true" />
                <h3>{t('manifestTools')}</h3>
              </div>
              <div className="path-list">
                <code>harbor init {selected.path ?? selected.name}</code>
                <code>harbor add {selected.manifestPath ?? 'dataset.toml'} tasks/*</code>
                <code>harbor remove {selected.manifestPath ?? 'dataset.toml'} task-name</code>
                <code>harbor sync {selected.manifestPath ?? 'dataset.toml'}</code>
              </div>
            </section>
            <section className="surface rail-card">
              <div className="rail-title">
                <Box aria-hidden="true" />
                <h3>{t('datasetTasks')}</h3>
              </div>
              <div className="mini-table">
                {selectedTasks.map((row) => (
                  <div key={row.name} className="mini-row task-row">
                    <span>{row.name}</span>
                    <span>{row.os}</span>
                    <span>{row.description}</span>
                    <span>{row.verifier}</span>
                    <div className="row-actions">
                      <button className="row-action">{t('runSingleTask')}</button>
                    </div>
                  </div>
                ))}
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
