import { useCallback, useEffect, useRef, useState } from 'react'
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
  JobDto,
  JobEventDto,
  LeaderboardDatasetDto,
  LeaderboardEntryDto,
  LeaderboardQuery,
  ListQuery,
  Operation,
  Page,
  SystemComponentDto,
  TrialDto,
} from './contract'
import type { WebUiClient } from './webUiClient'

export interface WebUiResource<T> {
  data: T | null
  error: ApiError | null
  loading: boolean
  refresh: () => Promise<void>
}

export interface WebUiOperation {
  error: ApiError | null
  operation: Operation | null
  refresh: () => Promise<void>
  submit: <T>(
    mutation: () => Promise<ApiResponse<T | null>>,
    operationFrom: (data: T) => Operation | undefined,
  ) => Promise<void>
}

export function useWebUiResource<T>(
  load: () => Promise<ApiResponse<T | null>>,
  dependencies: readonly unknown[],
  enabled = true,
): WebUiResource<T> {
  const [data, setData] = useState<T | null>(null)
  const [error, setError] = useState<ApiError | null>(null)
  const [loading, setLoading] = useState(true)
  const sequence = useRef(0)

  const refresh = useCallback(async () => {
    const request = ++sequence.current
    setLoading(true)
    setData(null)
    setError(null)
    const response = await load()
    if (request !== sequence.current) return
    setData(response.data)
    setError(response.error)
    setLoading(false)
  }, [load, ...dependencies])

  useEffect(() => {
    if (!enabled) {
      setData(null)
      setError(null)
      setLoading(false)
      return
    }
    void refresh()
  }, [enabled, refresh])

  return { data, error, loading, refresh }
}

export function useJobs(client: WebUiClient, query: ListQuery = {}): WebUiResource<Page<JobDto>> {
  const load = useCallback(() => client.listJobs(query), [client, query.cursor, query.limit, query.q])
  return useWebUiResource(load, [client, query.cursor, query.limit, query.q])
}

export function useDatasets(client: WebUiClient, query: ListQuery = {}): WebUiResource<Page<DatasetDto>> {
  const load = useCallback(() => client.listDatasets(query), [client, query.cursor, query.limit, query.q])
  return useWebUiResource(load, [client, query.cursor, query.limit, query.q])
}

export function useAgents(client: WebUiClient, query: AgentQuery = {}): WebUiResource<Page<AgentDto>> {
  const load = useCallback(() => client.listAgents(query), [client, query.cursor, query.limit, query.q, query.status, query.type])
  return useWebUiResource(load, [client, query.cursor, query.limit, query.q, query.status, query.type])
}

export function useAgent(client: WebUiClient, id?: string): WebUiResource<AgentDto> {
  const load = useCallback(() => client.getAgent(id ?? ''), [client, id])
  return useWebUiResource(load, [client, id], Boolean(id))
}

export function useDataset(client: WebUiClient, ref?: string): WebUiResource<DatasetDto> {
  const load = useCallback(() => client.getDataset(ref ?? ''), [client, ref])
  return useWebUiResource(load, [client, ref], Boolean(ref))
}

export function useEnvironments(client: WebUiClient, query: EnvironmentQuery = {}): WebUiResource<Page<EnvironmentDto>> {
  const load = useCallback(() => client.listEnvironments(query), [client, query.cursor, query.limit, query.q, query.type])
  return useWebUiResource(load, [client, query.cursor, query.limit, query.q, query.type])
}

export function useEnvironment(client: WebUiClient, id?: string): WebUiResource<EnvironmentDto> {
  const load = useCallback(() => client.getEnvironment(id ?? ''), [client, id])
  return useWebUiResource(load, [client, id], Boolean(id))
}

export function useDatasetTasks(
  client: WebUiClient,
  ref?: string,
  query: DatasetTaskQuery = {},
): WebUiResource<Page<DatasetTaskDto>> {
  const load = useCallback(() => client.listDatasetTasks(ref ?? '', query), [client, query.cursor, query.limit, query.q, query.split, ref])
  return useWebUiResource(load, [client, query.cursor, query.limit, query.q, query.split, ref], Boolean(ref))
}

