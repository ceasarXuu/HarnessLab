import { useEffect, useMemo, useState } from 'react'
import { AppShell, type PageKey } from './components/AppShell'
import {
  events,
  initialDraft,
  jobs as seedJobs,
  trialRows,
  type HarborJob,
} from './data/demo'
import { agentRows, datasetRows, taskRows } from './data/demoCatalog'
import { leaderboardRows, systemRows } from './data/demoSystem'
import { getTranslator, type Locale } from './i18n'
import { JobsPage } from './pages/JobsPage'
import { AgentsPage } from './pages/AgentsPage'
import { DatasetsPage } from './pages/DatasetsPage'
import { LeaderboardPage } from './pages/LeaderboardPage'
import { NewRunPage } from './pages/NewRunPage'
import { SystemPage } from './pages/SystemPage'

type JobView = 'list' | 'new'

interface RouteState {
  jobView: JobView
  page: PageKey
}

const pageKeys = new Set<PageKey>(['jobs', 'datasets', 'agents', 'leaderboard', 'system'])

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
    return leaderboardRows.filter((row) => row.dataset === leaderboardDataset)
  }, [leaderboardDataset])

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
      tokens: '0',
      tokenUsage: '0/M',
      runtimeDuration: '00:00:00',
      createdAt: '2026-06-29 04:30:00',
    }
    setJobs((current) => [newJob, ...current])
    setSelected(newJob)
    setJobDrawerOpen(true)
    navigate('jobs', 'list')
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
          onNewJob={() => navigate('jobs', 'new')}
          onSearch={setDatasetSearch}
        />
      )}
      {route.page === 'agents' && <AgentsPage rows={agentRows} t={t} />}
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
          draft={draft}
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
