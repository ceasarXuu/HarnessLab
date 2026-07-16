import type { AgentRow, EnvironmentRow, RunDraft } from '../domain/harbor'
import type { AgentInputDto, CreateJobRequestDto, EnvironmentDto } from './contract'
import { optional, parseKeyValues, parseMcpServers, seconds, splitList } from './formValueParsers'

export function runDraftToCreateJobRequest(draft: RunDraft): CreateJobRequestDto {
  return {
    config: {
      agentSetupTimeoutMultiplier: numberOr(draft.agentSetupTimeoutMultiplier, 1),
      agentName: draft.agent,
      agentTimeoutMultiplier: numberOr(draft.agentTimeoutMultiplier, 1),
      attempts: draft.attempts,
      concurrency: draft.concurrency,
      datasetRef: draft.source,
      debug: draft.debug,
      environmentPresetId: draft.environment,
      environmentBuildTimeoutMultiplier: numberOr(draft.environmentBuildTimeoutMultiplier, 1),
      extraInstructionPaths: splitList(draft.extraInstructions),
      includeInLeaderboard: draft.verifierMode === 'skip' ? false : draft.includeInLeaderboard,
      jobName: draft.jobName,
      jobsDir: draft.jobsDir,
      maxRetries: draft.maxRetries,
      metric: draft.metric,
      modelName: draft.model,
      notes: draft.notes,
      retryExclude: draft.retryExclude,
      retryInclude: draft.retryInclude,
      retryMaxWaitSeconds: numberOr(draft.retryMaxWaitSec, 30),
      retryMinWaitSeconds: numberOr(draft.retryMinWaitSec, 2),
      retryWaitMultiplier: numberOr(draft.retryWaitMultiplier, 1.5),
      selectedTaskNames: draft.selectedTaskNames,
      timeoutMultiplier: draft.timeoutMultiplier,
      verifierTimeoutMultiplier: numberOr(draft.verifierTimeoutMultiplier, 1),
      verifierMode: draft.verifierMode === 'skip' ? 'skip' : 'dataset-default',
    },
    runImmediately: true,
  }
}

export function agentRowToDto(agent: AgentRow): AgentInputDto {
  return {
    agentName: agent.agentName,
    authenticationMode: agent.authenticationMode,
    env: parseKeyValues(agent.env),
    harness: agent.harness,
    id: agent.id,
    importPath: agent.harness === 'custom-harness' ? optional(agent.adapter) : undefined,
    kwargs: agent.kwargs ?? '',
    maxTimeoutSeconds: seconds(agent.maxTimeout),
    mcpServers: parseMcpServers(agent.mcp),
    models: splitList(agent.models),
    setupTimeoutSeconds: seconds(agent.setupTimeout),
    timeoutSeconds: seconds(agent.timeout),
    skillSources: splitList(agent.skills),
    type: agent.type,
  }
}

export function environmentRowToDto(environment: EnvironmentRow): EnvironmentDto {
  return {
    allowedHosts: splitList(environment.allowedHosts),
    cpuPolicy: environment.cpuPolicy,
    deleteAfterRun: environment.deleteAfterRun,
    dockerComposePaths: splitList(environment.dockerCompose),
    env: parseKeyValues(environment.env),
    environmentType: environment.environmentType,
    forceBuild: environment.forceBuild,
    id: environment.id,
    importPath: optional(environment.importPath),
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

function numberOr(value: string, fallback: number): number {
  const parsed = Number(value)
  return Number.isFinite(parsed) ? parsed : fallback
}
