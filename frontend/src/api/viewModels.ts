import type { AgentRow, DatasetRow, DatasetTask, EnvironmentRow, EventLog, HarborJob, LeaderboardDataset, LeaderboardRow, SystemRow, TrialRow } from '../domain/harbor'
import type {
  AgentDto,
  DatasetDto,
  DatasetTaskDto,
  EnvironmentDto,
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
    cost: `$${job.costUsd.toFixed(2)}`,
    tokens: formatTokenUsage(job.tokenUsageM),
    tokenUsage: formatTokenUsage(job.tokenUsageM),
    runtimeDuration: formatDuration(job.runtimeSeconds),
    createdAt: formatDateTime(job.createdAt),
    includeInLeaderboard: job.includeInLeaderboard,
    jobDir: job.jobDir,
    eventLogPath: job.eventLogPath,
    artifactPaths: job.artifactPaths,
    split: job.split,
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
    size: dataset.download.sizeBytes === undefined ? undefined : formatBytes(dataset.download.sizeBytes),
    registryUrl: dataset.registryUrl,
    path: dataset.download.path,
    ref: dataset.ref,
    splits: dataset.splits,
  }
}

export function datasetTaskDtoToDatasetTask(task: DatasetTaskDto): DatasetTask {
  return { datasetRef: task.datasetRef, description: task.description, name: task.name, splits: task.splits }
}

export function agentDtoToRow(agent: AgentDto): AgentRow {
  const importPath = agent.importPath ?? 'none'
  return {
    adapter: importPath,
    agentName: agent.agentName,
    allowedHosts: formatList(agent.allowedHosts),
    apiKeyEnv: agent.apiKeyEnv,
    baseUrlEnv: agent.baseUrlEnv,
    compatibleModels: formatList(agent.supportedModels),
    contextLength: agent.contextLength === undefined ? undefined : String(agent.contextLength),
    env: formatKeyValues(agent.env),
    harness: agent.harness,
    id: agent.id,
    kwargs: agent.kwargs,
    maxTimeout: agent.maxTimeoutSeconds === undefined ? undefined : `${agent.maxTimeoutSeconds}s`,
    mcp: agent.mcpServers.length ? JSON.stringify(agent.mcpServers) : 'none',
    models: formatList(agent.models),
    reasoningEffort: formatList(agent.reasoningEfforts),
    reasoningSummary: agent.reasoningSummary,
    runtime: agent.runtime,
    setupTimeout: agent.setupTimeoutSeconds === undefined ? undefined : `${agent.setupTimeoutSeconds}s`,
    skills: agent.skillSources.length ? agent.skillSources.join('\n') : 'none',
    source: agent.type === 'built-in' ? 'Harbor built-in' : importPath,
    status: agent.status,
    temperature: agent.temperature === undefined ? undefined : String(agent.temperature),
    type: agent.type,
    updated: agent.updatedAt ?? '',
  }
}

export function environmentDtoToRow(environment: EnvironmentDto): EnvironmentRow {
  return {
    allowedHosts: formatList(environment.allowedHosts),
    cpuPolicy: environment.cpuPolicy,
    cpus: environment.cpus,
    deleteAfterRun: environment.deleteAfterRun,
    dockerCompose: environment.dockerComposePaths.length ? environment.dockerComposePaths.join('\n') : 'none',
    dockerImage: environment.dockerImage,
    env: formatKeyValues(environment.env),
    environmentType: environment.environmentType,
    extraAllowedHosts: formatList(environment.extraAllowedHosts),
    forceBuild: environment.forceBuild,
    gpus: environment.gpus,
    gpuTypes: environment.gpuTypes,
    healthcheck: environment.healthcheck,
    id: environment.id,
    importPath: environment.importPath ?? 'none',
    kwargs: environment.kwargs,
    memoryMb: environment.memoryMb,
    memoryPolicy: environment.memoryPolicy,
    mounts: environment.mounts,
    name: environment.name,
    networkMode: environment.networkMode,
    os: environment.os,
    overrideCpus: environment.overrideCpus,
    overrideGpus: environment.overrideGpus,
    overrideMemoryMb: environment.overrideMemoryMb,
    overrideStorageMb: environment.overrideStorageMb,
    overrideTpu: environment.overrideTpu,
    profileType: environment.profileType,
    skillsDir: 'none',
    storageMb: environment.storageMb,
    tpu: environment.tpu,
    workdir: environment.workdir,
  }
}

export function leaderboardEntryDtoToRow(entry: LeaderboardEntryDto): LeaderboardRow {
  return {
    agentName: entry.agentName,
    comparabilityKey: entry.comparabilityKey,
    cost: `$${entry.costUsd.toFixed(2)}`,
    dataset: entry.datasetRef,
    duration: formatDuration(entry.runtimeSeconds),
    harness: entry.harness,
    jobId: entry.jobId,
    metric: entry.metric,
    model: entry.model,
    rank: entry.rank,
    reportPath: entry.reportPath ?? '',
    score: formatScore(entry.score),
    split: entry.split,
    submitted: formatDateTime(entry.submittedAt),
    tokens: formatTokenUsage(entry.tokenUsageM),
    trials: `${entry.trial.completed} / ${entry.trial.total}`,
  }
}

export function leaderboardDatasetDtoToRow(dataset: LeaderboardDatasetDto): LeaderboardDataset {
  return { name: dataset.name, ref: dataset.ref, version: dataset.version }
}

export function systemComponentDtoToRow(component: SystemComponentDto): SystemRow {
  return {
    component: component.component,
    kind: component.kind,
    path: component.path,
    status: component.status,
    value: component.value,
  }
}

export function jobEventDtoToEventLog(event: JobEventDto): EventLog {
  return { level: event.level, message: event.message, time: event.occurredAt }
}

export function trialDtoToTrialRow(trial: TrialDto): TrialRow {
  return {
    analysisPath: '',
    artifactPath: '',
    cost: `$${trial.costUsd.toFixed(2)}`,
    duration: formatDuration(trial.runtimeSeconds),
    id: trial.id,
    jobId: trial.jobId,
    logPath: trial.logPath,
    progress: trial.status,
    result: trial.status,
    retries: trial.retryCount,
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

function formatTokenUsage(value: number): string {
  return `${Number(value.toFixed(4))}M`
}

function formatDuration(totalSeconds: number): string {
  const hours = Math.floor(totalSeconds / 3600)
  const minutes = Math.floor((totalSeconds % 3600) / 60)
  const seconds = totalSeconds % 60
  return [hours, minutes, seconds].map((value) => String(value).padStart(2, '0')).join(':')
}

function formatDateTime(value: string): string {
  return value.replace('T', ' ').replace(/Z$/, '')
}

function formatKeyValues(values: Array<{ key: string; value: string }>): string {
  return values.length ? values.map((entry) => `${entry.key}=${entry.value}`).join('\n') : 'none'
}

function formatList(values: string[]): string {
  return values.length ? values.join(', ') : 'none'
}

function formatBytes(bytes: number): string {
  const gigabytes = bytes / 1024 ** 3
  return gigabytes >= 1 ? `${gigabytes.toFixed(1)} GB` : `${(bytes / 1024 ** 2).toFixed(1)} MB`
}
