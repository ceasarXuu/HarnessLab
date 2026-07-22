import { events, jobs, trialRows } from '../mocks/demo'
import { agentRows, datasetRows, environmentRows, harnessTemplates, taskRows } from '../mocks/demoCatalog'
import { leaderboardRows, systemRows } from '../mocks/demoSystem'
import { fallbackAgentCapabilities } from '../domain/agentCapabilities'
import type {
  AgentDto,
  ApiError,
  ApiResponse,
  DatasetDto,
  CreateJobRequestDto,
  CreateJobResponseDto,
  JobDto,
  OperationResultDto,
  UpdateCheckResultDto,
  UpdateJobLeaderboardResponseDto,
} from './contract'
import { buildLeaderboardDatasets, toAgentDto, toDatasetDto, toDatasetTaskDto, toEnvironmentDto, toJobDto, toJobEventDto, toLeaderboardEntryDto, toSystemComponentDto, toTrialDto } from './mockMappers'
import { createMockOperationStore } from './mockOperations'
import { filterAgents, filterByQuery, filterDatasetTasks, filterEnvironments, page } from './mockQueries'
import type { WebUiClient } from './webUiClient'

const requestMeta = { requestId: 'mock-webui-client' }

export function createMockWebUiClient(): WebUiClient {
  let jobDtos = jobs.map(toJobDto)
  let agentDtos = agentRows.map(toAgentDto)
  let datasetDtos = datasetRows.map(toDatasetDto)
  let environmentDtos = environmentRows.map(toEnvironmentDto)
  let lastDatasetParent = '~/Datasets'
  const taskDtos = taskRows.map(toDatasetTaskDto)
  let eventDtos = events.map((event) => ({ event: toJobEventDto(event), jobId: event.jobId }))
  let leaderboardDtos = leaderboardRows.map(toLeaderboardEntryDto)
  let systemDtos = systemRows.map(toSystemComponentDto)
  let trialDtos = trialRows.map(toTrialDto)
  const operations = createMockOperationStore()
  const operationEffects = new Map<string, { onCompleted?: () => void; onRunning?: () => void }>()

  return {
    async cancelJob(id) {
      const job = jobDtos.find((item) => item.id === id)
      if (!job) return failure('JOB_NOT_FOUND', 'Job not found')
      if (isTerminalJob(job.status)) return failure('OPERATION_CONFLICT', 'Job is already terminal')
      jobDtos = jobDtos.map((job) => (job.id === id ? { ...job, status: 'cancelled' } : job))
      return operationResult(operations.complete('cancel-job', 'job', id, 'Job cancelled'))
    },
    async cancelDatasetDownload(ref) {
      const operation = operations.cancelActive('download-dataset', ref)
      if (!operation) return failure('INVALID_REQUEST', 'No active dataset download')
      operationEffects.delete(operation.id)
      datasetDtos = datasetDtos.map((item) => item.ref === ref
        ? { ...item, download: { status: 'not-downloaded' } }
        : item)
      return operationResult(operation)
    },
    async cancelOperation(id) {
      const operation = operations.cancel(id)
      operationEffects.delete(id)
      return operation ? operationResult(operation) : failure('OPERATION_NOT_CANCELLABLE', 'Operation cannot be cancelled')
    },
    async checkForSystemUpdate() {
      return success<UpdateCheckResultDto>({ currentVersion: '0.1.3', latestVersion: '0.1.3', updateAvailable: false })
    },
    async chooseDirectory() {
      return failure('NATIVE_DIRECTORY_PICKER_UNAVAILABLE', 'Native directory selection is available only in API mode.')
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
      return operationResult(operations.complete('copy-environment', 'environment', copy.id, 'Environment copied'))
    },
    async createAgent(agent) {
      agentDtos = [withMockAgentCapabilities({ ...agent, status: 'configured' }), ...agentDtos]
      return operationResult(operations.complete('create-agent', 'agent', agent.id, 'Agent created'))
    },
    async createEnvironment(environment) {
      environmentDtos = [environment, ...environmentDtos]
      return operationResult(operations.complete('create-environment', 'environment', environment.id, 'Environment created'))
    },
    async createJob(request) {
      const agent = agentDtos.find((item) => item.agentName === request.config.agentName)
      if (!agent) return failure('AGENT_NOT_FOUND', 'Agent not found')
      if (!agent.models.includes(request.config.modelName)) {
        return failure('INVALID_AGENT_MODEL', 'Selected model is not configured for this Agent')
      }
      const job = buildQueuedJob(jobDtos, request, agent)
      jobDtos = [job, ...jobDtos]
      const operation = operations.complete(
        request.runImmediately ? 'run-job' : 'create-job',
        'job',
        job.id,
        request.runImmediately ? 'Job queued' : 'Job created',
      )
      return success<CreateJobResponseDto>({ job, operation })
    },
    async deleteAgent(id) {
      const target = agentDtos.find((agent) => agent.id === id)
      if (!target) return failure('AGENT_NOT_FOUND', 'Agent not found')
      agentDtos = agentDtos.filter((agent) => agent.id !== id)
      return operationResult(operations.complete('delete-agent', 'agent', id, 'Agent deleted'))
    },
    async deleteEnvironment(id) {
      const target = environmentDtos.find((environment) => environment.id === id)
      if (!target) return failure('ENVIRONMENT_NOT_FOUND', 'Environment not found')
      if (target.profileType === 'built-in') return failure('ENVIRONMENT_BUILT_IN_IMMUTABLE', 'Built-in environments cannot be deleted')
      environmentDtos = environmentDtos.filter((environment) => environment.id !== id)
      return operationResult(operations.complete('delete-environment', 'environment', id, 'Environment deleted'))
    },
    async deleteLocalDataset(ref) {
      const dataset = datasetDtos.find((item) => item.ref === ref)
      if (!dataset) return failure('DATASET_NOT_FOUND', 'Dataset not found')
      if (dataset.download.storageKind !== 'managed') {
        return failure('DATASET_EXTERNAL_IMMUTABLE', 'External Dataset files cannot be deleted by OrnnLab')
      }
      datasetDtos = datasetDtos.map((item) => (item.ref === ref ? { ...item, download: { status: 'not-downloaded' } } : item))
      return operationResult(operations.complete('delete-local-dataset', 'dataset', ref, 'Local dataset removed'))
    },
    async deleteJob(id) {
      const target = jobDtos.find((job) => job.id === id)
      if (!target) return failure('JOB_NOT_FOUND', 'Job not found')
      if (!isTerminalJob(target.status)) {
        return failure('OPERATION_CONFLICT', 'Running or queued Jobs must be cancelled before deletion')
      }
      jobDtos = jobDtos.filter((job) => job.id !== id)
      eventDtos = eventDtos.filter((item) => item.jobId !== id)
      trialDtos = trialDtos.filter((item) => item.jobId !== id)
      leaderboardDtos = leaderboardDtos.filter((item) => item.jobId !== id)
      return success({ deletedJobId: id })
    },
    async downloadDataset(ref, request) {
      const dataset = datasetDtos.find((item) => item.ref === ref)
      if (!dataset) return failure('DATASET_NOT_FOUND', 'Dataset not found')
      const parentPath = request.parentPath.trim()
      if (!parentPath) return failure('INVALID_REQUEST', 'Dataset parent directory is required')
      lastDatasetParent = parentPath
      const operation = submitOperation('download-dataset', 'dataset', ref, {
        onCompleted: () => {
          datasetDtos = datasetDtos.map((item) => item.ref === ref ? {
            ...item,
            download: {
              path: `${parentPath}/${managedDatasetDirectory(ref)}`,
              sizeBytes: item.download.sizeBytes,
              status: 'downloaded',
              storageKind: 'managed',
              updatedAt: new Date().toISOString(),
            },
          } : item)
        },
      })
      datasetDtos = datasetDtos.map((item) => item.ref === ref
        ? { ...item, download: { status: 'downloading', progress: operation.progress ?? 0 } }
        : item)
      return operationResult(operation)
    },
    async getAgent(id) {
      return findById(agentDtos, id, 'AGENT_NOT_FOUND', 'Agent not found', 'id')
    },
    async getDataset(ref) {
      return findById(datasetDtos, ref, 'DATASET_NOT_FOUND', 'Dataset not found', 'ref')
    },
    async getDatasetDefaultParent() {
      return success({ parentPath: lastDatasetParent })
    },
    async getEnvironment(id) {
      return findById(environmentDtos, id, 'ENVIRONMENT_NOT_FOUND', 'Environment not found', 'id')
    },
    async getHubConnection() {
      return success({ status: 'connected' as const })
    },
    async getJobCopyConfig(id) {
      const job = jobDtos.find((item) => item.id === id)
      if (!job) return failure('JOB_NOT_FOUND', 'Job not found')
      return success(copyConfigFromJob(job))
    },
    async getJob(id) {
      return findById(jobDtos, id, 'JOB_NOT_FOUND', 'Job not found', 'id')
    },
    async getModelPricing(modelName) {
      return success({
        catalogModelName: modelName,
        inputCacheHitUsdPerMillion: 0.15,
        inputCacheMissUsdPerMillion: 1.5,
        modelName,
        outputUsdPerMillion: 6,
        source: 'litellm' as const,
        sourceUrl: 'https://example.test/pricing',
      })
    },
    async getOperation(id) {
      const operation = operations.get(id)
      if (operation) applyOperationEffects(operation)
      return operation ? success(operation) : failure('OPERATION_NOT_FOUND', 'Operation not found')
    },
    async importDataset(request) {
      const ref = `${request.name}@${request.version}`
      const dataset: DatasetDto = {
        download: { path: request.path, status: 'downloaded', storageKind: 'external', updatedAt: new Date().toISOString() },
        name: request.name,
        ref,
        source: 'local import',
        taskCount: request.taskCount,
        version: request.version,
        visibility: 'private',
      }
      return operationResult(submitOperation('import-dataset', 'dataset', ref, {
        onCompleted: () => { datasetDtos = [dataset, ...datasetDtos] },
      }))
    },
    async installSystemUpdate() {
      return operationResult(operations.submit('install-system-update', 'system', 'ornnlab-service'))
    },
    async moveDataset(ref, request) {
      const dataset = datasetDtos.find((item) => item.ref === ref)
      if (!dataset) return failure('DATASET_NOT_FOUND', 'Dataset not found')
      if (dataset.download.storageKind !== 'managed' || dataset.download.status !== 'downloaded') {
        return failure('DATASET_NOT_MOVABLE', 'Only downloaded managed Datasets can be moved')
      }
      const parentPath = request.parentPath.trim()
      if (!parentPath) return failure('INVALID_REQUEST', 'Dataset parent directory is required')
      lastDatasetParent = parentPath
      return operationResult(submitOperation('move-dataset', 'dataset', ref, {
        onCompleted: () => {
          datasetDtos = datasetDtos.map((item) => item.ref === ref ? {
            ...item,
            download: { ...item.download, path: `${parentPath}/${managedDatasetDirectory(ref)}`, updatedAt: new Date().toISOString() },
          } : item)
        },
      }))
    },
    async relocateDataset(ref, request) {
      const dataset = datasetDtos.find((item) => item.ref === ref)
      if (!dataset) return failure('DATASET_NOT_FOUND', 'Dataset not found')
      const path = request.path.trim()
      if (!path) return failure('INVALID_REQUEST', 'Dataset path is required')
      datasetDtos = datasetDtos.map((item) => item.ref === ref ? {
        ...item,
        download: { ...item.download, path, status: 'downloaded', updatedAt: new Date().toISOString() },
      } : item)
      return operationResult(operations.complete('relocate-dataset', 'dataset', ref, 'Dataset path updated'))
    },
    async removeDatasetRegistration(ref) {
      const dataset = datasetDtos.find((item) => item.ref === ref)
      if (!dataset) return failure('DATASET_NOT_FOUND', 'Dataset not found')
      if (dataset.download.storageKind === 'managed' && dataset.download.status === 'downloaded') {
        return failure('DATASET_MANAGED_REGISTRATION_REQUIRED', 'Managed Dataset must be deleted before removing its registration')
      }
      datasetDtos = dataset.source === 'local import'
        ? datasetDtos.filter((item) => item.ref !== ref)
        : datasetDtos.map((item) => item.ref === ref ? { ...item, download: { status: 'not-downloaded' } } : item)
      return operationResult(operations.complete('remove-dataset-registration', 'dataset', ref, 'Dataset registration removed'))
    },
    async listAgents(query) {
      return success(page(filterAgents(agentDtos, query), query))
    },
    async listHarnesses(query) {
      return success(page(filterByQuery(harnessTemplates, query, (harness) => [harness.name]), query))
    },
    async listDatasetTasks(ref, query) {
      return success(page(filterDatasetTasks(taskDtos, ref, query), query))
    },
    async getDatasetTask(ref, task) {
      return success(taskDtos.find((item) => item.datasetRef === ref && item.name === task) ?? null)
    },
    async listDatasets(query) {
      for (const active of operations.active('download-dataset')) {
        const operation = operations.get(active.id)
        if (operation) applyOperationEffects(operation)
      }
      return success(page(filterByQuery(datasetDtos, query, (dataset) => [dataset.name, dataset.version, dataset.source]), query))
    },
    async listEnvironments(query) {
      return success(page(filterEnvironments(environmentDtos, query), query))
    },
    async listJobEvents(id) {
      return success(eventDtos.filter((entry) => entry.jobId === id).map((entry) => entry.event))
    },
    async listJobTrials(id) {
      return success(trialDtos.filter((trial) => trial.jobId === id))
    },
    async listJobs(query) {
      return success(page(filterByQuery(jobDtos, query, (job) => [job.name, job.datasetRef, job.agentName, job.harness, job.model, job.status]), query))
    },
    async listLeaderboard(query) {
      const byDataset = leaderboardDtos.filter((entry) => entry.datasetRef === query.dataset)
      return success(page(filterByQuery(byDataset, query, (entry) => [entry.agentName, entry.harness, entry.model, entry.metric]), query))
    },
    async listLeaderboardDatasets(query) {
      const leaderboardDatasets = buildLeaderboardDatasets(leaderboardDtos)
      return success(page(filterByQuery(leaderboardDatasets, query, (dataset) => [dataset.ref, dataset.name, dataset.version]), query))
    },
    async listSystemHealth() {
      return success(page(systemDtos))
    },
    async restartSystemService() {
      return operationResult(operations.fail(
        'restart-system-service',
        'system',
        'ornnlab-service',
        'SERVICE_RESTART_UNAVAILABLE',
        'ORNNLAB_RESTART_COMMAND is not configured by the service supervisor',
      ))
    },
    async saveDockerStartCommand(command) {
      systemDtos = systemDtos.map((item) => item.kind === 'docker' ? { ...item, startCommand: command } : item)
      return success({ command })
    },
    async startDocker(command) {
      systemDtos = systemDtos.map((item) => item.kind === 'docker' ? { ...item, startCommand: command } : item)
      return operationResult(submitOperation('start-docker', 'system', 'docker', {
        onCompleted: () => {
          systemDtos = systemDtos.map((item) => item.kind === 'docker'
            ? { ...item, state: 'running', serverVersion: item.serverVersion ?? '27.5.1', error: null }
            : item)
        },
      }))
    },
    async resumeJob(id) {
      return operationResult(submitOperation('resume-job', 'job', id, {
        onRunning: () => { jobDtos = jobDtos.map((job) => (job.id === id ? { ...job, status: 'running' } : job)) },
      }))
    },
    async syncDataset(ref) {
      return operationResult(submitOperation('sync-dataset', 'dataset', ref))
    },
    async updateAgent(id, agent) {
      const target = agentDtos.find((item) => item.id === id)
      if (!target) return failure('AGENT_NOT_FOUND', 'Agent not found')
      if (target.harness !== agent.harness) return failure('AGENT_HARNESS_IMMUTABLE', 'Agent Harness cannot be changed')
      agentDtos = agentDtos.map((item) => (
        item.id === id
          ? withMockAgentCapabilities({ ...agent, capabilities: target.capabilities, id, status: 'configured' })
          : item
      ))
      return operationResult(operations.complete('update-agent', 'agent', id, 'Agent updated'))
    },
    async updateEnvironment(id, environment) {
      const target = environmentDtos.find((item) => item.id === id)
      if (!target) return failure('ENVIRONMENT_NOT_FOUND', 'Environment not found')
      if (target.profileType === 'built-in') return failure('ENVIRONMENT_BUILT_IN_IMMUTABLE', 'Built-in environments cannot be updated')
      environmentDtos = environmentDtos.map((item) => item.id === id ? { ...environment, id } : item)
      return operationResult(operations.complete('update-environment', 'environment', id, 'Environment updated'))
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
        operation: operations.complete('update-job-leaderboard', 'job', id, 'Leaderboard inclusion updated'),
      })
    },
  }

  function submitOperation(
    type: string,
    resourceType: OperationResultDto['operation']['resourceType'],
    resourceId?: string,
    effects?: { onCompleted?: () => void; onRunning?: () => void },
  ) {
    const operation = operations.submit(type, resourceType, resourceId)
    if (effects) operationEffects.set(operation.id, effects)
    return operation
  }

  function applyOperationEffects(operation: OperationResultDto['operation']) {
    const effects = operationEffects.get(operation.id)
    if (!effects) return
    if (operation.status === 'running') {
      effects.onRunning?.()
      if (operation.type === 'download-dataset') {
        datasetDtos = datasetDtos.map((item) => item.ref === operation.resourceId
          ? { ...item, download: { status: 'downloading', progress: operation.progress ?? 0 } }
          : item)
      }
    }
    if (operation.status === 'completed') {
      effects.onCompleted?.()
      operationEffects.delete(operation.id)
    }
  }
}

