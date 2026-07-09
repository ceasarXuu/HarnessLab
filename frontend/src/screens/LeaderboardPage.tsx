import { Search, Trophy } from 'lucide-react'
import { useState } from 'react'
import { useJob, useJobEvents, useJobTrials } from '../api/hooks'
import { jobDtoToHarborJob, jobEventDtoToEventLog, trialDtoToTrialRow } from '../api/viewModels'
import type { WebUiClient } from '../api/webUiClient'
import { CustomSelect } from '../ui/components/CustomSelect'
import { DetailDrawer } from '../ui/components/DetailDrawer'
import { DetailRail } from '../ui/components/DetailRail'
import { ResourceStatus } from '../ui/components/ResourceStatus'
import type { HarborJob, LeaderboardDataset, LeaderboardRow } from '../domain/harbor'
import type { Translate } from '../i18n'

interface LeaderboardPageProps {
  writesEnabled?: boolean
  client: WebUiClient
  dataset: string
  datasetSearch: string
  leaderboardDatasets: LeaderboardDataset[]
  jobs: HarborJob[]
  rows: LeaderboardRow[]
  t: Translate
  onDataset: (value: string) => void
  onDatasetSearch: (value: string) => void
  onJobAction: (jobId: string, action: 'cancel' | 'retry' | 'resume') => void
  onLeaderboardChange: (jobId: string, include: boolean) => void
  onRemove: (jobId: string) => void
}

export function LeaderboardPage({
  writesEnabled = true,
  client,
  dataset,
  datasetSearch,
  leaderboardDatasets,
  jobs,
  rows,
  t,
  onDataset,
  onDatasetSearch,
  onJobAction,
  onLeaderboardChange,
  onRemove,
}: LeaderboardPageProps) {
  const [selectedJob, setSelectedJob] = useState<HarborJob | null>(null)
  const [drawerOpen, setDrawerOpen] = useState(false)
  const detailResource = useJob(client, selectedJob?.id)
  const eventsResource = useJobEvents(client, selectedJob?.id)
  const trialsResource = useJobTrials(client, selectedJob?.id)
  const detailJob = detailResource.data ? jobDtoToHarborJob(detailResource.data) : selectedJob
  const events = eventsResource.data?.map(jobEventDtoToEventLog) ?? []
  const trials = trialsResource.data?.map(trialDtoToTrialRow) ?? []
  const selectedDataset = leaderboardDatasets.find((row) => row.ref === dataset)
  const visibleDatasets = leaderboardDatasets.filter((row) =>
    [row.name, row.version, row.ref].some((value) =>
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
              options={selectableDatasets.map((row) => ({ label: row.ref, value: row.ref }))}
              onChange={onDataset}
            />
          </div>
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
                <th>{t('metric')}</th>
                <th>{t('trialCount')}</th>
                <th>{t('cost')}</th>
                <th>{t('tokenUsage')}</th>
                <th>{t('duration')}</th>
                <th>{t('split')}</th>
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
                    <button className="secondary-button compact-action" disabled={!writesEnabled} onClick={() => onRemove(row.jobId)}>
                      {t('removeFromLeaderboard')}
                    </button>
                  </td>
                </tr>
              ))}
              {rows.length === 0 && (
                <tr>
                  <td className="empty-row" colSpan={13}>{t('noLeaderboardEntries')}</td>
                </tr>
              )}
            </tbody>
          </table>
        </div>
      </section>
      {detailJob && (
        <DetailDrawer label={t('selectedJob')} open={drawerOpen} onClose={() => setDrawerOpen(false)}>
          <>
            <DetailRail
              job={detailJob}
              events={events}
              trials={trials}
              t={t}
              writesEnabled={writesEnabled}
              onJobAction={onJobAction}
              onLeaderboardChange={onLeaderboardChange}
            />
            <ResourceStatus
              error={detailResource.error?.message ?? eventsResource.error?.message ?? trialsResource.error?.message ?? null}
              loading={detailResource.loading || eventsResource.loading || trialsResource.loading}
              loadingLabel={t('loadingJobs')}
            />
          </>
        </DetailDrawer>
      )}
    </main>
  )
}
