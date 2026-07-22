import { useCallback, useEffect, useMemo, useState } from 'react'
import { useAgents, useCachedServerSearch, useDatasets, useEnvironments, useHarnesses, useHubConnection, useJobs, useLeaderboard, useOperation, usePollingRefresh, useSystemHealth } from '../api/hooks'
import { runDraftToCreateJobRequest } from '../api/requestMappers'
import { createRuntimeWebUiClient, readWebUiDataMode, type WebUiDataMode } from '../api/runtimeClient'
import { agentDtoToRow, datasetDtoToRow, environmentDtoToRow, harnessDtoToTemplate, jobDtoToHarborJob, leaderboardEntryDtoToRow, systemComponentDtoToRow } from '../api/viewModels'
import type { WebUiClient } from '../api/webUiClient'
import { defaultRunDraft, jobConfigDtoToRunDraft, reconcileRunDraftResources } from '../domain/defaults'
import type { HarborJob } from '../domain/harbor'
import { AppShell, type PageKey } from '../ui/components/AppShell'
import { ResourceStatus } from '../ui/components/ResourceStatus'
import { getTranslator, type Locale } from '../i18n'
import { JobsPage } from '../screens/JobsPage'
import { AgentsPage } from '../screens/AgentsPage'
import { DatasetsPage } from '../screens/DatasetsPage'
import { EnvironmentsPage } from '../screens/EnvironmentsPage'
import { LeaderboardPage } from '../screens/LeaderboardPage'
import { NewRunPage } from '../screens/NewRunPage'
import { NewAgentPage } from '../screens/NewAgentPage'
import { SystemPage } from '../screens/SystemPage'

type JobView = 'list' | 'new'
type EnvironmentView = 'list' | 'new' | 'copy'
type AgentView = 'list' | 'new'

interface RouteState {
  agentView: AgentView
  environmentId?: string
  environmentView: EnvironmentView
  jobView: JobView
  page: PageKey
}

interface AppProps {
  client?: WebUiClient
  dataMode?: WebUiDataMode
}

const pageKeys = new Set<PageKey>(['jobs', 'datasets', 'agents', 'environments', 'leaderboard', 'system'])

function readRouteFromHash(): RouteState {
  const hash = window.location.hash.replace('#', '')
  if (hash === 'jobs/new' || hash === 'new-run') {
    return { page: 'jobs', jobView: 'new', agentView: 'list', environmentView: 'list' }
  }
  if (hash.startsWith('agents/new')) {
    return { page: 'agents', jobView: 'list', agentView: 'new', environmentView: 'list' }
  }
  if (hash === 'environments/new') {
    return { page: 'environments', jobView: 'list', agentView: 'list', environmentView: 'new' }
  }
  const environmentMatch = hash.match(/^environments\/([^/]+)\/copy$/)
  if (environmentMatch) {
    return {
      page: 'environments',
      jobView: 'list',
      agentView: 'list',
      environmentView: 'copy',
      environmentId: environmentMatch[1],
    }
  }
  return {
    page: pageKeys.has(hash as PageKey) ? (hash as PageKey) : 'jobs',
    jobView: 'list',
    agentView: 'list',
    environmentView: 'list',
  }
}

function readLocale(): Locale {
  return window.localStorage.getItem('ornnlab.locale') === 'zh' ? 'zh' : 'en'
}

function readTheme(): 'light' | 'dark' {
  return window.localStorage.getItem('ornnlab.theme') === 'dark' ? 'dark' : 'light'
}

