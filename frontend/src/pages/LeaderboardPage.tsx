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
        <div className="filter-strip">
          <label>
            Agent filter
            <input aria-label="Agent filter" placeholder="agent/model" />
          </label>
          <label>
            Status filter
            <input aria-label="Status filter" placeholder="submitted, local, pending" />
          </label>
          <label>
            Date range
            <input aria-label="Date range" placeholder="last 7 days" />
          </label>
          <label>
            Comparability key
            <input aria-label="Comparability key" value={rows[0]?.comparabilityKey ?? ''} readOnly />
          </label>
        </div>
        <div className="table-wrap">
          <table>
            <thead>
              <tr>
                <th>{t('rank')}</th>
                <th>{t('agentName')}</th>
                <th>{t('harness')}</th>
                <th>{t('model')}</th>
                <th>{t('score')}</th>
                <th>Metric</th>
                <th>{t('trialCount')}</th>
                <th>{t('cost')}</th>
                <th>{t('duration')}</th>
                <th>Split</th>
                <th>Submission</th>
                <th>Uploaded URL</th>
                <th>{t('reproducibility')}</th>
                <th>{t('job')}</th>
                <th>Actions</th>
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
                  <td>{row.agentName}</td>
                  <td>{row.harness}</td>
                  <td>{row.model}</td>
                  <td>{row.score}</td>
                  <td>{row.metric}</td>
                  <td>{row.trials}</td>
                  <td>{row.cost}</td>
                  <td>{row.duration}</td>
                  <td>{row.split}</td>
                  <td>{row.submitted}</td>
                  <td>
                    <small>{row.uploadedUrl}</small>
                  </td>
                  <td>
                    <small>{row.submissionId}</small>
                    <br />
                    <small>{row.configHash}</small>
                    <br />
                    <small>{row.agentSnapshotHash}</small>
                  </td>
                  <td>
                    <code>{row.jobId}</code>
                  </td>
                  <td>
                    <div className="row-actions">
                      <button className="row-action">Open job</button>
                      <button className="row-action">{t('openViewer')}</button>
                      <button className="row-action">{t('download')}</button>
                      <button className="row-action">{t('submit')}</button>
                      <button className="row-action">{t('share')}</button>
                    </div>
                    <small>{row.reportPath}</small>
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
