import { Box, Database, Download, Play, Search } from 'lucide-react'
import { useMemo, useState } from 'react'
import { DetailDrawer } from '../components/DetailDrawer'
import type { DatasetRow, TaskRow } from '../data/demo'
import type { Translate } from '../i18n'

interface DatasetsPageProps {
  rows: DatasetRow[]
  search: string
  taskRows: TaskRow[]
  t: Translate
  onNewJob: () => void
  onSearch: (value: string) => void
}

export function DatasetsPage({ rows, search, taskRows, t, onNewJob, onSearch }: DatasetsPageProps) {
  const [selected, setSelected] = useState<DatasetRow | null>(null)
  const [drawerOpen, setDrawerOpen] = useState(false)
  const selectedTasks = useMemo(
    () =>
      selected
        ? taskRows.filter((row) => row.dataset === selected.name || row.dataset === `${selected.name}@${selected.version}`)
        : [],
    [selected, taskRows],
  )

  return (
    <main className="workspace single-page">
      <section className="surface dataset-catalog">
        <div className="section-header">
          <div>
            <h1>{t('datasetCatalog')}</h1>
            <p>{t('datasetCatalogDesc')}</p>
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
            <button className="secondary-button">{t('import')}</button>
            <button className="primary-button">{t('download')}</button>
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
                <th>{t('digest')}</th>
                <th>{t('updated')}</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
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
                  <td>
                    <code>{row.digest}</code>
                  </td>
                  <td>{row.updated}</td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
      {selected && (
        <DetailDrawer label={t('selectedDataset')} open={drawerOpen} onClose={() => setDrawerOpen(false)}>
          <aside className="detail-rail dataset-detail">
            <section className="surface rail-card">
              <p className="panel-kicker">{t('selectedDataset')}</p>
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
                <Metric label={t('digest')} value={selected.digest} />
                <Metric label={t('updated')} value={selected.updated} />
              </div>
              <div className="button-row tight">
                <button className="secondary-button">
                  <Download aria-hidden="true" />
                  {t('download')}
                </button>
                <button className="primary-button" onClick={onNewJob}>
                  <Play aria-hidden="true" />
                  {t('newJob')}
                </button>
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
                    <button className="row-action">{t('runSingleTask')}</button>
                  </div>
                ))}
              </div>
            </section>
          </aside>
        </DetailDrawer>
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
