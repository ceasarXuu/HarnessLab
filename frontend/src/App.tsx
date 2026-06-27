import { useEffect, useMemo, useState } from 'react'
import { AppShell, type PageKey } from './components/AppShell'
import {
  events,
  initialDraft,
  jobs as seedJobs,
  systemRows,
  taskRows,
  trialRows,
  type HarborJob,
} from './data/demo'
import { getTranslator, type Locale } from './i18n'
import { JobsPage } from './pages/JobsPage'
import { NewRunPage } from './pages/NewRunPage'
import { SystemPage } from './pages/SystemPage'
import { TasksPage } from './pages/TasksPage'
import { TrialsPage } from './pages/TrialsPage'

const pageKeys = new Set<PageKey>(['jobs', 'new-run', 'tasks', 'trials', 'system'])

function readPageFromHash(): PageKey {
  const hash = window.location.hash.replace('#', '')
  return pageKeys.has(hash as PageKey) ? (hash as PageKey) : 'jobs'
}

function readLocale(): Locale {
  return window.localStorage.getItem('ornnlab.locale') === 'zh' ? 'zh' : 'en'
}

function readTheme(): 'light' | 'dark' {
  return window.localStorage.getItem('ornnlab.theme') === 'dark' ? 'dark' : 'light'
}

export function App() {
  const [activePage, setActivePage] = useState<PageKey>(readPageFromHash)
  const [jobs, setJobs] = useState(seedJobs)
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

  useEffect(() => {
    const onHashChange = () => setActivePage(readPageFromHash())
    window.addEventListener('hashchange', onHashChange)
    return () => window.removeEventListener('hashchange', onHashChange)
  }, [])

  useEffect(() => {
    document.documentElement.dataset.theme = theme
    document.documentElement.lang = language
    window.localStorage.setItem('ornnlab.theme', theme)
    window.localStorage.setItem('ornnlab.locale', language)
  }, [language, theme])

  function navigate(page: PageKey) {
    setActivePage(page)
    if (window.location.hash !== `#${page}`) {
      window.history.pushState(null, '', `#${page}`)
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
    navigate('jobs')
  }

  return (
    <AppShell
      activePage={activePage}
      language={language}
      theme={theme}
      t={t}
      onLanguage={setLanguage}
      onNavigate={navigate}
      onTheme={() => setTheme((current) => (current === 'light' ? 'dark' : 'light'))}
    >
      {activePage === 'jobs' && (
        <JobsPage
          events={events}
          jobs={filteredJobs}
          search={search}
          selected={selected}
          t={t}
          onNewJob={() => navigate('new-run')}
          onSearch={setSearch}
          onSelect={setSelected}
        />
      )}
      {activePage === 'new-run' && (
        <NewRunPage
          activeStep={activeStep}
          draft={draft}
          t={t}
          onDraft={setDraft}
          onLaunch={launchDraft}
          onStep={setActiveStep}
        />
      )}
      {activePage === 'tasks' && <TasksPage rows={taskRows} t={t} />}
      {activePage === 'trials' && <TrialsPage rows={trialRows} t={t} />}
      {activePage === 'system' && <SystemPage rows={systemRows} t={t} />}
    </AppShell>
  )
}
