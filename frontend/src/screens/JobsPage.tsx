import { useJob, useJobEvents, useJobTrials } from '../api/hooks'
import { jobDtoToHarborJob, jobEventDtoToEventLog, trialDtoToTrialRow } from '../api/viewModels'
import type { WebUiClient } from '../api/webUiClient'
import { DetailRail } from '../ui/components/DetailRail'
import { DetailDrawer } from '../ui/components/DetailDrawer'
import { JobsTable } from '../ui/components/JobsTable'
import type { HarborJob } from '../domain/harbor'
import type { Translate } from '../i18n'

interface JobsPageProps {
  allowMockWrites?: boolean
  client: WebUiClient
  jobs: HarborJob[]
  open: boolean
  search: string
  selected: HarborJob | null
  t: Translate
  onClose: () => void
  onNewJob: () => void
  onLeaderboardChange: (jobId: string, include: boolean) => void
  onSearch: (value: string) => void
  onSelect: (job: HarborJob) => void
}

export function JobsPage({
  allowMockWrites = true,
  client,
  jobs,
  open,
  search,
  selected,
  t,
  onClose,
  onNewJob,
  onLeaderboardChange,
  onSearch,
  onSelect,
}: JobsPageProps) {
  const detailResource = useJob(client, selected?.id)
  const eventsResource = useJobEvents(client, selected?.id)
  const trialsResource = useJobTrials(client, selected?.id)
  const detailJob = detailResource.data ? jobDtoToHarborJob(detailResource.data) : selected
  const events = eventsResource.data?.map(jobEventDtoToEventLog) ?? []
  const trials = trialsResource.data?.map(trialDtoToTrialRow) ?? []

  return (
    <main className="workspace single-page">
      <div className="content-column">
        <JobsTable
          jobs={jobs}
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
          <DetailRail
            job={detailJob}
            events={events}
            trials={trials}
            t={t}
            allowMockWrites={allowMockWrites}
            onLeaderboardChange={onLeaderboardChange}
          />
        </DetailDrawer>
      )}
    </main>
  )
}
