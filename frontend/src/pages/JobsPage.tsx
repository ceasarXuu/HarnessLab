import { DetailRail } from '../components/DetailRail'
import { JobsTable } from '../components/JobsTable'
import type { EventLog, HarborJob } from '../data/demo'
import type { Translate } from '../i18n'

interface JobsPageProps {
  events: EventLog[]
  jobs: HarborJob[]
  search: string
  selected: HarborJob
  t: Translate
  onNewJob: () => void
  onSearch: (value: string) => void
  onSelect: (job: HarborJob) => void
}

export function JobsPage({ events, jobs, search, selected, t, onNewJob, onSearch, onSelect }: JobsPageProps) {
  return (
    <main className="workspace jobs-workspace">
      <div className="content-column">
        <JobsTable
          jobs={jobs}
          selectedId={selected.id}
          search={search}
          t={t}
          onNewJob={onNewJob}
          onSearch={onSearch}
          onSelect={onSelect}
        />
      </div>
      <DetailRail job={selected} events={events} t={t} />
    </main>
  )
}
