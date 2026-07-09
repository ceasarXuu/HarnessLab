import type { DatasetRow, HarborJob } from '../domain/harbor'
import type { DatasetDto, JobDto, ScoreDto } from './contract'

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

function formatBytes(bytes: number): string {
  const gigabytes = bytes / 1024 ** 3
  return gigabytes >= 1 ? `${gigabytes.toFixed(1)} GB` : `${(bytes / 1024 ** 2).toFixed(1)} MB`
}
