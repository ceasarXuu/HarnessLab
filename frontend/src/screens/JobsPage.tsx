import { DetailRail } from '../ui/components/DetailRail'
import { DetailDrawer } from '../ui/components/DetailDrawer'
import { JobsTable } from '../ui/components/JobsTable'
import type { EventLog, HarborJob, TrialRow } from '../domain/harbor'
import type { Translate } from '../i18n'

interface JobsPageProps {
  events: EventLog[]
  jobs: HarborJob[]
  open: boolean
  search: string
  selected: HarborJob | null
  trialRows: TrialRow[]
  t: Translate
  onClose: () => void
  onNewJob: () => void
  onLeaderboardChange: (jobId: string, include: boolean) => void
  onSearch: (value: string) => void
  onSelect: (job: HarborJob) => void
}

export function JobsPage({
  events,
  jobs,
  open,
  search,
  selected,
  trialRows,
  t,
  onClose,
  onNewJob,
  onLeaderboardChange,
  onSearch,
  onSelect,
}: JobsPageProps) {
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
      {selected && (
        <DetailDrawer label={t('selectedJob')} open={open} onClose={onClose}>
          <DetailRail
            job={selected}
            events={events}
            trials={trialRows.filter((row) => row.jobId === selected.id)}
            t={t}
            onLeaderboardChange={onLeaderboardChange}
          />
        </DetailDrawer>
      )}
    </main>
  )
}
