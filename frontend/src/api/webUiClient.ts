import type {
  AgentDto,
  AgentQuery,
  ApiError,
  ApiResponse,
  DatasetDto,
  DatasetTaskDto,
  DatasetTaskQuery,
  EnvironmentDto,
  EnvironmentQuery,
  HubConnectionDto,
  CreateJobRequestDto,
  CreateJobResponseDto,
  DatasetImportRequestDto,
  JobDto,
  JobEventDto,
  LeaderboardDatasetDto,
  LeaderboardEntryDto,
  LeaderboardQuery,
  ListQuery,
  Page,
  Operation,
  OperationResultDto,
  SystemComponentDto,
  TrialDto,
  UpdateCheckResultDto,
  UpdateJobLeaderboardRequestDto,
  UpdateJobLeaderboardResponseDto,
} from './contract'

export interface WebUiClient {
  cancelJob(id: string): Promise<ApiResponse<OperationResultDto | null>>
  cancelDatasetDownload(ref: string): Promise<ApiResponse<OperationResultDto | null>>
  cancelOperation(id: string): Promise<ApiResponse<OperationResultDto | null>>
  checkForSystemUpdate(): Promise<ApiResponse<UpdateCheckResultDto | null>>
  cleanDockerCache(): Promise<ApiResponse<OperationResultDto | null>>
  cleanStorageCache(): Promise<ApiResponse<OperationResultDto | null>>
  copyEnvironment(id: string): Promise<ApiResponse<OperationResultDto | null>>
  createAgent(agent: AgentDto): Promise<ApiResponse<OperationResultDto | null>>
  createEnvironment(environment: EnvironmentDto): Promise<ApiResponse<OperationResultDto | null>>
  createJob(request: CreateJobRequestDto): Promise<ApiResponse<CreateJobResponseDto | null>>
  deleteAgent(id: string): Promise<ApiResponse<OperationResultDto | null>>
  deleteEnvironment(id: string): Promise<ApiResponse<OperationResultDto | null>>
  deleteLocalDataset(ref: string): Promise<ApiResponse<OperationResultDto | null>>
  downloadDataset(ref: string): Promise<ApiResponse<OperationResultDto | null>>
  getAgent(id: string): Promise<ApiResponse<AgentDto | null>>
  getDataset(ref: string): Promise<ApiResponse<DatasetDto | null>>
  getEnvironment(id: string): Promise<ApiResponse<EnvironmentDto | null>>
  getHubConnection(): Promise<ApiResponse<HubConnectionDto | null>>
  getJob(id: string): Promise<ApiResponse<JobDto | null>>
  getOperation(id: string): Promise<ApiResponse<Operation | null>>
  importDataset(request: DatasetImportRequestDto): Promise<ApiResponse<OperationResultDto | null>>
  installSystemUpdate(): Promise<ApiResponse<OperationResultDto | null>>
  listAgents(query?: AgentQuery): Promise<ApiResponse<Page<AgentDto> | null>>
  listDatasetTasks(ref: string, query?: DatasetTaskQuery): Promise<ApiResponse<Page<DatasetTaskDto> | null>>
  listDatasets(query?: ListQuery): Promise<ApiResponse<Page<DatasetDto> | null>>
  listEnvironments(query?: EnvironmentQuery): Promise<ApiResponse<Page<EnvironmentDto> | null>>
  listJobEvents(id: string): Promise<ApiResponse<JobEventDto[] | null>>
  listJobTrials(id: string): Promise<ApiResponse<TrialDto[] | null>>
  listJobs(query?: ListQuery): Promise<ApiResponse<Page<JobDto> | null>>
  listLeaderboard(query: LeaderboardQuery): Promise<ApiResponse<Page<LeaderboardEntryDto> | null>>
  listLeaderboardDatasets(query?: ListQuery): Promise<ApiResponse<Page<LeaderboardDatasetDto> | null>>
  listSystemHealth(): Promise<ApiResponse<Page<SystemComponentDto> | null>>
  restartSystemService(): Promise<ApiResponse<OperationResultDto | null>>
  retryJob(id: string): Promise<ApiResponse<OperationResultDto | null>>
  resumeJob(id: string): Promise<ApiResponse<OperationResultDto | null>>
  runDatasetTask(ref: string, taskName: string): Promise<ApiResponse<OperationResultDto | null>>
  syncDataset(ref: string): Promise<ApiResponse<OperationResultDto | null>>
  updateAgent(id: string, agent: AgentDto): Promise<ApiResponse<OperationResultDto | null>>
  updateEnvironment(id: string, environment: EnvironmentDto): Promise<ApiResponse<OperationResultDto | null>>
  updateJobLeaderboard(id: string, request: UpdateJobLeaderboardRequestDto): Promise<ApiResponse<UpdateJobLeaderboardResponseDto | null>>
}

