import { events, jobs, trialRows } from '../mocks/demo'
import { agentRows, datasetRows, environmentRows, taskRows } from '../mocks/demoCatalog'
import { leaderboardRows, systemRows } from '../mocks/demoSystem'
import type {
  ApiError,
  ApiResponse,
  DatasetDto,
  DatasetTaskDto,
  CreateJobRequestDto,
  CreateJobResponseDto,
  JobDto,
  Page,
  OperationResultDto,
  UpdateCheckResultDto,
  UpdateJobLeaderboardResponseDto,
} from './contract'
import type { DatasetTaskQuery, ListQuery } from './contract'
import { buildLeaderboardDatasets, toAgentDto, toDatasetDto, toDatasetTaskDto, toEnvironmentDto, toJobDto, toJobEventDto, toLeaderboardEntryDto, toSystemComponentDto, toTrialDto } from './mockMappers'
import { createMockOperationStore } from './mockOperations'
import type { WebUiClient } from './webUiClient'

const requestMeta = { requestId: 'mock-webui-client' }

export function createMockWebUiClient(): WebUiClient {
  let jobDtos = jobs.map(toJobDto)
  let agentDtos = agentRows.map(toAgentDto)
  let datasetDtos = datasetRows.map(toDatasetDto)
  let environmentDtos = environmentRows.map(toEnvironmentDto)
  const taskDtos = taskRows.map(toDatasetTaskDto)
  const eventDtos = events.map((event) => ({ event: toJobEventDto(event), jobId: event.jobId }))
  let leaderboardDtos = leaderboardRows.map(toLeaderboardEntryDto)
  const systemDtos = systemRows.map(toSystemComponentDto)
  const trialDtos = trialRows.map(toTrialDto)
  const operations = createMockOperationStore()

  return {
    async cancelJob(id) {
      jobDtos = jobDtos.map((job) => (job.id === id ? { ...job, status: 'failed' } : job))
      return operationResult(operations.submit('cancel-job', 'job', id))
    },
    async cancelDatasetDownload(ref) {
      datasetDtos = datasetDtos.map((dataset) => (dataset.ref === ref ? { ...dataset, download: { status: 'not-downloaded' } } : dataset))
      return operationResult(operations.submit('cancel-dataset-download', 'dataset', ref))
    },
    async cancelOperation(id) {
      const operation = operations.cancel(id)
      return operation ? operationResult(operation) : failure('OPERATION_NOT_CANCELLABLE', 'Operation cannot be cancelled')
    },
    async checkForSystemUpdate() {
      return success<UpdateCheckResultDto>({ currentVersion: '0.1.3', latestVersion: '0.1.3', updateAvailable: false })
    },
    async cleanDockerCache() {
      return operationResult(operations.submit('clean-docker-cache', 'system', 'docker'))
    },
    async cleanStorageCache() {
      return operationResult(operations.submit('clean-storage-cache', 'system', 'storage'))
    },
    async copyEnvironment(id) {
      const source = environmentDtos.find((environment) => environment.id === id)
      if (!source) return failure('ENVIRONMENT_NOT_FOUND', 'Environment not found')
      const copy = { ...source, id: uniqueId(environmentDtos, `${id}-copy`), name: `${source.name} copy`, profileType: 'custom' as const }
      environmentDtos = [...environmentDtos, copy]
      return operationResult(operations.submit('copy-environment', 'environment', copy.id))
    },
    async createAgent(agent) {
      agentDtos = [agent, ...agentDtos]
      return operationResult(operations.submit('create-agent', 'agent', agent.id))
    },
    async createEnvironment(environment) {
      environmentDtos = [environment, ...environmentDtos]
      return operationResult(operations.submit('create-environment', 'environment', environment.id))
    },
    async createJob(request) {
      const job = buildQueuedJob(jobDtos, request)
      jobDtos = [job, ...jobDtos]
      const operation = operations.submit('create-job', 'job', job.id)
      return success<CreateJobResponseDto>({ job, operation })
    },
    async deleteAgent(id) {
      const target = agentDtos.find((agent) => agent.id === id)
      if (!target) return failure('AGENT_NOT_FOUND', 'Agent not found')
      if (target.type === 'built-in') return failure('AGENT_BUILT_IN_IMMUTABLE', 'Built-in agents cannot be deleted')
      agentDtos = agentDtos.filter((agent) => agent.id !== id)
      return operationResult(operations.submit('delete-agent', 'agent', id))
    },
    async deleteEnvironment(id) {
      const target = environmentDtos.find((environment) => environment.id === id)
      if (!target) return failure('ENVIRONMENT_NOT_FOUND', 'Environment not found')
      if (target.profileType === 'built-in') return failure('ENVIRONMENT_BUILT_IN_IMMUTABLE', 'Built-in environments cannot be deleted')
      environmentDtos = environmentDtos.filter((environment) => environment.id !== id)
      return operationResult(operations.submit('delete-environment', 'environment', id))
    },
    async deleteLocalDataset(ref) {
      datasetDtos = datasetDtos.map((dataset) => (dataset.ref === ref ? { ...dataset, download: { status: 'not-downloaded' } } : dataset))
      return operationResult(operations.submit('delete-local-dataset', 'dataset', ref))
    },
    async downloadDataset(ref) {
      const dataset = datasetDtos.find((item) => item.ref === ref)
      if (!dataset) return failure('DATASET_NOT_FOUND', 'Dataset not found')
      datasetDtos = datasetDtos.map((item) => item.ref === ref ? {
        ...item,
        download: { path: `~/.cache/harbor/datasets/${ref.replace('@', '-')}`, sizeBytes: item.download.sizeBytes, status: 'downloaded' },
      } : item)
      return operationResult(operations.submit('download-dataset', 'dataset', ref))
    },
    async getAgent(id) {
      return findById(agentDtos, id, 'AGENT_NOT_FOUND', 'Agent not found', 'id')
    },
    async getDataset(ref) {
      return findById(datasetDtos, ref, 'DATASET_NOT_FOUND', 'Dataset not found', 'ref')
    },
    async getEnvironment(id) {
      return findById(environmentDtos, id, 'ENVIRONMENT_NOT_FOUND', 'Environment not found', 'id')
    },
    async getHubConnection() {
      return success({ status: 'connected' as const })
    },
    async getJob(id) {
      return findById(jobDtos, id, 'JOB_NOT_FOUND', 'Job not found', 'id')
    },
    async getOperation(id) {
      const operation = operations.get(id)
      return operation ? success(operation) : failure('OPERATION_NOT_FOUND', 'Operation not found')
    },
    async importDataset(request) {
      const ref = `${request.name}@${request.version}`
      const dataset: DatasetDto = {
        download: { path: request.path, status: 'downloaded' },
        name: request.name,
        ref,
        source: 'local import',
        splits: ['local'],
        taskCount: request.taskCount,
        version: request.version,
        visibility: 'private',
      }
      datasetDtos = [dataset, ...datasetDtos]
      return operationResult(operations.submit('import-dataset', 'dataset', ref))
    },
    async installSystemUpdate() {
      return operationResult(operations.submit('install-system-update', 'system', 'ornnlab-service'))
    },
    async listAgents(query) {
      return success(page(filterByQuery(agentDtos, query, (agent) => [agent.agentName, agent.harness, agent.status, agent.type])))
    },
    async listDatasetTasks(ref, query) {
      return success(page(filterDatasetTasks(taskDtos, ref, query)))
    },
    async listDatasets(query) {
      return success(page(filterByQuery(datasetDtos, query, (dataset) => [dataset.name, dataset.version, dataset.source])))
    },
    async listEnvironments(query) {
      return success(page(filterByQuery(environmentDtos, query, (environment) => [environment.name, environment.environmentType, environment.profileType])))
    },
    async listJobEvents(id) {
      return success(eventDtos.filter((entry) => entry.jobId === id).map((entry) => entry.event))
    },
    async listJobTrials(id) {
      return success(trialDtos.filter((trial) => trial.jobId === id))
    },
    async listJobs(query) {
      return success(page(filterByQuery(jobDtos, query, (job) => [job.name, job.datasetRef, job.agentName, job.harness, job.model, job.status])))
    },
    async listLeaderboard(query) {
      const byDataset = leaderboardDtos.filter((entry) => entry.datasetRef === query.dataset)
      return success(page(filterByQuery(byDataset, query, (entry) => [entry.agentName, entry.harness, entry.model, entry.metric, entry.split])))
    },
    async listLeaderboardDatasets(query) {
      const leaderboardDatasets = buildLeaderboardDatasets(leaderboardDtos)
      return success(page(filterByQuery(leaderboardDatasets, query, (dataset) => [dataset.ref, dataset.name, dataset.version])))
    },
    async listSystemHealth() {
      return success(page(systemDtos))
    },
    async restartSystemService() {
      return operationResult(operations.submit('restart-service', 'system', 'ornnlab-service'))
    },
    async retryJob(id) {
      jobDtos = jobDtos.map((job) => (job.id === id ? { ...job, status: 'queued' } : job))
      return operationResult(operations.submit('retry-job', 'job', id))
    },
    async resumeJob(id) {
      jobDtos = jobDtos.map((job) => (job.id === id ? { ...job, status: 'running' } : job))
      return operationResult(operations.submit('resume-job', 'job', id))
    },
    async runDatasetTask(ref, taskName) {
      return operationResult(operations.submit('run-dataset-task', 'job', `${ref}:${taskName}`))
    },
    async syncDataset(ref) {
      return operationResult(operations.submit('sync-dataset', 'dataset', ref))
    },
    async updateAgent(id, agent) {
      const target = agentDtos.find((item) => item.id === id)
      if (!target) return failure('AGENT_NOT_FOUND', 'Agent not found')
      if (target.type === 'built-in') return failure('AGENT_BUILT_IN_IMMUTABLE', 'Built-in agents cannot be updated')
      agentDtos = agentDtos.map((item) => item.id === id ? { ...agent, id } : item)
      return operationResult(operations.submit('update-agent', 'agent', id))
    },
    async updateEnvironment(id, environment) {
      const target = environmentDtos.find((item) => item.id === id)
      if (!target) return failure('ENVIRONMENT_NOT_FOUND', 'Environment not found')
      if (target.profileType === 'built-in') return failure('ENVIRONMENT_BUILT_IN_IMMUTABLE', 'Built-in environments cannot be updated')
      environmentDtos = environmentDtos.map((item) => item.id === id ? { ...environment, id } : item)
      return operationResult(operations.submit('update-environment', 'environment', id))
    },
    async updateJobLeaderboard(id, request) {
      const target = jobDtos.find((job) => job.id === id)
      if (!target) return failure('JOB_NOT_FOUND', 'Job not found')
      const job = { ...target, includeInLeaderboard: request.includeInLeaderboard }
      jobDtos = jobDtos.map((item) => item.id === id ? job : item)
      if (!request.includeInLeaderboard) leaderboardDtos = leaderboardDtos.filter((entry) => entry.jobId !== id)
      return success<UpdateJobLeaderboardResponseDto>({
        job,
        leaderboard: leaderboardDtos.filter((entry) => entry.datasetRef === job.datasetRef),
        operation: operations.submit('update-job-leaderboard', 'job', id),
      })
    },
  }
}

