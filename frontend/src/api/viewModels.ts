import type { AgentRow, DatasetRow, DatasetTask, EnvironmentRow, EventLog, HarborJob, HarnessTemplate, LeaderboardDataset, LeaderboardRow, SystemRow, TrialRow } from '../domain/harbor'
import { fallbackAgentCapabilities } from '../domain/agentCapabilities'
import type {
  AgentDto,
  DatasetDto,
  DatasetTaskDto,
  EnvironmentDto,
  HarnessDto,
  JobDto,
  JobEventDto,
  LeaderboardEntryDto,
  LeaderboardDatasetDto,
  ScoreDto,
  SystemComponentDto,
  TrialDto,
} from './contract'

export function jobDtoToHarborJob(job: JobDto): HarborJob {
  return {
    id: job.id,
    name: job.name,
    status: job.status,
    dataset: job.datasetRef,
    agent: job.agentName,
    model: job.model,
    environment: job.environmentName,
    trials: `${job.trial.completed} / ${job.trial.total}`,
    score: formatScore(job.score),
    cost: formatCost(job.costUsd),
    tokens: formatTokenUsage(job.tokenUsageM),
    tokenUsage: formatTokenUsage(job.tokenUsageM),
    runtimeDuration: formatDuration(job.runtimeSeconds),
    createdAt: formatDateTime(job.createdAt),
    includeInLeaderboard: job.includeInLeaderboard,
    canResume: job.canResume,
    jobDir: job.jobDir,
    eventLogPath: job.eventLogPath,
    artifactPaths: job.artifactPaths,
    failureCode: job.failureCode,
  }
}

export function datasetDtoToRow(dataset: DatasetDto): DatasetRow {
  return {
    name: dataset.name,
    version: dataset.version,
    visibility: dataset.visibility,
    tasks: dataset.taskCount,
    source: dataset.source,
    downloadStatus: dataset.download.status,
    downloadPath: dataset.download.path,
    downloadedAt: dataset.download.updatedAt,
    storageKind: dataset.download.storageKind,
    size: dataset.download.sizeBytes === undefined ? undefined : formatBytes(dataset.download.sizeBytes),
    registryUrl: dataset.registryUrl,
    path: dataset.download.path,
    ref: dataset.ref,
  }
}

export function datasetTaskDtoToDatasetTask(task: DatasetTaskDto): DatasetTask {
  return { datasetRef: task.datasetRef, description: task.description, environment: task.environment, name: task.name }
}

export function agentDtoToRow(agent: AgentDto): AgentRow {
  const importPath = agent.importPath ?? 'none'
  return {
    adapter: importPath,
    agentName: agent.agentName,
    authenticationMode: agent.authenticationMode,
    capabilities: agent.capabilities ?? fallbackAgentCapabilities(),
    env: formatKeyValues(agent.env),
    harness: agent.harness,
    id: agent.id,
    kwargs: agent.kwargs,
    maxTimeout: agent.maxTimeoutSeconds === undefined ? undefined : `${agent.maxTimeoutSeconds}s`,
    mcp: agent.mcpServers.length ? JSON.stringify(agent.mcpServers) : 'none',
    models: agent.models.length ? agent.models.join(', ') : '-',
    setupTimeout: agent.setupTimeoutSeconds === undefined ? undefined : `${agent.setupTimeoutSeconds}s`,
    timeout: agent.timeoutSeconds === undefined ? undefined : `${agent.timeoutSeconds}s`,
    skills: agent.skillSources.length ? agent.skillSources.join('\n') : 'none',
    source: importPath,
    status: agent.status,
    updated: '',
  }
}

export function harnessDtoToTemplate(harness: HarnessDto): HarnessTemplate {
  return harness
}