export function createWebUiHttpClient(baseUrl = '/api/webui/v1', request = fetch): WebUiClient {
  return {
    cancelJob: (id) => post<OperationResultDto>(request, `${baseUrl}/jobs/${encodeURIComponent(id)}/cancel`),
    cancelDatasetDownload: (ref) => post<OperationResultDto>(request, `${baseUrl}/datasets/${encodeURIComponent(ref)}/download/cancel`),
    cancelOperation: (id) => post<OperationResultDto>(request, `${baseUrl}/operations/${encodeURIComponent(id)}/cancel`),
    checkForSystemUpdate: () => post<UpdateCheckResultDto>(request, `${baseUrl}/system/service/update/check`),
    cleanDockerCache: () => post<OperationResultDto>(request, `${baseUrl}/system/cache/docker/clean`),
    cleanStorageCache: () => post<OperationResultDto>(request, `${baseUrl}/system/cache/storage/clean`),
    copyEnvironment: (id) => post<OperationResultDto>(request, `${baseUrl}/environments/${encodeURIComponent(id)}/copy`),
    createAgent: (agent) => post<OperationResultDto>(request, `${baseUrl}/agents`, agent),
    createEnvironment: (environment) => post<OperationResultDto>(request, `${baseUrl}/environments`, environment),
    createJob: (body) => post<CreateJobResponseDto>(request, `${baseUrl}/jobs`, body),
    deleteAgent: (id) => send<OperationResultDto>(request, `${baseUrl}/agents/${encodeURIComponent(id)}`, 'DELETE'),
    deleteEnvironment: (id) => send<OperationResultDto>(request, `${baseUrl}/environments/${encodeURIComponent(id)}`, 'DELETE'),
    deleteLocalDataset: (ref) => send<OperationResultDto>(request, `${baseUrl}/datasets/${encodeURIComponent(ref)}/local`, 'DELETE'),
    downloadDataset: (ref) => post<OperationResultDto>(request, `${baseUrl}/datasets/${encodeURIComponent(ref)}/download`),
    getAgent: (id) => requestJson<AgentDto | null>(request, `${baseUrl}/agents/${encodeURIComponent(id)}`),
    getDataset: (ref) => requestJson<DatasetDto | null>(request, `${baseUrl}/datasets/${encodeURIComponent(ref)}`),
    getEnvironment: (id) => requestJson<EnvironmentDto | null>(request, `${baseUrl}/environments/${encodeURIComponent(id)}`),
    getHubConnection: () => requestJson<HubConnectionDto | null>(request, `${baseUrl}/system/hub-connection`),
    getJob: (id) => requestJson<JobDto | null>(request, `${baseUrl}/jobs/${encodeURIComponent(id)}`),
    getOperation: (id) => requestJson<Operation | null>(request, `${baseUrl}/operations/${encodeURIComponent(id)}`),
    importDataset: (body) => post<OperationResultDto>(request, `${baseUrl}/datasets/import`, body),
    installSystemUpdate: () => post<OperationResultDto>(request, `${baseUrl}/system/service/update`),
    listAgents: (query) => requestJson<Page<AgentDto>>(request, `${baseUrl}/agents${toSearch(query)}`),
    listDatasetTasks: (ref, query) =>
      requestJson<Page<DatasetTaskDto>>(request, `${baseUrl}/datasets/${encodeURIComponent(ref)}/tasks${toSearch(query)}`),
    listDatasets: (query) => requestJson<Page<DatasetDto>>(request, `${baseUrl}/datasets${toSearch(query)}`),
    listEnvironments: (query) => requestJson<Page<EnvironmentDto>>(request, `${baseUrl}/environments${toSearch(query)}`),
    listJobEvents: (id) => requestJson<JobEventDto[]>(request, `${baseUrl}/jobs/${encodeURIComponent(id)}/events`),
    listJobTrials: (id) => requestJson<TrialDto[]>(request, `${baseUrl}/jobs/${encodeURIComponent(id)}/trials`),
    listJobs: (query) => requestJson<Page<JobDto>>(request, `${baseUrl}/jobs${toSearch(query)}`),
    listLeaderboard: (query) => requestJson<Page<LeaderboardEntryDto>>(request, `${baseUrl}/leaderboard${toSearch(query)}`),
    listLeaderboardDatasets: (query) => requestJson<Page<LeaderboardDatasetDto>>(request, `${baseUrl}/leaderboard/datasets${toSearch(query)}`),
    listSystemHealth: () => requestJson<Page<SystemComponentDto>>(request, `${baseUrl}/system/health`),
    restartSystemService: () => post<OperationResultDto>(request, `${baseUrl}/system/service/restart`),
    retryJob: (id) => post<OperationResultDto>(request, `${baseUrl}/jobs/${encodeURIComponent(id)}/retry`),
    resumeJob: (id) => post<OperationResultDto>(request, `${baseUrl}/jobs/${encodeURIComponent(id)}/resume`),
    runDatasetTask: (ref, taskName) => post<OperationResultDto>(request, `${baseUrl}/datasets/${encodeURIComponent(ref)}/tasks/${encodeURIComponent(taskName)}/run`),
    syncDataset: (ref) => post<OperationResultDto>(request, `${baseUrl}/datasets/${encodeURIComponent(ref)}/sync`),
    updateAgent: (id, agent) => send<OperationResultDto>(request, `${baseUrl}/agents/${encodeURIComponent(id)}`, 'PATCH', agent),
    updateEnvironment: (id, environment) => send<OperationResultDto>(request, `${baseUrl}/environments/${encodeURIComponent(id)}`, 'PATCH', environment),
    updateJobLeaderboard: (id, body) => send<UpdateJobLeaderboardResponseDto>(request, `${baseUrl}/jobs/${encodeURIComponent(id)}/leaderboard`, 'PATCH', body),
  }
}

