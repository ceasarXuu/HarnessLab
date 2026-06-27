import { useEffect, useMemo, useState } from 'react'
import { AppShell, type PageKey } from './components/AppShell'
import {
  events,
  datasetRows,
  initialDraft,
  jobs as seedJobs,
  systemRows,
  taskRows,
  trialRows,
  type HarborJob,
} from './data/demo'
import { getTranslator, type Locale } from './i18n'
import { JobsPage } from './pages/JobsPage'
import { DatasetsPage } from './pages/DatasetsPage'
import { NewRunPage } from './pages/NewRunPage'
import { SystemPage } from './pages/SystemPage'
import { TasksPage } from './pages/TasksPage'
import { TrialsPage } from './pages/TrialsPage'

type JobView = 'list' | 'new'

interface RouteState {
  jobView: JobView
  page: PageKey
}

const pageKeys = new Set<PageKey>(['datasets', 'jobs', 'tasks', 'trials', 'system'])

function readRouteFromHash(): RouteState {
  const hash = window.location.hash.replace('#', '')
  if (hash === 'jobs/new' || hash === 'new-run') {
    return { page: 'jobs', jobView: 'new' }
  }
  return {
    page: pageKeys.has(hash as PageKey) ? (hash as PageKey) : 'datasets',
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
  const [selected, setSelected] = useState(seedJobs[0])
  const [search, setSearch] = useState('')
  const [activeStep, setActiveStep] = useState('Source')
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
      updated: 'just now',
    }
    setJobs((current) => [newJob, ...current])
    setSelected(newJob)
    setActiveStep('Review')
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
      onNewJob={() => navigate('jobs', 'new')}
      onTheme={() => setTheme((current) => (current === 'light' ? 'dark' : 'light'))}
    >
      {route.page === 'datasets' && (
        <DatasetsPage rows={filteredDatasets} search={datasetSearch} t={t} onSearch={setDatasetSearch} />
      )}
      {route.page === 'jobs' && route.jobView === 'list' && (
        <JobsPage
          events={events}
          jobs={filteredJobs}
          search={search}
          selected={selected}
          t={t}
          onNewJob={() => navigate('jobs', 'new')}
          onSearch={setSearch}
          onSelect={setSelected}
        />
      )}
      {route.page === 'jobs' && route.jobView === 'new' && (
        <NewRunPage
          activeStep={activeStep}
          draft={draft}
          t={t}
          onDraft={setDraft}
          onLaunch={launchDraft}
          onStep={setActiveStep}
        />
      )}
      {route.page === 'tasks' && <TasksPage rows={taskRows} t={t} />}
      {route.page === 'trials' && <TrialsPage rows={trialRows} t={t} />}
      {route.page === 'system' && <SystemPage rows={systemRows} t={t} />}
    </AppShell>
  )
}