export function environmentDtoToRow(environment: EnvironmentDto): EnvironmentRow {
  return {
    allowedHosts: formatList(environment.allowedHosts),
    cpuPolicy: environment.cpuPolicy,
    deleteAfterRun: environment.deleteAfterRun,
    dockerCompose: environment.dockerComposePaths.length ? environment.dockerComposePaths.join('\n') : 'none',
    env: formatKeyValues(environment.env),
    environmentType: environment.environmentType,
    forceBuild: environment.forceBuild,
    id: environment.id,
    importPath: environment.importPath ?? 'none',
    kwargs: environment.kwargs,
    memoryPolicy: environment.memoryPolicy,
    mounts: environment.mounts,
    name: environment.name,
    overrideCpus: environment.overrideCpus,
    overrideGpus: environment.overrideGpus,
    overrideMemoryMb: environment.overrideMemoryMb,
    overrideStorageMb: environment.overrideStorageMb,
    overrideTpu: environment.overrideTpu,
    profileType: environment.profileType,
  }
}

export function leaderboardEntryDtoToRow(entry: LeaderboardEntryDto): LeaderboardRow {
  return {
    agentName: entry.agentName,
    comparabilityKey: entry.comparabilityKey,
    cost: formatCost(entry.costUsd),
    dataset: entry.datasetRef,
    duration: formatDuration(entry.runtimeSeconds),
    harness: entry.harness,
    jobId: entry.jobId,
    metric: entry.metric,
    model: entry.model,
    rank: entry.rank,
    reportPath: entry.reportPath ?? '',
    score: formatScore(entry.score),
    submitted: formatDateTime(entry.submittedAt),
    tokens: formatTokenUsage(entry.tokenUsageM),
    trials: `${entry.trial.completed} / ${entry.trial.total}`,
  }
}

export function leaderboardDatasetDtoToRow(dataset: LeaderboardDatasetDto): LeaderboardDataset {
  return { name: dataset.name, ref: dataset.ref, version: dataset.version }
}

export function systemComponentDtoToRow(component: SystemComponentDto): SystemRow {
  return component
}

export function jobEventDtoToEventLog(event: JobEventDto): EventLog {
  return { level: event.level, message: event.message, time: event.occurredAt }
}

export function trialDtoToTrialRow(trial: TrialDto): TrialRow {
  return {
    analysisPath: '',
    artifactPath: '',
    cost: formatCost(trial.costUsd),
    duration: formatDuration(trial.runtimeSeconds),
    id: trial.id,
    jobId: trial.jobId,
    logPath: trial.logPath ?? '',
    progress: trial.status,
    result: trial.status,
    retries: trial.retryCount ?? 0,
    score: formatScore(trial.score),
    task: trial.taskName,
    tokens: formatTokenUsage(trial.tokenUsageM),
    verifierEvidence: '',
  }
}

function formatScore(score: ScoreDto | null): string {
  if (!score) return '-'
  return score.kind === 'percentage' ? `${score.value}%` : `${score.value}/${score.maximum}`
}

function formatTokenUsage(value: number | null): string {
  if (value === null) return '-'
  return `${Number(value.toFixed(4))}M`
}

function formatDuration(totalSeconds: number | null): string {
  if (totalSeconds === null) return '-'
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60
  return [hours, minutes, seconds].map((value) => String(value).padStart(2, '0')).join(':')
}

function formatCost(value: number | null): string {
  return value === null ? '-' : `$${value.toFixed(2)}`
}

function formatDateTime(value: string): string {
  return value.replace('T', ' ').replace(/Z$/, '')
}

function formatKeyValues(values: Array<{ key: string; value: string | null }>): string {
  return values.length
    ? values.map((entry) => entry.value === null ? entry.key : `${entry.key}=${entry.value}`).join('\n')
    : 'none'
}

function formatList(values: string[]): string {
  return values.length ? values.join(', ') : 'none'
}

function formatBytes(bytes: number): string {
  const gigabytes = bytes / 1024 ** 3
  return gigabytes >= 1 ? `${gigabytes.toFixed(1)} GB` : `${(bytes / 1024 ** 2).toFixed(1)} MB`
}