function success<T>(data: T): ApiResponse<T> {
  return { data, error: null, meta: requestMeta }
}

function failure(code: string, message: string): ApiResponse<null> {
  const error: ApiError = { code, message }
  return { data: null, error, meta: requestMeta }
}

function page<T>(items: T[]): Page<T> {
  return { items, total: items.length }
}

function operationResult(operation: OperationResultDto['operation']): ApiResponse<OperationResultDto> {
  return success({ operation })
}

function uniqueId(items: Array<{ id: string }>, base: string): string {
  const ids = new Set(items.map((item) => item.id))
  if (!ids.has(base)) return base
  let index = 2
  let candidate = `${base}-${index}`
  while (ids.has(candidate)) {
    index += 1
    candidate = `${base}-${index}`
  }
  return candidate
}

function buildQueuedJob(existing: JobDto[], request: CreateJobRequestDto): JobDto {
  const id = uniqueId(existing, request.config.jobName.toLowerCase().replace(/[^a-z0-9]+/g, '-'))
  const root = `/Users/xuzhang/.ornnlab/HarnessLab/${request.config.jobsDir}`
  const selectedTaskCount = request.config.selectedTaskNames?.length ?? 0
  return {
    agentName: request.config.agentName,
    artifactPaths: [`${root}/harbor.config.json`, `${root}/result.json`, `${root}/job.log`, root],
    costUsd: 0,
    createdAt: new Date().toISOString(),
    datasetRef: request.config.datasetRef,
    environmentName: request.config.environmentPresetId,
    eventLogPath: `${root}/job.log`,
    harness: request.config.agentName,
    id,
    includeInLeaderboard: request.config.includeInLeaderboard,
    jobDir: request.config.jobsDir,
    model: request.config.model,
    name: request.config.jobName,
    runtimeSeconds: 0,
    score: null,
    split: request.config.split,
    status: 'queued',
    tokenUsageM: 0,
    trial: { completed: 0, total: selectedTaskCount },
  }
}

function findById<T, K extends keyof T>(
  items: T[],
  id: T[K],
  code: string,
  message: string,
  key: K,
): ApiResponse<T | null> {
  const item = items.find((candidate) => candidate[key] === id)
  return item ? success(item) : failure(code, message)
}

function filterByQuery<T>(items: T[], query: ListQuery | undefined, fields: (item: T) => string[]): T[] {
  const needle = query?.q?.trim().toLowerCase()
  if (!needle) return items
  return items.filter((item) => fields(item).some((field) => field.toLowerCase().includes(needle)))
}

function filterDatasetTasks(tasks: DatasetTaskDto[], ref: string, query: DatasetTaskQuery | undefined): DatasetTaskDto[] {
  const byDataset = tasks.filter((task) => task.datasetRef === ref)
  const bySplit = query?.split ? byDataset.filter((task) => task.splits.includes(query.split ?? '')) : byDataset
  return filterByQuery(bySplit, query, (task) => [task.name, task.description])
}