function success<T>(data: T): ApiResponse<T> {
  return { data, error: null, meta: requestMeta }
}

function failure(code: string, message: string): ApiResponse<null> {
  const error: ApiError = { code, message }
  return { data: null, error, meta: requestMeta }
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

function managedDatasetDirectory(ref: string): string {
  return ref.replace('/', '--')
}

function withMockAgentCapabilities(agent: Omit<AgentDto, 'capabilities'> & Partial<Pick<AgentDto, 'capabilities'>>): AgentDto {
  return { ...agent, capabilities: agent.capabilities ?? fallbackAgentCapabilities() }
}

function buildQueuedJob(existing: JobDto[], request: CreateJobRequestDto, agent: AgentDto): JobDto {
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
    harness: agent.harness,
    id,
    includeInLeaderboard: request.config.includeInLeaderboard,
    canResume: false,
    jobDir: request.config.jobsDir,
    model: request.config.modelName,
    name: request.config.jobName,
    runtimeSeconds: 0,
    score: null,
    status: 'queued',
    tokenUsageM: 0,
    trial: { completed: 0, errored: 0, notPassed: 0, passed: 0, total: selectedTaskCount },
  }
}

function copyConfigFromJob(job: JobDto): import('./contract').JobConfigDto {
  return {
    agentSetupTimeoutMultiplier: 1,
    agentName: job.agentName,
    agentTimeoutMultiplier: 1,
    attempts: 1,
    concurrency: 4,
    datasetRef: job.datasetRef,
    debug: false,
    environmentPresetId: job.environmentName,
    environmentBuildTimeoutMultiplier: 1,
    extraInstructionPaths: [],
    includeInLeaderboard: job.includeInLeaderboard,
    jobName: `${job.name}-copy`,
    jobsDir: job.jobDir ?? 'jobs/new-job',
    maxRetries: 0,
    metric: 'mean',
    modelName: job.model,
    notes: '',
    retryExclude: '',
    retryInclude: '',
    retryMaxWaitSeconds: 30,
    retryMinWaitSeconds: 2,
    retryWaitMultiplier: 1.5,
    selectedTaskNames: null,
    timeoutMultiplier: 1,
    verifierTimeoutMultiplier: 1,
    verifierMode: 'dataset-default',
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

function isTerminalJob(status: JobDto['status']) {
  return status === 'completed' || status === 'failed' || status === 'cancelled' || status === 'interrupted'
}
