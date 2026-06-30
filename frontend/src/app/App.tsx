import { useEffect, useMemo, useState } from 'react'
import { AppShell, type PageKey } from '../ui/components/AppShell'
import {
  events,
  initialDraft,
  jobs as seedJobs,
  trialRows,
  type HarborJob,
  type LeaderboardRow,
} from '../mocks/demo'
import { agentRows, datasetRows, environmentRows, taskRows } from '../mocks/demoCatalog'
import { leaderboardRows, systemRows } from '../mocks/demoSystem'
import { getTranslator, type Locale } from '../i18n'
import { JobsPage } from '../screens/JobsPage'
import { AgentsPage } from '../screens/AgentsPage'
import { DatasetsPage } from '../screens/DatasetsPage'
import { EnvironmentsPage } from '../screens/EnvironmentsPage'
import { LeaderboardPage } from '../screens/LeaderboardPage'
import { NewRunPage } from '../screens/NewRunPage'
import { SystemPage } from '../screens/SystemPage'

type JobView = 'list' | 'new'

interface RouteState {
  jobView: JobView
  page: PageKey
}

const pageKeys = new Set<PageKey>(['jobs', 'datasets', 'agents', 'environments', 'leaderboard', 'system'])

function readRouteFromHash(): RouteState {
  const hash = window.location.hash.replace('#', '')
  if (hash === 'jobs/new' || hash === 'new-run') {
    return { page: 'jobs', jobView: 'new' }
  }
  return {
    page: pageKeys.has(hash as PageKey) ? (hash as PageKey) : 'jobs',
    jobView: 'list',
  }
}

function readLocale(): Locale {
  return window.localStorage.getItem('ornnlab.locale') === 'zh' ? 'zh' : 'en'
}

function readTheme(): 'light' | 'dark' {
  return window.localStorage.getItem('ornnlab.theme') === 'dark' ? 'dark' : 'light'
}

