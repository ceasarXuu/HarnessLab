import { Search, Trophy } from 'lucide-react'
import { CustomSelect } from '../components/CustomSelect'
import type { DatasetRow, LeaderboardRow } from '../data/demo'
import type { Translate } from '../i18n'

interface LeaderboardPageProps {
  dataset: string
  datasetSearch: string
  datasets: DatasetRow[]
  rows: LeaderboardRow[]
  t: Translate
  onDataset: (value: string) => void
  onDatasetSearch: (value: string) => void
}

export function LeaderboardPage({
  dataset,
  datasetSearch,
  datasets,
  rows,
  t,
  onDataset,
  onDatasetSearch,
}: LeaderboardPageProps) {
  const selectedDataset = datasets.find((row) => `${row.name}@${row.version}` === dataset)
  const visibleDatasets = datasets.filter((row) =>
    [row.name, row.version, `${row.name}@${row.version}`].some((value) =>
      value.toLowerCase().includes(datasetSearch.trim().toLowerCase()),
    ),
  )
  const selectableDatasets =
    selectedDataset && !visibleDatasets.includes(selectedDataset) ? [selectedDataset, ...visibleDatasets] : visibleDatasets

  return (
    <main className="workspace single-page">
      <section className="surface leaderboard-page">
        <div className="section-header">
          <div>
            <h1>{t('leaderboardTitle')}</h1>
            <p>{t('leaderboardDesc')}</p>
          </div>
          <div className="toolbar leaderboard-toolbar">
            <label className="search-field dataset-filter">
              <Search aria-hidden="true" />
              <input
                aria-label={t('searchDatasets')}
                value={datasetSearch}
                onChange={(event) => onDatasetSearch(event.target.value)}
                placeholder={t('searchDatasetsPlaceholder')}
              />
            </label>
            <CustomSelect
              ariaLabel={t('selectDataset')}
              className="toolbar-select"
              visibleLabel={t('dataset')}
              value={dataset}
              options={selectableDatasets.map((row) => {
                const value = `${row.name}@${row.version}`
                return { label: value, value }
              })}
              onChange={onDataset}
            />
          </div>
        </div>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t('rank')}</th>
                <th>{t('agent')}</th>
                <th>{t('model')}</th>
                <th>{t('score')}</th>
                <th>{t('trialCount')}</th>
                <th>{t('cost')}</th>
                <th>{t('duration')}</th>
                <th>{t('job')}</th>
              </tr>
            </thead>
            <tbody>
              {rows.map((row) => (
                <tr key={`${row.dataset}-${row.rank}-${row.jobId}`}>
                  <td>
                    <span className="cell-title">
                      <Trophy aria-hidden="true" />
                      #{row.rank}
                    </span>
                  </td>
                  <td>{row.agent}</td>
                  <td>{row.model}</td>
                  <td>{row.score}</td>
                  <td>{row.trials}</td>
                  <td>{row.cost}</td>
                  <td>{row.duration}</td>
                  <td>
                    <code>{row.jobId}</code>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
    </main>
  )
}
