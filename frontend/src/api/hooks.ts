import { useCallback, useEffect, useRef, useState } from 'react'
import type { ApiError, ApiResponse, DatasetDto, DatasetTaskDto, DatasetTaskQuery, JobDto, JobEventDto, ListQuery, Page, TrialDto } from './contract'
import type { WebUiClient } from './webUiClient'

export interface WebUiResource<T> {
  data: T | null
  error: ApiError | null
  loading: boolean
  refresh: () => Promise<void>
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

export function useDataset(client: WebUiClient, ref?: string): WebUiResource<DatasetDto> {
  const load = useCallback(() => client.getDataset(ref ?? ''), [client, ref])
  return useWebUiResource(load, [client, ref], Boolean(ref))
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
