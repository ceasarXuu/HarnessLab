import { useState } from 'react'
import { useJob, useJobEvents, useJobTrials, usePollingRefresh } from '../api/hooks'
import { jobDtoToHarborJob, jobEventDtoToEventLog, trialDtoToTrialRow } from '../api/viewModels'
import type { WebUiClient } from '../api/webUiClient'
import { DetailRail } from '../ui/components/DetailRail'
import { DetailDrawer } from '../ui/components/DetailDrawer'
import { ConfirmDialog } from '../ui/components/ConfirmDialog'
import { JobsTable } from '../ui/components/JobsTable'
import { ResourceStatus } from '../ui/components/ResourceStatus'
import { usePaginatedItems } from '../ui/pagination'
import type { HarborJob } from '../domain/harbor'
import type { Translate } from '../i18n'

interface JobsPageProps {
  writesEnabled?: boolean
  client: WebUiClient
  jobs: HarborJob[]
  open: boolean
  search: string
  selected: HarborJob | null
  actionError?: string | null
  t: Translate
  onClose: () => void
  onJobAction: (jobId: string, action: 'cancel' | 'resume') => void
  onCopyJob?: (jobId: string) => void
  onNewJob: () => void
  onRefresh?: () => Promise<void>
  onLeaderboardChange: (jobId: string, include: boolean) => void
  onSearch: (value: string) => void
  onSelect: (job: HarborJob) => void
}

export function JobsPage({
  writesEnabled = true,
  client,
  jobs,
  open,
  search,
  selected,
  actionError = null,
  t,
  onClose,
  onJobAction,
  onCopyJob = () => undefined,
  onNewJob,
  onRefresh = async () => undefined,
  onLeaderboardChange,
  onSearch,
  onSelect,
}: JobsPageProps) {
  const [cancelTarget, setCancelTarget] = useState<HarborJob | null>(null)
  const [deleteTarget, setDeleteTarget] = useState<HarborJob | null>(null)
  const [deleteError, setDeleteError] = useState<string | null>(null)
  const detailResource = useJob(client, selected?.id)
  const eventsResource = useJobEvents(client, selected?.id)
  const trialsResource = useJobTrials(client, selected?.id)
  const loadedDetailJob = detailResource.data ? jobDtoToHarborJob(detailResource.data) : selected
  const detailJob = loadedDetailJob && selected
    ? { ...loadedDetailJob, status: selected.status, canResume: selected.canResume }
    : loadedDetailJob
  const events = eventsResource.data?.map(jobEventDtoToEventLog) ?? []
  const trials = trialsResource.data?.map(trialDtoToTrialRow) ?? []
  const pagination = usePaginatedItems({ items: jobs, resetKey: search })
  const live = open && (selected?.status === 'queued' || selected?.status === 'running')

  usePollingRefresh(detailResource.refresh, live)
  usePollingRefresh(eventsResource.refresh, live)
  usePollingRefresh(trialsResource.refresh, live)

  const requestJobAction = (jobId: string, action: 'cancel' | 'resume') => {
    if (action !== 'cancel') {
      onJobAction(jobId, action)
      return
    }
    const target = detailJob?.id === jobId ? detailJob : jobs.find((job) => job.id === jobId)
    if (target) setCancelTarget(target)
  }

  const confirmCancellation = () => {
    if (!cancelTarget) return
    onJobAction(cancelTarget.id, 'cancel')
    setCancelTarget(null)
  }

  const confirmDeletion = async () => {
    if (!deleteTarget) return
    const target = deleteTarget
    setDeleteTarget(null)
    setDeleteError(null)
    const response = await client.deleteJob(target.id)
    if (response.error) {
      setDeleteError(response.error.message)
      return
    }
    onClose()
    await onRefresh()
  }

  return (
    <main className="workspace single-page">
      <div className="content-column">
        <JobsTable
          jobs={pagination.items}
          pagination={pagination}
          selectedId={selected?.id}
          search={search}
          t={t}
          onNewJob={onNewJob}
          onSearch={onSearch}
          onSelect={onSelect}
        />
      </div>
      {detailJob && (
        <DetailDrawer label={t('selectedJob')} open={open} onClose={onClose}>
          <>
            <DetailRail
              job={detailJob}
              events={events}
              trials={trials}
              t={t}
              writesEnabled={writesEnabled}
              onJobAction={requestJobAction}
              onCopyJob={onCopyJob}
              onDeleteJob={setDeleteTarget}
              onLeaderboardChange={onLeaderboardChange}
            />
            <ResourceStatus
              error={deleteError ?? actionError ?? detailResource.error?.message ?? eventsResource.error?.message ?? trialsResource.error?.message ?? null}
              loading={
                (detailResource.loading && !detailResource.data)
                || (eventsResource.loading && !eventsResource.data)
                || (trialsResource.loading && !trialsResource.data)
              }
              loadingLabel={t('loadingJobs')}
            />
          </>
        </DetailDrawer>
      )}
      {cancelTarget && (
        <ConfirmDialog
          cancelLabel={t('cancel')}
          confirmLabel={t('confirmCancelJob')}
          impacts={[t('cancelJobImpact'), cancelTarget.name]}
          title={t('cancelJobTitle')}
          onCancel={() => setCancelTarget(null)}
          onConfirm={confirmCancellation}
        />
      )}
      {deleteTarget && (
        <ConfirmDialog
          cancelLabel={t('cancel')}
          confirmLabel={t('confirmDeleteJob')}
          impacts={[t('deleteJobRecordsImpact'), t('deleteJobArtifactsImpact'), t('deleteJobLeaderboardImpact')]}
          title={t('deleteJobTitle')}
          onCancel={() => setDeleteTarget(null)}
          onConfirm={() => { void confirmDeletion() }}
        />
      )}
    </main>
  )
}
