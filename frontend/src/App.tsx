import { useMemo, useState } from 'react'
import { AppShell } from './components/AppShell'
import { DetailRail, TaskPreview } from './components/DetailRail'
import { JobsTable } from './components/JobsTable'
import { RunBuilder } from './components/RunBuilder'
import { events, initialDraft, jobs as seedJobs, taskRows, type HarborJob } from './data/demo'

export function App() {
  const [jobs, setJobs] = useState(seedJobs)
  const [selected, setSelected] = useState(seedJobs[0])
  const [search, setSearch] = useState('')
  const [activeStep, setActiveStep] = useState('Source')
  const [draft, setDraft] = useState(initialDraft)

  const filteredJobs = useMemo(() => {
    const query = search.trim().toLowerCase()
    if (!query) return jobs
    return jobs.filter((job) =>
      [job.name, job.dataset, job.agent, job.model, job.status].some((value) =>
        value.toLowerCase().includes(query),
      ),
    )
  }, [jobs, search])

  function launchDraft() {
    const newJob: HarborJob = {
      id: `job_${Math.floor(Math.random() * 9000 + 1000)}`,
      name: `${draft.source.split('@')[0]}-draft`,
      status: 'queued',
      dataset: draft.source,
      agent: draft.agent,
      model: draft.model.split('/').at(-1) ?? draft.model,
      environment: draft.environment,
      trials: '0 / 64',
      score: '-',
      cost: '$0.00',
      updated: 'just now',
    }
    setJobs((current) => [newJob, ...current])
    setSelected(newJob)
    setActiveStep('Review')
  }

  return (
    <AppShell>
      <main className="workspace">
        <div className="content-column">
          <JobsTable
            jobs={filteredJobs}
            selectedId={selected.id}
            search={search}
            onSearch={setSearch}
            onSelect={setSelected}
          />
          <div className="lower-grid">
            <RunBuilder
              draft={draft}
              activeStep={activeStep}
              onStep={setActiveStep}
              onDraft={setDraft}
              onLaunch={launchDraft}
            />
            <TaskPreview rows={taskRows} />
          </div>
        </div>
        <DetailRail job={selected} events={events} />
      </main>
    </AppShell>
  )
}
