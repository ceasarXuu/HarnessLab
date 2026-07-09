import type { AgentRow, EnvironmentRow, RunDraft } from '../domain/harbor'
import type { AgentDto, CreateJobRequestDto, EnvironmentDto, KeyValueDto, McpServerDto } from './contract'

export function runDraftToCreateJobRequest(draft: RunDraft): CreateJobRequestDto {
  return {
    config: {
      agentEnv: parseKeyValues(draft.agentEnv),
      agentImportPath: optional(draft.agentImportPath),
      agentKwargs: draft.agentKwargs,
      agentName: draft.agent,
      attempts: draft.attempts,
      concurrency: draft.concurrency,
      datasetRef: draft.source,
      debug: draft.debug,
      environmentPresetId: draft.environment,
      includeInLeaderboard: draft.verifierMode === 'skip' ? false : draft.includeInLeaderboard,
      jobName: draft.jobName,
      jobsDir: draft.jobsDir,
      maxRetries: draft.maxRetries,
      metric: draft.metric,
      model: draft.model,
      notes: draft.notes,
      retryExclude: draft.retryExclude,
      retryInclude: draft.retryInclude,
      retryIntervalPolicy: draft.retryIntervalPolicy,
      retryMaxWaitSeconds: numberOr(draft.retryMaxWaitSec, 30),
      retryMinWaitSeconds: numberOr(draft.retryMinWaitSec, 2),
      retryWaitMultiplier: numberOr(draft.retryWaitMultiplier, 1.5),
      selectedTaskNames: draft.selectedTaskNames,
      split: draft.split,
      timeoutMultiplier: draft.timeoutMultiplier,
      timeoutPolicy: draft.timeoutPolicy,
      verifierMode: draft.verifierMode,
    },
    runImmediately: true,
  }
}

export function agentRowToDto(agent: AgentRow): AgentDto {
  return {
    agentName: agent.agentName,
    allowedHosts: splitList(agent.allowedHosts),
    apiKeyEnv: optional(agent.apiKeyEnv),
    baseUrlEnv: optional(agent.baseUrlEnv),
    contextLength: optionalNumber(agent.contextLength),
    env: parseKeyValues(agent.env),
    harness: agent.harness,
    id: agent.id,
    importPath: optional(agent.adapter),
    kwargs: agent.kwargs ?? '',
    maxTimeoutSeconds: seconds(agent.maxTimeout),
    mcpServers: parseMcpServers(agent.mcp),
    models: splitList(agent.models),
    reasoningEfforts: splitList(agent.reasoningEffort),
    reasoningSummary: optional(agent.reasoningSummary),
    runtime: optional(agent.runtime),
    setupTimeoutSeconds: seconds(agent.setupTimeout),
    skillSources: splitList(agent.skills),
    status: agent.status,
    supportedModels: splitList(agent.compatibleModels),
    temperature: optionalNumber(agent.temperature),
    type: agent.type,
    updatedAt: optional(agent.updated),
  }
}

export function environmentRowToDto(environment: EnvironmentRow): EnvironmentDto {
  return {
    allowedHosts: splitList(environment.allowedHosts),
    cpuPolicy: environment.cpuPolicy,
    cpus: environment.cpus,
    deleteAfterRun: environment.deleteAfterRun,
    dockerComposePaths: splitList(environment.dockerCompose),
    dockerImage: environment.dockerImage,
    env: parseKeyValues(environment.env),
    environmentType: environment.environmentType,
    extraAllowedHosts: splitList(environment.extraAllowedHosts),
    forceBuild: environment.forceBuild,
    gpus: environment.gpus,
    gpuTypes: environment.gpuTypes,
    healthcheck: environment.healthcheck,
    id: environment.id,
    importPath: optional(environment.importPath),
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
    storageMb: environment.storageMb,
    tpu: environment.tpu,
    workdir: environment.workdir,
  }
}

function parseKeyValues(value: string | undefined): KeyValueDto[] {
  if (!value || value === 'none') return []
  return value.split('\n').map((line) => {
    const [key, ...rest] = line.split('=')
    return { key: key.trim(), value: rest.join('=').trim() }
  }).filter((entry) => entry.key)
}

function parseMcpServers(value: string | undefined): McpServerDto[] {
  if (!value || value === 'none') return []
  try {
    const parsed = JSON.parse(value)
    return Array.isArray(parsed) ? parsed as McpServerDto[] : []
  } catch {
    return []
  }
}

function splitList(value: string | undefined): string[] {
  if (!value || value === 'none') return []
  return value.split(/\n|,/).map((item) => item.trim()).filter(Boolean)
}

function seconds(value: string | undefined): number | undefined {
  if (!value || value === 'none') return undefined
  return numberOr(value.replace(/s$/, ''), 0)
}

function optionalNumber(value: string | undefined): number | undefined {
  if (!value || value === 'none') return undefined
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : undefined
}

function numberOr(value: string, fallback: number): number {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : fallback
}

function optional(value: string | undefined): string | undefined {
  return value && value !== 'none' ? value : undefined
}