export function App({ client: injectedClient, dataMode: injectedDataMode }: AppProps) {
  const [route, setRoute] = useState<RouteState>(readRouteFromHash)
  const dataMode = injectedDataMode ?? readWebUiDataMode()
  const client = useMemo(() => injectedClient ?? createRuntimeWebUiClient(dataMode), [dataMode, injectedClient])
  const writesEnabled = true
  const [search, setSearch] = useState('')
  const jobSearchQuery = search.trim() || undefined
  const loadJobSearch = useCallback((query: string) => client.listJobs({ limit: 100, q: query }), [client])
  const jobSearchResource = useCachedServerSearch('jobs', jobSearchQuery, loadJobSearch)
  const agentsResource = useAgents(client, { limit: 100 })
  const harnessesResource = useHarnesses(client, { limit: 100 })
  const jobsResource = useJobs(client, { limit: 100 })
  const datasetsResource = useDatasets(client, { limit: 100 })
  const environmentsResource = useEnvironments(client, { limit: 100 })
  const hubConnectionResource = useHubConnection(client)
  const [datasetSearch, setDatasetSearch] = useState('')
  const datasetSearchQuery = datasetSearch.trim() || undefined
  const loadDatasetSearch = useCallback((query: string) => client.listDatasets({ limit: 100, q: query }), [client])
  const datasetSearchResource = useCachedServerSearch('datasets', datasetSearchQuery, loadDatasetSearch)
  const [leaderboardDataset, setLeaderboardDataset] = useState('')
  const [leaderboardDatasetSearch, setLeaderboardDatasetSearch] = useState('')
  const leaderboardResource = useLeaderboard(client, { dataset: leaderboardDataset })
  const systemResource = useSystemHealth(client)
  const jobOperation = useOperation(client)
  const [selected, setSelected] = useState<HarborJob | null>(null)
  const [jobDrawerOpen, setJobDrawerOpen] = useState(false)
  const [copyJobError, setCopyJobError] = useState<string | null>(null)
  const [draft, setDraft] = useState(defaultRunDraft)
  const [language, setLanguage] = useState<Locale>(readLocale)
  const [theme, setTheme] = useState<'light' | 'dark'>(readTheme)
  const t = useMemo(() => getTranslator(language), [language])
  const agents = useMemo(() => agentsResource.data?.items.map(agentDtoToRow) ?? [], [agentsResource.data])
  const harnesses = useMemo(
    () => harnessesResource.data?.items.map(harnessDtoToTemplate) ?? [],
    [harnessesResource.data],
  )
  const configuredAgents = agents
  const datasets = useMemo(() => datasetsResource.data?.items.map(datasetDtoToRow) ?? [], [datasetsResource.data])
  const environmentProfiles = useMemo(
    () => environmentsResource.data?.items.map(environmentDtoToRow) ?? [],
    [environmentsResource.data],
  )
  const jobs = useMemo(() => jobsResource.data?.items.map(jobDtoToHarborJob) ?? [], [jobsResource.data])
  const selectedJob = useMemo(
    () => selected ? jobs.find((job) => job.id === selected.id) ?? selected : null,
    [jobs, selected],
  )
  const leaderboardEntries = useMemo(
    () => leaderboardResource.data?.items.map(leaderboardEntryDtoToRow) ?? [],
    [leaderboardResource.data],
  )
  const hubConnection = hubConnectionResource.loading
    ? 'loading'
    : hubConnectionResource.data?.status ?? 'unavailable'
  const hasLiveJob = jobs.some((job) => job.status === 'queued' || job.status === 'running')
    || selectedJob?.status === 'queued'
    || selectedJob?.status === 'running'

  usePollingRefresh(jobsResource.refresh, route.page === 'jobs' && route.jobView === 'list' && hasLiveJob)

  useEffect(() => {
    if (agentsResource.loading || datasetsResource.loading || environmentsResource.loading) return
    setDraft((current) => {
      const next = reconcileRunDraftResources(current, {
        agents: configuredAgents,
        datasets,
        environments: environmentProfiles,
      })
      return next.agent === current.agent
        && next.environment === current.environment
        && next.model === current.model
        && next.source === current.source
        ? current
        : next
    })
  }, [agentsResource.loading, configuredAgents, datasets, datasetsResource.loading, environmentProfiles, environmentsResource.loading])

  useEffect(() => {
    const status = jobOperation.operation?.status
    if (status !== 'completed' && status !== 'failed' && status !== 'cancelled') return
    void jobsResource.refresh()
    if (status === 'completed') void leaderboardResource.refresh()
  }, [jobOperation.operation?.id, jobOperation.operation?.status, jobsResource.refresh, leaderboardResource.refresh])

  const selectedJobActionError = jobOperation.operation?.type === 'resume-job'
    && jobOperation.operation.resourceId === selectedJob?.id
    && jobOperation.operation.status === 'failed'
    ? t('resumeJobFailed')
    : copyJobError ?? jobOperation.error?.message ?? null

  const filteredJobs = useMemo(() => {
    if (!jobSearchQuery) return jobs
    if (jobSearchResource.data) return jobSearchResource.data.items.map(jobDtoToHarborJob)
    const query = jobSearchQuery.toLowerCase()
    return jobs.filter((job) =>
      [job.name, job.dataset, job.agent, job.model, job.status].some((value) =>
        value.toLowerCase().includes(query),
      ),
    )
  }, [jobSearchQuery, jobSearchResource.data, jobs])

  const datasetSearchFallback = useMemo(() => {
    if (!datasetSearchQuery) return datasets
    const needle = datasetSearchQuery.toLowerCase()
    return datasets.filter((row) =>
      [row.name, row.version, row.visibility, row.source, row.digest].some((value) =>
        (value ?? '').toLowerCase().includes(needle),
      ),
    )
  }, [datasetSearchQuery, datasets])

  const filteredDatasets = useMemo(
    () => (datasetSearchQuery ? datasetSearchResource.data?.items.map(datasetDtoToRow) ?? datasetSearchFallback : datasets),
    [datasetSearchFallback, datasetSearchQuery, datasetSearchResource.data?.items, datasets],
  )

  const refreshDatasets = useCallback(
    async () => {
      if (datasetSearchQuery) {
        await datasetSearchResource.refresh()
        return
      }
      await datasetsResource.refresh()
    },
    [datasetSearchQuery, datasetSearchResource.refresh, datasetsResource.refresh],
  )
  const hasActiveDatasetDownload = filteredDatasets.some(
    (dataset) => dataset.downloadStatus === 'downloading',
  )
  usePollingRefresh(
    refreshDatasets,
    route.page === 'datasets' && hasActiveDatasetDownload,
  )

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

  const navigate = useCallback((page: PageKey, jobView: JobView = 'list') => {
    const nextRoute: RouteState = {
      page,
      jobView: page === 'jobs' ? jobView : 'list',
      agentView: 'list',
      environmentView: 'list',
    }
    const nextHash = nextRoute.page === 'jobs' && nextRoute.jobView === 'new' ? '#jobs/new' : `#${nextRoute.page}`
    setRoute(nextRoute)
    if (window.location.hash !== nextHash) {
      window.history.pushState(null, '', nextHash)
    }
  }, [])

  const navigateAgent = useCallback((agentView: AgentView) => {
    const nextRoute: RouteState = { page: 'agents', jobView: 'list', agentView, environmentView: 'list' }
    const nextHash = agentView === 'new' ? '#agents/new' : '#agents'
    setRoute(nextRoute)
    if (window.location.hash !== nextHash) {
      window.history.pushState(null, '', nextHash)
    }
  }, [])

  const navigateEnvironment = useCallback((environmentView: EnvironmentView, environmentId?: string) => {
    const nextRoute: RouteState = { page: 'environments', jobView: 'list', agentView: 'list', environmentView, environmentId }
    const nextHash =
      environmentView === 'list'
        ? '#environments'
        : environmentView === 'new'
          ? '#environments/new'
        : `#environments/${environmentId}/copy`
    setRoute(nextRoute)
    if (window.location.hash !== nextHash) {
      window.history.pushState(null, '', nextHash)
    }
  }, [])

  async function launchDraft() {
    if (!writesEnabled) return
    const response = await client.createJob(runDraftToCreateJobRequest(draft))
    const created = await jobOperation.submit(
      () => Promise.resolve(response),
      ({ operation }) => operation,
    )
    if (!created || !response.data) return
    setSelected(jobDtoToHarborJob(response.data.job))
    setJobDrawerOpen(true)
    navigate('jobs', 'list')
    void jobsResource.refresh()
  }

  function copyJobConfig() {
    if (!navigator.clipboard) return
    void navigator.clipboard.writeText(JSON.stringify(runDraftToCreateJobRequest(draft).config, null, 2))
  }

  async function removeFromLeaderboard(jobId: string) {
    await updateJobLeaderboardInclusion(jobId, false)
  }

  async function updateJobLeaderboardInclusion(jobId: string, includeInLeaderboard: boolean) {
    if (!writesEnabled) return
    await jobOperation.submit(
      () => client.updateJobLeaderboard(jobId, { includeInLeaderboard }),
      ({ operation }) => operation,
    )
  }

  async function runJobAction(jobId: string, action: 'cancel' | 'resume') {
    if (!writesEnabled) return
    const mutation = action === 'cancel'
      ? () => client.cancelJob(jobId)
      : () => client.resumeJob(jobId)
    await jobOperation.submit(mutation, ({ operation }) => operation)
  }

  async function copyExistingJob(jobId: string) {
    setCopyJobError(null)
    const response = await client.getJobCopyConfig(jobId)
    if (!response.data) {
      setCopyJobError(t('copyJobFailed'))
      return
    }
    const copied = jobConfigDtoToRunDraft(response.data)
    const resourcesReady = !agentsResource.loading
      && !datasetsResource.loading
      && !environmentsResource.loading
    setDraft(resourcesReady
      ? reconcileRunDraftResources(copied, {
          agents: configuredAgents,
          datasets,
          environments: environmentProfiles,
        })
      : copied)
    setJobDrawerOpen(false)
    navigate('jobs', 'new')
  }

  return (
    <AppShell
      activePage={route.page}
      hubConnection={hubConnection}
      language={language}
      theme={theme}
      t={t}
      onLanguage={setLanguage}
      onNavigate={navigate}
      onTheme={() => setTheme((current) => (current === 'light' ? 'dark' : 'light'))}
    >
      {route.page === 'datasets' && (
        <>
          <DatasetsPage
            writesEnabled={writesEnabled}
            client={client}
            rows={filteredDatasets}
            search={datasetSearch}
            t={t}
            onRefresh={refreshDatasets}
            onPrepareTaskRun={(datasetRef, taskName) => {
              setDraft((current) => ({ ...current, source: datasetRef, selectedTaskNames: [taskName] }))
              navigate('jobs', 'new')
            }}
            onSearch={setDatasetSearch}
          />
          <ResourceStatus
            error={(datasetSearchQuery ? datasetSearchResource.error : datasetsResource.error) ? t('unableToLoadDatasets') : null}
          />
        </>
      )}
      {route.page === 'agents' && route.agentView === 'list' && (
        <>
          <AgentsPage
            writesEnabled={writesEnabled}
            client={client}
            rows={agents}
            t={t}
            onNewAgent={() => navigateAgent('new')}
            onRefresh={agentsResource.refresh}
          />
          <ResourceStatus
            error={agentsResource.error ? t('unableToLoadAgents') : null}
          />
        </>
      )}
      {route.page === 'agents' && route.agentView === 'new' && (
        <NewAgentPage
          canSave={writesEnabled}
          client={client}
          harnesses={harnesses}
          rows={agents}
          t={t}
          onAgents={() => navigateAgent('list')}
          onRefresh={agentsResource.refresh}
        />
      )}
      {route.page === 'environments' && (
        <>
          <EnvironmentsPage
            writesEnabled={writesEnabled}
            client={client}
            environmentId={route.environmentId}
            rows={environmentProfiles}
            t={t}
            view={route.environmentView}
            onRefresh={environmentsResource.refresh}
            onView={navigateEnvironment}
          />
          <ResourceStatus
            error={environmentsResource.error ? t('unableToLoadEnvironments') : null}
          />
        </>
      )}
      {route.page === 'leaderboard' && (
        <>
          <LeaderboardPage
            writesEnabled={writesEnabled}
            actionError={selectedJobActionError}
            dataset={leaderboardDataset}
            datasetSearch={leaderboardDatasetSearch}
            leaderboardDatasets={datasets}
            client={client}
            jobs={jobs}
            rows={filteredLeaderboard}
            t={t}
            onDataset={setLeaderboardDataset}
            onDatasetSearch={setLeaderboardDatasetSearch}
            onJobAction={runJobAction}
            onCopyJob={copyExistingJob}
            onLeaderboardChange={updateJobLeaderboardInclusion}
            onRemove={removeFromLeaderboard}
          />
          <ResourceStatus error={jobOperation.error?.message ?? null} />
          <ResourceStatus
            error={datasetsResource.error || leaderboardResource.error ? t('unableToLoadLeaderboard') : null}
          />
        </>
      )}
      {route.page === 'jobs' && route.jobView === 'list' && (
        <>
          <JobsPage
            writesEnabled={writesEnabled}
            client={client}
            jobs={filteredJobs}
            open={jobDrawerOpen}
            search={search}
            selected={selectedJob}
            actionError={selectedJobActionError}
            t={t}
            onClose={() => setJobDrawerOpen(false)}
            onLeaderboardChange={updateJobLeaderboardInclusion}
            onJobAction={runJobAction}
            onCopyJob={copyExistingJob}
            onNewJob={() => navigate('jobs', 'new')}
            onRefresh={jobsResource.refresh}
            onSearch={setSearch}
            onSelect={(job) => {
              setCopyJobError(null)
              setSelected(job)
              setJobDrawerOpen(true)
            }}
          />
          <ResourceStatus
            error={(jobSearchQuery ? jobSearchResource.error : jobsResource.error) ? t('unableToLoadJobs') : null}
          />
        </>
      )}
      {route.page === 'jobs' && route.jobView === 'new' && (
        <>
          <NewRunPage
            canLaunch={
              writesEnabled
              && jobOperation.operation?.status !== 'queued'
              && jobOperation.operation?.status !== 'running'
            }
            submitError={jobOperation.error?.message}
            agents={configuredAgents}
            datasets={datasets}
            client={client}
            draft={draft}
            environments={environmentProfiles}
            t={t}
            onDraft={setDraft}
            onJobs={() => navigate('jobs', 'list')}
            onCopyJobConfig={copyJobConfig}
            onLaunch={launchDraft}
            onReset={() => setDraft(defaultRunDraft)}
          />
        </>
      )}
      {route.page === 'system' && (
        <>
          <SystemPage
            writesEnabled={writesEnabled}
            client={client}
            rows={systemResource.data?.items.map(systemComponentDtoToRow) ?? []}
            t={t}
            onRefresh={systemResource.refresh}
          />
          <ResourceStatus
            error={systemResource.error ? t('unableToLoadSystem') : null}
          />
        </>
      )}
    </AppShell>
  )
}
