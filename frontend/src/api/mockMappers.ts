import { events } from '../mocks/demo'
import { datasetRows } from '../mocks/demoCatalog'
import type { AgentRow, DatasetRow, EnvironmentRow, HarborJob, LeaderboardRow, SystemRow, TaskRow, TrialRow } from '../domain/harbor'
import type { AgentDto, DatasetDto, DatasetTaskDto, EnvironmentDto, JobDto, JobEventDto, LeaderboardDatasetDto, LeaderboardEntryDto, ScoreDto, SystemComponentDto, TrialDto } from './contract'

export function toJobDto(job: HarborJob): JobDto {
  return {
    id: job.id, name: job.name, status: job.status, datasetRef: job.dataset, agentName: job.agent, harness: job.agent,
    model: job.model, environmentName: job.environment, trial: parseTrial(job.trials), score: parseScore(job.score),
    costUsd: parseNumber(job.cost), tokenUsageM: parseNumber(job.tokenUsage), runtimeSeconds: parseDuration(job.runtimeDuration),
    createdAt: job.createdAt, includeInLeaderboard: job.includeInLeaderboard, jobDir: job.jobDir, eventLogPath: job.eventLogPath,
    artifactPaths: job.artifactPaths, split: job.split, failureCode: job.failureCode,
  }
}

export function toAgentDto(agent: AgentRow): AgentDto {
  return {
    agentName: agent.agentName, allowedHosts: splitList(agent.allowedHosts), apiKeyEnv: agent.apiKeyEnv, baseUrlEnv: agent.baseUrlEnv,
    contextLength: parseOptionalNumber(agent.contextLength), env: parseKeyValues(agent.env), harness: agent.harness, id: agent.id,
    importPath: optional(agent.adapter), kwargs: agent.kwargs ?? '', maxTimeoutSeconds: parseOptionalDuration(agent.maxTimeout),
    mcpServers: parseMcpServers(agent.mcp), models: splitList(agent.models), reasoningEfforts: splitList(agent.reasoningEffort),
    reasoningSummary: optional(agent.reasoningSummary), runtime: optional(agent.runtime), setupTimeoutSeconds: parseOptionalDuration(agent.setupTimeout),
    skillSources: splitList(agent.skills), status: agent.status, supportedModels: splitList(agent.compatibleModels),
    temperature: parseOptionalNumber(agent.temperature), type: agent.type, updatedAt: optional(agent.updated),
  }
}

export function toDatasetDto(dataset: DatasetRow): DatasetDto {
  return {
    ref: `${dataset.name}@${dataset.version}`, name: dataset.name, version: dataset.version, visibility: dataset.visibility,
    taskCount: dataset.tasks, source: dataset.source,
    download: { status: dataset.downloadStatus, path: dataset.downloadPath ?? dataset.path ?? dataset.downloadDir, sizeBytes: parseSizeBytes(dataset.size) },
    registryUrl: dataset.registryUrl, splits: dataset.splits ?? [],
  }
}

export function toEnvironmentDto(environment: EnvironmentRow): EnvironmentDto {
  return {
    allowedHosts: splitList(environment.allowedHosts), cpuPolicy: environment.cpuPolicy, cpus: environment.cpus,
    deleteAfterRun: environment.deleteAfterRun, dockerComposePaths: splitList(environment.dockerCompose), dockerImage: environment.dockerImage,
    env: parseKeyValues(environment.env), environmentType: environment.environmentType, extraAllowedHosts: splitList(environment.extraAllowedHosts),
    forceBuild: environment.forceBuild, gpus: environment.gpus, gpuTypes: environment.gpuTypes, healthcheck: environment.healthcheck,
    id: environment.id, importPath: optional(environment.importPath), kwargs: environment.kwargs, memoryMb: environment.memoryMb,
    memoryPolicy: environment.memoryPolicy, mounts: environment.mounts, name: environment.name, networkMode: environment.networkMode, os: environment.os,
    overrideCpus: environment.overrideCpus, overrideGpus: environment.overrideGpus, overrideMemoryMb: environment.overrideMemoryMb,
    overrideStorageMb: environment.overrideStorageMb, overrideTpu: environment.overrideTpu, profileType: environment.profileType,
    storageMb: environment.storageMb, tpu: environment.tpu, workdir: environment.workdir,
  }
}

