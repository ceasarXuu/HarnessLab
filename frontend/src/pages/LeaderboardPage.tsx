import { Search, Trophy } from 'lucide-react'
import { useState } from 'react'
import { CustomSelect } from '../components/CustomSelect'
import { DetailDrawer } from '../components/DetailDrawer'
import { DetailRail } from '../components/DetailRail'
import type { DatasetRow, EventLog, HarborJob, LeaderboardRow, TrialRow } from '../data/demo'
import type { Translate } from '../i18n'

interface LeaderboardPageProps {
  dataset: string
  datasetSearch: string
  datasets: DatasetRow[]
  events: EventLog[]
  jobs: HarborJob[]
  rows: LeaderboardRow[]
  t: Translate
  trialRows: TrialRow[]
  onDataset: (value: string) => void
  onDatasetSearch: (value: string) => void
  onRemove: (jobId: string) => void
}

export function LeaderboardPage({
  dataset,
  datasetSearch,
  datasets,
  events,
  jobs,
  rows,
  t,
  trialRows,
  onDataset,
  onDatasetSearch,
  onRemove,
}: LeaderboardPageProps) {
  const [selectedJob, setSelectedJob] = useState<HarborJob | null>(null)
  const [drawerOpen, setDrawerOpen] = useState(false)
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
                <th>Tokens (M)</th>
                <th>{t('duration')}</th>
                <th>Split</th>
                <th>{t('job')}</th>
                <th>{t('actions')}</th>
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
                  <td>{row.tokens}</td>
                  <td>{row.duration}</td>
                  <td>{row.split}</td>
                  <td>
                    <button className="row-button" onClick={() => {
                      const job = jobs.find((item) => item.id === row.jobId)
                      if (!job) return
                      setSelectedJob(job)
                      setDrawerOpen(true)
                    }}>
                      <code>{row.jobId}</code>
                    </button>
                  </td>
                  <td>
                    <button className="secondary-button compact-action" onClick={() => onRemove(row.jobId)}>
                      {t('removeFromLeaderboard')}
                    </button>
                  </td>
                </tr>
              ))}
            </tbody>
          </table>
        </div>
      </section>
      {selectedJob && (
        <DetailDrawer label={t('selectedJob')} open={drawerOpen} onClose={() => setDrawerOpen(false)}>
          <DetailRail job={selectedJob} events={events} trials={trialRows.filter((row) => row.jobId === selectedJob.id)} t={t} />
        </DetailDrawer>
      )}
    </main>
  )
}
