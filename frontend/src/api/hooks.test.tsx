import { act, renderHook, waitFor } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type { ApiResponse } from './contract'
import { useCachedServerSearch, useOperation, usePollingRefresh, useWebUiResource } from './hooks'
import { createMockWebUiClient } from './mockClient'

describe('useWebUiResource', () => {
  it('polls an enabled live resource and stops when disabled', async () => {
    vi.useFakeTimers()
    try {
      const refresh = vi.fn().mockResolvedValue(undefined)
      const { rerender } = renderHook(
        ({ enabled }) => usePollingRefresh(refresh, enabled, 1_000),
        { initialProps: { enabled: true } },
      )

      await act(async () => vi.advanceTimersByTimeAsync(1_000))
      expect(refresh).toHaveBeenCalledTimes(1)

      rerender({ enabled: false })
      await act(async () => vi.advanceTimersByTimeAsync(2_000))
      expect(refresh).toHaveBeenCalledTimes(1)
    } finally {
      vi.useRealTimers()
    }
  })

  it('exposes successful API data after loading completes', async () => {
    const load = vi.fn<() => Promise<ApiResponse<string | null>>>().mockResolvedValue({
      data: 'loaded',
      error: null,
    })

    const { result } = renderHook(() => useWebUiResource(load, []))

    expect(result.current.loading).toBe(true)
    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.data).toBe('loaded')
    expect(result.current.error).toBeNull()
  })

  it('exposes contract errors instead of keeping stale successful data', async () => {
    const load = vi.fn<() => Promise<ApiResponse<string | null>>>().mockResolvedValue({
      data: null,
      error: { code: 'NETWORK_REQUEST_FAILED', message: 'The API request could not be completed.' },
    })

    const { result } = renderHook(() => useWebUiResource(load, []))

    await waitFor(() => expect(result.current.loading).toBe(false))
    expect(result.current.data).toBeNull()
    expect(result.current.error?.code).toBe('NETWORK_REQUEST_FAILED')
  })

  it('does not invoke a disabled resource loader, including manual refresh', async () => {
    const load = vi.fn<() => Promise<ApiResponse<string | null>>>().mockResolvedValue({
      data: 'should not load',
      error: null,
    })
    const { result } = renderHook(() => useWebUiResource(load, [], false))

    await waitFor(() => expect(result.current.loading).toBe(false))
    await act(async () => result.current.refresh())

    expect(load).not.toHaveBeenCalled()
    expect(result.current.data).toBeNull()
  })

  it('keeps the last successful resource while a refresh is pending', async () => {
    let resolveRefresh: ((response: ApiResponse<string | null>) => void) | undefined
    const load = vi.fn<() => Promise<ApiResponse<string | null>>>()
      .mockResolvedValueOnce({ data: 'loaded', error: null })
      .mockImplementationOnce(() => new Promise((resolve) => { resolveRefresh = resolve }))
    const { result } = renderHook(() => useWebUiResource(load, []))

    await waitFor(() => expect(result.current.data).toBe('loaded'))
    let refresh: Promise<void>
    act(() => { refresh = result.current.refresh() })

    await waitFor(() => expect(result.current.loading).toBe(true))
    expect(result.current.data).toBe('loaded')
    await act(async () => resolveRefresh?.({ data: 'updated', error: null }))
    await refresh!
    expect(result.current.data).toBe('updated')
  })

  it('caches a server search result by scope and keyword', async () => {
    const loadPage = vi.fn<(query: string) => Promise<ApiResponse<{ items: string[]; total: number } | null>>>()
      .mockResolvedValue({ data: { items: ['swebench-verified'], total: 1 }, error: null })
    const { result, rerender } = renderHook(
      ({ query }) => useCachedServerSearch('datasets', query, loadPage),
      { initialProps: { query: 'swe' } },
    )

    await waitFor(() => expect(result.current.data?.items).toEqual(['swebench-verified']))
    rerender({ query: '' })
    await waitFor(() => expect(result.current.data).toBeNull())
    rerender({ query: 'swe' })
    await waitFor(() => expect(result.current.data?.items).toEqual(['swebench-verified']))

    expect(loadPage).toHaveBeenCalledTimes(1)
    expect(loadPage).toHaveBeenCalledWith('swe')
  })

  it('tracks an Operation through queued, running, and completed states', async () => {
    const client = createMockWebUiClient()
    const { result } = renderHook(() => useOperation(client))

    await act(async () => {
      await result.current.submit(() => client.cleanStorageCache(), (data) => data.operation)
    })
    await waitFor(() => expect(result.current.operation?.status).toBe('queued'))

    await act(async () => {
      await result.current.refresh()
    })
    await waitFor(() => expect(result.current.operation?.status).toBe('running'))

    await act(async () => {
      await result.current.refresh()
    })
    await waitFor(() => expect(result.current.operation?.status).toBe('completed'))
  })

  it('keeps polling an active Operation after a transient polling failure', async () => {
    vi.useFakeTimers()
    try {
      const client = createMockWebUiClient()
      const getOperation = client.getOperation.bind(client)
      vi.spyOn(client, 'getOperation')
        .mockResolvedValueOnce({
          data: null,
          error: { code: 'NETWORK_REQUEST_FAILED', message: 'The API request could not be completed.' },
        })
        .mockImplementation(getOperation)
      const { result } = renderHook(() => useOperation(client))

      await act(async () => {
        await result.current.submit(() => client.cleanStorageCache(), (data) => data.operation)
      })
      await act(async () => {
        await vi.advanceTimersByTimeAsync(500)
      })
      expect(result.current.operation?.status).toBe('queued')
      expect(result.current.error?.code).toBe('NETWORK_REQUEST_FAILED')

      await act(async () => {
        await vi.advanceTimersByTimeAsync(1_000)
      })
      expect(result.current.operation?.status).toBe('running')
      expect(result.current.error).toBeNull()
    } finally {
      vi.useRealTimers()
    }
  })
})