async function requestJson<T>(request: typeof fetch, url: string, init?: RequestInit): Promise<ApiResponse<T | null>> {
  try {
    const response = await request(url, init)
    const payload: unknown = await response.json()
    if (isApiResponse<T>(payload)) return payload
    return contractFailure('INVALID_API_RESPONSE', 'The server returned an invalid API response.')
  } catch {
    return contractFailure('NETWORK_REQUEST_FAILED', 'The API request could not be completed.')
  }
}

function post<T>(request: typeof fetch, url: string, body?: unknown): Promise<ApiResponse<T | null>> {
  return send<T>(request, url, 'POST', body)
}

function send<T>(request: typeof fetch, url: string, method: 'DELETE' | 'PATCH' | 'POST', body?: unknown): Promise<ApiResponse<T | null>> {
  return requestJson<T>(request, url, {
    body: body === undefined ? undefined : JSON.stringify(body),
    headers: body === undefined ? undefined : { 'content-type': 'application/json' },
    method,
  })
}

function toSearch(query: AgentQuery | DatasetTaskQuery | EnvironmentQuery | LeaderboardQuery | ListQuery | undefined): string {
  if (!query) return ''
  const params = new URLSearchParams()
  for (const [key, value] of Object.entries(query)) {
    if (value !== undefined && value !== '') params.set(key, String(value))
  }
  const result = params.toString()
  return result ? `?${result}` : ''
}

function contractFailure(code: string, message: string): ApiResponse<null> {
  const error: ApiError = { code, message }
  return { data: null, error }
}

function isApiResponse<T>(value: unknown): value is ApiResponse<T> {
  if (!value || typeof value !== 'object') return false
  const payload = value as Record<string, unknown>
  const error = payload.error
  return 'data' in payload && (error === null || (typeof error === 'object' && error !== null))
}
