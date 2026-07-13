import { useEffect, useMemo, useState } from 'react'

export const DEFAULT_PAGE_SIZE = 20

interface PaginationInput<T> {
  items: T[]
  pageSize?: number
  resetKey?: string
}

export interface PaginationState<T> {
  endItem: number
  items: T[]
  page: number
  pageSize: number
  setPage: (page: number) => void
  startItem: number
  totalItems: number
  totalPages: number
}

export function usePaginatedItems<T>({
  items,
  pageSize = DEFAULT_PAGE_SIZE,
  resetKey = '',
}: PaginationInput<T>): PaginationState<T> {
  const [pageState, setPageState] = useState({ page: 1, resetKey })
  const requestedPage = pageState.resetKey === resetKey ? pageState.page : 1
  const totalItems = items.length
  const totalPages = Math.max(1, Math.ceil(totalItems / pageSize))
  const currentPage = Math.min(requestedPage, totalPages)
  const startIndex = (currentPage - 1) * pageSize

  useEffect(() => {
    if (pageState.resetKey !== resetKey) setPageState({ page: 1, resetKey })
  }, [pageState.resetKey, resetKey])

  useEffect(() => {
    if (pageState.resetKey === resetKey && pageState.page > totalPages) {
      setPageState({ page: totalPages, resetKey })
    }
  }, [pageState.page, pageState.resetKey, resetKey, totalPages])

  const pagedItems = useMemo(
    () => items.slice(startIndex, startIndex + pageSize),
    [items, pageSize, startIndex],
  )

  return {
    endItem: totalItems === 0 ? 0 : Math.min(startIndex + pageSize, totalItems),
    items: pagedItems,
    page: currentPage,
    pageSize,
    setPage: (page) => setPageState({ page, resetKey }),
    startItem: totalItems === 0 ? 0 : startIndex + 1,
    totalItems,
    totalPages,
  }
}
