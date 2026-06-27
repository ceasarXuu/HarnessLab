import { Database, Search } from 'lucide-react'
import type { DatasetRow } from '../data/demo'
import type { Translate } from '../i18n'

interface DatasetsPageProps {
  rows: DatasetRow[]
  search: string
  t: Translate
  onSearch: (value: string) => void
}

export function DatasetsPage({ rows, search, t, onSearch }: DatasetsPageProps) {
  return (
    <main className="workspace single-page">
      <section className="surface">
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
                <tr key={`${row.name}-${row.version}`}>
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
    </main>
  )
}
