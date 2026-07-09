import type { ApiError, ApiResponse, DatasetDto, DatasetTaskDto, DatasetTaskQuery, JobDto, JobEventDto, ListQuery, Page, TrialDto } from './contract'

export interface WebUiClient {
  getDataset(ref: string): Promise<ApiResponse<DatasetDto | null>>
  getJob(id: string): Promise<ApiResponse<JobDto | null>>
  listDatasetTasks(ref: string, query?: DatasetTaskQuery): Promise<ApiResponse<Page<DatasetTaskDto> | null>>
  listDatasets(query?: ListQuery): Promise<ApiResponse<Page<DatasetDto> | null>>
  listJobEvents(id: string): Promise<ApiResponse<JobEventDto[] | null>>
  listJobTrials(id: string): Promise<ApiResponse<TrialDto[] | null>>
  listJobs(query?: ListQuery): Promise<ApiResponse<Page<JobDto> | null>>
}

export function createWebUiHttpClient(baseUrl = '/api/webui/v1', request = fetch): WebUiClient {
  return {
    getDataset: (ref) => requestJson<DatasetDto | null>(request, `${baseUrl}/datasets/${encodeURIComponent(ref)}`),
    getJob: (id) => requestJson<JobDto | null>(request, `${baseUrl}/jobs/${encodeURIComponent(id)}`),
    listDatasetTasks: (ref, query) =>
      requestJson<Page<DatasetTaskDto>>(request, `${baseUrl}/datasets/${encodeURIComponent(ref)}/tasks${toSearch(query)}`),
    listDatasets: (query) => requestJson<Page<DatasetDto>>(request, `${baseUrl}/datasets${toSearch(query)}`),
    listJobEvents: (id) => requestJson<JobEventDto[]>(request, `${baseUrl}/jobs/${encodeURIComponent(id)}/events`),
    listJobTrials: (id) => requestJson<TrialDto[]>(request, `${baseUrl}/jobs/${encodeURIComponent(id)}/trials`),
    listJobs: (query) => requestJson<Page<JobDto>>(request, `${baseUrl}/jobs${toSearch(query)}`),
  }
}

async function requestJson<T>(request: typeof fetch, url: string): Promise<ApiResponse<T | null>> {
  try {
    const response = await request(url)
    const payload: unknown = await response.json()
    if (isApiResponse<T>(payload)) return payload
    return contractFailure('INVALID_API_RESPONSE', 'The server returned an invalid API response.')
  } catch {
    return contractFailure('NETWORK_REQUEST_FAILED', 'The API request could not be completed.')
  }
}

function toSearch(query: ListQuery | DatasetTaskQuery | undefined): string {
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
