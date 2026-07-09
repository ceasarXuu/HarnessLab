import { events, jobs, trialRows } from '../mocks/demo'
import { datasetRows, taskRows } from '../mocks/demoCatalog'
import type { HarborJob, DatasetRow, TaskRow, TrialRow } from '../domain/harbor'
import type { ApiError, ApiResponse, DatasetDto, DatasetTaskDto, JobDto, JobEventDto, Page, ScoreDto, TrialDto } from './contract'
import type { DatasetTaskQuery, ListQuery } from './contract'
import type { WebUiClient } from './webUiClient'

const requestMeta = { requestId: 'mock-webui-client' }

export function createMockWebUiClient(): WebUiClient {
  const jobDtos = jobs.map(toJobDto)
  const datasetDtos = datasetRows.map(toDatasetDto)
  const taskDtos = taskRows.map(toDatasetTaskDto)
  const eventDtos = events.map((event) => ({ event: toJobEventDto(event), jobId: event.jobId }))
  const trialDtos = trialRows.map(toTrialDto)

  return {
    async getDataset(ref) {
      return datasetDtos.find((dataset) => dataset.ref === ref)
        ? success(datasetDtos.find((dataset) => dataset.ref === ref) ?? null)
        : failure('DATASET_NOT_FOUND', 'Dataset not found')
    },
    async getJob(id) {
      return jobDtos.find((job) => job.id === id)
        ? success(jobDtos.find((job) => job.id === id) ?? null)
        : failure('JOB_NOT_FOUND', 'Job not found')
    },
    async listDatasetTasks(ref, query) {
      return success(page(filterDatasetTasks(taskDtos, ref, query)))
    },
    async listDatasets(query) {
      return success(page(filterByQuery(datasetDtos, query, (dataset) => [dataset.name, dataset.version, dataset.source])))
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

function toJobDto(job: HarborJob): JobDto {
  return {
    id: job.id,
    name: job.name,
    status: job.status,
    datasetRef: job.dataset,
    agentName: job.agent,
    harness: job.agent,
    model: job.model,
    environmentName: job.environment,
    trial: parseTrial(job.trials),
    score: parseScore(job.score),
    costUsd: parseNumber(job.cost),
    tokenUsageM: parseNumber(job.tokenUsage),
    runtimeSeconds: parseDuration(job.runtimeDuration),
    createdAt: job.createdAt,
    includeInLeaderboard: job.includeInLeaderboard,
    jobDir: job.jobDir,
    eventLogPath: job.eventLogPath,
    artifactPaths: job.artifactPaths,
    split: job.split,
    failureCode: job.failureCode,
  }
}

function toDatasetDto(dataset: DatasetRow): DatasetDto {
  return {
    ref: `${dataset.name}@${dataset.version}`,
    name: dataset.name,
    version: dataset.version,
    visibility: dataset.visibility,
    taskCount: dataset.tasks,
    source: dataset.source,
    download: {
      status: dataset.downloadStatus,
      path: dataset.downloadPath ?? dataset.path ?? dataset.downloadDir,
      sizeBytes: parseSizeBytes(dataset.size),
    },
    registryUrl: dataset.registryUrl,
    splits: dataset.splits ?? [],
  }
}

function toDatasetTaskDto(task: TaskRow): DatasetTaskDto {
  const dataset = datasetRows.find((row) => row.name === task.dataset)
  return {
    datasetRef: dataset ? `${dataset.name}@${dataset.version}` : task.dataset,
    description: task.description,
    name: task.name,
    splits: task.splits ?? [],
  }
}

function toJobEventDto(event: typeof events[number]): JobEventDto {
  return { level: event.level, message: event.message, occurredAt: event.time }
}

function toTrialDto(trial: TrialRow): TrialDto {
  return {
    costUsd: parseNumber(trial.cost),
    id: trial.id,
    jobId: trial.jobId,
    logPath: trial.logPath,
    retryCount: trial.retries,
    runtimeSeconds: parseDuration(trial.duration),
    score: parseScore(trial.score),
    status: trial.result as TrialDto['status'],
    taskName: trial.task,
    tokenUsageM: parseTokenUsageM(trial.tokens),
  }
}

function parseTrial(value: string) {
  const [completed = '0', total = '0'] = value.split('/').map((part) => part.trim())
  return { completed: Number(completed), total: Number(total) }
}

function parseScore(value: string): ScoreDto | null {
  if (value === '-') return null
  if (value.endsWith('%')) return { kind: 'percentage', value: parseNumber(value) }
  const [score, maximum] = value.split('/')
  return { kind: 'points', value: Number(score), maximum: Number(maximum) }
}

function parseDuration(value: string): number {
  if (value.endsWith('h')) return Number(value.slice(0, -1)) * 3600
  if (value.endsWith('m')) return Number(value.slice(0, -1)) * 60
  if (value.endsWith('s')) return Number(value.slice(0, -1))
  const [hours, minutes, seconds] = value.split(':').map(Number)
  return hours * 3600 + minutes * 60 + seconds
}

function parseTokenUsageM(value: string): number {
  if (value === '-') return 0
  if (value.endsWith('M')) return parseNumber(value)
  if (value.endsWith('k')) return parseNumber(value) / 1_000
  return parseNumber(value) / 1_000_000
}

function parseNumber(value: string): number {
  return Number(value.replace(/[^0-9.]/g, ''))
}

function parseSizeBytes(value: string | undefined): number | undefined {
  if (!value) return undefined
  const number = parseNumber(value)
  if (value.includes('GB')) return number * 1024 ** 3
  if (value.includes('MB')) return number * 1024 ** 2
  return number
}
