export class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
    readonly payload: unknown,
  ) {
    super(message)
    this.name = 'ApiError'
  }
}

export interface ApiClient {
  get<TResponse>(path: string): Promise<TResponse>
  post<TRequest, TResponse>(path: string, payload: TRequest): Promise<TResponse>
}

export interface ExperimentRun {
  id: string
  experiment_id: string
  status: 'cancelled' | 'completed' | 'draft' | 'failed' | 'interrupted' | 'queued' | 'running'
  run_order: number
  agent_id: string
  benchmark_name: string
  benchmark_version: string | null
  split: string | null
  n_tasks: number | null
  n_attempts: number
  n_concurrent: number
  score: number | null
  report_path: string | null
  failure_class: string | null
  failure_code: string | null
}

export interface Experiment {
  id: string
  name: string
  kind: 'batch' | 'comparison' | 'single'
  status: string
  requested_run_count: number
  mode: string
  created_at: string
  updated_at: string
}

export interface ExperimentStateResponse {
  experiment: Experiment
  runs: ExperimentRun[]
}

export interface LeaderboardEntryResponse {
  id: string
  agent_id: string
  benchmark_name: string
  benchmark_version: string | null
  split: string | null
  finished_at: string | null
  score: number | null
  comparability_key: string
  report_path: string | null
}

export interface TemplateResponse {
  id: string
  name: string
  config: Record<string, unknown>
  created_at: string
  updated_at: string
  deleted_at?: string | null
}

export interface ReportSummaryResponse {
  run_id: string
  status: string
  score: number | null
  failure_class: string | null
  failure_code: string | null
  artifact_links: string[]
}

export interface RunReportResponse {
  run: ExperimentRun
  summary: ReportSummaryResponse
}

export interface ExperimentReportResponse {
  experiment: Experiment
  reports: Array<{
    run_id: string
    report_path: string
    summary: ReportSummaryResponse
  }>
}

const joinPath = (basePath: string, path: string) => {
  const normalizedBase = basePath.endsWith('/') ? basePath.slice(0, -1) : basePath
  const normalizedPath = path.startsWith('/') ? path : `/${path}`
  return `${normalizedBase}${normalizedPath}`
}

export const createApiClient = (basePath = '/api'): ApiClient => {
  const request = async <TResponse>(
    path: string,
    init?: RequestInit,
  ): Promise<TResponse> => {
    const response = await fetch(joinPath(basePath, path), {
      headers: {
        'Content-Type': 'application/json',
        ...(init?.headers ?? {}),
      },
      ...init,
    })

    if (!response.ok) {
      const payload = await response.text()
      throw new ApiError(`API request failed for ${path}`, response.status, payload)
    }

    return (await response.json()) as TResponse
  }

  return {
    get: <TResponse>(path: string) => request<TResponse>(path),
    post: <TRequest, TResponse>(path: string, payload: TRequest) =>
      request<TResponse>(path, {
        method: 'POST',
        body: JSON.stringify(payload),
      }),
  }
}

export const apiClient = createApiClient('/api')

export const harnessLabApi = {
  experiments: () => apiClient.get<Experiment[]>('/experiments'),
  experiment: (id: string) => apiClient.get<ExperimentStateResponse>(`/experiments/${id}`),
  runExperiment: (id: string) => apiClient.post<Record<string, never>, ExperimentStateResponse>(
    `/experiments/${id}/run`,
    {},
  ),
  cancelExperiment: (id: string) => apiClient.post<Record<string, never>, ExperimentStateResponse>(
    `/experiments/${id}/cancel`,
    {},
  ),
  cloneExperiment: (id: string) => apiClient.post<Record<string, never>, ExperimentStateResponse>(
    `/experiments/${id}/clone`,
    {},
  ),
  saveExperimentTemplate: (id: string, name: string) =>
    apiClient.post<{ name: string }, TemplateResponse>(
      `/experiments/${id}/save-template`,
      { name },
    ),
  experimentReport: (id: string) => apiClient.get<ExperimentReportResponse>(
    `/experiments/${id}/report`,
  ),
  run: (id: string) => apiClient.get<ExperimentRun>(`/runs/${id}`),
  cancelRun: (id: string) => apiClient.post<Record<string, never>, ExperimentRun>(
    `/runs/${id}/cancel`,
    {},
  ),
  runReport: (id: string) => apiClient.get<RunReportResponse>(`/runs/${id}/report`),
  templates: () => apiClient.get<TemplateResponse[]>('/templates'),
  createTemplate: (name: string, config: Record<string, unknown>) =>
    apiClient.post<{ name: string; config: Record<string, unknown> }, TemplateResponse>(
      '/templates',
      { name, config },
    ),
  leaderboard: (benchmark?: string) => {
    const suffix = benchmark ? `?benchmark=${encodeURIComponent(benchmark)}` : ''
    return apiClient.get<LeaderboardEntryResponse[]>(`/leaderboard${suffix}`)
  },
}
