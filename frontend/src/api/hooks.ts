import { useCallback, useEffect, useRef, useState } from 'react'
import type { ApiError, ApiResponse, DatasetDto, JobDto, ListQuery, Page } from './contract'
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
): WebUiResource<T> {
  const [data, setData] = useState<T | null>(null)
  const [error, setError] = useState<ApiError | null>(null)
  const [loading, setLoading] = useState(true)
  const sequence = useRef(0)

  const refresh = useCallback(async () => {
    const request = ++sequence.current
    setLoading(true)
    const response = await load()
    if (request !== sequence.current) return
    setData(response.data)
    setError(response.error)
    setLoading(false)
  }, [load, ...dependencies])

  useEffect(() => {
    void refresh()
  }, [refresh])

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
