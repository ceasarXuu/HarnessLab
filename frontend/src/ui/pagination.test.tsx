import { act, renderHook } from '@testing-library/react'
import { describe, expect, it } from 'vitest'
import { usePaginatedItems } from './pagination'

describe('usePaginatedItems', () => {
  it('paginates after filtering and resets when the search key changes', () => {
    const items = Array.from({ length: 45 }, (_, index) => index + 1)
    const { result, rerender } = renderHook(
      ({ resetKey, values }) => usePaginatedItems({ items: values, resetKey }),
      { initialProps: { resetKey: '', values: items } },
    )

    act(() => result.current.setPage(3))
    expect(result.current.items).toEqual([41, 42, 43, 44, 45])

    rerender({ resetKey: 'needle', values: items.slice(20) })

    expect(result.current.page).toBe(1)
    expect(result.current.items).toEqual(items.slice(20, 40))
  })
})