export function App() {
  const [route, setRoute] = useState<RouteState>(readRouteFromHash)
  const [jobs, setJobs] = useState(seedJobs)
  const [environmentProfiles, setEnvironmentProfiles] = useState(environmentRows)
  const [leaderboardEntries, setLeaderboardEntries] = useState(leaderboardRows)
  const [datasetSearch, setDatasetSearch] = useState('')
  const [leaderboardDataset, setLeaderboardDataset] = useState('terminal-bench@2.0')
  const [leaderboardDatasetSearch, setLeaderboardDatasetSearch] = useState('')
  const [selected, setSelected] = useState<HarborJob | null>(null)
  const [jobDrawerOpen, setJobDrawerOpen] = useState(false)
  const [search, setSearch] = useState('')
  const [draft, setDraft] = useState(initialDraft)
  const [language, setLanguage] = useState<Locale>(readLocale)
  const [theme, setTheme] = useState<'light' | 'dark'>(readTheme)
  const t = useMemo(() => getTranslator(language), [language])

  const filteredJobs = useMemo(() => {
    const query = search.trim().toLowerCase()
    if (!query) return jobs
    return jobs.filter((job) =>
      [job.name, job.dataset, job.agent, job.model, job.status].some((value) =>
        value.toLowerCase().includes(query),
      ),
    )
  }, [jobs, search])

  const filteredDatasets = useMemo(() => {
    const query = datasetSearch.trim().toLowerCase()
    if (!query) return datasetRows
    return datasetRows.filter((row) =>
      [row.name, row.version, row.visibility, row.source, row.digest].some((value) =>
        value.toLowerCase().includes(query),
      ),
    )
  }, [datasetSearch])

  const filteredLeaderboard = useMemo(() => {
    const excludedJobIds = new Set(jobs.filter((job) => !job.includeInLeaderboard).map((job) => job.id))
    return leaderboardEntries
      .filter((row) => row.dataset === leaderboardDataset && !excludedJobIds.has(row.jobId))
      .map((row, index) => ({ ...row, rank: index + 1 }))
  }, [jobs, leaderboardDataset, leaderboardEntries])

  useEffect(() => {
    const onHashChange = () => setRoute(readRouteFromHash())
    window.addEventListener('hashchange', onHashChange)
    return () => window.removeEventListener('hashchange', onHashChange)
  }, [])

  useEffect(() => {
    document.documentElement.dataset.theme = theme
    document.documentElement.lang = language
    window.localStorage.setItem('ornnlab.theme', theme)
    window.localStorage.setItem('ornnlab.locale', language)
  }, [language, theme])

  function navigate(page: PageKey, jobView: JobView = 'list') {
    const nextRoute: RouteState = { page, jobView: page === 'jobs' ? jobView : 'list' }
    const nextHash = nextRoute.page === 'jobs' && nextRoute.jobView === 'new' ? '#jobs/new' : `#${nextRoute.page}`
    setRoute(nextRoute)
    if (window.location.hash !== nextHash) {
      window.history.pushState(null, '', nextHash)
    }
  }

  function launchDraft() {
    const nextJobId = `job_${Math.floor(Math.random() * 9000 + 1000)}`
    const nextJobRoot = `/Users/xuzhang/.ornnlab/HarnessLab/${draft.jobsDir}`
    const draftDataset = datasetRows.find((row) => `${row.name}@${row.version}` === draft.source)
    const draftEnvironment = environmentProfiles.find((row) => row.id === draft.environment)
    const draftTaskRows = taskRows.filter((row) => row.dataset === draftDataset?.name || row.dataset === draft.source)
    const selectedTaskCount = draft.selectedTaskNames?.length ?? draftTaskRows.length
    const newJob: HarborJob = {
      id: nextJobId,
      name: `${draft.source.split('@')[0]}-draft`,
      status: 'queued',
      dataset: draft.source,
      agent: draft.agent,
      model: draft.model.split('/').at(-1) ?? draft.model,
      environment: draftEnvironment?.name ?? draft.environment,
      trials: `0 / ${selectedTaskCount}`,
      score: '-',
      cost: '$0.00',
      tokens: '0',
      tokenUsage: '0M',
      runtimeDuration: '00:00:00',
      createdAt: '2026-06-29 04:30:00',
      includeInLeaderboard: draft.includeInLeaderboard,
      jobDir: draft.jobsDir,
      eventLogPath: `${nextJobRoot}/job.log`,
      artifactPaths: [
        `${nextJobRoot}/harbor.config.json`,
        `${nextJobRoot}/harbor.capability.json`,
        `${nextJobRoot}/result.json`,
        `${nextJobRoot}/job.log`,
        nextJobRoot,
        `/Users/xuzhang/.ornnlab/HarnessLab/trials/${nextJobId}`,
      ],
      split: draft.split,
    }
    setJobs((current) => [newJob, ...current])
    if (draft.includeInLeaderboard) {
      setLeaderboardEntries((current) => [...current, buildLeaderboardEntryFromJob(newJob, draft.metric)])
    }
    setSelected(newJob)
    setJobDrawerOpen(true)
    navigate('jobs', 'list')
  }

  function updateEnvironmentProfiles(nextRows: typeof environmentRows) {
    setEnvironmentProfiles(nextRows)
    if (!nextRows.some((row) => row.id === draft.environment)) {
      setDraft((current) => ({ ...current, environment: nextRows[0]?.id ?? current.environment }))
    }
  }

  function removeFromLeaderboard(jobId: string) {
    setJobs((current) =>
      current.map((job) => (job.id === jobId ? { ...job, includeInLeaderboard: false } : job)),
    )
    setLeaderboardEntries((current) => current.filter((row) => row.jobId !== jobId))
  }

  function updateJobLeaderboardInclusion(jobId: string, includeInLeaderboard: boolean) {
    setJobs((current) =>
      current.map((job) => (job.id === jobId ? { ...job, includeInLeaderboard } : job)),
    )
    setSelected((current) => (current?.id === jobId ? { ...current, includeInLeaderboard } : current))
    if (!includeInLeaderboard) {
      setLeaderboardEntries((current) => current.filter((row) => row.jobId !== jobId))
      return
    }
    const targetJob = jobs.find((job) => job.id === jobId)
    if (!targetJob) {
      return
    }
    setLeaderboardEntries((current) => {
      if (current.some((row) => row.jobId === jobId)) {
        return current
      }
      return [...current, buildLeaderboardEntryFromJob({ ...targetJob, includeInLeaderboard })]
    })
  }

  return (
    <AppShell
      activePage={route.page}
      language={language}
      theme={theme}
      t={t}
      onLanguage={setLanguage}
      onNavigate={navigate}
      onTheme={() => setTheme((current) => (current === 'light' ? 'dark' : 'light'))}
    >
      {route.page === 'datasets' && (
        <DatasetsPage
          rows={filteredDatasets}
          search={datasetSearch}
          taskRows={taskRows}
          t={t}
          onSearch={setDatasetSearch}
        />
      )}
      {route.page === 'agents' && <AgentsPage rows={agentRows} t={t} />}
      {route.page === 'environments' && (
        <EnvironmentsPage rows={environmentProfiles} t={t} onRowsChange={updateEnvironmentProfiles} />
      )}
      {route.page === 'leaderboard' && (
        <LeaderboardPage
          dataset={leaderboardDataset}
          datasetSearch={leaderboardDatasetSearch}
          datasets={datasetRows}
          events={events}
          jobs={jobs}
          rows={filteredLeaderboard}
          t={t}
          trialRows={trialRows}
          onDataset={setLeaderboardDataset}
          onDatasetSearch={setLeaderboardDatasetSearch}
          onLeaderboardChange={updateJobLeaderboardInclusion}
          onRemove={removeFromLeaderboard}
        />
      )}
      {route.page === 'jobs' && route.jobView === 'list' && (
        <JobsPage
          events={events}
          jobs={filteredJobs}
          open={jobDrawerOpen}
          search={search}
          selected={selected}
          trialRows={trialRows}
          t={t}
          onClose={() => setJobDrawerOpen(false)}
          onLeaderboardChange={updateJobLeaderboardInclusion}
          onNewJob={() => navigate('jobs', 'new')}
          onSearch={setSearch}
          onSelect={(job) => {
            setSelected(job)
            setJobDrawerOpen(true)
          }}
        />
      )}
      {route.page === 'jobs' && route.jobView === 'new' && (
        <NewRunPage
          datasets={datasetRows}
          draft={draft}
          environments={environmentProfiles}
          taskRows={taskRows}
          t={t}
          onDraft={setDraft}
          onJobs={() => navigate('jobs', 'list')}
          onLaunch={launchDraft}
        />
      )}
      {route.page === 'system' && <SystemPage rows={systemRows} t={t} />}
    </AppShell>
  )
}

function buildLeaderboardEntryFromJob(job: HarborJob, metric = 'pass@1 mean'): LeaderboardRow {
  return {
    dataset: job.dataset,
    rank: 0,
    agentName: job.agent,
    harness: job.agent,
    model: job.model,
    score: job.score,
    trials: job.trials.split('/')[0]?.trim() ?? job.trials,
    cost: job.cost,
    tokens: job.tokenUsage,
    duration: job.runtimeDuration,
    jobId: job.id,
    split: job.split || 'default',
    metric,
    submitted: 'local only',
    reportPath: `reports/${job.id}.json`,
    comparabilityKey: `${job.dataset}:${job.split || 'default'}:${metric}`,
    uploadedUrl: '-',
    submissionId: '-',
    configHash: `cfg_${job.id}`,
    agentSnapshotHash: `agent_${job.agent}`,
  }
}
