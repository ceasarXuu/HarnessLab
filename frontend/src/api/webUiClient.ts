import type { ApiResponse, DatasetDto, DatasetTaskDto, DatasetTaskQuery, JobDto, ListQuery, Page } from './contract'

export interface WebUiClient {
  getDataset(ref: string): Promise<ApiResponse<DatasetDto | null>>
  getJob(id: string): Promise<ApiResponse<JobDto | null>>
  listDatasetTasks(ref: string, query?: DatasetTaskQuery): Promise<ApiResponse<Page<DatasetTaskDto>>>
  listDatasets(query?: ListQuery): Promise<ApiResponse<Page<DatasetDto>>>
  listJobs(query?: ListQuery): Promise<ApiResponse<Page<JobDto>>>
}

export function createWebUiHttpClient(baseUrl = '/api/webui/v1', request = fetch): WebUiClient {
  return {
    getDataset: (ref) => requestJson<DatasetDto | null>(request, `${baseUrl}/datasets/${encodeURIComponent(ref)}`),
    getJob: (id) => requestJson<JobDto | null>(request, `${baseUrl}/jobs/${encodeURIComponent(id)}`),
    listDatasetTasks: (ref, query) =>
      requestJson<Page<DatasetTaskDto>>(request, `${baseUrl}/datasets/${encodeURIComponent(ref)}/tasks${toSearch(query)}`),
    listDatasets: (query) => requestJson<Page<DatasetDto>>(request, `${baseUrl}/datasets${toSearch(query)}`),
    listJobs: (query) => requestJson<Page<JobDto>>(request, `${baseUrl}/jobs${toSearch(query)}`),
  }
}

async function requestJson<T>(request: typeof fetch, url: string): Promise<ApiResponse<T>> {
  const response = await request(url)
  return response.json() as Promise<ApiResponse<T>>
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
