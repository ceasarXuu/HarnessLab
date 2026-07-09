import { renderHook, waitFor } from '@testing-library/react'
import { describe, expect, it, vi } from 'vitest'
import type { ApiResponse } from './contract'
import { useWebUiResource } from './hooks'

describe('useWebUiResource', () => {
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
})
