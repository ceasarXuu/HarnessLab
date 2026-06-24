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

export type QueryParams = Record<string, string | number | boolean | undefined | null>

export interface ApiClient {
  get<TResponse>(path: string, query?: QueryParams): Promise<TResponse>
  post<TRequest, TResponse>(path: string, payload: TRequest, query?: QueryParams): Promise<TResponse>
  put<TRequest, TResponse>(path: string, payload: TRequest, query?: QueryParams): Promise<TResponse>
  delete<TResponse>(path: string, query?: QueryParams): Promise<TResponse>
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
  job_dir: string | null
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

export interface AgentResponse {
  id: string
  name: string
  kind: string
  harbor_agent_name: string | null
  harbor_import_path: string | null
  model_name: string | null
  status: string
  profile_path: string
  created_at: string
  updated_at: string
}

export interface BenchmarkResponse {
  name: string
  version: string | null
  source: string
}

export interface SystemStatusResponse {
  ok?: boolean
  // 后端 DoctorService 返回结构会随版本演化；保留 unknown 子字段集合
  [key: string]: unknown
}

export interface EventResponse {
  id?: number
  scope_type?: string
  scope_id?: string
  event_type?: string
  payload?: Record<string, unknown>
  created_at?: string
  [key: string]: unknown
}

const joinPath = (basePath: string, path: string) => {
  const normalizedBase = basePath.endsWith('/') ? basePath.slice(0, -1) : basePath
  const normalizedPath = path.startsWith('/') ? path : `/${path}`
  return `${normalizedBase}${normalizedPath}`
}

const buildQuery = (query?: QueryParams): string => {
  if (!query) return ''
  const params = new URLSearchParams()
  for (const [key, value] of Object.entries(query)) {
    if (value === undefined || value === null) continue
    params.append(key, String(value))
  }
  const qs = params.toString()
  return qs ? `?${qs}` : ''
}

export const createApiClient = (basePath = '/api'): ApiClient => {
  const request = async <TResponse>(
    path: string,
    query: QueryParams | undefined,
    init?: RequestInit,
  ): Promise<TResponse> => {
    const url = joinPath(basePath, path) + buildQuery(query)
    const response = await fetch(url, {
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

    // DELETE 等接口可能返回空 body；fetch 在空 body 上 .json() 会抛错。
    const text = await response.text()
    if (!text) return undefined as unknown as TResponse
    try {
      return JSON.parse(text) as TResponse
    } catch (err) {
      throw new ApiError(`API response is not valid JSON for ${path}`, response.status, text)
    }
  }

  return {
    get: <TResponse>(path: string, query?: QueryParams) => request<TResponse>(path, query),
    post: <TRequest, TResponse>(path: string, payload: TRequest, query?: QueryParams) =>
      request<TResponse>(path, query, {
        method: 'POST',
        body: JSON.stringify(payload),
      }),
    put: <TRequest, TResponse>(path: string, payload: TRequest, query?: QueryParams) =>
      request<TResponse>(path, query, {
        method: 'PUT',
        body: JSON.stringify(payload),
      }),
    delete: <TResponse>(path: string, query?: QueryParams) =>
      request<TResponse>(path, query, { method: 'DELETE' }),
  }
}

export const apiClient = createApiClient('/api')

// 按 docs/releases/v0.1.4/web-connectivity/03-contract-gap-vs-backend.md 31 端点表组织。
// 覆盖：system / agents / benchmarks / experiments / runs / templates / leaderboard。
// Deferred：experiments/{id}/events/stream（SSE，见 bugfix/04）。
export const ornnLabApi = {
  // ---- system ----
  system: {
    status: () => apiClient.get<SystemStatusResponse>('/system/status'),
    doctor: (logs = false) =>
      apiClient.post<Record<string, never>, SystemStatusResponse>('/system/doctor', {}, { logs }),
    dockerOrphans: () => apiClient.get<Record<string, unknown>>('/system/docker-orphans'),
  },

  // ---- agents ----
  agents: {
    list: () => apiClient.get<AgentResponse[]>('/agents'),
    create: (payload: Record<string, unknown>) =>
      apiClient.post<Record<string, unknown>, AgentResponse>('/agents', payload),
    get: (id: string) => apiClient.get<AgentResponse>(`/agents/${id}`),
    compile: (id: string) =>
      apiClient.post<Record<string, never>, Record<string, unknown>>(`/agents/${id}/compile`, {}),
    validate: (payload: Record<string, unknown>) =>
      apiClient.post<Record<string, unknown>, { valid: boolean; errors: unknown[] }>(
        '/agents/validate',
        payload,
      ),
    update: (id: string, payload: Record<string, unknown>) =>
      apiClient.put<Record<string, unknown>, AgentResponse>(`/agents/${id}`, payload),
    delete: (id: string) => apiClient.delete<AgentResponse>(`/agents/${id}`),
  },

  // ---- benchmarks ----
  benchmarks: {
    list: () => apiClient.get<BenchmarkResponse[]>('/benchmarks'),
  },

  // ---- experiments ----
  experiments: () => apiClient.get<Experiment[]>('/experiments'),
  experiment: (id: string) => apiClient.get<ExperimentStateResponse>(`/experiments/${id}`),
  createExperiment: (payload: Record<string, unknown>) =>
    apiClient.post<Record<string, unknown>, ExperimentStateResponse>('/experiments', payload),
  runExperiment: (id: string, wait = false) =>
    apiClient.post<Record<string, never>, ExperimentStateResponse>(
      `/experiments/${id}/run`,
      {},
      { wait },
    ),
  cancelExperiment: (id: string) =>
    apiClient.post<Record<string, never>, ExperimentStateResponse>(
      `/experiments/${id}/cancel`,
      {},
    ),
  deleteExperiment: (id: string) => apiClient.delete<ExperimentStateResponse>(`/experiments/${id}`),
  cloneExperiment: (id: string) =>
    apiClient.post<Record<string, never>, ExperimentStateResponse>(
      `/experiments/${id}/clone`,
      {},
    ),
  saveExperimentTemplate: (id: string, name: string) =>
    apiClient.post<{ name: string }, TemplateResponse>(
      `/experiments/${id}/save-template`,
      { name },
    ),
  experimentReport: (id: string) =>
    apiClient.get<ExperimentReportResponse>(`/experiments/${id}/report`),
  experimentEvents: (id: string, after = 0) =>
    apiClient.get<EventResponse[]>(`/experiments/${id}/events`, { after }),
  experimentRuns: (id: string) =>
    apiClient.get<ExperimentRun[]>(`/experiments/${id}/runs`),
  // experiments/{id}/events/stream Deferred（SSE，见 bugfix/04-sse-stream-not-realtime.md）

  // ---- runs ----
  run: (id: string) => apiClient.get<ExperimentRun>(`/runs/${id}`),
  cancelRun: (id: string) =>
    apiClient.post<Record<string, never>, ExperimentRun>(`/runs/${id}/cancel`, {}),
  runEvents: (id: string, after = 0) =>
    apiClient.get<EventResponse[]>(`/runs/${id}/events`, { after }),
  runReport: (id: string) => apiClient.get<RunReportResponse>(`/runs/${id}/report`),

  // ---- templates ----
  templates: () => apiClient.get<TemplateResponse[]>('/templates'),
  createTemplate: (name: string, config: Record<string, unknown>) =>
    apiClient.post<{ name: string; config: Record<string, unknown> }, TemplateResponse>(
      '/templates',
      { name, config },
    ),
  deleteTemplate: (id: string) => apiClient.delete<TemplateResponse>(`/templates/${id}`),

  // ---- leaderboard ----
  leaderboard: (benchmark?: string) =>
    apiClient.get<LeaderboardEntryResponse[]>(
      '/leaderboard',
      benchmark ? { benchmark } : undefined,
    ),
}