export function useJob(client: WebUiClient, id?: string): WebUiResource<JobDto> {
  const load = useCallback(() => client.getJob(id ?? ''), [client, id])
  return useWebUiResource(load, [client, id], Boolean(id))
}

export function useJobEvents(client: WebUiClient, id?: string): WebUiResource<JobEventDto[]> {
  const load = useCallback(() => client.listJobEvents(id ?? ''), [client, id])
  return useWebUiResource(load, [client, id], Boolean(id))
}

export function useJobTrials(client: WebUiClient, id?: string): WebUiResource<TrialDto[]> {
  const load = useCallback(() => client.listJobTrials(id ?? ''), [client, id])
  return useWebUiResource(load, [client, id], Boolean(id))
}

export function useLeaderboard(client: WebUiClient, query?: LeaderboardQuery): WebUiResource<Page<LeaderboardEntryDto>> {
  const enabled = Boolean(query?.dataset)
  const load = useCallback(
    () => client.listLeaderboard(query ?? { dataset: '' }),
    [client, query?.cursor, query?.dataset, query?.limit, query?.metric, query?.q, query?.split],
  )
  return useWebUiResource(load, [client, query?.cursor, query?.dataset, query?.limit, query?.metric, query?.q, query?.split], enabled)
}

export function useLeaderboardDatasets(client: WebUiClient, query: ListQuery = {}): WebUiResource<Page<LeaderboardDatasetDto>> {
  const load = useCallback(() => client.listLeaderboardDatasets(query), [client, query.cursor, query.limit, query.q])
  return useWebUiResource(load, [client, query.cursor, query.limit, query.q])
}

export function useSystemHealth(client: WebUiClient): WebUiResource<Page<SystemComponentDto>> {
  const load = useCallback(() => client.listSystemHealth(), [client])
  return useWebUiResource(load, [client])
}

export function useHubConnection(client: WebUiClient): WebUiResource<HubConnectionDto> {
  const load = useCallback(() => client.getHubConnection(), [client])
  return useWebUiResource(load, [client])
}

export function useOperation(client: WebUiClient): WebUiOperation {
  const [operation, setOperation] = useState<Operation | null>(null)
  const [error, setError] = useState<ApiError | null>(null)
  const [pollFailureCount, setPollFailureCount] = useState(0)

  const refresh = useCallback(async () => {
    if (!operation) return
    const response = await client.getOperation(operation.id)
    if (response.data) {
      setOperation(response.data)
      setError(response.error)
      setPollFailureCount(0)
      return
    }
    setError(response.error ?? { code: 'OPERATION_POLL_FAILED', message: 'The operation could not be refreshed.' })
    setPollFailureCount((current) => current + 1)
  }, [client, operation])

  const submit = useCallback(async <T,>(
    mutation: () => Promise<ApiResponse<T | null>>,
    operationFrom: (data: T) => Operation | undefined,
  ) => {
    const response = await mutation()
    if (!response.data) {
      setOperation(null)
      setPollFailureCount(0)
      setError(response.error ?? { code: 'OPERATION_MISSING', message: 'The mutation did not return an operation.' })
      return
    }
    const nextOperation = operationFrom(response.data)
    if (!nextOperation) {
      setOperation(null)
      setPollFailureCount(0)
      setError({ code: 'OPERATION_MISSING', message: 'The mutation did not return an operation.' })
      return
    }
    setError(response.error)
    setPollFailureCount(0)
    setOperation(nextOperation)
  }, [])

  useEffect(() => {
    if (!operation || (operation.status !== 'queued' && operation.status !== 'running')) return undefined
    const delay = Math.min(500 * 2 ** pollFailureCount, 5_000)
    const timer = window.setTimeout(() => void refresh(), delay)
    return () => window.clearTimeout(timer)
  }, [operation, pollFailureCount, refresh])

  return { error, operation, refresh, submit }
}