export function toDatasetTaskDto(task: TaskRow): DatasetTaskDto {
  const dataset = datasetRows.find((row) => row.name === task.dataset)
  return { datasetRef: dataset ? `${dataset.name}@${dataset.version}` : task.dataset, description: task.description, name: task.name, splits: task.splits ?? [] }
}

export function toJobEventDto(event: typeof events[number]): JobEventDto {
  return { level: event.level, message: event.message, occurredAt: event.time }
}

export function toTrialDto(trial: TrialRow): TrialDto {
  return {
    costUsd: parseNumber(trial.cost), id: trial.id, jobId: trial.jobId, logPath: trial.logPath, retryCount: trial.retries,
    runtimeSeconds: parseDuration(trial.duration), score: parseScore(trial.score), status: trial.result as TrialDto['status'],
    taskName: trial.task, tokenUsageM: parseTokenUsageM(trial.tokens),
  }
}

export function toLeaderboardEntryDto(row: LeaderboardRow): LeaderboardEntryDto {
  return {
    agentName: row.agentName, comparabilityKey: row.comparabilityKey, costUsd: parseNumber(row.cost), datasetRef: row.dataset,
    harness: row.harness, jobId: row.jobId, metric: row.metric, model: row.model, rank: row.rank, reportPath: optional(row.reportPath),
    runtimeSeconds: parseDuration(row.duration), score: parseScore(row.score), split: row.split, submittedAt: row.submitted,
    tokenUsageM: parseTokenUsageM(row.tokens), trial: parseTrial(row.trials),
  }
}

export function buildLeaderboardDatasets(entries: LeaderboardEntryDto[]): LeaderboardDatasetDto[] {
  return [...new Set(entries.map((entry) => entry.datasetRef))].map((ref) => {
    const [name, version = ''] = ref.split('@')
    return { name, ref, version }
  })
}

export function toSystemComponentDto(row: SystemRow): SystemComponentDto {
  return { actions: systemActionsFor(row.kind), component: row.component, kind: row.kind, path: row.path, status: row.status, value: row.value }
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
  if (!value || value === '-') return 0
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

function parseOptionalNumber(value: string | undefined): number | undefined {
  if (!value || value === 'none') return undefined
  const parsed = parseNumber(value)
  return Number.isFinite(parsed) ? parsed : undefined
}

function parseOptionalDuration(value: string | undefined): number | undefined {
  if (!value || value === 'none') return undefined
  return parseDuration(value)
}

function parseKeyValues(value: string | undefined) {
  if (!value || value === 'none') return []
  return value.split('\n').map((line) => {
    const [key, ...rest] = line.split('=')
    return { key: key.trim(), value: rest.join('=').trim() }
  }).filter((entry) => entry.key)
}

function parseMcpServers(value: string | undefined) {
  if (!value || value === 'none') return []
  try {
    const parsed = JSON.parse(value)
    return Array.isArray(parsed) ? parsed : []
  } catch {
    return []
  }
}

function splitList(value: string | undefined): string[] {
  if (!value || value === 'none') return []
  return value.split(/\n|,/).map((item) => item.trim()).filter(Boolean)
}

function optional(value: string | undefined): string | undefined {
  return value && value !== 'none' ? value : undefined
}

function systemActionsFor(kind: SystemRow['kind']): SystemComponentDto['actions'] {
  if (kind === 'ornnlab-service') return ['check-update', 'restart-service']
  if (kind === 'docker') return ['clean-docker-cache']
  if (kind === 'storage') return ['clean-storage-cache']
  return []
}

function parseSizeBytes(value: string | undefined): number | undefined {
  if (!value) return undefined
  const number = parseNumber(value)
  if (value.includes('GB')) return number * 1024 ** 3
  if (value.includes('MB')) return number * 1024 ** 2
  return number
}
